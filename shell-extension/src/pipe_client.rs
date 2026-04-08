use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Storage::FileSystem::*;

const PIPE_NAME: &str = r"\\.\pipe\oss-share";

pub fn send_upload_request(files: &[String]) -> Result<()> {
    let msg = serde_json::json!({
        "action": "upload",
        "files": files
    });
    let msg_str = msg.to_string();
    let msg_bytes = msg_str.as_bytes();

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
        )?
    };

    unsafe {
        let mut written = 0u32;
        WriteFile(handle, Some(msg_bytes), Some(&mut written), None)?;
        CloseHandle(handle)?;
    }

    Ok(())
}

/// Try to launch oss-share.exe if pipe is not available
pub fn try_launch_main_process() {
    use windows::Win32::System::Registry::*;

    let subkey = HSTRING::from(r"SOFTWARE\OSSShare");
    let mut hkey = HKEY::default();
    let result = unsafe {
        RegOpenKeyExW(HKEY_CURRENT_USER, &subkey, 0, KEY_READ, &mut hkey)
    };
    if result != ERROR_SUCCESS {
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
    unsafe {
        let _ = RegCloseKey(hkey);
    }

    if result == ERROR_SUCCESS {
        let len = (buf_size as usize / 2).saturating_sub(1);
        let path = String::from_utf16_lossy(&buf[..len]);
        let exe_path = format!(r"{}\oss-share.exe", path);
        let _ = std::process::Command::new(&exe_path).spawn();
        // Wait a moment for the pipe server to start
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
