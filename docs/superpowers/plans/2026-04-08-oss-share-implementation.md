# OSS Share Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Windows system tray app that uploads files to Alibaba Cloud OSS via right-click context menu and copies a presigned share link to clipboard.

**Architecture:** Tauri 2.x main process (system tray, OSS upload, config UI) + standalone COM DLL shell extension (IExplorerCommand for Win11 context menu). Communication via Named Pipe.

**Tech Stack:** Rust, Tauri 2.x, Vue 3 + TypeScript, windows-rs, aliyun-oss-client, NSIS installer

---

## File Structure

```
oss-share/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs           # App entry, tray setup, plugin registration
│   │   ├── tray.rs           # System tray icon, menu, event handling
│   │   ├── oss.rs            # OSS client: upload, list, delete, presign
│   │   ├── ipc.rs            # Named Pipe server (listens for shell extension)
│   │   ├── config.rs         # Config read/write with DPAPI encryption
│   │   └── commands.rs       # Tauri IPC commands (frontend ↔ backend)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   └── icons/
│       └── icon.ico
├── src/
│   ├── main.ts               # Vue app entry
│   ├── App.vue               # Root component with tab navigation
│   ├── views/
│   │   ├── Settings.vue      # Settings page
│   │   └── Files.vue         # File browser page
│   └── components/
│       └── FileRow.vue       # Single file row in file list
├── shell-extension/
│   ├── src/
│   │   ├── lib.rs            # DLL entry, COM registration exports
│   │   ├── command.rs        # IExplorerCommand implementation
│   │   └── pipe_client.rs    # Named Pipe client (send file paths)
│   └── Cargo.toml
├── installer/
│   └── nsis/
│       └── installer.nsi     # NSIS install/uninstall script
├── package.json
├── tsconfig.json
└── vite.config.ts
```

---

## Task 1: Project Scaffolding

**Files:**
- Create: `package.json`, `tsconfig.json`, `vite.config.ts`
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/build.rs`
- Create: `src-tauri/src/main.rs`
- Create: `src/main.ts`, `src/App.vue`

- [ ] **Step 1: Initialize Tauri project**

```bash
npm create tauri-app@latest oss-share -- --template vue-ts
cd oss-share
```

- [ ] **Step 2: Update `package.json` dependencies**

```json
{
  "name": "oss-share",
  "private": true,
  "version": "0.1.0",
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "tauri": "tauri"
  },
  "dependencies": {
    "vue": "^3.5.0",
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-clipboard-manager": "^2.0.0",
    "@tauri-apps/plugin-notification": "^2.0.0"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^5.0.0",
    "typescript": "^5.5.0",
    "vite": "^6.0.0",
    "vue-tsc": "^2.0.0"
  }
}
```

- [ ] **Step 3: Configure `src-tauri/Cargo.toml`**

```toml
[package]
name = "oss-share"
version = "0.1.0"
edition = "2021"

[lib]
name = "oss_share_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-notification = "2"
tauri-plugin-clipboard-manager = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
tokio = { version = "1", features = ["full"] }
aliyun-oss-client = "0.13"
windows = { version = "0.58", features = [
    "Win32_Security_Cryptography",
    "Win32_System_Pipes",
    "Win32_System_Memory",
    "Win32_Storage_FileSystem",
    "Win32_Foundation"
] }
base64 = "0.22"
dirs = "5"
```

- [ ] **Step 3b: Create `src-tauri/build.rs`**

```rust
fn main() {
    tauri_build::build()
}
```

- [ ] **Step 4: Configure `src-tauri/tauri.conf.json`**

```json
{
  "$schema": "https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri-cli/schema.json",
  "productName": "OSS Share",
  "version": "0.1.0",
  "identifier": "com.oss-share.app",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "windows": [
      {
        "title": "OSS Share",
        "width": 680,
        "height": 500,
        "visible": false,
        "resizable": true
      }
    ],
    "trayIcon": {
      "iconPath": "icons/icon.ico",
      "iconAsTemplate": false
    },
    "security": {
      "csp": null
    }
  },
  "plugins": {
    "notification": {
      "all": true
    },
    "clipboard-manager": {
      "all": true
    }
  }
}
```

- [ ] **Step 5: Create minimal `src-tauri/src/main.rs`**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 6: Create minimal `src/main.ts` and `src/App.vue`**

`src/main.ts`:
```typescript
import { createApp } from "vue";
import App from "./App.vue";

createApp(App).mount("#app");
```

`src/App.vue`:
```vue
<template>
  <div class="app">
    <h1>OSS Share</h1>
    <p>配置和文件管理界面即将到来</p>
  </div>
</template>

<script setup lang="ts">
</script>
```

- [ ] **Step 7: Verify project builds**

```bash
npm install
npm run tauri build -- --debug
```

Expected: Build succeeds, produces `oss-share.exe`.

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat: scaffold Tauri 2.x project with Vue 3 frontend"
```

---

## Task 2: Config Module

