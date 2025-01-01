use chrono::{DateTime, Utc};
use rusqlite::{named_params, Connection};
use serde_derive::{Deserialize, Serialize};
use serde_rusqlite as sq_serde;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Token {
    pub partition: String,
    pub token_type: String,
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
}

impl Token {
    pub fn find(db: &Connection, partition: String) -> Result<Option<Token>, rusqlite::Error> {
        let mut statement = db
            .prepare("SELECT * FROM tokens WHERE partition = :partition")
            .unwrap();
        let rows = statement.query_and_then(named_params! {":partition": partition}, |row| {
            sq_serde::from_row::<Token>(row)
        });
        match rows {
            Err(e) => match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                _ => Err(e),
            },
            Ok(r) => {
                for raw in r.into_iter() {
                    match raw {
                        Ok(i) => {
                            if i.expires_at < Utc::now() {
                                continue;
                            }
                            return Ok(Some(i));
                        }
                        Err(e) => {
                            log::error!("sql_serde error on token: {}", e);
                        }
                    }
                }
                Ok(None)
            }
        }
    }

    pub fn insert(&self, db: &Connection) -> Result<(), rusqlite::Error> {
        db.execute(
            "INSERT INTO tokens (partition, token_type, access_token, expires_at)
            VALUES (:partition, :token_type, :access_token, :expires_at)
            ON CONFLICT (partition, token_type) DO UPDATE SET
                access_token = excluded.access_token,
                expires_at = excluded.expires_at
            ",
            sq_serde::to_params_named(self)
                .unwrap()
                .to_slice()
                .as_slice(),
        )?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Registration {
    pub partition: String,
    pub client_id: String,
    pub client_secret: String,
    pub expires_at: DateTime<Utc>,
    pub issued_at: DateTime<Utc>,
}

impl Registration {
    pub fn find(
        db: &Connection,
        partition: String,
    ) -> Result<Option<Registration>, rusqlite::Error> {
        let mut statement = db.prepare(
            "SELECT * FROM registrations WHERE partition = :partition AND expires_at > :now LIMIT 1").unwrap();
        let rows = statement.query_and_then(
            named_params! {":partition": partition, ":now": Utc::now()},
            sq_serde::from_row::<Registration>,
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

    pub fn insert(&self, db: &Connection) -> Result<(), rusqlite::Error> {
        db.execute(
            "INSERT INTO registrations (partition, client_id, client_secret, expires_at, issued_at) \
            VALUES (:partition, :client_id, :client_secret, :expires_at, :issued_at)
            ON CONFLICT (partition) DO UPDATE SET
                client_id = excluded.client_id,
                client_secret = excluded.client_secret,
                expires_at = excluded.expires_at,
                issued_at = excluded.issued_at
            ",
            sq_serde::to_params_named(self
            )
                .unwrap()
                .to_slice()
                .as_slice(),
        )?;
        Ok(())
    }
}
