use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::ProcessStatus::{K32EmptyWorkingSet, K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
use windows_sys::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_QUOTA,
};

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
}

pub fn enumerate_processes() -> Result<Vec<ProcessInfo>, String> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(format!("CreateToolhelp32Snapshot failed: {}", GetLastError()));
        }

        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        let mut list = Vec::new();

        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                let null_pos = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..null_pos]);

                if entry.th32ProcessID > 4 {
                    list.push(ProcessInfo {
                        pid: entry.th32ProcessID,
                        name,
                    });
                }

                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
        Ok(list)
    }
}

pub fn trim_process_working_set(pid: u32) -> Result<(), String> {
    unsafe {
        let handle = OpenProcess(
            PROCESS_SET_QUOTA | PROCESS_QUERY_LIMITED_INFORMATION,
            0,
            pid,
        );
        if handle.is_null() {
            return Err(format!("OpenProcess({}) failed: {}", pid, GetLastError()));
        }

        let ret = K32EmptyWorkingSet(handle);
        CloseHandle(handle);

        if ret != 0 {
            Ok(())
        } else {
            Err(format!("EmptyWorkingSet({}) failed: {}", pid, GetLastError()))
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessMemUsage {
    pub pid: u32,
    pub name: String,
    pub working_set_mb: u64,
}

pub fn get_top_memory_processes(limit: usize) -> Vec<ProcessMemUsage> {
    let procs = enumerate_processes().unwrap_or_default();
    let mut usage_list = Vec::with_capacity(procs.len());

    for p in procs {
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, p.pid);
            if !handle.is_null() {
                let mut pmc: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
                let cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
                if K32GetProcessMemoryInfo(handle, &mut pmc, cb) != 0 {
                    let mb = (pmc.WorkingSetSize as u64) / (1024 * 1024);
                    if mb >= 1 {
                        usage_list.push(ProcessMemUsage {
                            pid: p.pid,
                            name: p.name,
                            working_set_mb: mb,
                        });
                    }
                }
                CloseHandle(handle);
            }
        }
    }

    usage_list.sort_by(|a, b| b.working_set_mb.cmp(&a.working_set_mb));
    usage_list.truncate(limit);
    usage_list
}