**Files:**
- Create: `src-tauri/src/config.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Create `src-tauri/src/config.rs`**

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use windows::Win32::Security::Cryptography::*;
use windows::core::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub credentials: Credentials,
    pub oss: OssConfig,
    pub sharing: SharingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub access_key_id: String,
    pub access_key_secret: String, // DPAPI encrypted, base64 encoded
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OssConfig {
    pub region: String,
    pub bucket: String,
    pub prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingConfig {
    pub expire_seconds: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            credentials: Credentials {
                access_key_id: String::new(),
                access_key_secret: String::new(),
            },
            oss: OssConfig {
                region: "oss-cn-hangzhou".into(),
                bucket: String::new(),
                prefix: "share/".into(),
            },
            sharing: SharingConfig {
                expire_seconds: 604800,
            },
        }
    }
}

pub fn config_dir() -> PathBuf {
    let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("oss-share");
    dir
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        toml::from_str(&content).unwrap_or_default()
    } else {
        AppConfig::default()
    }
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let dir = config_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let content = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(config_path(), content).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn encrypt_secret(plain: &str) -> Result<String, String> {
    use base64::Engine;
    let plain_bytes = plain.as_bytes();
    let mut blob = CRYPTOAPI_BLOB {
        cbData: plain_bytes.len() as u32,
        pbData: plain_bytes.as_ptr() as *mut u8,
    };
    let mut encrypted = CRYPTOAPI_BLOB::default();
    unsafe {
        CryptProtectData(
            &mut blob,
            None,
            None,
            None,
            None,
            0,
            &mut encrypted,
        ).map_err(|e| format!("DPAPI encrypt failed: {e}"))?;
        let slice = std::slice::from_raw_parts(
            encrypted.pbData,
            encrypted.cbData as usize,
        );
        let encoded = base64::engine::general_purpose::STANDARD.encode(slice);
        windows::Win32::System::Memory::LocalFree(
            windows::Win32::Foundation::HLOCAL(encrypted.pbData as *mut _)
        );
        Ok(encoded)
    }
}

pub fn decrypt_secret(encrypted_b64: &str) -> Result<String, String> {
    use base64::Engine;
    let encrypted_bytes = base64::engine::general_purpose::STANDARD
        .decode(encrypted_b64)
        .map_err(|e| format!("base64 decode failed: {e}"))?;
    let mut blob = CRYPTOAPI_BLOB {
        cbData: encrypted_bytes.len() as u32,
        pbData: encrypted_bytes.as_ptr() as *mut u8,
    };
    let mut decrypted = CRYPTOAPI_BLOB::default();
    unsafe {
        CryptUnprotectData(
            &mut blob,
            None,
            None,
            None,
            None,
            0,
            &mut decrypted,
        ).map_err(|e| format!("DPAPI decrypt failed: {e}"))?;
        let slice = std::slice::from_raw_parts(
            decrypted.pbData,
            decrypted.cbData as usize,
        );
        let plain = String::from_utf8(slice.to_vec())
            .map_err(|e| format!("UTF-8 decode failed: {e}"))?;
        windows::Win32::System::Memory::LocalFree(
            windows::Win32::Foundation::HLOCAL(decrypted.pbData as *mut _)
        );
        Ok(plain)
    }
}
```

- [ ] **Step 2: Register module in `main.rs`**

Add at top of `src-tauri/src/main.rs`:
```rust
mod config;
```

- [ ] **Step 4: Verify it compiles**

```bash
cd src-tauri && cargo check
```

Expected: No errors.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add config module with DPAPI encryption"
```

---

## Task 3: OSS Module

**Files:**
- Create: `src-tauri/src/oss.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Create `src-tauri/src/oss.rs`**

```rust
use aliyun_oss_client::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OssFileInfo {
    pub key: String,
    pub size: u64,
    pub last_modified: String,
}

pub struct OssService {
    client: Client,
    bucket: String,
    prefix: String,
    endpoint: String,
}

impl OssService {
    pub fn new(
        access_key_id: &str,
        access_key_secret: &str,
        region: &str,
        bucket: &str,
        prefix: &str,
    ) -> Result<Self, String> {
        let endpoint = format!("https://{}.aliyuncs.com", region);
        let client = Client::new(
            access_key_id.to_string(),
            access_key_secret.to_string(),
            endpoint.clone(),
            bucket.to_string(),
        ).map_err(|e| format!("Failed to create OSS client: {e}"))?;

        Ok(Self {
            client,
            bucket: bucket.to_string(),
            prefix: prefix.to_string(),
            endpoint,
        })
    }

    pub async fn upload_file(&self, file_path: &Path) -> Result<String, String> {
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("Invalid file name")?;
        let object_key = format!("{}{}", self.prefix, file_name);

        let content = tokio::fs::read(file_path)
            .await
            .map_err(|e| format!("Failed to read file: {e}"))?;

        self.client
            .put_object(&object_key, content)
            .await
            .map_err(|e| format!("Upload failed: {e}"))?;

        Ok(object_key)
    }

    pub fn generate_presigned_url(
        &self,
        object_key: &str,
        expire_seconds: u64,
    ) -> Result<String, String> {
        self.client
            .presign_url(object_key, expire_seconds)
            .map_err(|e| format!("Failed to generate URL: {e}"))
    }

    pub async fn list_files(&self) -> Result<Vec<OssFileInfo>, String> {
        let objects = self.client
            .list_objects(&self.prefix)
            .await
            .map_err(|e| format!("Failed to list objects: {e}"))?;

        let files = objects
            .into_iter()
            .map(|obj| OssFileInfo {
                key: obj.key,
                size: obj.size,
                last_modified: obj.last_modified,
            })
            .collect();

        Ok(files)
    }

    pub async fn delete_file(&self, object_key: &str) -> Result<(), String> {
        self.client
            .delete_object(object_key)
            .await
            .map_err(|e| format!("Failed to delete: {e}"))
    }

    pub async fn test_connection(&self) -> Result<(), String> {
        self.client
            .list_objects(&self.prefix)
            .await
            .map(|_| ())
            .map_err(|e| format!("Connection failed: {e}"))
    }
}
```

> **Note:** The exact `aliyun-oss-client` API may differ from what's shown above. During implementation, check the crate's docs and adjust method names accordingly. The key operations needed are: `put_object`, `list_objects`, `delete_object`, and `presign_url` (or equivalent). If `presign_url` is not available in the crate, implement it manually using HMAC-SHA1 signing per OSS REST API spec.

- [ ] **Step 2: Register module in `main.rs`**

Add to `src-tauri/src/main.rs`:
```rust
mod oss;
```

- [ ] **Step 3: Verify it compiles**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add OSS service module (upload, list, delete, presign)"
```

---

## Task 4: System Tray Module

**Files:**
- Create: `src-tauri/src/tray.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Create `src-tauri/src/tray.rs`**

```rust
use tauri::{
    AppHandle, Manager,
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::TrayIconBuilder,
};

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let open_files = MenuItemBuilder::with_id("open_files", "打开文件管理").build(app)?;
    let settings = MenuItemBuilder::with_id("settings", "设置").build(app)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&open_files)
        .item(&settings)
        .item(&separator)
        .item(&quit)
        .build()?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("OSS Share")
        .menu(&menu)
        .on_menu_event(move |app, event| {
            match event.id().as_ref() {
                "open_files" => {
                    show_window(app, "files");
                }
                "settings" => {
                    show_window(app, "settings");
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}

fn show_window(app: &AppHandle, tab: &str) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.eval(&format!("window.__navigateTo && window.__navigateTo('{tab}')"));
    }
}
```

