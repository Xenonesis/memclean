use serde::{Deserialize, Serialize};
use std::ffi::c_void;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress, LoadLibraryA};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentProcessId, OpenProcess, PROCESS_ALL_ACCESS,
    PROCESS_QUERY_INFORMATION, PROCESS_SET_INFORMATION,
};

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
#[allow(non_snake_case)]
pub struct D3DKMT_TRIM_FLAGS {
    pub Value: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[allow(non_snake_case)]
pub struct D3DKMT_TRIMPROCESSCOMMITMENT {
    pub cbSize: u32,
    pub hProcess: HANDLE,
    pub Flags: D3DKMT_TRIM_FLAGS,
    pub DecommitRequested: u64,
    pub NumBytesDecommitted: u64,
}

type D3DKMTTrimProcessCommitmentFn =
    unsafe extern "system" fn(params: *mut D3DKMT_TRIMPROCESSCOMMITMENT) -> i32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuAdapterInfo {
    pub index: u32,
    pub description: String,
    pub dedicated_vram_mb: u64,
    pub dedicated_system_mb: u64,
    pub shared_system_mb: u64,
    pub vendor_id: u32,
    pub device_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VramTrimReport {
    pub processes_trimmed: usize,
    pub bytes_decommitted: u64,
    pub mb_decommitted: f64,
}

/// Trims GPU VRAM commitment for a single process given its PID
pub fn trim_process_vram(pid: u32) -> Result<u64, String> {
    unsafe {
        let h_gdi32 = GetModuleHandleA(b"gdi32.dll\0".as_ptr());
        if h_gdi32.is_null() {
            return Err("Failed to obtain gdi32.dll handle".to_string());
        }

        let proc_addr = GetProcAddress(h_gdi32, b"D3DKMTTrimProcessCommitment\0".as_ptr());
        if proc_addr.is_none() {
            return Err("D3DKMTTrimProcessCommitment not supported on this Windows driver".to_string());
        }

        let trim_fn: D3DKMTTrimProcessCommitmentFn = std::mem::transmute(proc_addr);

        let h_proc = if pid == GetCurrentProcessId() {
            GetCurrentProcess()
        } else {
            let h = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);
            if h.is_null() {
                OpenProcess(PROCESS_SET_INFORMATION | PROCESS_QUERY_INFORMATION, 0, pid)
            } else {
                h
            }
        };

        if h_proc.is_null() {
            return Err(format!("Failed to open process PID {} for VRAM trimming", pid));
        }

        let mut params = D3DKMT_TRIMPROCESSCOMMITMENT {
            cbSize: std::mem::size_of::<D3DKMT_TRIMPROCESSCOMMITMENT>() as u32,
            hProcess: h_proc,
            Flags: D3DKMT_TRIM_FLAGS { Value: 0 },
            DecommitRequested: u64::MAX, // Request maximum eligible decommitment
            NumBytesDecommitted: 0,
        };

        let status = trim_fn(&mut params);

        if pid != GetCurrentProcessId() && !h_proc.is_null() {
            CloseHandle(h_proc);
        }

        if status == 0 {
            Ok(params.NumBytesDecommitted)
        } else {
            Err(format!("D3DKMTTrimProcessCommitment failed with NTSTATUS: 0x{:08X}", status as u32))
        }
    }
}

/// Trims GPU video memory commitments across all eligible running processes
pub fn trim_all_vram(exclusions: &[String]) -> VramTrimReport {
    let mut report = VramTrimReport::default();

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return report;
        }

        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                let pid = entry.th32ProcessID;
                if pid > 4 {
                    let len = entry
                        .szExeFile
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(entry.szExeFile.len());
                    let name = String::from_utf16_lossy(&entry.szExeFile[..len]).to_lowercase();

                    let is_excluded = exclusions.iter().any(|ex| ex.to_lowercase() == name);
                    if !is_excluded {
                        if let Ok(bytes) = trim_process_vram(pid) {
                            report.processes_trimmed += 1;
                            report.bytes_decommitted += bytes;
                        }
                    }
                }

                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
    }

    report.mb_decommitted = report.bytes_decommitted as f64 / (1024.0 * 1024.0);
    report
}

// Low-level COM helper to query GPU adapter information via DXGI
#[repr(C)]
struct DXGI_ADAPTER_DESC1 {
    description: [u16; 128],
    vendor_id: u32,
    device_id: u32,
    subsys_id: u32,
    revision: u32,
    dedicated_video_memory: usize,
    dedicated_system_memory: usize,
    shared_system_memory: usize,
    adapter_luid: [u32; 2],
    flags: u32,
}

#[repr(C)]
struct IUnknownVtbl {
    query_interface: unsafe extern "system" fn(this: *mut c_void, riid: *const c_void, ppv: *mut *mut c_void) -> i32,
    add_ref: unsafe extern "system" fn(this: *mut c_void) -> u32,
    release: unsafe extern "system" fn(this: *mut c_void) -> u32,
}

