// // #![allow(non_snake_case)]
// // #![recursion_limit = "512"]

// mod cmd;
// mod constants;

// use once_cell::sync::OnceCell;
// use tauri::{AppHandle, Manager as _};

// pub static APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();

// /// Application initialization helper functions
// mod app_init {
//     use super::*;

//     /// Setup plugins for the Tauri builder
//     pub fn setup_plugins(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
//         #[allow(unused_mut)]
//         let mut builder = builder
//             .plugin(tauri_plugin_opener::init())
//             .plugin(tauri_plugin_clipboard_manager::init())
//             .plugin(tauri_plugin_notification::init())
//             .plugin(tauri_plugin_updater::Builder::new().build())
//             .plugin(tauri_plugin_process::init())
//             .plugin(tauri_plugin_fs::init())
//             .plugin(tauri_plugin_dialog::init())
//             .plugin(tauri_plugin_shell::init());

//         builder
//     }

//     /// Generate command handlers
//     pub fn generate_handlers()
//     -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
//         tauri::generate_handler![
//             cmd::greet,
//             cmd::system::get_system_hostname,
//             cmd::system::get_network_interface,
//             cmd::system::get_system_info,
//         ]
//     }
// }

// #[cfg_attr(mobile, tauri::mobile_entry_point)]
// pub fn run() {
//     let builder = app_init::setup_plugins(tauri::Builder::default())
//         .setup(|app| {
//             #[allow(clippy::expect_used)]
//             APP_HANDLE
//                 .set(app.app_handle().clone())
//                 .expect("failed to set global app handle");

//             println!("Starting application initialization...");

//             Ok(())
//         })
//         .invoke_handler(app_init::generate_handlers());

//     let app = builder
//         .build(tauri::generate_context!())
//         .unwrap_or_else(|e| {
//             println!("Failed to build Tauri application: {}", e);
//             std::process::exit(1);
//         });

//     app.run(|_app_handle, e| match e {
//         tauri::RunEvent::Ready | tauri::RunEvent::Resumed => {
//             println!("Application ready");
//         }
//         tauri::RunEvent::Exit => {
//             println!("Application exiting");
//         }
//         _ => {}
//     });
// }
mod config;