- [ ] **Step 2: Update `main.rs` to use tray**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod oss;
mod tray;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            tray::setup_tray(&app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Verify it compiles**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add system tray with menu (files, settings, quit)"
```

---

## Task 5: Tauri IPC Commands (Frontend ↔ Backend)

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Create `src-tauri/src/commands.rs`**

```rust
use crate::config::{self, AppConfig, decrypt_secret, encrypt_secret, load_config, save_config};
use crate::oss::{OssFileInfo, OssService};
use tauri::State;
use std::sync::Mutex;

pub struct AppState {
    pub config: Mutex<AppConfig>,
}

#[tauri::command]
pub fn get_config(state: State<AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    // Return config with masked secret
    let mut c = config.clone();
    if !c.credentials.access_key_secret.is_empty() {
        c.credentials.access_key_secret = "••••••••".to_string();
    }
    Ok(c)
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
    if let Some(secret) = access_key_secret {
        if secret != "••••••••" {
            config.credentials.access_key_secret = encrypt_secret(&secret)?;
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
    let config = state.config.lock().map_err(|e| e.to_string())?.clone();
    let secret = decrypt_secret(&config.credentials.access_key_secret)?;
    let svc = OssService::new(
        &config.credentials.access_key_id,
        &secret,
        &config.oss.region,
        &config.oss.bucket,
        &config.oss.prefix,
    )?;
    svc.test_connection().await
}

#[tauri::command]
pub async fn list_files(state: State<'_, AppState>) -> Result<Vec<OssFileInfo>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?.clone();
    let secret = decrypt_secret(&config.credentials.access_key_secret)?;
    let svc = OssService::new(
        &config.credentials.access_key_id,
        &secret,
        &config.oss.region,
        &config.oss.bucket,
        &config.oss.prefix,
    )?;
    svc.list_files().await
}

#[tauri::command]
pub async fn get_share_link(
    state: State<'_, AppState>,
    object_key: String,
) -> Result<String, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?.clone();
    let secret = decrypt_secret(&config.credentials.access_key_secret)?;
    let svc = OssService::new(
        &config.credentials.access_key_id,
        &secret,
        &config.oss.region,
        &config.oss.bucket,
        &config.oss.prefix,
    )?;
    svc.generate_presigned_url(&object_key, config.sharing.expire_seconds)
}

#[tauri::command]
pub async fn delete_file(
    state: State<'_, AppState>,
    object_key: String,
) -> Result<(), String> {
    let config = state.config.lock().map_err(|e| e.to_string())?.clone();
    let secret = decrypt_secret(&config.credentials.access_key_secret)?;
    let svc = OssService::new(
        &config.credentials.access_key_id,
        &secret,
        &config.oss.region,
        &config.oss.bucket,
        &config.oss.prefix,
    )?;
    svc.delete_file(&object_key).await
}

#[tauri::command]
pub async fn upload_file(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<String, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?.clone();
    let secret = decrypt_secret(&config.credentials.access_key_secret)?;
    let svc = OssService::new(
        &config.credentials.access_key_id,
        &secret,
        &config.oss.region,
        &config.oss.bucket,
        &config.oss.prefix,
    )?;
    let path = std::path::Path::new(&file_path);
    let object_key = svc.upload_file(path).await?;
    let url = svc.generate_presigned_url(&object_key, config.sharing.expire_seconds)?;
    Ok(url)
}
```

- [ ] **Step 2: Update `main.rs` to register commands and state**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod oss;
mod tray;

use commands::AppState;
use std::sync::Mutex;

fn main() {
    let app_config = config::load_config();

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState {
            config: Mutex::new(app_config),
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_settings,
            commands::test_connection,
            commands::list_files,
            commands::get_share_link,
            commands::delete_file,
            commands::upload_file,
        ])
        .setup(|app| {
            tray::setup_tray(&app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Verify it compiles**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add Tauri IPC commands (config, upload, list, delete, share link)"
```

---

## Task 6: Named Pipe IPC Server

**Files:**
- Create: `src-tauri/src/ipc.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Create `src-tauri/src/ipc.rs`**

```rust
use serde::Deserialize;
use std::io::Read;
use tauri::AppHandle;
use tauri::Manager;
use windows::Win32::Foundation::*;
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::System::Pipes::*;
use windows::core::*;

const PIPE_NAME: &str = r"\\.\pipe\oss-share";

#[derive(Debug, Deserialize)]
struct PipeMessage {
    action: String,
    files: Vec<String>,
}

pub fn start_pipe_server(app_handle: AppHandle) {
    std::thread::spawn(move || {
        loop {
            match create_and_listen(&app_handle) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Pipe error: {e}");
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }
        }
    });
}

fn create_and_listen(app_handle: &AppHandle) -> Result<()> {
    let pipe_name = HSTRING::from(PIPE_NAME);
    let pipe = unsafe {
        CreateNamedPipeW(
            &pipe_name,
            PIPE_ACCESS_INBOUND,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
            1,
            4096,
            4096,
            0,
            None,
        )?
    };

    // Wait for client connection
    unsafe { ConnectNamedPipe(pipe, None)? };

    // Read message
    let mut buffer = vec![0u8; 65536];
    let mut total_read = 0usize;
    loop {
        let mut bytes_read = 0u32;
        let ok = unsafe {
            ReadFile(
                pipe,
                Some(&mut buffer[total_read..]),
                Some(&mut bytes_read),
                None,
            )
        };
        total_read += bytes_read as usize;
        if ok.is_err() || bytes_read == 0 {
            break;
        }
    }

    unsafe {
        DisconnectNamedPipe(pipe)?;
        CloseHandle(pipe)?;
    };

    if total_read > 0 {
        let msg_str = String::from_utf8_lossy(&buffer[..total_read]);
        if let Ok(msg) = serde_json::from_str::<PipeMessage>(msg_str.trim()) {
            if msg.action == "upload" {
                handle_upload_request(app_handle, msg.files);
            }
        }
    }

    Ok(())
}

fn handle_upload_request(app_handle: &AppHandle, files: Vec<String>) {
    let handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        for file_path in files {
            let state = handle.state::<crate::commands::AppState>();
            let config = match state.config.lock() {
                Ok(c) => c.clone(),
                Err(_) => continue,
            };
            let secret = match crate::config::decrypt_secret(&config.credentials.access_key_secret) {
                Ok(s) => s,
                Err(e) => {
                    notify_error(&handle, &format!("解密失败: {e}"));
                    continue;
                }
            };
            let svc = match crate::oss::OssService::new(
                &config.credentials.access_key_id,
                &secret,
                &config.oss.region,
                &config.oss.bucket,
                &config.oss.prefix,
            ) {
                Ok(s) => s,
                Err(e) => {
                    notify_error(&handle, &format!("OSS 连接失败: {e}"));
                    continue;
                }
            };

            let path = std::path::Path::new(&file_path);
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            match svc.upload_file(path).await {
                Ok(object_key) => {
                    match svc.generate_presigned_url(&object_key, config.sharing.expire_seconds) {
                        Ok(url) => {
                            // Copy to clipboard
                            let _ = handle.emit("copy-to-clipboard", &url);
                            notify_success(&handle, file_name);
                        }
                        Err(e) => notify_error(&handle, &format!("生成链接失败: {e}")),
                    }
                }
                Err(e) => notify_error(&handle, &format!("{file_name} 上传失败: {e}")),
            }
        }
    });
}

