use aws_config;
use aws_sdk_sso::{self, Error as SsoError};
use chrono::{serde::ts_milliseconds, DateTime, Utc};

use crate::{
    configuration::Partition,
    domain::{AccountInfo, RoleInfo, Storage},
};

pub async fn list_roles(
    partition: Partition,
    storage: Storage,
    account_id: String,
) -> Vec<RoleInfo> {
    let token = match storage.valid_token(partition.clone()) {
        Some(t) => t,
        None => return vec![],
    };

    let config = aws_config::load_from_env().await;
    let client = aws_sdk_sso::Client::new(&config);

    let mut roles: Vec<RoleInfo> = vec![];
    let mut tries: i32 = 0;
    while tries < 10 {
        let req = client
            .list_account_roles()
            .access_token(token.access_token.clone())
            .account_id(account_id.clone());
        match req.send().await.map_err(SsoError::from) {
            Err(SsoError::TooManyRequestsException(e)) => {
                log::warn!("Slow down: {:?}", e);
                std::thread::sleep(std::time::Duration::from_millis(750));
                tries += 1;
            }
            Err(e) => {
                log::warn!("Failed to get roles in {}: {:?}", account_id.clone(), e);
            }
            Ok(pgn) => match pgn.role_list() {
                None => {}
                Some(l) => {
                    for role in l {
                        roles.push(RoleInfo {
                            alias: None,
                            account_id: role.account_id.clone().unwrap(),
                            role_name: role.role_name.clone().unwrap(),
                            partition: partition.slug(),
                        })
                    }
                    break;
                }
            },
        };
    }
    log::debug!(
        "Found roles for {}: {:?}",
        account_id,
        roles
            .iter()
            .map(|r| { r.role_name.clone() })
            .collect::<Vec<String>>()
    );
    roles
}

pub async fn list_accounts(partition: Partition, storage: Storage) -> Vec<AccountInfo> {
    let token = match storage.valid_token(partition.clone()) {
        Some(t) => t,
        None => return vec![],
    };

    let config = aws_config::load_from_env().await;
    let client = aws_sdk_sso::Client::new(&config);

    let mut accounts: Vec<AccountInfo> = vec![];
    let mut resp = client
        .list_accounts()
        .access_token(token.access_token.clone())
        .send()
        .await
        .unwrap();

    for l in resp.account_list().unwrap() {
        accounts.push(AccountInfo {
            account_id: l.account_id.clone().unwrap(),
            account_name: l.account_name.clone().unwrap(),
            email_address: l.email_address.clone().unwrap(),
            alias: None,
        });
    }

    while resp.next_token().is_some() {
        resp = client
            .list_accounts()
            .access_token(token.access_token.clone())
            .next_token(resp.next_token().unwrap().to_string())
            .send()
            .await
            .unwrap();
        for l in resp.account_list().unwrap() {
            accounts.push(AccountInfo {
                account_id: l.account_id.clone().unwrap(),
                account_name: l.account_name.clone().unwrap(),
                email_address: l.email_address.clone().unwrap(),
                alias: None,
            });
        }
    }

    log::info!(
        "Found accounts for {}: {:?}",
        partition.slug(),
        accounts
            .iter()
            .map(|a| { a.account_name.clone() })
            .collect::<Vec<String>>()
    );
    accounts
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Credentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: String,
    #[serde(with = "ts_milliseconds")]
    pub expires_at: DateTime<Utc>,
}

pub async fn get_credentials(
    partition: Partition,
    role_name: String,
    account_id: String,
    storage: Storage,
) -> Credentials {
    let token = match storage.valid_token(partition.clone()) {
        Some(t) => t,
        None => todo!("handle missing token"),
    };

    let config = aws_config::load_from_env().await;
    let client = aws_sdk_sso::Client::new(&config);

    let resp = client
        .get_role_credentials()
        .access_token(token.access_token)
        .role_name(role_name)
        .account_id(account_id)
        .send()
        .await
        .unwrap();
    let creds = resp.role_credentials().unwrap();
    Credentials {
        access_key_id: creds.access_key_id().unwrap().to_string(),
        secret_access_key: creds.secret_access_key().unwrap().to_string(),
        session_token: creds.session_token().unwrap().to_string(),
        expires_at: DateTime::from_timestamp(creds.expiration(), 0).unwrap(),
    }
}
