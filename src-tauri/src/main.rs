// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use arsd::session::login::ConfirmationInfo;
use tauri::Listener;
use tauri::{AppHandle, Emitter, EventTarget, Manager, State};
use tauri_plugin_log::Target as LogTarget;
use tauri_plugin_log::TargetKind as LogTargetKind;

use arsd::configuration::{get_configuration, Settings};
use arsd::domain::{AccountInfo, RoleInfo};
use arsd::session::{account, account::Credentials, console_url, events, login};
use arsd::sql;
use arsd::sql::ServiceAccess;

#[derive(Clone, serde::Serialize)]
struct SingletonPayload {
    args: Vec<String>,
    cwd: String,
}

#[derive(serde::Serialize)]
struct AppDirs {
    data: String,
    config: String,
    logs: String,
}
#[tauri::command]
fn storage_path(app: tauri::AppHandle, settings: State<Settings>) -> AppDirs {
    AppDirs {
        data: app
            .path()
            .app_data_dir()
            .unwrap()
            .to_string_lossy()
            .to_string(),
        config: settings.path.to_string_lossy().to_string(),
        logs: app
            .path()
            .app_log_dir()
            .unwrap()
            .to_string_lossy()
            .to_string(),
    }
}

#[tauri::command]
async fn open_web_console(
    config: State<'_, Settings>,
    app: tauri::AppHandle,
    role_name: String,
    partition: String,
    account_id: String,
) -> Result<String, String> {
    app.db_mut(|db| {
        sql::models::HistoryNew {
            partition: partition.clone(),
            account: account_id.clone(),
            role: role_name.clone(),
            style: sql::models::AssumeStyle::WebConsole,
            service: None,
        }
        .insert(db)
    })
    .unwrap();
    match config.partition(partition) {
        None => todo!("Bad partition"),
        Some(part) => Ok(console_url::get_console_url(account_id, role_name, part, app).await),
    }
}

#[tauri::command]
async fn list_accounts(
    config: State<'_, Settings>,
    app: tauri::AppHandle,
    partition: String,
) -> Result<Vec<AccountInfo>, String> {
    match config.partition(partition.clone()) {
        None => todo!("Bad partition"),
        Some(part) => {
            return Ok(account::list_accounts(part, app.clone())
                .await
                .iter()
                .map(|a| AccountInfo {
                    account_id: a.account_id.clone(),
                    account_name: a.account_name.clone(),
                    email_address: a.email_address.clone(),
                    alias: config.aliases.map_account(a.email_address.clone()),
                    score: app.db(|db| {
                        sql::models::Account::score(db, partition.clone(), a.account_id.clone())
                    }),
                })
                .collect());
        }
    }
}

#[tauri::command]
async fn settings_get_sort(app: tauri::AppHandle) -> i32 {
    app.db(|db| sql::models::SettingSort::get(db).unwrap_or(0))
}

#[tauri::command]
async fn settings_save_sort(sort: i32, app: tauri::AppHandle) -> Result<(), String> {
    let extant = app.db_mut(|db| match sort {
        0 => sql::models::SettingSort {
            value: sql::models::SortOrder::ALPHA,
        }
        .insert(db),
        1 => sql::models::SettingSort {
            value: sql::models::SortOrder::FRECENCY,
        }
        .insert(db),
        _ => Err(rusqlite::Error::InvalidQuery),
    });
    match extant {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Failed to delete local data: {:?}", e);
            Err(String::from("Delete failed"))
        }
    }
}

#[tauri::command]
async fn delete_cache(app: tauri::AppHandle) -> Result<(), String> {
    let extant = app.db(|db| {
        db.execute_batch(
            "BEGIN;
            DELETE FROM roles WHERE true;
            DELETE FROM registrations WHERE true;
            DELETE FROM accounts WHERE true;
            DELETE FROM tokens WHERE true;
            COMMIT;",
        )
    });
    match extant {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Failed to delete local data: {:?}", e);
            Err(String::from("Delete failed"))
        }
    }
}

