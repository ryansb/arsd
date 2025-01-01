use aws_sdk_ssooidc::{self, Error as SsoIdcError};
use chrono::{serde::ts_milliseconds, DateTime, Utc};
use tauri::{AppHandle, Manager};

use crate::configuration::Partition;
use crate::domain::storage::client_name;
use crate::{sql, sql::ServiceAccess};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
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
}

#[derive(Clone, Debug)]
pub enum Event {
    RegisterDevice,
    StartDeviceAuthorization,
    ConfirmDeviceAuthorization(ConfirmationInfo),
}

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Start,                                  // nothing has been checked
    Registered,                             // non-expired registration is available
    AwaitingConfirmation(ConfirmationInfo), // a token was requested, but the user has to confirm
    Ready,                                  // non-expired token is available
    Failed { message: String },
}

pub struct SessionState {
    partition: Partition,
    app: AppHandle,
    oidc: aws_sdk_ssooidc::Client,
    state: State,
}
impl SessionState {
    /*
     Event loop for each partition:
     - `authorize_device` emitted by `main` on app start
     - `confirm_device` emitted by SSOSession if user confirmation is required
     - `partition_state` emitted by SSOSession when the partition state changes
    */
    pub async fn new(
        app: AppHandle,
        partition: Partition,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        log::info!("starting login check");
        let config = partition.aws_config().await;
        Ok(Self {
            partition,
            app,
            state: State::Start,
            oidc: aws_sdk_ssooidc::Client::new(&config),
        })
    }

    pub async fn next(&mut self, event: Event) -> State {
        if self
            .app
            .db(|db| sql::models::Token::find(db, self.partition.slug()))
            .unwrap()
            .is_some()
        {
            self.state = State::Ready;
            log::info!("found valid token, short-circuiting login");
            return State::Ready;
        };

        match (self.state.clone(), event.clone()) {
            (State::Start, Event::RegisterDevice) => {
                if self
                    .app
                    .db(|db| sql::models::Registration::find(db, self.partition.slug()))
                    .unwrap()
                    .is_none()
                {
                    let req = self
                        .oidc
                        .register_client()
                        .client_name(client_name())
                        .client_type("public");
                    log::info!(
                        "sending req for new {} secret name={:?}, type={:?}, scopes={:?}",
                        self.partition.slug(),
                        req.get_client_name(),
                        req.get_client_type(),
                        req.get_scopes()
                    );
                    let r = req.send().await.expect("Error registering client");
                    self.app.db(|db| {
                        let m = sql::models::Registration {
                            partition: self.partition.slug(),
                            client_id: r.client_id().unwrap().to_string(),
                            client_secret: r.client_secret().unwrap().to_string(),
                            issued_at: DateTime::<Utc>::from_timestamp(r.client_id_issued_at(), 0)
                                .expect("client ID issue timestamp should parse"),
                            expires_at: DateTime::<Utc>::from_timestamp(
                                r.client_secret_expires_at(),
                                0,
                            )
                            .expect("client ID expiry timestamp should parse"),
                        };
                        m.insert(db).expect("Failed to save registration for the current partition");
                    });
                };
                self.state = State::Registered;
                self.app
                    .emit_all("needs_confirmation", self.partition.slug())
                    .unwrap();
                State::Registered
                // learn what boxing is if we want this state machine to be recursive
                // self.next(Event::StartDeviceAuthorization).await
            }
            (State::Registered, Event::StartDeviceAuthorization) => {
                if self
                    .app
                    .db(|db| sql::models::Token::find(db, self.partition.slug()))
                    .unwrap()
                    .is_some()
                {
                    self.state = State::Ready;
                    return State::Ready;
                };

                log::info!("no valid token found");
                let registration = match self
                    .app
                    .db(|db| sql::models::Registration::find(db, self.partition.slug()))
                    .unwrap()
                {
                    None => {
                        log::warn!("no registration found, returning to start");
                        self.state = State::Start;
                        return State::Start;
                    }
                    Some(r) => r,
                };
                let req = self
                    .oidc
                    .start_device_authorization()
                    .client_id(registration.client_id)
                    .client_secret(registration.client_secret)
                    .start_url(self.partition.sso_start_url());

                log::info!(
                    "starting device req id={:?} url={:?} secret={:?}",
                    req.get_client_id(),
                    req.get_start_url(),
                    req.get_client_secret(),
                );
                let confirmation = match req.send().await {
                    Err(e) => {
                        log::error!(
                            "Failed to start device auth for {}: {:?}",
                            self.partition.slug(),
                            e
                        );
                        return State::Failed {
                            message: "Failed to start device auth".to_string(),
                        };
                    }
                    Ok(resp) => ConfirmationInfo {
                        partition: self.partition.slug(),
                        user_code: resp.user_code().unwrap().to_string(),
                        device_code: resp.device_code().unwrap().to_string(),
                        expires_at: Utc::now()
                            + chrono::Duration::seconds(resp.expires_in().into()),
                        confirmation_url: resp.verification_uri_complete().unwrap().to_string(),
                        polling_interval: resp.interval(),
                    },
                };

                self.state = State::AwaitingConfirmation(confirmation.clone());
                State::AwaitingConfirmation(confirmation)
            }
            (_, Event::ConfirmDeviceAuthorization(cc)) => {
                let registration = match self
                    .app
                    .db(|db| sql::models::Registration::find(db, self.partition.slug()))
                    .unwrap()
                {
                    None => {
                        log::warn!("no registration found, returning to start");
                        self.state = State::Start;
                        return State::Start;
                    }
                    Some(r) => r,
                };

                let req = self
                    .oidc
                    .create_token()
                    .client_id(registration.client_id)
                    .client_secret(registration.client_secret)
                    .device_code(cc.device_code.clone())
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
                        self.app.db_mut(|db| {
                            sql::models::Token {
                                partition: self.partition.slug(),
                                token_type: resp.token_type().unwrap().to_string(),
                                access_token: resp.access_token().unwrap().to_string(),
                                expires_at: Utc::now()
                                    + chrono::Duration::seconds(
                                        resp.expires_in().into()
                                    ),
                            }
                            .insert(db)
                            .unwrap()
                        });
                        self.state = State::Ready;
                        State::Ready
                    }
                    Err(SsoIdcError::AuthorizationPendingException(e)) => {
                        log::info!("Auth Pending: {:?}", e);
                        State::AwaitingConfirmation(cc)
                    }
                    Err(SsoIdcError::SlowDownException(e)) => {
                        log::info!("Slow down: {:?}", e);
                        State::AwaitingConfirmation(cc)
                    }
                    Err(e) => {
                        log::error!("Error confirming device registration: {:?}", e);
                        State::AwaitingConfirmation(cc)
                    }
                }
            }
            _ => todo!("event={:?} is not legal for state={:?}", event, self.state),
        }
    }
}
