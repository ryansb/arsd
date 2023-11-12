use whoami;

pub fn client_name() -> String {
    format!(
        "{} arsd {}@{}",
        whoami::platform(),
        whoami::username(),
        whoami::hostname(),
    )
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AccountInfo {
    pub account_id: String,
    pub account_name: String,
    pub email_address: String,
    pub alias: Option<String>,
    pub score: Option<i64>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RoleInfo {
    pub account_id: String,
    pub role_name: String,
    pub partition: String,
    pub alias: Option<String>,
}