#[tauri::command]
async fn list_roles_for(
    config: State<'_, Settings>,
    app: tauri::AppHandle,
    partition: String,
    account_id: String,
) -> Result<Vec<RoleInfo>, String> {
    match config.partition(partition) {
        None => todo!("Bad partition"),
        Some(part) => {
            let extant = app
                .db(|db| sql::models::Role::list(db, part.slug(), account_id.clone()))
                .unwrap()
                .iter()
                .map(|r| RoleInfo {
                    role_name: r.role_name.clone(),
                    alias: Some(config.aliases.map_role(r.role_name.clone())),
                    account_id: r.account_id.clone(),
                    partition: r.partition.clone(),
                })
                .collect::<Vec<RoleInfo>>();
            if !extant.is_empty() {
                log::debug!(
                    "Found roles for {} in db: {:?}",
                    account_id.clone(),
                    extant.len()
                );
                return Ok(extant);
            }
            let token = match app
                .db(|db| sql::models::Token::find(db, part.slug()))
                .unwrap()
            {
                None => {
                    log::warn!("No token found for {}", part.slug());
                    return Ok(vec![]);
                }
                Some(t) => t,
            };
            let roles =
                account::list_roles(part.clone(), token.access_token, account_id.clone()).await;
            for r in roles.iter() {
                log::warn!("Inserting role: {:?}", r.clone());
                app.db_mut(|db| {
                    sql::models::Role {
                        partition: part.slug(),
                        account_id: account_id.clone(),
                        role_name: r.role_name.clone(),
                        updated_at: chrono::Utc::now(),
                    }
                    .insert(db)
                })
                .unwrap();
            }

            return Ok(roles
                .iter()
                .map(|r| RoleInfo {
                    role_name: r.role_name.clone(),
                    alias: Some(config.aliases.map_role(r.role_name.clone())),
                    account_id: r.account_id.clone(),
                    partition: r.partition.clone(),
                })
                .collect());
        }
    }
}

#[derive(serde::Serialize)]
struct PartitionDisplay {
    pub start_url: String,
    pub account_id: Option<String>,
    pub region: String,
    pub slug: String,
}

#[tauri::command]
fn get_partitions(config: State<Settings>) -> Vec<PartitionDisplay> {
    config
        .partitions
        .iter()
        .map(|p| PartitionDisplay {
            start_url: p.start_url.clone(),
            account_id: p.account_id.clone(),
            region: p.region.clone(),
            slug: p.slug(),
        })
        .collect::<Vec<PartitionDisplay>>()
}

#[tauri::command]
async fn get_credentials_for(
    partition: String,
    account_id: String,
    role_name: String,
    app: AppHandle,
    config: State<'_, Settings>,
) -> Result<Credentials, String> {
    app.db_mut(|db| {
        sql::models::HistoryNew {
            partition: partition.clone(),
            account: account_id.clone(),
            role: role_name.clone(),
            style: sql::models::AssumeStyle::LinuxCopy,
            service: None,
        }
        .insert(db)
    })
    .unwrap();
    Ok(account::get_credentials(
        config.partition(partition).unwrap(),
        role_name,
        account_id,
        app,
    )
    .await)
}

#[tauri::command]
async fn check_device_token(
    token_event: ConfirmationInfo,
    app: AppHandle,
    config: State<'_, Settings>,
) -> Result<String, String> {
    match config.partition(token_event.partition.clone()) {
        None => Err(format!("No partition found for {}", token_event.partition)),
        Some(partition) => {
            log::info!(
                "Checking device token for partition: {:?}",
                partition.sso_start_url()
            );
            let mut sess = login::SessionState::new(app.clone(), partition.clone())
                .await
                .unwrap();
            match sess
                .next(login::Event::ConfirmDeviceAuthorization(token_event))
                .await
            {
                login::State::AwaitingConfirmation(c) => {
                    log::info!("check_device_token still awaiting confirmation: {:?}", c);
                    Ok(String::from("Pending"))
                }
                login::State::Ready => {
                    app.emit_to(
                        EventTarget::any(),
                        "token_ready",
                        events::AuthorizeDevice {
                            partition_name: partition.slug(),
                        },
                    )
                    .unwrap();
                    Ok(String::from("Done"))
                }
                _ => todo!("Handle other states"),
            }
        }
    }
}

