use rusqlite::{named_params, Connection};
use serde_derive::{Deserialize, Serialize};
use serde_rusqlite as sq_serde;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum SortOrder {
    ALPHA,
    FRECENCY,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SettingSort {
    pub value: SortOrder,
}

impl SettingSort {
    pub fn get(db: &Connection) -> Result<i32, rusqlite::Error> {
        // 0 for ALPHA, 1 for FRECENCY
        let mut statement = db
            .prepare("SELECT * FROM settings WHERE key = :key")
            .unwrap();
        let rows = statement.query_and_then(
            named_params! {":key": "SORT_ORDER".to_string()},
            sq_serde::from_row::<SettingSort>,
        );
        match rows {
            Err(e) => match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(0),
                _ => Err(e),
            },
            Ok(r) => match r.into_iter().next() {
                Some(i) => match i.expect("Something went wrong with sql_serde").value {
                    SortOrder::ALPHA => Ok(0),
                    SortOrder::FRECENCY => Ok(1),
                },
                None => Ok(0),
            },
        }
    }
    pub fn insert(&self, db: &Connection) -> Result<(), rusqlite::Error> {
        db.execute(
            "INSERT INTO settings (key, value) VALUES ('SORT_ORDER', :value)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            sq_serde::to_params_named_with_fields(self, &["key", "value"])
                .unwrap()
                .to_slice()
                .as_slice(),
        )?;
        Ok(())
    }
}
