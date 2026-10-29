use std::ffi::c_void;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

type NtSetSystemInformationFn = unsafe extern "system" fn(
    system_information_class: u32,
    system_information: *const c_void,
    system_information_length: u32,
) -> i32;

#[test]
fn test_priv_diag() {
    // Test 1: Try with SeProfileSingleProcessPrivilege enabled
    let r1 = memclean::core::privileges::enable_memory_privileges();
    println!("enable_memory_privileges: {:?}", r1);

    // Direct call
    unsafe {
        let h = GetModuleHandleA(b"ntdll.dll\0".as_ptr());
        let proc = GetProcAddress(h, b"NtSetSystemInformation\0".as_ptr());
        let fn_ptr: NtSetSystemInformationFn = std::mem::transmute(proc);
        let cmd: u32 = 4u32;
        let status = fn_ptr(80u32, &cmd as *const u32 as *const c_void, 4u32);
        println!("Status after enable: {:#010X} ({})", status, status);
        assert!(status >= 0, "NtSetSystemInformation failed: {:#010X}", status);
    }
}
