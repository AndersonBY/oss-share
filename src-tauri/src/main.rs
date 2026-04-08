#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod ipc;
mod oss;
mod tray;
mod upload_state;

use commands::AppState;
use upload_state::UploadTracker;
use std::sync::Mutex;
use windows::Win32::Foundation::*;
use windows::Win32::System::Threading::*;
use windows::core::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // If called with --upload, send file paths to the running instance via pipe and exit
    if args.len() >= 3 && args[1] == "--upload" {
        let files: Vec<String> = args[2..].to_vec();
        ipc::send_upload_via_pipe(&files);
        return;
    }

    // Single-instance check using a named mutex
    let mutex_name = HSTRING::from("OSSShare_SingleInstance_Mutex");
    let mutex = unsafe { CreateMutexW(None, true, &mutex_name) };
    match mutex {
        Ok(handle) => {
            if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
                // Another instance is running, exit silently
                let _ = unsafe { CloseHandle(handle) };
                return;
            }
            // Keep handle alive for the lifetime of the process (don't close it)
        }
        Err(_) => {
            // Failed to create mutex, continue anyway
        }
    }

    let app_config = config::load_config();

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState {
            config: Mutex::new(app_config),
            uploads: UploadTracker::default(),
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_settings,
            commands::test_connection,
            commands::list_files,
            commands::get_share_link,
            commands::delete_file,
            commands::upload_file,
            commands::get_uploads,
            commands::clear_uploads,
            commands::enqueue_uploads,
        ])
        .setup(|app| {
            tray::setup_tray(&app.handle())?;
            ipc::start_pipe_server(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