fn notify_success(app: &AppHandle, file_name: &str) {
    use tauri_plugin_notification::NotificationExt;
    let _ = app.notification()
        .builder()
        .title("OSS Share")
        .body(format!("{file_name} — 链接已复制到剪贴板"))
        .show();
}

fn notify_error(app: &AppHandle, msg: &str) {
    use tauri_plugin_notification::NotificationExt;
    let _ = app.notification()
        .builder()
        .title("OSS Share - 错误")
        .body(msg)
        .show();
}
```

- [ ] **Step 2: Update `main.rs` to start pipe server**

Add `mod ipc;` and in `setup` closure, after tray setup:

```rust
.setup(|app| {
    tray::setup_tray(&app.handle())?;
    ipc::start_pipe_server(app.handle().clone());
    Ok(())
})
```

- [ ] **Step 3: Add clipboard copy event listener in frontend**

In `src/App.vue`, add to `<script setup>`:
```typescript
import { listen } from "@tauri-apps/api/event";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

listen<string>("copy-to-clipboard", async (event) => {
  await writeText(event.payload);
});
```

- [ ] **Step 4: Verify it compiles**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add Named Pipe IPC server for shell extension communication"
```

---

## Task 7: Frontend — Settings Page

**Files:**
- Create: `src/views/Settings.vue`
- Modify: `src/App.vue`

- [ ] **Step 1: Create `src/views/Settings.vue`**

```vue
<template>
  <div class="settings">
    <div class="form-group">
      <label>Access Key ID</label>
      <input v-model="form.accessKeyId" type="text" placeholder="LTAI5t..." />
    </div>
    <div class="form-group">
      <label>Access Key Secret</label>
      <input v-model="form.accessKeySecret" type="password" placeholder="输入 Access Key Secret" />
    </div>
    <div class="form-group">
      <label>Region</label>
      <input v-model="form.region" type="text" placeholder="oss-cn-hangzhou" />
    </div>
    <div class="form-group">
      <label>Bucket</label>
      <input v-model="form.bucket" type="text" placeholder="my-share-bucket" />
    </div>
    <div class="form-group">
      <label>上传路径前缀</label>
      <input v-model="form.prefix" type="text" placeholder="share/" />
    </div>
    <div class="form-group">
      <label>链接有效期</label>
      <div class="expire-options">
        <button
          v-for="opt in expireOptions"
          :key="opt.value"
          :class="{ active: form.expireSeconds === opt.value }"
          @click="form.expireSeconds = opt.value"
        >
          {{ opt.label }}
        </button>
      </div>
    </div>
    <div class="actions">
      <button class="btn-secondary" @click="testConnection" :disabled="testing">
        {{ testing ? '测试中...' : '测试连接' }}
      </button>
      <button class="btn-primary" @click="save" :disabled="saving">
        {{ saving ? '保存中...' : '保存' }}
      </button>
    </div>
    <p v-if="message" :class="['message', messageType]">{{ message }}</p>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const form = ref({
  accessKeyId: "",
  accessKeySecret: "",
  region: "oss-cn-hangzhou",
  bucket: "",
  prefix: "share/",
  expireSeconds: 604800,
});

const expireOptions = [
  { label: "24h", value: 86400 },
  { label: "7d", value: 604800 },
  { label: "30d", value: 2592000 },
];

const testing = ref(false);
const saving = ref(false);
const message = ref("");
const messageType = ref<"success" | "error">("success");

onMounted(async () => {
  try {
    const config: any = await invoke("get_config");
    form.value.accessKeyId = config.credentials.access_key_id;
    form.value.accessKeySecret = config.credentials.access_key_secret;
    form.value.region = config.oss.region;
    form.value.bucket = config.oss.bucket;
    form.value.prefix = config.oss.prefix;
    form.value.expireSeconds = config.sharing.expire_seconds;
  } catch (e) {
    message.value = `加载配置失败: ${e}`;
    messageType.value = "error";
  }
});

async function save() {
  saving.value = true;
  message.value = "";
  try {
    await invoke("save_settings", {
      accessKeyId: form.value.accessKeyId,
      accessKeySecret: form.value.accessKeySecret,
      region: form.value.region,
      bucket: form.value.bucket,
      prefix: form.value.prefix,
      expireSeconds: form.value.expireSeconds,
    });
    message.value = "保存成功";
    messageType.value = "success";
  } catch (e) {
    message.value = `保存失败: ${e}`;
    messageType.value = "error";
  } finally {
    saving.value = false;
  }
}

async function testConnection() {
  testing.value = true;
  message.value = "";
  try {
    await invoke("test_connection");
    message.value = "连接成功";
    messageType.value = "success";
  } catch (e) {
    message.value = `连接失败: ${e}`;
    messageType.value = "error";
  } finally {
    testing.value = false;
  }
}
</script>

<style scoped>
.settings {
  padding: 20px;
}
.form-group {
  display: grid;
  grid-template-columns: 120px 1fr;
  align-items: center;
  gap: 12px;
  margin-bottom: 14px;
}
.form-group label {
  text-align: right;
  color: #aaa;
  font-size: 13px;
}
.form-group input {
  background: #1a1a2e;
  border: 1px solid #444;
  border-radius: 4px;
  padding: 6px 10px;
  color: #e0e0e0;
  font-size: 13px;
}
.expire-options {
  display: flex;
  gap: 8px;
}
.expire-options button {
  padding: 6px 14px;
  background: #1a1a2e;
  border: 1px solid #444;
  border-radius: 4px;
  color: #888;
  cursor: pointer;
}
.expire-options button.active {
  background: #3a3a5c;
  border-color: #6c6cff;
  color: #fff;
}
.actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 20px;
}
.btn-primary {
  padding: 6px 20px;
  background: #4c8bf5;
  border: none;
  border-radius: 4px;
  color: #fff;
  cursor: pointer;
}
.btn-secondary {
  padding: 6px 20px;
  background: #3a3a5c;
  border: none;
  border-radius: 4px;
  color: #fff;
  cursor: pointer;
}
.message {
  margin-top: 12px;
  font-size: 13px;
}
.message.success { color: #4caf50; }
.message.error { color: #ff6b6b; }
</style>
```

