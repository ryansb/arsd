use rusqlite::Connection;
use std::fs;
use tauri::AppHandle;

const CURRENT_DB_VERSION: u32 = 1;

/// Initializes the database connection, creating the .sqlite file if needed, and upgrading the database
/// if it's out of date.
pub fn initialize_database(app_handle: &AppHandle) -> Result<Connection, rusqlite::Error> {
    let app_dir = app_handle
        .path_resolver()
        .app_data_dir()
        .expect("The app data directory should exist.");
    fs::create_dir_all(&app_dir).expect("The app data directory should be created.");
    let sqlite_path = app_dir.join("arsd.sqlite");

    let mut db = Connection::open(sqlite_path)?;

    let mut user_pragma = db.prepare("PRAGMA user_version")?;
    let existing_user_version: u32 = user_pragma.query_row([], |row| row.get(0))?;
    drop(user_pragma);

    upgrade_database_if_needed(&mut db, existing_user_version)?;

    Ok(db)
}

/// Upgrades the database to the current version.
pub fn upgrade_database_if_needed(
    db: &mut Connection,
    existing_version: u32,
) -> Result<(), rusqlite::Error> {
    if existing_version < CURRENT_DB_VERSION {
        db.pragma_update(None, "journal_mode", "WAL")?;

        let tx = db.transaction()?;

        tx.pragma_update(None, "user_version", CURRENT_DB_VERSION)?;

        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                partition TEXT NOT NULL,
                account TEXT NOT NULL,
                role TEXT NOT NULL,
                service TEXT,
                style TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS tokens (
                partition TEXT NOT NULL,
                token_type TEXT NOT NULL,
                access_token TEXT NOT NULL,
                expires_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (partition, token_type)
            );
            CREATE TABLE IF NOT EXISTS registrations (
                partition TEXT PRIMARY KEY,
                client_id TEXT NOT NULL,
                client_secret TEXT NOT NULL,
                expires_at TIMESTAMP NOT NULL,
                issued_at TIMESTAMP NOT NULL
            );
            CREATE TABLE IF NOT EXISTS roles (
                partition TEXT NOT_NULL,
                account_id TEXT NOT NULL,
                role_name TEXT NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (partition, account_id, role_name)
            );
            CREATE TABLE IF NOT EXISTS accounts (
                partition TEXT NOT NULL,
                account_id TEXT NOT NULL,
                email_address TEXT NOT NULL,
                account_name TEXT NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (partition, account_id)
            );
            ",
        )?;

        tx.commit()?;
    }

    Ok(())
}
