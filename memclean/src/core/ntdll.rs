use std::ffi::c_void;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

type NtSetSystemInformationFn = unsafe extern "system" fn(
    system_information_class: u32,
    system_information: *const c_void,
    system_information_length: u32,
) -> i32;

type NtQuerySystemInformationFn = unsafe extern "system" fn(
    system_information_class: u32,
    system_information: *mut c_void,
    system_information_length: u32,
    return_length: *mut u32,
) -> i32;

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct SystemMemoryListInformation {
    pub zero_page_count: usize,
    pub free_page_count: usize,
    pub modified_page_count: usize,
    pub modified_no_write_page_count: usize,
    pub bad_page_count: usize,
    pub page_count_by_priority: [usize; 8],
    pub repurposed_pages_by_priority: [usize; 8],
    pub modified_page_count_page_file: usize,
}

pub fn query_system_memory_list() -> Result<SystemMemoryListInformation, String> {
    unsafe {
        let h_ntdll = GetModuleHandleA(b"ntdll.dll\0".as_ptr());
        if h_ntdll.is_null() {
            return Err("Failed to obtain ntdll.dll module handle".to_string());
        }

        let proc = GetProcAddress(h_ntdll, b"NtQuerySystemInformation\0".as_ptr());
        if proc.is_none() {
            return Err("Failed to resolve NtQuerySystemInformation in ntdll".to_string());
        }

        let nt_query_sys_info: NtQuerySystemInformationFn = std::mem::transmute(proc);
        let mut info = SystemMemoryListInformation::default();
        let mut ret_len = 0u32;
        let status = nt_query_sys_info(
            SYSTEM_MEMORY_LIST_INFORMATION,
            &mut info as *mut _ as *mut c_void,
            std::mem::size_of::<SystemMemoryListInformation>() as u32,
            &mut ret_len,
        );

        if status >= 0 {
            Ok(info)
        } else {
            Err(format!("NtQuerySystemInformation failed with NTSTATUS: {:#010X}", status))
        }
    }
}

const SYSTEM_MEMORY_LIST_INFORMATION: u32 = 80;

#[repr(u32)]
pub enum MemoryListCommand {
    EmptyWorkingSets = 2,
    FlushModifiedList = 3,
    PurgeStandbyList = 4,
    PurgeLowPriorityStandbyList = 5,
}

fn execute_memory_list_command(command: MemoryListCommand) -> Result<(), String> {
    unsafe {
        let h_ntdll = GetModuleHandleA(b"ntdll.dll\0".as_ptr());
        if h_ntdll.is_null() {
            return Err("Failed to obtain ntdll.dll module handle".to_string());
        }

        let proc = GetProcAddress(h_ntdll, b"NtSetSystemInformation\0".as_ptr());
        if proc.is_none() {
            return Err("Failed to resolve NtSetSystemInformation in ntdll".to_string());
        }

        let nt_set_system_info: NtSetSystemInformationFn = std::mem::transmute(proc);
        let cmd_val = command as u32;
        let status = nt_set_system_info(
            SYSTEM_MEMORY_LIST_INFORMATION,
            &cmd_val as *const u32 as *const c_void,
            std::mem::size_of::<u32>() as u32,
        );

        if status >= 0 {
            Ok(())
        } else {
            Err(format!("NtSetSystemInformation failed with NTSTATUS: {:#010X}", status))
        }
    }
}

pub fn purge_standby_list() -> Result<(), String> {
    execute_memory_list_command(MemoryListCommand::PurgeStandbyList)
}

pub fn purge_low_priority_standby_list() -> Result<(), String> {
    execute_memory_list_command(MemoryListCommand::PurgeLowPriorityStandbyList)
}

pub fn flush_modified_list() -> Result<(), String> {
    execute_memory_list_command(MemoryListCommand::FlushModifiedList)
}

pub fn empty_system_working_sets() -> Result<(), String> {
    execute_memory_list_command(MemoryListCommand::EmptyWorkingSets)
}

pub fn trim_system_file_cache() -> Result<(), String> {
    extern "system" {
        fn SetSystemFileCacheSize(
            minimumfilesize: usize,
            maximumfilesize: usize,
            flags: u32,
        ) -> i32;
    }
    unsafe {
        let res = SetSystemFileCacheSize(usize::MAX, usize::MAX, 0);
        if res != 0 {
            Ok(())
        } else {
            Err("SetSystemFileCacheSize failed".to_string())
        }
    }
}