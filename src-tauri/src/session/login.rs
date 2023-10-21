use aws_config;
use aws_sdk_ssooidc::{self, Error as SsoIdcError};
use chrono::{serde::ts_milliseconds, DateTime, Utc};

use crate::{configuration::Partition, domain::Storage};

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct ConfirmationInfo {
    pub partition: String,
    pub user_code: String,
    pub device_code: String,
    pub confirmation_url: String,
    pub polling_interval: i32,
    #[serde(with = "ts_milliseconds")]
    pub expires_at: DateTime<Utc>,
}

impl ConfirmationInfo {
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct RefreshInfo {}
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct SuccessInfo {
    #[serde(with = "ts_milliseconds")]
    pub expires_at: DateTime<Utc>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum DeviceAuthState {
    NeedsConfirmation(ConfirmationInfo),
    NeedsRefresh(RefreshInfo),
    Success(SuccessInfo),
    Pending,
    Failure,
}

pub struct SSOSession {
    partition: Partition,
    storage: Storage,
    oidc: aws_sdk_ssooidc::Client,
}
impl SSOSession {
    /*
     Event loop for each partition:
     - `authorize_device` emitted by `main` on app start
     - `confirm_device` emitted by SSOSession if user confirmation is required
     - `partition_state` emitted by SSOSession when the partition state changes
    */
    pub async fn new(
        storage: Storage,
        partition: Partition,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        log::info!("starting login check");
        let config = aws_config::load_from_env().await;
        Ok(Self {
            partition,
            storage,
            oidc: aws_sdk_ssooidc::Client::new(&config),
        })
    }

    pub async fn confirm_device_registration(
        &self,
        device_code: String,
    ) -> Result<DeviceAuthState, String> {
        if let Some(registration) = self.storage.valid_registration(self.partition.clone()) {
            let req = self
                .oidc
                .create_token()
                .client_id(registration.client_id)
                .client_secret(registration.client_secret)
                .device_code(device_code.clone())
                .grant_type(String::from("urn:ietf:params:oauth:grant-type:device_code"));
            log::info!(
                "Sending CreateToken request: client_id={:?}, client_secret={:?}, device_code={:?}, code={:?}, grant_type={:?}, refresh_token={:?}, scope={:?}",
                req.get_client_id(),
                req.get_client_secret().is_some(),
                req.get_device_code(),
                req.get_code(),
                req.get_grant_type(),
                req.get_refresh_token(),
                req.get_scope()
            );
            match req.send().await.map_err(SsoIdcError::from) {
                Ok(resp) => {
                    log::info!("Got response: {:?}", resp);
                    let token = self.storage.token(
                        self.partition.clone(),
                        resp.access_token().unwrap().to_string(),
                        resp.expires_in().try_into().unwrap(),
                        resp.token_type().unwrap().to_string(),
                    );
                    return Ok(DeviceAuthState::Success(SuccessInfo {
                        expires_at: token.expires_at,
                    }));
                }
                Err(SsoIdcError::AuthorizationPendingException(e)) => {
                    log::info!("Auth Pending: {:?}", e);
                    return Ok(DeviceAuthState::Pending);
                }
                Err(SsoIdcError::SlowDownException(e)) => {
                    log::info!("Slow down: {:?}", e);
                    return Ok(DeviceAuthState::Pending);
                }
                Err(e) => {
                    let message = format!("Error confirming device registration: {:?}", e);
                    log::warn!("{}", message);
                    return Err(message);
                }
            }
        }
        Err("No registration found".to_string())
    }

    pub async fn authorize_device(&self) -> Result<DeviceAuthState, Box<dyn std::error::Error>> {
        /*
          1. check locally for a client ID/client secret
          1a. if there is none, send a ssooidc::RegisterClient call
          2. check locally for a device bearer token and refresh token
          2a. check expiry of bearer token
          2b. exit with NeedsRefresh
          3. if no local token, use ssooidc::StartDeviceAuthorization to get a URL for the user to visit and confirm
          3a. exit with NeedsConfirmation
        */
        let req = self
            .oidc
            .register_client()
            .client_name(self.storage.client_name())
            //.set_scopes(Some(self.partition.scopes()))
            .client_type("public");

        let registration = match self.storage.valid_registration(self.partition.clone()) {
            None => {
                log::info!(
                    "sending req for new {} secret name={:?}, type={:?}, scopes={:?}",
                    self.partition.slug(),
                    req.get_client_name(),
                    req.get_client_type(),
                    req.get_scopes()
                );
                let r = req.send().await?;
                self.storage.register(
                    self.partition.clone(),
                    r.client_id().unwrap().to_string(),
                    r.client_id_issued_at(),
                    r.client_secret().unwrap().to_string(),
                    r.client_secret_expires_at(),
                )
            }
            Some(registration) => {
                log::info!("registration already valid");
                registration
            }
        };

        match self.storage.valid_token(self.partition.clone()) {
            None => {
                log::info!("no valid token found");
                let req = self
                    .oidc
                    .start_device_authorization()
                    .client_id(registration.client_id)
                    .client_secret(registration.client_secret)
                    .start_url(self.partition.sso_start_url());

                log::info!(
                    "starting device req id={:?} url={:?} secret={:?}",
                    req.get_client_id(),
                    req.get_client_secret(),
                    req.get_start_url(),
                );
                let resp = req.send().await?;

                Ok(DeviceAuthState::NeedsConfirmation(ConfirmationInfo {
                    partition: self.partition.slug(),
                    user_code: resp.user_code().unwrap().to_string(),
                    device_code: resp.device_code().unwrap().to_string(),
                    expires_at: Utc::now() + chrono::Duration::seconds(resp.expires_in().into()),
                    confirmation_url: resp.verification_uri_complete().unwrap().to_string(),
                    polling_interval: resp.interval(),
                }))
            }
            Some(token) => {
                log::info!("found valid token");
                Ok(DeviceAuthState::Success(SuccessInfo {
                    expires_at: token.expires_at,
                }))
            }
        }
    }
}
