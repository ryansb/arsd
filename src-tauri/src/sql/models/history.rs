use rusqlite::Connection;
use serde_derive::{Deserialize, Serialize};
use serde_rusqlite as sq_serde;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum AssumeStyle {
    WebConsole,
    WindowsCopy,
    LinuxCopy,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct HistoryNew {
    pub partition: String,
    pub account: String,
    pub role: String,
    pub style: AssumeStyle,
    pub service: Option<String>,
}

impl HistoryNew {
    pub fn insert(&self, db: &Connection) -> Result<(), rusqlite::Error> {
        db.execute(
            "INSERT INTO history (partition, account, role, style) \
            VALUES (:partition, :account, :role, :style)",
            sq_serde::to_params_named_with_fields(self, &["partition", "account", "role", "style"])
                .unwrap()
                .to_slice()
                .as_slice(),
        )?;
        Ok(())
    }
}
