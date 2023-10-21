use chrono::{serde::ts_milliseconds, DateTime, Duration, Utc};
use std::path::PathBuf;

use chrono;
use log;
use serde_json::json;
use tauri::AppHandle;
use tauri_plugin_store;
use whoami;

use std::sync::Mutex;

use crate::configuration::Partition;

enum StoreType {
    Registration(Partition),
    Token(Partition),
    Client,
}

pub struct Storage {
    dir: PathBuf,
    app: AppHandle,
    write: Mutex<()>,
}
impl Storage {
    pub fn new(app: AppHandle) -> Self {
        Self {
            dir: app.path_resolver().app_data_dir().unwrap(),
            app,
            write: Mutex::new(()),
        }
    }

    fn store(&self, t: StoreType) -> PathBuf {
        let suffix = match t {
            StoreType::Client => String::from("client-name.bin"),
            StoreType::Registration(p) => format!("{}-registration.bin", p.slug()),
            StoreType::Token(p) => format!("{}-token.bin", p.slug()),
        };
        self.dir.join(suffix)
    }

    pub fn valid_registration(&self, partition: Partition) -> Option<RegistrationInfo> {
        match self.get(StoreType::Registration(partition), "registration") {
            Some(v) => match serde_json::from_value::<RegistrationInfo>(v) {
                Ok(reg) => {
                    if reg.client_secret_expires_at
                        > (Utc::now().timestamp() + 300/* 5 minute buffer */)
                    {
                        Some(reg)
                    } else {
                        None
                    }
                }
                Err(e) => {
                    log::warn!("Error deserializing registration struct: {}", e);
                    None
                }
            },
            None => None,
        }
    }

    pub fn register(
        &self,
        partition: Partition,
        client_id: String,
        client_id_issued_at: i64,
        client_secret: String,
        client_secret_expires_at: i64,
    ) -> RegistrationInfo {
        let reg = RegistrationInfo {
            client_id,
            client_id_issued_at,
            client_secret,
            client_secret_expires_at,
        };
        self.insert(
            StoreType::Registration(partition),
            "registration",
            serde_json::to_value::<&RegistrationInfo>(&reg).unwrap(),
        );
        reg
    }

    pub fn valid_token(&self, partition: Partition) -> Option<TokenInfo> {
        match self.get(StoreType::Token(partition), "authn") {
            Some(v) => match serde_json::from_value::<TokenInfo>(v) {
                Ok(token) => {
                    if token.expires_at > Utc::now() {
                        Some(token)
                    } else {
                        None
                    }
                }
                Err(e) => {
                    log::warn!("Error deserializing registration struct: {}", e);
                    None
                }
            },
            None => None,
        }
    }

    pub fn token(
        &self,
        partition: Partition,
        access_token: String,
        expires_in: i64,
        token_type: String,
    ) -> TokenInfo {
        if token_type != "Bearer" {
            panic!("token_type must be Bearer");
        }
        let token = TokenInfo {
            access_token,
            expires_at: Utc::now() + Duration::seconds(expires_in),
            token_type: String::from("Bearer"),
            refresh_token: None,
            id_token: None,
        };
        self.insert(
            StoreType::Token(partition),
            "authn",
            serde_json::to_value::<&TokenInfo>(&token).unwrap(),
        );
        token
    }

    pub fn client_name(&self) -> String {
        if let Some(v) = self.get(StoreType::Client, "client_name") {
            match serde_json::from_value::<String>(v) {
                Ok(v) => return v,
                Err(e) => {
                    log::warn!("Error deserializing client_name: {}", e);
                }
            }
        }
        let name = format!(
            "{} arsd {}@{}",
            whoami::platform(),
            whoami::username(),
            whoami::hostname(),
        );
        self.insert(StoreType::Client, "client_name", json!(name));
        name
    }

    pub fn path(&self) -> String {
        self.dir.join("store.bin").to_string_lossy().to_string()
    }

    fn get(&self, t: StoreType, key: &str) -> Option<serde_json::Value> {
        let _l = self.write.lock();
        let path = self.store(t);
        let mut store =
            tauri_plugin_store::StoreBuilder::new(self.app.clone(), path.clone()).build();
        if let Err(e) = store.load() {
            log::info!("Store not initialized at {:?}: {}", path, e);
        }
        store.get(key).cloned()
    }

    fn insert(&self, t: StoreType, key: &str, value: serde_json::Value) {
        let _l = self.write.lock();
        let mut store =
            tauri_plugin_store::StoreBuilder::new(self.app.clone(), self.store(t)).build();
        store.insert(key.to_string(), value).unwrap();
        store.save().unwrap();
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RegistrationInfo {
    pub client_id: String,
    pub client_id_issued_at: i64,
    pub client_secret: String,
    pub client_secret_expires_at: i64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TokenInfo {
    pub access_token: String,
    #[serde(with = "ts_milliseconds")]
    pub expires_at: DateTime<Utc>,
    // always `BearerToken`
    pub token_type: String,
    pub refresh_token: Option<String>,
    // not implemented by AWS as of 2023-11
    // https://docs.aws.amazon.com/singlesignon/latest/OIDCAPIReference/API_CreateToken.html#API_CreateToken_ResponseSyntax
    pub id_token: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AccountInfo {
    pub account_id: String,
    pub account_name: String,
    pub email_address: String,
    pub alias: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RoleInfo {
    pub account_id: String,
    pub role_name: String,
    pub partition: String,
    pub alias: Option<String>,
}
