use crate::pipe_client;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Shell::*;

// GUID for our shell extension
// {A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
pub const CLSID_OSS_SHARE: GUID = GUID::from_u128(0xA1B2C3D4_E5F6_7890_ABCD_EF1234567890);

#[windows::core::implement(IExplorerCommand)]
pub struct OssShareCommand;

impl IExplorerCommand_Impl for OssShareCommand_Impl {
    fn GetTitle(&self, _psiitemarray: Option<&IShellItemArray>) -> Result<PWSTR> {
        alloc_pwstr("共享到 OSS")
    }

    fn GetIcon(&self, _psiitemarray: Option<&IShellItemArray>) -> Result<PWSTR> {
        alloc_pwstr("%SystemRoot%\\System32\\shell32.dll,-16817")
    }

    fn GetToolTip(&self, _psiitemarray: Option<&IShellItemArray>) -> Result<PWSTR> {
        alloc_pwstr("上传文件到阿里云 OSS 并生成共享链接")
    }

    fn GetCanonicalName(&self) -> Result<GUID> {
        Ok(CLSID_OSS_SHARE)
    }

    fn GetState(
        &self,
        _psiitemarray: Option<&IShellItemArray>,
        _foktobeslow: BOOL,
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
            let path = unsafe { item.GetDisplayName(SIGDN_FILESYSPATH)? };
            let path_str = unsafe { path.to_string() }
                .map_err(|_| Error::from(E_FAIL))?;
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

/// Allocate a wide string via CoTaskMemAlloc and return as PWSTR.
/// The caller (COM runtime / Explorer) is responsible for freeing.
fn alloc_pwstr(s: &str) -> Result<PWSTR> {
    let wide: Vec<u16> = s.encode_utf16().chain(std::iter::once(0)).collect();
    let byte_len = wide.len() * 2;
    let ptr = unsafe { CoTaskMemAlloc(byte_len) as *mut u16 };
    if ptr.is_null() {
        return Err(E_OUTOFMEMORY.into());
    }
    unsafe {
        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
    }
    Ok(PWSTR(ptr))
}
