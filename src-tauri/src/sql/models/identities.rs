use chrono::{DateTime, Utc};
use rusqlite::{named_params, Connection, OptionalExtension};
use serde_derive::{Deserialize, Serialize};
use serde_rusqlite as sq_serde;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Account {
    pub partition: String,
    pub account_id: String,
    pub email_address: String,
    pub account_name: String,
    pub updated_at: DateTime<Utc>,
}

impl Account {
    pub fn find(
        db: &Connection,
        partition: String,
        account_id: String,
    ) -> Result<Option<Account>, rusqlite::Error> {
        let mut statement = db
            .prepare(
                "SELECT * FROM accounts WHERE partition = :partition AND account_id = :account_id",
            )
            .unwrap();
        let rows = statement.query_and_then(
            named_params! {":account_id": account_id, ":partition": partition},
            sq_serde::from_row::<Account>,
        );
        match rows {
            Err(e) => match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                _ => Err(e),
            },
            Ok(r) => match r.into_iter().next() {
                Some(i) => Ok(Some(i.expect("Something went wrong with sql_serde"))),
                None => Ok(None),
            },
        }
    }

    pub fn score(db: &Connection, partition: String, account_id: String) -> Option<i64> {
        let mut statement = db
            .prepare(
                "with ranks as (
                    SELECT 
                        t.account, (
                            SELECT COUNT(*) + 1
                            FROM (SELECT account, COUNT(*) AS frequency 
                                  FROM history 
                                  GROUP BY account) AS sub_t
                            WHERE sub_t.frequency < t.frequency
                        ) AS rank
                    FROM (
                        SELECT account, COUNT(*) AS frequency 
                        FROM history 
                        WHERE partition = :partition
                        GROUP BY account) AS t
                    )
                SELECT rank FROM ranks WHERE account = :account_id",
            )
            .unwrap();
        match statement
            .query_row(
                named_params! {":account_id": account_id, ":partition": partition},
                |r| r.get(0),
            )
            .optional()
        {
            Err(e) => {
                log::warn!("rank failed for account {}: {:?}", account_id, e);
                None
            }
            Ok(r) => r,
        }
    }

    pub fn list(db: &Connection, partition: String) -> Result<Vec<Account>, rusqlite::Error> {
        let mut statement = db
            .prepare("SELECT * FROM accounts WHERE partition = :partition")
            .unwrap();
        let rows = statement.query_and_then(named_params! {":partition": partition, }, |row| {
            sq_serde::from_row::<Account>(row)
        });
        match rows {
            Err(e) => match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(vec![]),
                _ => Err(e),
            },
            Ok(r) => Ok(r
                .into_iter()
                .filter_map(|i| match i {
                    Err(e) => {
                        log::error!("sql_serde error on account: {}", e);
                        None
                    }
                    Ok(i) => Some(i),
                })
                .collect()),
        }
    }

    pub fn insert(&self, db: &Connection) -> Result<(), rusqlite::Error> {
        db.execute(
            "INSERT INTO accounts (partition, account_id, email_address, account_name, updated_at)
            VALUES (:partition, :account_id, :email_address, :account_name, :updated_at)
            ON CONFLICT (partition, account_id) DO UPDATE SET
                email_address = excluded.email_address,
                account_name = excluded.account_name,
                updated_at = excluded.updated_at",
            sq_serde::to_params_named(self)
                .unwrap()
                .to_slice()
                .as_slice(),
        )?;
        Ok(())
    }

    pub fn as_info(&self) -> crate::domain::AccountInfo {
        crate::domain::AccountInfo {
            account_id: self.account_id.clone(),
            account_name: self.account_name.clone(),
            email_address: self.email_address.clone(),
            alias: None,
            score: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Role {
    pub partition: String,
    pub account_id: String,
    pub role_name: String,
    pub updated_at: DateTime<Utc>,
}

impl Role {
    pub fn list(
        db: &Connection,
        partition: String,
        account_id: String,
    ) -> Result<Vec<Role>, rusqlite::Error> {
        let mut statement = db
            .prepare(
                "SELECT * FROM roles WHERE partition = :partition AND account_id = :account_id",
            )
            .unwrap();
        let rows = statement.query_and_then(
            named_params! {":partition": partition, ":account_id": account_id},
            sq_serde::from_row::<Role>,
        );
        match rows {
            Err(e) => match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(vec![]),
                _ => Err(e),
            },
            Ok(r) => Ok(r
                .into_iter()
                .filter_map(|i| match i {
                    Err(e) => {
                        log::error!("sql_serde error on role: {}", e);
                        None
                    }
                    Ok(i) => Some(i),
                })
                .collect()),
        }
    }

    pub fn insert(&self, db: &Connection) -> Result<(), rusqlite::Error> {
        db.execute(
            "INSERT INTO roles (partition, account_id, role_name, updated_at)
            VALUES (:partition, :account_id, :role_name, :updated_at)
            ON CONFLICT (partition, account_id, role_name) DO UPDATE SET
                updated_at = excluded.updated_at
            ",
            sq_serde::to_params_named(self)
                .unwrap()
                .to_slice()
                .as_slice(),
        )?;
        Ok(())
    }

    pub fn from_info(info: crate::domain::RoleInfo) -> Self {
        Role {
            partition: info.partition,
            account_id: info.account_id,
            role_name: info.role_name,
            updated_at: Utc::now(),
        }
    }

    pub fn as_info(self) -> crate::domain::RoleInfo {
        crate::domain::RoleInfo {
            partition: self.partition.clone(),
            account_id: self.account_id.clone(),
            role_name: self.role_name,
            alias: None,
        }
    }
}
