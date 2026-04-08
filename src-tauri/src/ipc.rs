use crate::commands::AppState;
use crate::config::decrypt_secret;
use crate::oss::OssService;
use crate::tray;
use serde::Deserialize;
use std::path::Path;
use tauri::{AppHandle, Manager};
use windows::core::HSTRING;
use windows::Win32::Foundation::*;
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::System::Pipes::*;

const PIPE_NAME: &str = r"\\.\pipe\oss-share";
const BUFFER_SIZE: u32 = 4096;

/// Called from main() when --upload is passed.
pub fn send_upload_via_pipe(files: &[String]) {
    let msg = serde_json::json!({
        "action": "upload",
        "files": files
    });
    let msg_bytes = msg.to_string();

    let pipe_name = HSTRING::from(PIPE_NAME);
    let handle = unsafe {
        CreateFileW(
            &pipe_name,
            GENERIC_WRITE.0,
            FILE_SHARE_NONE,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            HANDLE::default(),
        )
    };

    match handle {
        Ok(h) => {
            let mut written = 0u32;
            let _ = unsafe { WriteFile(h, Some(msg_bytes.as_bytes()), Some(&mut written), None) };
            let _ = unsafe { CloseHandle(h) };
        }
        Err(_) => {
            eprintln!("Failed to connect to OSS Share pipe");
        }
    }
}

#[derive(Debug, Deserialize)]
struct PipeMessage {
    action: String,
    files: Vec<String>,
}

pub fn start_pipe_server(app_handle: AppHandle) {
    std::thread::spawn(move || loop {
        match create_and_listen(&app_handle) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Pipe error: {e}");
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    });
}

fn create_and_listen(app_handle: &AppHandle) -> Result<(), String> {
    let pipe_name = HSTRING::from(PIPE_NAME);

    let pipe = unsafe {
        CreateNamedPipeW(
            &pipe_name,
            PIPE_ACCESS_INBOUND,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
            PIPE_UNLIMITED_INSTANCES,
            BUFFER_SIZE,
            BUFFER_SIZE,
            0,
            None,
        )
    };

    if pipe == INVALID_HANDLE_VALUE {
        return Err("CreateNamedPipeW failed".into());
    }

    let connect_result = unsafe { ConnectNamedPipe(pipe, None) };
    if let Err(e) = connect_result {
        let _ = unsafe { CloseHandle(pipe) };
        return Err(format!("ConnectNamedPipe failed: {e}"));
    }

    let mut all_data = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        let mut bytes_read: u32 = 0;
        let read_result =
            unsafe { ReadFile(pipe, Some(&mut buf), Some(&mut bytes_read), None) };
        if bytes_read > 0 {
            all_data.extend_from_slice(&buf[..bytes_read as usize]);
        }
        if read_result.is_err() || bytes_read == 0 {
            break;
        }
    }

    let _ = unsafe { DisconnectNamedPipe(pipe) };
    let _ = unsafe { CloseHandle(pipe) };

    if all_data.is_empty() {
        return Ok(());
    }

    let message: PipeMessage = serde_json::from_slice(&all_data)
        .map_err(|e| format!("Failed to parse pipe message: {e}"))?;

    if message.action == "upload" {
        handle_upload_request(app_handle, message.files);
    }

    Ok(())
}

fn handle_upload_request(app_handle: &AppHandle, files: Vec<String>) {
    enqueue_uploads(app_handle, files);
}

pub fn enqueue_uploads(app_handle: &AppHandle, files: Vec<String>) {
    if files.is_empty() {
        return;
    }

    let queued_uploads = {
        let state = app_handle.state::<AppState>();
        state.uploads.begin_batch(&files)
    };

    tray::show_window(app_handle, "files");

    let app = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        for queued_upload in queued_uploads {
            match upload_single_file(&app, &queued_upload.file_path).await {
                Ok(url) => {
                    app.state::<AppState>()
                        .uploads
                        .mark_done(queued_upload.id, url);
                }
                Err(e) => {
                    app.state::<AppState>()
                        .uploads
                        .mark_error(queued_upload.id, e);
                }
            }
        }
    });
}

async fn upload_single_file(app: &AppHandle, file_path: &str) -> Result<String, String> {
    let config = {
        let state = app.state::<AppState>();
        let guard = state.config.lock().map_err(|e| e.to_string())?;
        guard.clone()
    };

    let secret = decrypt_secret(&config.credentials.access_key_secret)?;
    let service = OssService::new(
        &config.credentials.access_key_id,
        &secret,
        &config.oss.region,
        &config.oss.bucket,
        &config.oss.prefix,
    )?;

    let path = Path::new(file_path);
    let object_key = service.upload_file(path).await?;
    let url = service.generate_presigned_url(&object_key, config.sharing.expire_seconds)?;

    // Copy last URL to clipboard
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        let _ = clipboard.set_text(&url);
    }

    Ok(url)
}
