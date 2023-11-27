use aws_sdk_sso::{self, Error as SsoError};
use chrono::{serde::ts_milliseconds, DateTime, Utc};

use crate::{
    configuration::Partition,
    domain::{AccountInfo, RoleInfo},
    sql,
    sql::ServiceAccess,
};

pub async fn list_roles(partition: Partition, token: String, account_id: String) -> Vec<RoleInfo> {
    let config = partition.aws_config().await;
    let client = aws_sdk_sso::Client::new(&config);

    let mut roles: Vec<RoleInfo> = vec![];
    let mut tries: i32 = 0;
    while tries < 10 {
        let req = client
            .list_account_roles()
            .access_token(token.clone())
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
            Ok(pgn) => {
                let l = pgn.role_list();
                if pgn.role_list.is_none() {
                    log::warn!("No roles found for {}", account_id);
                    break;
                }
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

pub async fn list_accounts(partition: Partition, app: tauri::AppHandle) -> Vec<AccountInfo> {
    let candidates = app
        .db(|db| sql::models::Account::list(db, partition.slug()))
        .unwrap();
    if !candidates.is_empty()
        // only requery if the data is more than 5 hours old
        && candidates.iter().map(|a| a.updated_at).min().unwrap()
            > (Utc::now() - chrono::Duration::hours(5))
    {
        log::info!(
            "early-return accounts for {}: {}",
            partition.slug(),
            candidates
                .iter()
                .map(|a| { a.account_name.clone() })
                .collect::<Vec<String>>()
                .join(", ")
        );
        return candidates.iter().map(|a| a.as_info()).collect();
    }

    let token = match app
        .db(|db| sql::models::Token::find(db, partition.slug()))
        .unwrap()
    {
        Some(t) => t,
        None => return candidates.iter().map(|a| a.as_info()).collect(),
    };

    let config = partition.aws_config().await;
    let client = aws_sdk_sso::Client::new(&config);

    let mut accounts: Vec<sql::models::Account> = vec![];
    let mut resp = match client
        .list_accounts()
        .access_token(token.access_token.clone())
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            log::error!("Failed to get accounts in {}: {:?}", partition.slug(), e);
            return candidates.iter().map(|a| a.as_info()).collect();
        }
    };

    for l in resp.account_list() {
        accounts.push(sql::models::Account {
            partition: partition.slug(),
            account_id: l.account_id.clone().unwrap(),
            account_name: l.account_name.clone().unwrap(),
            email_address: l.email_address.clone().unwrap(),
            updated_at: Utc::now(),
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
        for l in resp.account_list() {
            accounts.push(sql::models::Account {
                partition: partition.slug(),
                account_id: l.account_id.clone().unwrap(),
                account_name: l.account_name.clone().unwrap(),
                email_address: l.email_address.clone().unwrap(),
                updated_at: Utc::now(),
            });
        }
    }
    app.db(|db| {
        for a in &accounts {
            a.insert(db).expect("Failed to insert account in local DB");
        }
    });

    log::debug!(
        "Found accounts for {}: {:?}",
        partition.slug(),
        accounts
            .iter()
            .map(|a| { a.account_name.clone() })
            .collect::<Vec<String>>()
    );
    accounts.iter().map(|a| a.as_info()).collect()
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
    app: tauri::AppHandle,
) -> Credentials {
    let token = match app
        .db(|db| sql::models::Token::find(db, partition.slug()))
        .unwrap()
    {
        Some(t) => t,
        None => todo!("handle missing token"),
    };

    let config = partition.aws_config().await;
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
