// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{AppHandle, Manager, State};
use tauri_plugin_log::{LogTarget, RotationStrategy};

use arsd::configuration::{get_configuration, Settings};
use arsd::domain::{AccountInfo, RoleInfo, Storage};
use arsd::session::{account, account::Credentials, console_url, events, login};

#[derive(Clone, serde::Serialize)]
struct SingletonPayload {
    args: Vec<String>,
    cwd: String,
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn check_sso_status(settings: State<Settings>) -> String {
    log::info!("cfg={:?}", settings);
    format!("cfg={:?}", settings)
}

#[tauri::command]
fn config_path(settings: State<Settings>) -> String {
    settings.path.to_string_lossy().to_string()
}

#[tauri::command]
fn data_path(storage: State<Storage>) -> String {
    format!("{} -- {}", storage.client_name(), storage.path())
}

#[tauri::command]
async fn open_web_console(
    config: State<'_, Settings>,
    app: tauri::AppHandle,
    role_name: String,
    partition: String,
    account_id: String,
) -> Result<String, String> {
    match config.partition(partition) {
        None => todo!("Bad partition"),
        Some(part) => {
            let st = Storage::new(app);
            Ok(console_url::get_console_url(account_id, role_name, part, st).await)
        }
    }
}

#[tauri::command]
async fn list_accounts(
    config: State<'_, Settings>,
    app: tauri::AppHandle,
    partition: String,
) -> Result<Vec<AccountInfo>, String> {
    match config.partition(partition) {
        None => todo!("Bad partition"),
        Some(part) => {
            let st = Storage::new(app);
            return Ok(account::list_accounts(part, st)
                .await
                .iter()
                .map(|a| AccountInfo {
                    account_id: a.account_id.clone(),
                    account_name: a.account_name.clone(),
                    email_address: a.email_address.clone(),
                    alias: config.aliases.map_account(a.email_address.clone()),
                })
                .collect());
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
            let st = Storage::new(app);
            let roles = account::list_roles(part, st, account_id).await;

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
    Ok(account::get_credentials(
        config.partition(partition).unwrap(),
        role_name,
        account_id,
        Storage::new(app),
    )
    .await)
}

#[derive(serde::Deserialize, Clone)]
struct TokenCheckEvent {
    device_code: String,
    partition: String,
}

#[tauri::command]
async fn check_device_token(
    token_event: TokenCheckEvent,
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
            let sess = login::SSOSession::new(Storage::new(app), partition)
                .await
                .unwrap();
            match sess
                .confirm_device_registration(token_event.device_code.clone())
                .await
            {
                Err(e) => Ok(e),
                Ok(state) => {
                    log::info!("Device registration done. state: {:?}", state);
                    match state {
                        login::DeviceAuthState::NeedsConfirmation(c) => {
                            log::info!("Needs confirmation: {:?}", c);
                            Ok(serde_json::to_string(&c).unwrap())
                        }
                        login::DeviceAuthState::NeedsRefresh(_) => todo!(),
                        login::DeviceAuthState::Success(_) => Ok(String::from("Done")),
                        login::DeviceAuthState::Pending => Ok(String::from("Pending")),
                        login::DeviceAuthState::Failure => todo!(),
                    }
                }
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
            let sess = login::SSOSession::new(Storage::new(app), partition)
                .await
                .unwrap();
            match sess.authorize_device().await {
                Ok(r) => match r.clone() {
                    login::DeviceAuthState::Failure => Err("Auth step failed".to_string()),
                    login::DeviceAuthState::NeedsConfirmation(c) => {
                        log::info!("Needs confirmation: {:?}", c);
                        Ok(r)
                    }
                    login::DeviceAuthState::NeedsRefresh(_) => todo!(),
                    login::DeviceAuthState::Success(s) => {
                        log::info!("Token is ready {}", s.expires_at);
                        Ok(r)
                    }
                    login::DeviceAuthState::Pending => todo!(),
                },
                Err(e) => Err(format!(
                    "Error authorizing device for {}: {}",
                    auth_event.partition_name, e
                )),
            }
        }
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            println!("{}, {argv:?}, {cwd}", app.package_info().name);
            app.emit_all("single-instance", SingletonPayload { args: argv, cwd })
                .unwrap();
        }))
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .rotation_strategy(RotationStrategy::KeepOne)
                .targets([LogTarget::LogDir])
                .build(),
        )
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Warn)
                .rotation_strategy(RotationStrategy::KeepOne)
                .targets([LogTarget::Stdout])
                .build(),
        )
        .setup(|app| {
            app.manage(Storage::new(app.handle()));

            if let Some(config_path) = app.path_resolver().app_config_dir() {
                if !config_path.is_dir() {
                    // make the config directory if it doesn't exist
                    if let Err(e) = std::fs::create_dir_all(config_path) {
                        panic!("Error creating config directory: {}", e);
                    }
                }
            }
            let config;
            match get_configuration(
                app.path_resolver()
                    .app_config_dir()
                    .unwrap()
                    .join("config.yaml"),
            ) {
                Ok(c) => config = c,
                Err(e) => {
                    log::error!(
                        "Error loading configuration from {:?}",
                        app.path_resolver()
                            .app_config_dir()
                            .unwrap()
                            .join("config.yaml")
                    );
                    panic!("Error loading configuration: {}", e)
                }
            };

            app.manage(config.clone());

            app.listen_global("authorize_device", |event| {
                log::info!("Received `authorize_device` event: {:?}", event);
                match event.payload() {
                    None => log::error!("No payload for `authorize_device` event"),
                    Some(payload) => {
                        match serde_json::from_str::<events::AuthorizeDevice>(payload) {
                            Ok(auth) => {
                                log::info!(
                                    "Kicking off `authorize_device` for {}",
                                    auth.partition_name
                                );
                            }
                            Err(e) => {
                                log::error!("Error deserializing `authorize_device` event: {}", e);
                            }
                        }
                    }
                }
            });

            let main_window = app.get_window("main").unwrap();
            let handle = app.app_handle();
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
                    handle
                        .emit_all(
                            "authorize_device",
                            events::AuthorizeDevice {
                                partition_name: partition.slug(),
                            },
                        )
                        .unwrap();
                    //handle.trigger_global(
                    //    "authorize_device",
                    //    Some(
                    //        serde_json::to_string(&events::AuthorizeDevice {
                    //            partition_name: partition.slug(),
                    //        })
                    //        .unwrap(),
                    //    ),
                    //);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            authorize_device,
            check_device_token,
            check_sso_status,
            config_path,
            data_path,
            get_credentials_for,
            get_partitions,
            list_accounts,
            list_roles_for,
            open_web_console
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