#[repr(C)]
struct IDXGIFactory1Vtbl {
    unknown: IUnknownVtbl,
    set_private_data: unsafe extern "system" fn(this: *mut c_void, name: *const c_void, data_size: u32, data: *const c_void) -> i32,
    set_private_data_interface: unsafe extern "system" fn(this: *mut c_void, name: *const c_void, unknown: *const c_void) -> i32,
    get_private_data: unsafe extern "system" fn(this: *mut c_void, name: *const c_void, data_size: *mut u32, data: *mut c_void) -> i32,
    get_parent: unsafe extern "system" fn(this: *mut c_void, riid: *const c_void, parent: *mut *mut c_void) -> i32,
    enum_adapters: unsafe extern "system" fn(this: *mut c_void, adapter: u32, pp_adapter: *mut *mut c_void) -> i32,
    make_window_association: unsafe extern "system" fn(this: *mut c_void, window_handle: *mut c_void, flags: u32) -> i32,
    get_window_association: unsafe extern "system" fn(this: *mut c_void, window_handle: *mut *mut c_void) -> i32,
    create_swap_chain: unsafe extern "system" fn(this: *mut c_void, device: *mut c_void, desc: *mut c_void, swap_chain: *mut *mut c_void) -> i32,
    create_software_adapter: unsafe extern "system" fn(this: *mut c_void, module: *mut c_void, pp_adapter: *mut *mut c_void) -> i32,
    enum_adapters1: unsafe extern "system" fn(this: *mut c_void, adapter: u32, pp_adapter: *mut *mut c_void) -> i32,
}

#[repr(C)]
struct IDXGIAdapter1Vtbl {
    unknown: IUnknownVtbl,
    set_private_data: unsafe extern "system" fn(this: *mut c_void, name: *const c_void, data_size: u32, data: *const c_void) -> i32,
    set_private_data_interface: unsafe extern "system" fn(this: *mut c_void, name: *const c_void, unknown: *const c_void) -> i32,
    get_private_data: unsafe extern "system" fn(this: *mut c_void, name: *const c_void, data_size: *mut u32, data: *mut c_void) -> i32,
    get_parent: unsafe extern "system" fn(this: *mut c_void, riid: *const c_void, parent: *mut *mut c_void) -> i32,
    enum_outputs: unsafe extern "system" fn(this: *mut c_void, output: u32, pp_output: *mut *mut c_void) -> i32,
    get_desc: unsafe extern "system" fn(this: *mut c_void, desc: *mut c_void) -> i32,
    check_interface_support: unsafe extern "system" fn(this: *mut c_void, interface_name: *const c_void, umd_version: *mut u64) -> i32,
    get_desc1: unsafe extern "system" fn(this: *mut c_void, desc: *mut DXGI_ADAPTER_DESC1) -> i32,
}

// IID_IDXGIFactory1: 770aae78-f26f-4dba-a829-253c83d1b387
const IID_IDXGI_FACTORY1: [u8; 16] = [
    0x78, 0xae, 0x0a, 0x77, 0x6f, 0xf2, 0xba, 0x4d, 0xa8, 0x29, 0x25, 0x3c, 0x83, 0xd1, 0xb3, 0x87,
];

type CreateDXGIFactory1Fn =
    unsafe extern "system" fn(riid: *const c_void, pp_factory: *mut *mut c_void) -> i32;

/// Enumerate physical GPU adapters and their video memory allocations
pub fn get_gpu_adapters() -> Vec<GpuAdapterInfo> {
    let mut adapters = Vec::new();

    unsafe {
        let h_dxgi = LoadLibraryA(b"dxgi.dll\0".as_ptr());
        if h_dxgi.is_null() {
            return adapters;
        }

        let proc = GetProcAddress(h_dxgi, b"CreateDXGIFactory1\0".as_ptr());
        if proc.is_none() {
            return adapters;
        }

        let create_factory: CreateDXGIFactory1Fn = std::mem::transmute(proc);
        let mut factory_ptr: *mut c_void = std::ptr::null_mut();

        if create_factory(IID_IDXGI_FACTORY1.as_ptr() as _, &mut factory_ptr) != 0 || factory_ptr.is_null() {
            return adapters;
        }

        let factory_vtbl = *(factory_ptr as *mut *mut IDXGIFactory1Vtbl);
        let mut index: u32 = 0;

        loop {
            let mut adapter_ptr: *mut c_void = std::ptr::null_mut();
            let hr = ((*factory_vtbl).enum_adapters1)(factory_ptr, index, &mut adapter_ptr);
            if hr != 0 || adapter_ptr.is_null() {
                break;
            }

            let adapter_vtbl = *(adapter_ptr as *mut *mut IDXGIAdapter1Vtbl);
            let mut desc: DXGI_ADAPTER_DESC1 = std::mem::zeroed();

            if ((*adapter_vtbl).get_desc1)(adapter_ptr, &mut desc) == 0 {
                let desc_len = desc
                    .description
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(desc.description.len());
                let description = String::from_utf16_lossy(&desc.description[..desc_len]);

                adapters.push(GpuAdapterInfo {
                    index,
                    description,
                    dedicated_vram_mb: (desc.dedicated_video_memory / (1024 * 1024)) as u64,
                    dedicated_system_mb: (desc.dedicated_system_memory / (1024 * 1024)) as u64,
                    shared_system_mb: (desc.shared_system_memory / (1024 * 1024)) as u64,
                    vendor_id: desc.vendor_id,
                    device_id: desc.device_id,
                });
            }

            ((*adapter_vtbl).unknown.release)(adapter_ptr);
            index += 1;
        }

        ((*factory_vtbl).unknown.release)(factory_ptr);
    }

    adapters
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_process_commitment_struct_layout() {
        assert_eq!(std::mem::size_of::<D3DKMT_TRIM_FLAGS>(), 4);
        assert_eq!(std::mem::size_of::<D3DKMT_TRIMPROCESSCOMMITMENT>(), 40);
    }

    #[test]
    fn test_get_gpu_adapters_returns_hardware() {
        let adapters = get_gpu_adapters();
        assert!(!adapters.is_empty(), "Expected at least one GPU adapter");
        for a in &adapters {
            assert!(!a.description.is_empty());
        }
    }

    #[test]
    fn test_trim_process_vram_self() {
        let pid = std::process::id();
        let res = trim_process_vram(pid);
        assert!(res.is_ok(), "Self VRAM trim should succeed: {:?}", res);
    }
}
