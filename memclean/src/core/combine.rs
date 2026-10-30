use std::ffi::c_void;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

pub const SYSTEM_COMBINE_PHYSICAL_MEMORY_INFORMATION: u32 = 130;

#[repr(C)]
#[derive(Debug, Default)]
pub struct MemoryCombineInformationEx {
    pub handle: *mut c_void,
    pub pages_combined: usize,
    pub flags: u32,
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct CombineResult {
    pub pages_combined: usize,
    pub mb_saved: f32,
}

pub fn combine_physical_memory() -> Result<CombineResult, String> {
    unsafe {
        let h_ntdll = GetModuleHandleA(b"ntdll.dll\0".as_ptr());
        if h_ntdll.is_null() {
            return Err("Failed to obtain ntdll.dll handle".to_string());
        }

        let proc = GetProcAddress(h_ntdll, b"NtSetSystemInformation\0".as_ptr());
        if proc.is_none() {
            return Err("Failed to resolve NtSetSystemInformation in ntdll".to_string());
        }

        let nt_set_sys_info: unsafe extern "system" fn(u32, *mut c_void, u32) -> i32 =
            std::mem::transmute(proc);

        let mut info = MemoryCombineInformationEx {
            handle: std::ptr::null_mut(),
            pages_combined: 0,
            flags: 0,
        };

        let status = nt_set_sys_info(
            SYSTEM_COMBINE_PHYSICAL_MEMORY_INFORMATION,
            &mut info as *mut _ as *mut c_void,
            std::mem::size_of::<MemoryCombineInformationEx>() as u32,
        );

        if status >= 0 {
            let mb = (info.pages_combined as f32 * 4.0) / 1024.0;
            Ok(CombineResult {
                pages_combined: info.pages_combined,
                mb_saved: mb,
            })
        } else {
            Err(format!(
                "NtSetSystemInformation(130) failed with NTSTATUS: {:#010X}",
                status
            ))
        }
    }
}
