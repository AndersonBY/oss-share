use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use windows::Win32::Foundation::{HLOCAL, LocalFree};
use windows::Win32::Security::Cryptography::*;

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
    let blob_in = CRYPT_INTEGER_BLOB {
        cbData: plain_bytes.len() as u32,
        pbData: plain_bytes.as_ptr() as *mut u8,
    };
    let mut blob_out = CRYPT_INTEGER_BLOB::default();
    unsafe {
        CryptProtectData(
            &blob_in,
            None,
            None,
            None,
            None,
            0,
            &mut blob_out,
        )
        .map_err(|e| format!("DPAPI encrypt failed: {e}"))?;
        let slice = std::slice::from_raw_parts(blob_out.pbData, blob_out.cbData as usize);
        let encoded = base64::engine::general_purpose::STANDARD.encode(slice);
        LocalFree(HLOCAL(blob_out.pbData as *mut _));
        Ok(encoded)
    }
}

pub fn decrypt_secret(encrypted_b64: &str) -> Result<String, String> {
    use base64::Engine;
    if encrypted_b64.is_empty() {
        return Ok(String::new());
    }
    let encrypted_bytes = base64::engine::general_purpose::STANDARD
        .decode(encrypted_b64)
        .map_err(|e| format!("base64 decode failed: {e}"))?;
    let blob_in = CRYPT_INTEGER_BLOB {
        cbData: encrypted_bytes.len() as u32,
        pbData: encrypted_bytes.as_ptr() as *mut u8,
    };
    let mut blob_out = CRYPT_INTEGER_BLOB::default();
    unsafe {
        CryptUnprotectData(
            &blob_in,
            None,
            None,
            None,
            None,
            0,
            &mut blob_out,
        )
        .map_err(|e| format!("DPAPI decrypt failed: {e}"))?;
        let slice = std::slice::from_raw_parts(blob_out.pbData, blob_out.cbData as usize);
        let plain =
            String::from_utf8(slice.to_vec()).map_err(|e| format!("UTF-8 decode failed: {e}"))?;
        LocalFree(HLOCAL(blob_out.pbData as *mut _));
        Ok(plain)
    }
}