#[tauri::command]
async fn authorize_device(
    auth_event: events::AuthorizeDevice,
    app: AppHandle,
    config: State<'_, Settings>,
) -> Result<login::DeviceAuthState, String> {
    match config.partition(auth_event.partition_name.clone()) {
        None => Err(format!(
            "No partition found for {}",
            auth_event.partition_name
        )),
        Some(partition) => {
            log::debug!("Found partition: {:?}", partition.sso_start_url());
            let mut sess = login::SessionState::new(app.clone(), partition.clone())
                .await
                .unwrap();
            let mut event: login::Event = login::Event::RegisterDevice;
            loop {
                let st = sess.next(event.clone()).await;
                log::info!(
                    "Stepped state event: {:?} and machine: {:?}",
                    event.clone(),
                    st
                );
                match st {
                    login::State::Ready => {
                        log::info!("Token is ready");
                        let token = app
                            .db(|db| sql::models::Token::find(db, partition.slug()).unwrap())
                            .unwrap();
                        return Ok(login::DeviceAuthState::Success(login::SuccessInfo {
                            expires_at: token.expires_at,
                        }));
                    }
                    login::State::Registered => {
                        log::info!("Device is registered");
                        event = login::Event::StartDeviceAuthorization;
                    }
                    login::State::AwaitingConfirmation(c) => {
                        log::info!("Needs confirmation: {:?}", c);
                        return Ok(login::DeviceAuthState::NeedsConfirmation(c));
                    }
                    login::State::Failed { message } => return Err(message),
                    _ => todo!("Handle other states"),
                }
            }
        }
    }
}

fn main() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init());
    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .targets([
                    LogTarget::new(LogTargetKind::Stdout),
                    LogTarget::new(LogTargetKind::LogDir {
                        file_name: Some("arsd".into()),
                    }),
                ])
                .build(),
        );
    }
    #[cfg(not(debug_assertions))]
    {
        builder = builder.plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepOne)
                .targets([LogTarget::new(LogTargetKind::LogDir {
                    file_name: Some("arsd".into()),
                })])
                .build(),
        );
    }
    builder
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            println!("{}, {argv:?}, {cwd}", app.package_info().name);
            app.emit_to(
                EventTarget::any(),
                "single-instance",
                SingletonPayload { args: argv, cwd },
            )
            .unwrap();
        }))
        .setup(|app| {
            if let Ok(config_path) = app.path().app_config_dir() {
                if !config_path.is_dir() {
                    // make the config directory if it doesn't exist
                    if let Err(e) = std::fs::create_dir_all(config_path) {
                        panic!("Error creating config directory: {}", e);
                    }
                }
            }
            let config =
                match get_configuration(app.path().app_config_dir().unwrap().join("config.yaml")) {
                    Ok(c) => c,
                    Err(e) => {
                        log::error!(
                            "Error loading configuration from {:?}",
                            app.path().app_config_dir().unwrap().join("config.yaml")
                        );
                        log::error!("Failed to load configuration: {:?}", e);
                        app.handle().exit(-1);
                        return Ok(());
                    }
                };
            app.manage(config.clone());

            app.listen_any("authorize_device", |event| {
                log::info!("Received `authorize_device` event: {:?}", event);
                match serde_json::from_str::<events::AuthorizeDevice>(event.payload()) {
                    Ok(auth) => {
                        log::info!("Kicking off `authorize_device` for {}", auth.partition_name);
                    }
                    Err(e) => {
                        log::error!("Error deserializing `authorize_device` event: {}", e);
                    }
                }
            });

            let main_window = app.get_webview_window("main").unwrap();

            let sql_state = sql::connect::SqlRepo {
                conn: std::sync::Mutex::new(None),
            };
            *sql_state.conn.lock().unwrap() = Some(
                sql::database::initialize_database(
                    app.path()
                        .app_data_dir()
                        .expect("data dir must exist for us to make the DB"),
                )
                .expect("Database should initialize"),
            );
            app.manage(sql_state);

            tauri::async_runtime::spawn(async move {
                main_window.show().unwrap();
                #[cfg(debug_assertions)] // for debug builds, open the devtools by default
                {
                    main_window.open_devtools();
                }

                for partition in config.partitions {
                    log::info!(
                        "Checking session for url={} region={}",
                        partition.start_url,
                        partition.region
                    );
                    main_window
                        .emit_to(
                            EventTarget::any(),
                            "authorize_device",
                            events::AuthorizeDevice {
                                partition_name: partition.slug(),
                            },
                        )
                        .unwrap();
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            authorize_device,
            check_device_token,
            delete_cache,
            get_credentials_for,
            get_partitions,
            list_accounts,
            list_roles_for,
            open_web_console,
            settings_get_sort,
            settings_save_sort,
            storage_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
