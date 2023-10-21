#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AuthorizeDevice {
    pub partition_name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ConfirmDevice {
    pub partition_name: String,
    pub confirmation_code: String,
    pub confirmation_url: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct PartitionState {
    pub partition_name: String,
    pub state: String,
}
