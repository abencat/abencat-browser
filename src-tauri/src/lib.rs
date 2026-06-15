// Many store/profile/proxy helpers are only reachable through the GUI command
// layer; in the headless build that layer is compiled out, so allow dead code.
#![cfg_attr(not(feature = "gui"), allow(dead_code))]

//! Cloak Fingerprint Controller — backend.
//!
//! The core (store/launch/proxy/api/crypto/license/…) is Tauri-agnostic and
//! compiles without the GUI. The desktop app lives behind the `gui` feature;
//! the `cloak-headless` binary reuses the same core with `--no-default-features`
//! so it builds on a headless Ubuntu server without Tauri/webkit.

mod api;
#[cfg(feature = "gui")]
mod commands;
mod crypto;
mod fingerprint;
mod inject;
mod launch;
mod models;
mod profile;
mod proxy;
mod store;
mod util;

use launch::RuntimeState;
use std::sync::Arc;

#[cfg(feature = "gui")]
pub fn run() {
    let runtime = Arc::new(RuntimeState::default());
    let api_runtime = runtime.clone();
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .manage(runtime)
        .setup(move |_app| {
            // Best-effort: the desktop UI still works if the API port is busy.
            let _ = api::start(api_runtime.clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::save_settings,
            commands::create_project,
            commands::rename_project,
            commands::delete_project,
            commands::set_active_project,
            commands::create_profile,
            commands::batch_create_profiles,
            commands::save_profile,
            commands::clone_profile,
            commands::delete_profile,
            commands::batch_delete_profiles,
            commands::move_profile_to_project,
            commands::batch_move_profiles,
            commands::batch_launch,
            commands::batch_stop,
            commands::test_proxy,
            commands::extract_proxy_from_api,
            commands::export_profiles,
            commands::import_profiles,
            commands::reorder_profiles,
            commands::launch_profile,
            commands::launch_profile_at,
            commands::stop_profile,
            commands::build_launch_command,
            commands::open_profile_folder,
            commands::audit_fingerprint,
            commands::save_proxy,
            commands::delete_proxy,
            commands::import_proxies,
            commands::check_proxy_entry,
            commands::assign_proxy_to_profiles,
            commands::get_automation_info,
            commands::get_security_status,
            commands::unlock,
            commands::set_master_password,
            commands::remove_master_password
        ])
        .run(tauri::generate_context!())
        .expect("error while running Abencat Browser");
}

/// Headless server entry point. Starts the automation API (browsers launch with
/// `--headless=new`) and blocks. Driven entirely over HTTP.
pub fn headless_main() {
    let runtime = Arc::new(RuntimeState {
        headless: true,
        ..Default::default()
    });
    match api::start(runtime.clone()) {
        Some(port) => {
            let token = runtime.api_token.lock().unwrap().clone();
            let settings = store::load_settings();
            println!("Abencat Browser — headless automation API ready");
            println!("  endpoint : http://127.0.0.1:{port}");
            println!("  token    : {token}");
            println!("  browser  : {}", settings.browser_path);
            println!("  dataRoot : {}", settings.data_root);
            println!("Press Ctrl+C to stop.");
        }
        None => {
            eprintln!("error: failed to bind the automation API port");
            std::process::exit(1);
        }
    }
    loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
}
