mod command;
mod pipe_client;

use command::{OssShareCommand, CLSID_OSS_SHARE};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::Com::*;
use windows::Win32::System::Registry::*;
use windows::Win32::UI::Shell::IExplorerCommand;

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
        unsafe {
            let hr = cmd.query(&*riid, ppvobject);
            hr.ok()
        }
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
        unsafe {
            DLL_MODULE = hinstance;
        }
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
    format!(
        "{{{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        CLSID_OSS_SHARE.data1,
        CLSID_OSS_SHARE.data2,
        CLSID_OSS_SHARE.data3,
        CLSID_OSS_SHARE.data4[0],
        CLSID_OSS_SHARE.data4[1],
        CLSID_OSS_SHARE.data4[2],
        CLSID_OSS_SHARE.data4[3],
        CLSID_OSS_SHARE.data4[4],
        CLSID_OSS_SHARE.data4[5],
        CLSID_OSS_SHARE.data4[6],
        CLSID_OSS_SHARE.data4[7],
    )
}

fn get_dll_path() -> Result<String> {
    let mut buf = [0u16; 512];
    let len = unsafe {
        windows::Win32::System::LibraryLoader::GetModuleFileNameW(DLL_MODULE, &mut buf)
    };
    Ok(String::from_utf16_lossy(&buf[..len as usize]))
}

fn register_server() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let guid = guid_string();
    let dll_path = get_dll_path()?;

    // Register CLSID\{guid}\InProcServer32 under HKCU
    let clsid_key = format!(r"SOFTWARE\Classes\CLSID\{}\InProcServer32", guid);
    let clsid_key_h = HSTRING::from(&clsid_key);
    let mut hkey = HKEY::default();
    unsafe {
        let err = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            &clsid_key_h,
            0,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut hkey,
            None,
        );
        if err != ERROR_SUCCESS {
            return Err(format!("RegCreateKeyExW failed: {:?}", err).into());
        }

        // Default value = DLL path
        let dll_wide: Vec<u16> = OsStr::new(&dll_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let dll_bytes = std::slice::from_raw_parts(
            dll_wide.as_ptr() as *const u8,
            dll_wide.len() * 2,
        );
        let _ = RegSetValueExW(hkey, PCWSTR::null(), 0, REG_SZ, Some(dll_bytes));

        // ThreadingModel = Apartment
        let threading = HSTRING::from("ThreadingModel");
        let apartment: Vec<u16> = OsStr::new("Apartment")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let apartment_bytes = std::slice::from_raw_parts(
            apartment.as_ptr() as *const u8,
            apartment.len() * 2,
        );
        let _ = RegSetValueExW(hkey, &threading, 0, REG_SZ, Some(apartment_bytes));

        let _ = RegCloseKey(hkey);
    }

    // Register IExplorerCommand via *\shell\OSSShare verb
    // Use HKCU\SOFTWARE\Classes to avoid needing admin privileges
    let verb_key = r"SOFTWARE\Classes\*\shell\OSSShare";
    let verb_key_h = HSTRING::from(verb_key);
    unsafe {
        let err = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            &verb_key_h,
            0,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut hkey,
            None,
        );
        if err != ERROR_SUCCESS {
            return Err(format!("RegCreateKeyExW (verb) failed: {:?}", err).into());
        }

        // Display name
        let name = HSTRING::from("MUIVerb");
        let display: Vec<u16> = OsStr::new("共享到 OSS")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let display_bytes = std::slice::from_raw_parts(
            display.as_ptr() as *const u8,
            display.len() * 2,
        );
        let _ = RegSetValueExW(hkey, &name, 0, REG_SZ, Some(display_bytes));

        // ExplorerCommandHandler = CLSID
        let handler_name = HSTRING::from("ExplorerCommandHandler");
        let guid_wide: Vec<u16> = OsStr::new(&guid)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let guid_bytes = std::slice::from_raw_parts(
            guid_wide.as_ptr() as *const u8,
            guid_wide.len() * 2,
        );
        let _ = RegSetValueExW(hkey, &handler_name, 0, REG_SZ, Some(guid_bytes));

        // Icon
        let icon_name = HSTRING::from("Icon");
        let icon_val: Vec<u16> = OsStr::new("%SystemRoot%\\System32\\shell32.dll,-16817")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let icon_bytes = std::slice::from_raw_parts(
            icon_val.as_ptr() as *const u8,
            icon_val.len() * 2,
        );
        let _ = RegSetValueExW(hkey, &icon_name, 0, REG_SZ, Some(icon_bytes));

        let _ = RegCloseKey(hkey);
    }

    Ok(())
}

fn unregister_server() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let guid = guid_string();

    // Remove CLSID\{guid} from HKCU
    let clsid_key = format!(r"SOFTWARE\Classes\CLSID\{}", guid);
    let clsid_key_h = HSTRING::from(&clsid_key);
    unsafe {
        let _ = RegDeleteTreeW(HKEY_CURRENT_USER, &clsid_key_h);
    }

    // Remove *\shell\OSSShare from HKCU
    let verb_key = HSTRING::from(r"SOFTWARE\Classes\*\shell\OSSShare");
    unsafe {
        let _ = RegDeleteTreeW(HKEY_CURRENT_USER, &verb_key);
    }

    // Also clean up old HKCR registrations if they exist
    let old_clsid = format!(r"CLSID\{}", guid);
    let old_clsid_h = HSTRING::from(&old_clsid);
    unsafe {
        let _ = RegDeleteTreeW(HKEY_CLASSES_ROOT, &old_clsid_h);
    }
    let old_handler = HSTRING::from(r"*\shellex\ContextMenuHandlers\OSSShare");
    unsafe {
        let _ = RegDeleteTreeW(HKEY_CLASSES_ROOT, &old_handler);
    }

    Ok(())
}
