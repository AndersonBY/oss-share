use crate::config::{AppConfig, decrypt_secret, encrypt_secret, save_config};
use crate::ipc;
use crate::oss::{OssFileInfo, OssService};
use crate::upload_state::{UploadSnapshot, UploadTracker};
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use tauri::{AppHandle, State};

pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub uploads: UploadTracker,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EnqueueUploadsResult {
    pub accepted_files: Vec<String>,
    pub rejected_directories: Vec<String>,
}

/// Build an OssService from the current AppConfig.
fn create_oss_service(config: &AppConfig) -> Result<OssService, String> {
    let secret = decrypt_secret(&config.credentials.access_key_secret)?;
    OssService::new(
        &config.credentials.access_key_id,
        &secret,
        &config.oss.region,
        &config.oss.bucket,
        &config.oss.prefix,
    )
}

const MASKED_SECRET: &str = "••••••••";

#[tauri::command]
pub fn get_config(state: State<AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let mut masked = config.clone();
    if !masked.credentials.access_key_secret.is_empty() {
        masked.credentials.access_key_secret = MASKED_SECRET.to_string();
    }
    Ok(masked)
}

#[tauri::command]
pub fn save_settings(
    state: State<AppState>,
    access_key_id: String,
    access_key_secret: Option<String>,
    region: String,
    bucket: String,
    prefix: String,
    expire_seconds: u64,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;

    config.credentials.access_key_id = access_key_id;

    // Only update the secret if a new value is provided and it's not the mask
    if let Some(ref secret) = access_key_secret {
        if secret != MASKED_SECRET && !secret.is_empty() {
            config.credentials.access_key_secret = encrypt_secret(secret)?;
        }
    }

    config.oss.region = region;
    config.oss.bucket = bucket;
    config.oss.prefix = prefix;
    config.sharing.expire_seconds = expire_seconds;

    save_config(&config)?;

    Ok(())
}

#[tauri::command]
pub async fn test_connection(state: State<'_, AppState>) -> Result<(), String> {
    let config = {
        let guard = state.config.lock().map_err(|e| e.to_string())?;
        guard.clone()
    };

    let service = create_oss_service(&config)?;
    service.test_connection().await
}

#[tauri::command]
pub async fn list_files(state: State<'_, AppState>) -> Result<Vec<OssFileInfo>, String> {
    let config = {
        let guard = state.config.lock().map_err(|e| e.to_string())?;
        guard.clone()
    };

    let service = create_oss_service(&config)?;
    service.list_files().await
}

#[tauri::command]
pub async fn get_share_link(
    state: State<'_, AppState>,
    object_key: String,
) -> Result<String, String> {
    let config = {
        let guard = state.config.lock().map_err(|e| e.to_string())?;
        guard.clone()
    };

    let service = create_oss_service(&config)?;
    service.generate_presigned_url(&object_key, config.sharing.expire_seconds)
}

#[tauri::command]
pub async fn delete_file(
    state: State<'_, AppState>,
    object_key: String,
) -> Result<(), String> {
    let config = {
        let guard = state.config.lock().map_err(|e| e.to_string())?;
        guard.clone()
    };

    let service = create_oss_service(&config)?;
    service.delete_file(&object_key).await
}

#[tauri::command]
pub async fn upload_file(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<String, String> {
    let config = {
        let guard = state.config.lock().map_err(|e| e.to_string())?;
        guard.clone()
    };

    let service = create_oss_service(&config)?;
    let path = Path::new(&file_path);
    let object_key = service.upload_file(path).await?;
    let url = service.generate_presigned_url(&object_key, config.sharing.expire_seconds)?;
    Ok(url)
}

#[tauri::command]
pub fn get_uploads(state: State<AppState>) -> Result<Vec<UploadSnapshot>, String> {
    Ok(state.uploads.snapshot())
}

#[tauri::command]
pub fn clear_uploads(state: State<AppState>, ids: Vec<u64>) -> Result<(), String> {
    state.uploads.clear_finished(&ids);
    Ok(())
}

#[tauri::command]
pub fn enqueue_uploads(
    app_handle: AppHandle,
    files: Vec<String>,
) -> Result<EnqueueUploadsResult, String> {
    let result = split_upload_paths(files);

    if !result.accepted_files.is_empty() {
        ipc::enqueue_uploads(&app_handle, result.accepted_files.clone());
    }

    Ok(result)
}

fn split_upload_paths(paths: Vec<String>) -> EnqueueUploadsResult {
    let mut accepted_files = Vec::new();
    let mut rejected_directories = Vec::new();

    for path in paths {
        if let Ok(metadata) = fs::metadata(&path) {
            if metadata.is_file() {
                accepted_files.push(path);
            } else if metadata.is_dir() {
                rejected_directories.push(path);
            }
        }
    }

    EnqueueUploadsResult {
        accepted_files,
        rejected_directories,
    }
}

#[cfg(test)]
mod tests {
    use super::{EnqueueUploadsResult, split_upload_paths};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn split_upload_paths_separates_files_from_directories() {
        let test_root = unique_test_path("drag_drop_split");
        let file_path = test_root.join("sample.txt");
        let dir_path = test_root.join("folder");

        fs::create_dir_all(&test_root).unwrap();
        fs::write(&file_path, "hello").unwrap();
        fs::create_dir_all(&dir_path).unwrap();

        let result = split_upload_paths(vec![
            file_path.to_string_lossy().to_string(),
            dir_path.to_string_lossy().to_string(),
        ]);

        assert_eq!(
            result,
            EnqueueUploadsResult {
                accepted_files: vec![file_path.to_string_lossy().to_string()],
                rejected_directories: vec![dir_path.to_string_lossy().to_string()],
            }
        );

        let _ = fs::remove_dir_all(&test_root);
    }

    #[test]
    fn split_upload_paths_ignores_missing_paths() {
        let missing_path = unique_test_path("drag_drop_missing").join("missing.txt");

        let result = split_upload_paths(vec![missing_path.to_string_lossy().to_string()]);

        assert_eq!(
            result,
            EnqueueUploadsResult {
                accepted_files: Vec::new(),
                rejected_directories: Vec::new(),
            }
        );
    }

    fn unique_test_path(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("oss-share-{prefix}-{nanos}"))
    }
}
