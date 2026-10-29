use std::ffi::c_void;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

type NtSetSystemInformationFn = unsafe extern "system" fn(
    system_information_class: u32,
    system_information: *const c_void,
    system_information_length: u32,
) -> i32;

#[test]
fn test_ntdll_debug() {
    unsafe {
        let h = GetModuleHandleA(b"ntdll.dll\0".as_ptr());
        let proc = GetProcAddress(h, b"NtSetSystemInformation\0".as_ptr());
        let fn_ptr: NtSetSystemInformationFn = std::mem::transmute(proc);
        let cmd: u32 = 4u32;
        let status = fn_call(fn_ptr, 80u32, &cmd as *const u32 as *const c_void, 4u32);
        println!("NtSetSystemInformation status: {:#010X} ({})", status, status);
        if status < 0 {
            println!("FAILED - status negative");
        } else {
            println!("SUCCESS");
        }
    }
}

unsafe fn fn_call(f: NtSetSystemInformationFn, c: u32, ptr: *const c_void, len: u32) -> i32 {
    f(c, ptr, len)
}