- [ ] **Step 2: Update `src/App.vue` with tab navigation**

```vue
<template>
  <div class="app">
    <div class="tabs">
      <button
        :class="{ active: currentTab === 'settings' }"
        @click="currentTab = 'settings'"
      >
        设置
      </button>
      <button
        :class="{ active: currentTab === 'files' }"
        @click="currentTab = 'files'"
      >
        文件管理
      </button>
    </div>
    <Settings v-if="currentTab === 'settings'" />
    <div v-else>
      <p style="padding: 20px; color: #888;">文件管理页面即将到来</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import Settings from "./views/Settings.vue";

const currentTab = ref("settings");

// Handle clipboard copy from backend
listen<string>("copy-to-clipboard", async (event) => {
  await writeText(event.payload);
});

// Handle navigation from tray menu
(window as any).__navigateTo = (tab: string) => {
  currentTab.value = tab;
};
</script>

<style>
body {
  margin: 0;
  background: #0f0f1a;
  color: #e0e0e0;
  font-family: 'Segoe UI', sans-serif;
}
.app {
  min-height: 100vh;
}
.tabs {
  display: flex;
  border-bottom: 2px solid #333;
  padding: 0;
}
.tabs button {
  padding: 10px 24px;
  background: transparent;
  border: none;
  color: #888;
  cursor: pointer;
  font-size: 14px;
}
.tabs button.active {
  color: #fff;
  background: #3a3a5c;
  border-radius: 6px 6px 0 0;
}
</style>
```

- [ ] **Step 3: Verify frontend builds**

```bash
npm run build
```

Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add Settings page with config form and tab navigation"
```

---

## Task 8: Frontend — Files Page

**Files:**
- Create: `src/components/FileRow.vue`
- Create: `src/views/Files.vue`
- Modify: `src/App.vue`

- [ ] **Step 1: Create `src/components/FileRow.vue`**

```vue
<template>
  <div class="file-row">
    <span class="file-name">{{ file.key.split('/').pop() }}</span>
    <span class="file-size">{{ formatSize(file.size) }}</span>
    <span class="file-date">{{ file.last_modified }}</span>
    <span class="file-actions">
      <button @click="$emit('copy-link', file.key)" title="复制链接">🔗</button>
      <button @click="$emit('delete', file.key)" title="删除">🗑</button>
    </span>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  file: { key: string; size: number; last_modified: string };
}>();

defineEmits<{
  (e: "copy-link", key: string): void;
  (e: "delete", key: string): void;
}>();

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}
</script>

<style scoped>
.file-row {
  display: grid;
  grid-template-columns: 1fr 100px 160px 80px;
  padding: 10px 12px;
  border-bottom: 1px solid #2a2a2a;
  align-items: center;
}
.file-name { color: #e0e0e0; font-size: 13px; overflow: hidden; text-overflow: ellipsis; }
.file-size, .file-date { color: #aaa; font-size: 12px; }
.file-actions { display: flex; gap: 6px; }
.file-actions button {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 14px;
  padding: 2px 4px;
}
</style>
```

- [ ] **Step 2: Create `src/views/Files.vue`**

```vue
<template>
  <div class="files">
    <div class="toolbar">
      <span class="file-count">{{ filteredFiles.length }} 个文件</span>
      <input
        v-model="search"
        type="text"
        placeholder="搜索文件名..."
        class="search-input"
      />
    </div>
    <div class="file-list">
      <div class="file-header">
        <span>文件名</span>
        <span>大小</span>
        <span>上传时间</span>
        <span>操作</span>
      </div>
      <div v-if="loading" class="loading">加载中...</div>
      <div v-else-if="filteredFiles.length === 0" class="empty">暂无文件</div>
      <FileRow
        v-for="file in filteredFiles"
        :key="file.key"
        :file="file"
        @copy-link="copyLink"
        @delete="deleteFile"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import FileRow from "../components/FileRow.vue";

interface OssFile {
  key: string;
  size: number;
  last_modified: string;
}

const files = ref<OssFile[]>([]);
const search = ref("");
const loading = ref(true);

const filteredFiles = computed(() => {
  if (!search.value) return files.value;
  const q = search.value.toLowerCase();
  return files.value.filter((f) =>
    f.key.toLowerCase().includes(q)
  );
});

onMounted(loadFiles);

async function loadFiles() {
  loading.value = true;
  try {
    files.value = await invoke("list_files");
  } catch (e) {
    console.error("Failed to list files:", e);
  } finally {
    loading.value = false;
  }
}

async function copyLink(key: string) {
  try {
    const url: string = await invoke("get_share_link", { objectKey: key });
    await writeText(url);
  } catch (e) {
    console.error("Failed to get share link:", e);
  }
}

async function deleteFile(key: string) {
  try {
    await invoke("delete_file", { objectKey: key });
    files.value = files.value.filter((f) => f.key !== key);
  } catch (e) {
    console.error("Failed to delete file:", e);
  }
}
</script>

<style scoped>
.files { padding: 20px; }
.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}
.file-count { color: #aaa; font-size: 13px; }
.search-input {
  background: #1a1a2e;
  border: 1px solid #444;
  border-radius: 4px;
  padding: 4px 10px;
  color: #ccc;
  font-size: 12px;
  width: 180px;
}
.file-list {
  border: 1px solid #333;
  border-radius: 6px;
  overflow: hidden;
}
.file-header {
  display: grid;
  grid-template-columns: 1fr 100px 160px 80px;
  padding: 8px 12px;
  background: #1a1a2e;
  color: #888;
  font-size: 12px;
  border-bottom: 1px solid #333;
}
.loading, .empty {
  padding: 40px;
  text-align: center;
  color: #666;
}
</style>
```

- [ ] **Step 3: Update `src/App.vue` to import Files**

Replace the `v-else` placeholder block:

```vue
<template>
  <div class="app">
    <div class="tabs">
      <button
        :class="{ active: currentTab === 'settings' }"
        @click="currentTab = 'settings'"
      >
        设置
      </button>
      <button
        :class="{ active: currentTab === 'files' }"
        @click="currentTab = 'files'"
      >
        文件管理
      </button>
    </div>
    <Settings v-if="currentTab === 'settings'" />
    <Files v-else />
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import Settings from "./views/Settings.vue";
import Files from "./views/Files.vue";

const currentTab = ref("settings");

listen<string>("copy-to-clipboard", async (event) => {
  await writeText(event.payload);
});

(window as any).__navigateTo = (tab: string) => {
  currentTab.value = tab;
};
</script>
```

- [ ] **Step 4: Verify frontend builds**

```bash
npm run build
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add Files page with list, search, copy link, delete"
```

---

## Task 9: Shell Extension — COM DLL Project Setup

**Files:**
- Create: `shell-extension/Cargo.toml`
- Create: `shell-extension/src/lib.rs`
- Create: `shell-extension/src/pipe_client.rs`
- Create: `shell-extension/src/command.rs`

- [ ] **Step 1: Create `shell-extension/Cargo.toml`**

```toml
[package]
name = "oss-share-shell"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
windows = { version = "0.58", features = [
    "Win32_UI_Shell",
    "Win32_System_Com",
    "Win32_System_Registry",
    "Win32_System_Pipes",
    "Win32_Storage_FileSystem",
    "Win32_Foundation",
    "Win32_System_LibraryLoader",
    "implement"
] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

- [ ] **Step 2: Create `shell-extension/src/pipe_client.rs`**

```rust
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::Win32::Foundation::*;
use windows::Win32::Storage::FileSystem::*;
use windows::core::*;

const PIPE_NAME: &str = r"\\.\pipe\oss-share";

pub fn send_upload_request(files: &[String]) -> Result<()> {
    let msg = serde_json::json!({
        "action": "upload",
        "files": files
    });
    let msg_bytes = msg.to_string();
    let msg_bytes = msg_bytes.as_bytes();

    let pipe_name: Vec<u16> = OsStr::new(PIPE_NAME)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let pipe_name = HSTRING::from_wide(&pipe_name[..pipe_name.len() - 1])?;

    let handle = unsafe {
        CreateFileW(
            &pipe_name,
            GENERIC_WRITE.0,
            FILE_SHARE_NONE,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        )?
    };

    let mut written = 0u32;
    unsafe {
        WriteFile(handle, Some(msg_bytes), Some(&mut written), None)?;
        CloseHandle(handle)?;
    }

    Ok(())
}

/// Try to launch oss-share.exe if pipe is not available
pub fn try_launch_main_process() {
    // Read install path from registry
    use windows::Win32::System::Registry::*;
    let subkey = HSTRING::from(r"SOFTWARE\OSSShare");
    let mut hkey = HKEY::default();
    let result = unsafe {
        RegOpenKeyExW(HKEY_LOCAL_MACHINE, &subkey, 0, KEY_READ, &mut hkey)
    };
    if result.is_err() {
        return;
    }

    let value_name = HSTRING::from("InstallPath");
    let mut buf = [0u16; 512];
    let mut buf_size = (buf.len() * 2) as u32;
    let mut reg_type = REG_VALUE_TYPE::default();
    let result = unsafe {
        RegQueryValueExW(
            hkey,
            &value_name,
            None,
            Some(&mut reg_type),
            Some(buf.as_mut_ptr() as *mut u8),
            Some(&mut buf_size),
        )
    };
    unsafe { let _ = RegCloseKey(hkey); }

    if result.is_ok() {
        let len = (buf_size as usize / 2).saturating_sub(1);
        let path = String::from_utf16_lossy(&buf[..len]);
        let exe_path = format!(r"{}\oss-share.exe", path);
        let _ = std::process::Command::new(&exe_path).spawn();
        // Wait a moment for the pipe server to start
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
```

- [ ] **Step 3: Create `shell-extension/src/command.rs`**

```rust
use crate::pipe_client;
use windows::Win32::UI::Shell::*;
use windows::Win32::System::Com::*;
use windows::Win32::Foundation::*;
use windows::core::*;

// GUID for our shell extension — generate a unique one
// {A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
pub const CLSID_OSS_SHARE: GUID = GUID::from_u128(0xA1B2C3D4_E5F6_7890_ABCD_EF1234567890);

#[windows::core::implement(IExplorerCommand)]
pub struct OssShareCommand;

impl IExplorerCommand_Impl for OssShareCommand_Impl {
    fn GetTitle(
        &self,
        _psiitemarray: Option<&IShellItemArray>,
    ) -> Result<PWSTR> {
        let title: Vec<u16> = "共享到 OSS"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let ptr = unsafe {
            let p = windows::Win32::System::Com::CoTaskMemAlloc(title.len() * 2) as *mut u16;
            std::ptr::copy_nonoverlapping(title.as_ptr(), p, title.len());
            p
        };
        Ok(PWSTR(ptr))
    }

    fn GetIcon(
        &self,
        _psiitemarray: Option<&IShellItemArray>,
    ) -> Result<PWSTR> {
        // Use a cloud icon from shell32.dll
        let icon: Vec<u16> = "%SystemRoot%\\System32\\shell32.dll,-16817"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let ptr = unsafe {
            let p = windows::Win32::System::Com::CoTaskMemAlloc(icon.len() * 2) as *mut u16;
            std::ptr::copy_nonoverlapping(icon.as_ptr(), p, icon.len());
            p
        };
        Ok(PWSTR(ptr))
    }

    fn GetToolTip(
        &self,
        _psiitemarray: Option<&IShellItemArray>,
    ) -> Result<PWSTR> {
        let tip: Vec<u16> = "上传文件到阿里云 OSS 并生成共享链接"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let ptr = unsafe {
            let p = windows::Win32::System::Com::CoTaskMemAlloc(tip.len() * 2) as *mut u16;
            std::ptr::copy_nonoverlapping(tip.as_ptr(), p, tip.len());
            p
        };
        Ok(PWSTR(ptr))
    }

    fn GetCanonicalName(&self) -> Result<GUID> {
        Ok(CLSID_OSS_SHARE)
    }

    fn GetState(
        &self,
        _psiitemarray: Option<&IShellItemArray>,
        _foktobedefault: BOOL,
    ) -> Result<u32> {
        // ECS_ENABLED = 0
        Ok(0)
    }

    fn GetFlags(&self) -> Result<u32> {
        // ECF_DEFAULT = 0
        Ok(0)
    }

    fn EnumSubCommands(&self) -> Result<IEnumExplorerCommand> {
        Err(E_NOTIMPL.into())
    }

    fn Invoke(
        &self,
        psiitemarray: Option<&IShellItemArray>,
        _pbc: Option<&IBindCtx>,
    ) -> Result<()> {
        let items = match psiitemarray {
            Some(items) => items,
            None => return Ok(()),
        };

        let count = unsafe { items.GetCount()? };
        let mut files = Vec::new();

        for i in 0..count {
            let item = unsafe { items.GetItemAt(i)? };
            let path = unsafe {
                item.GetDisplayName(SIGDN_FILESYSPATH)?
            };
            let path_str = unsafe { path.to_string()? };
            unsafe { CoTaskMemFree(Some(path.0 as *const _)) };
            files.push(path_str);
        }

        if !files.is_empty() {
            // Try to send via pipe, launch main process if needed
            if pipe_client::send_upload_request(&files).is_err() {
                pipe_client::try_launch_main_process();
                let _ = pipe_client::send_upload_request(&files);
            }
        }

        Ok(())
    }
}
```

- [ ] **Step 4: Create `shell-extension/src/lib.rs`**

```rust
mod command;
mod pipe_client;

use command::{OssShareCommand, CLSID_OSS_SHARE};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::Win32::Foundation::*;
use windows::Win32::System::Com::*;
use windows::Win32::System::Registry::*;
use windows::core::*;

static mut DLL_MODULE: HMODULE = HMODULE(std::ptr::null_mut());

#[windows::core::implement(IClassFactory)]
struct OssShareClassFactory;

impl IClassFactory_Impl for OssShareClassFactory_Impl {
    fn CreateInstance(
        &self,
        _punkouter: Option<&IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut std::ffi::c_void,
    ) -> Result<()> {
        let cmd: IExplorerCommand = OssShareCommand.into();
        unsafe { cmd.query(&*riid, ppvobject) }
    }

    fn LockServer(&self, _flock: BOOL) -> Result<()> {
        Ok(())
    }
}

#[no_mangle]
extern "system" fn DllMain(
    hinstance: HMODULE,
    reason: u32,
    _reserved: *mut std::ffi::c_void,
) -> BOOL {
    if reason == 1 {
        // DLL_PROCESS_ATTACH
        unsafe { DLL_MODULE = hinstance; }
    }
    TRUE
}

#[no_mangle]
extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut std::ffi::c_void,
) -> HRESULT {
    unsafe {
        if *rclsid == CLSID_OSS_SHARE {
            let factory: IClassFactory = OssShareClassFactory.into();
            factory.query(&*riid, ppv)
                .map(|_| S_OK)
                .unwrap_or(E_NOINTERFACE)
        } else {
            CLASS_E_CLASSNOTAVAILABLE
        }
    }
}

#[no_mangle]
extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_FALSE
}

#[no_mangle]
extern "system" fn DllRegisterServer() -> HRESULT {
    match register_server() {
        Ok(_) => S_OK,
        Err(_) => E_FAIL,
    }
}

#[no_mangle]
extern "system" fn DllUnregisterServer() -> HRESULT {
    match unregister_server() {
        Ok(_) => S_OK,
        Err(_) => E_FAIL,
    }
}

fn guid_string() -> String {
    format!("{{{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        CLSID_OSS_SHARE.data1,
        CLSID_OSS_SHARE.data2,
        CLSID_OSS_SHARE.data3,
        CLSID_OSS_SHARE.data4[0], CLSID_OSS_SHARE.data4[1],
        CLSID_OSS_SHARE.data4[2], CLSID_OSS_SHARE.data4[3],
        CLSID_OSS_SHARE.data4[4], CLSID_OSS_SHARE.data4[5],
        CLSID_OSS_SHARE.data4[6], CLSID_OSS_SHARE.data4[7],
    )
}

fn get_dll_path() -> Result<String> {
    let mut buf = [0u16; 512];
    let len = unsafe {
        windows::Win32::System::LibraryLoader::GetModuleFileNameW(
            DLL_MODULE,
            &mut buf,
        )
    };
    Ok(String::from_utf16_lossy(&buf[..len as usize]))
}

fn register_server() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let guid = guid_string();
    let dll_path = get_dll_path()?;

    // Register CLSID
    let clsid_key = format!(r"CLSID\{}\InProcServer32", guid);
    let clsid_key_h = HSTRING::from(&clsid_key);
    let mut hkey = HKEY::default();
    unsafe {
        RegCreateKeyExW(
            HKEY_CLASSES_ROOT,
            &clsid_key_h,
            0, None,
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut hkey,
            None,
        )?;
        let dll_wide: Vec<u16> = OsStr::new(&dll_path)
            .encode_wide().chain(std::iter::once(0)).collect();
        RegSetValueExW(hkey, None, 0, REG_SZ,
            Some(std::slice::from_raw_parts(
                dll_wide.as_ptr() as *const u8,
                dll_wide.len() * 2,
            )))?;
        let threading = HSTRING::from("ThreadingModel");
        let apartment: Vec<u16> = OsStr::new("Apartment")
            .encode_wide().chain(std::iter::once(0)).collect();
        RegSetValueExW(hkey, &threading, 0, REG_SZ,
            Some(std::slice::from_raw_parts(
                apartment.as_ptr() as *const u8,
                apartment.len() * 2,
            )))?;
        RegCloseKey(hkey)?;
    }

    // Register context menu handler for all files
    let handler_key = format!(r"*\shellex\ContextMenuHandlers\OSSShare");
    let handler_key_h = HSTRING::from(&handler_key);
    unsafe {
        RegCreateKeyExW(
            HKEY_CLASSES_ROOT,
            &handler_key_h,
            0, None,
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut hkey,
            None,
        )?;
        let guid_wide: Vec<u16> = OsStr::new(&guid)
            .encode_wide().chain(std::iter::once(0)).collect();
        RegSetValueExW(hkey, None, 0, REG_SZ,
            Some(std::slice::from_raw_parts(
                guid_wide.as_ptr() as *const u8,
                guid_wide.len() * 2,
            )))?;
        RegCloseKey(hkey)?;
    }

    Ok(())
}

fn unregister_server() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let guid = guid_string();

    let clsid_key = format!(r"CLSID\{}", guid);
    let clsid_key_h = HSTRING::from(&clsid_key);
    unsafe {
        let _ = RegDeleteTreeW(HKEY_CLASSES_ROOT, &clsid_key_h);
    }

    let handler_key = HSTRING::from(r"*\shellex\ContextMenuHandlers\OSSShare");
    unsafe {
        let _ = RegDeleteTreeW(HKEY_CLASSES_ROOT, &handler_key);
    }

    Ok(())
}
```

- [ ] **Step 5: Verify DLL compiles**

```bash
cd shell-extension && cargo build
```

Expected: Produces `target/debug/oss_share_shell.dll`.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add shell extension COM DLL with IExplorerCommand"
```

---

## Task 10: NSIS Installer

**Files:**
- Create: `installer/nsis/installer.nsi`

- [ ] **Step 1: Create `installer/nsis/installer.nsi`**

```nsis
!include "MUI2.nsh"

Name "OSS Share"
OutFile "OSSShare-Setup.exe"
InstallDir "$PROGRAMFILES64\OSSShare"
RequestExecutionLevel admin

!define MUI_ICON "..\..\src-tauri\icons\icon.ico"
!define MUI_UNICON "..\..\src-tauri\icons\icon.ico"

!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "SimpChinese"

Section "Install"
    SetOutPath $INSTDIR

    ; Kill running instance
    nsExec::ExecToLog 'taskkill /f /im oss-share.exe'

    ; Copy files
    File "..\..\src-tauri\target\release\oss-share.exe"
    File "..\..\shell-extension\target\release\oss_share_shell.dll"

    ; Register COM DLL
    nsExec::ExecToLog 'regsvr32 /s "$INSTDIR\oss_share_shell.dll"'

    ; Write install path to registry (for DLL to find exe)
    WriteRegStr HKLM "SOFTWARE\OSSShare" "InstallPath" "$INSTDIR"

    ; Auto-start on login
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Run" \
        "OSSShare" '"$INSTDIR\oss-share.exe"'

    ; Uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"

    ; Add/Remove Programs entry
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\OSSShare" \
        "DisplayName" "OSS Share"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\OSSShare" \
        "UninstallString" '"$INSTDIR\Uninstall.exe"'
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\OSSShare" \
        "DisplayIcon" '"$INSTDIR\oss-share.exe"'
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\OSSShare" \
        "Publisher" "OSS Share"

    ; Launch app
    Exec '"$INSTDIR\oss-share.exe"'
SectionEnd

Section "Uninstall"
    ; Kill running instance
    nsExec::ExecToLog 'taskkill /f /im oss-share.exe'

    ; Unregister COM DLL
    nsExec::ExecToLog 'regsvr32 /u /s "$INSTDIR\oss_share_shell.dll"'

    ; Remove files
    Delete "$INSTDIR\oss-share.exe"
    Delete "$INSTDIR\oss_share_shell.dll"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir "$INSTDIR"

    ; Clean registry
    DeleteRegKey HKLM "SOFTWARE\OSSShare"
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "OSSShare"
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\OSSShare"

    ; Notify shell of changes
    System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
SectionEnd
```

- [ ] **Step 2: Verify NSIS script syntax**

```bash
makensis /HDRINFO  # Check NSIS is installed
```

If NSIS is not installed, install it:
```bash
winget install NSIS.NSIS
```

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat: add NSIS installer script"
```

---

## Task 11: Build & Integration Test

**Files:**
- Modify: `package.json` (add build scripts)

- [ ] **Step 1: Add build scripts to `package.json`**

Add to `scripts` section:
```json
{
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "tauri": "tauri",
    "build:shell": "cd shell-extension && cargo build --release",
    "build:app": "npm run tauri build",
    "build:all": "npm run build:shell && npm run build:app",
    "build:installer": "makensis installer/nsis/installer.nsi"
  }
}
```

- [ ] **Step 2: Build shell extension**

```bash
cd shell-extension && cargo build --release
```

Expected: `shell-extension/target/release/oss_share_shell.dll` produced.

- [ ] **Step 3: Build Tauri app**

```bash
npm run tauri build
```

Expected: `src-tauri/target/release/oss-share.exe` produced.

- [ ] **Step 4: Manual integration test**

1. Run `oss-share.exe` — verify tray icon appears
2. Right-click tray icon — verify menu shows (打开文件管理, 设置, 退出)
3. Click "设置" — verify settings window opens with form
4. Fill in OSS credentials, click "测试连接" — verify connection result
5. Click "保存" — verify config saved to `%APPDATA%/oss-share/config.toml`
6. Register DLL manually: `regsvr32 oss_share_shell.dll`
7. Right-click a file in Explorer — verify "共享到 OSS" appears
8. Click "共享到 OSS" — verify file uploads and link copied to clipboard
9. Click "文件管理" in tray — verify uploaded file appears in list
10. Click 🔗 on a file — verify link copied to clipboard
11. Click 🗑 on a file — verify file deleted from list

- [ ] **Step 5: Build installer (optional)**

```bash
makensis installer/nsis/installer.nsi
```

Expected: `installer/nsis/OSSShare-Setup.exe` produced.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add build scripts and finalize integration"
```

---

## Summary

| Task | Description | Key Files |
|------|-------------|-----------|
| 1 | Project scaffolding | package.json, Cargo.toml, tauri.conf.json, main.rs |
| 2 | Config module (DPAPI) | config.rs |
| 3 | OSS service module | oss.rs |
| 4 | System tray | tray.rs |
| 5 | Tauri IPC commands | commands.rs, main.rs |
| 6 | Named Pipe IPC server | ipc.rs |
| 7 | Settings page (Vue) | Settings.vue, App.vue |
| 8 | Files page (Vue) | Files.vue, FileRow.vue |
| 9 | Shell Extension COM DLL | lib.rs, command.rs, pipe_client.rs |
| 10 | NSIS installer | installer.nsi |
| 11 | Build & integration test | package.json, manual testing |
