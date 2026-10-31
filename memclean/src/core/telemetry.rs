use serde::{Deserialize, Serialize};
use std::ffi::c_void;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use windows_sys::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};

type NtQuerySystemInformationFn = unsafe extern "system" fn(
    system_information_class: u32,
    system_information: *mut c_void,
    system_information_length: u32,
    return_length: *mut u32,
) -> i32;

const SYSTEM_PERFORMANCE_INFORMATION: u32 = 2;
const SYSTEM_PERF_BUF_SIZE: usize = 312;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct SystemPerformanceRaw {
    pub page_fault_count: u32,
    pub page_read_count: u32, // Hard page faults requiring disk fetch
    pub page_read_io_count: u32,
    pub context_switches: u32,
    pub system_calls: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StutterRiskLevel {
    Smooth,       // < 100 hard faults/sec
    Moderate,     // 100 - 500 hard faults/sec
    Elevated,     // 500 - 2,000 hard faults/sec
    Severe,       // > 2,000 hard faults/sec (disk thrashing, heavy frame drops)
}

impl StutterRiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Smooth => "Smooth (Zero Disk Stutter)",
            Self::Moderate => "Moderate (Minor Latency)",
            Self::Elevated => "Elevated (Frame-Drop Risk)",
            Self::Severe => "Severe (Disk Thrashing / Micro-Stutters)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessFaultInfo {
    pub pid: u32,
    pub name: String,
    pub page_fault_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveFaultTelemetry {
    pub sample_duration_ms: u64,
    pub total_page_faults_per_sec: u64,
    pub hard_faults_per_sec: u64,
    pub hard_fault_io_per_sec: u64,
    pub context_switches_per_sec: u64,
    pub stutter_risk: StutterRiskLevel,
    pub top_faulting_processes: Vec<ProcessFaultInfo>,
}

/// Query low-level NT kernel memory performance information
pub fn query_system_performance() -> Result<SystemPerformanceRaw, String> {
    unsafe {
        let h_ntdll = GetModuleHandleA(b"ntdll.dll\0".as_ptr());
        if h_ntdll.is_null() {
            return Err("Failed to obtain ntdll.dll handle".to_string());
        }

        let proc = GetProcAddress(h_ntdll, b"NtQuerySystemInformation\0".as_ptr());
        if proc.is_none() {
            return Err("Failed to resolve NtQuerySystemInformation in ntdll".to_string());
        }

        let nt_query_fn: NtQuerySystemInformationFn = std::mem::transmute(proc);

        let mut buffer = [0u8; SYSTEM_PERF_BUF_SIZE];
        let mut return_length: u32 = 0;

        let status = nt_query_fn(
            SYSTEM_PERFORMANCE_INFORMATION,
            buffer.as_mut_ptr() as _,
            SYSTEM_PERF_BUF_SIZE as u32,
            &mut return_length,
        );

        if status != 0 {
            return Err(format!("NtQuerySystemInformation failed with status: 0x{:08X}", status as u32));
        }

        // Layout offsets in 64-bit Windows:
        // PageFaultCount at offset 44
        // PageReadCount (Hard Faults) at offset 64
        // PageReadIoCount at offset 68
        // ContextSwitches at offset 284
        // SystemCalls at offset 296
        let page_fault_count = u32::from_ne_bytes(buffer[44..48].try_into().unwrap_or([0; 4]));
        let page_read_count = u32::from_ne_bytes(buffer[64..68].try_into().unwrap_or([0; 4]));
        let page_read_io_count = u32::from_ne_bytes(buffer[68..72].try_into().unwrap_or([0; 4]));
        let context_switches = u32::from_ne_bytes(buffer[284..288].try_into().unwrap_or([0; 4]));
        let system_calls = u32::from_ne_bytes(buffer[296..300].try_into().unwrap_or([0; 4]));

        Ok(SystemPerformanceRaw {
            page_fault_count,
            page_read_count,
            page_read_io_count,
            context_switches,
            system_calls,
        })
    }
}

/// Enumerate top processes by cumulative page fault count
pub fn get_top_faulting_processes(limit: usize) -> Vec<ProcessFaultInfo> {
    let mut procs = Vec::new();

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
            return procs;
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
                    let name = String::from_utf16_lossy(&entry.szExeFile[..len]);

                    let h_proc = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
                    if !h_proc.is_null() {
                        let mut pmc: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
                        pmc.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;

                        if GetProcessMemoryInfo(h_proc, &mut pmc, pmc.cb) != 0 {
                            procs.push(ProcessFaultInfo {
                                pid,
                                name,
                                page_fault_count: pmc.PageFaultCount as u32,
                            });
                        }
                        CloseHandle(h_proc);
                    }
                }

                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
    }

    procs.sort_by(|a, b| b.page_fault_count.cmp(&a.page_fault_count));
    if procs.len() > limit {
        procs.truncate(limit);
    }
    procs
}

/// Sample system memory performance over a duration and compute per-second rates
pub fn sample_fault_telemetry(duration: Duration) -> Result<LiveFaultTelemetry, String> {
    let s1 = query_system_performance()?;
    let t1 = Instant::now();

    std::thread::sleep(duration);

    let s2 = query_system_performance()?;
    let elapsed = t1.elapsed().as_secs_f64();
    let sample_duration_ms = t1.elapsed().as_millis() as u64;

    let delta_pf = s2.page_fault_count.wrapping_sub(s1.page_fault_count) as f64;
    let delta_hf = s2.page_read_count.wrapping_sub(s1.page_read_count) as f64;
    let delta_hf_io = s2.page_read_io_count.wrapping_sub(s1.page_read_io_count) as f64;
    let delta_cs = s2.context_switches.wrapping_sub(s1.context_switches) as f64;

    let total_page_faults_per_sec = (delta_pf / elapsed).max(0.0) as u64;
    let hard_faults_per_sec = (delta_hf / elapsed).max(0.0) as u64;
    let hard_fault_io_per_sec = (delta_hf_io / elapsed).max(0.0) as u64;
    let context_switches_per_sec = (delta_cs / elapsed).max(0.0) as u64;

    let stutter_risk = if hard_faults_per_sec > 2000 {
        StutterRiskLevel::Severe
    } else if hard_faults_per_sec > 500 {
        StutterRiskLevel::Elevated
    } else if hard_faults_per_sec > 100 {
        StutterRiskLevel::Moderate
    } else {
        StutterRiskLevel::Smooth
    };

    let top_faulting_processes = get_top_faulting_processes(5);

    Ok(LiveFaultTelemetry {
        sample_duration_ms,
        total_page_faults_per_sec,
        hard_faults_per_sec,
        hard_fault_io_per_sec,
        context_switches_per_sec,
        stutter_risk,
        top_faulting_processes,
    })
}

// Global cached state for zero-overhead background dashboard queries
static LAST_SAMPLE: Mutex<Option<(Instant, SystemPerformanceRaw)>> = Mutex::new(None);

/// Query instantaneous fault rates using cached previous tick with zero sleeping delay
pub fn get_instant_fault_telemetry() -> (u64, StutterRiskLevel) {
    let now = Instant::now();
    let current_raw = match query_system_performance() {
        Ok(raw) => raw,
        Err(_) => return (0, StutterRiskLevel::Smooth),
    };

    let mut lock = LAST_SAMPLE.lock().unwrap();
    if let Some((last_time, last_raw)) = *lock {
        let elapsed = now.duration_since(last_time).as_secs_f64();
        if elapsed >= 0.5 {
            let delta_hf = current_raw.page_read_count.wrapping_sub(last_raw.page_read_count) as f64;
            let hard_faults_per_sec = (delta_hf / elapsed).max(0.0) as u64;
            let risk = if hard_faults_per_sec > 2000 {
                StutterRiskLevel::Severe
            } else if hard_faults_per_sec > 500 {
                StutterRiskLevel::Elevated
            } else if hard_faults_per_sec > 100 {
                StutterRiskLevel::Moderate
            } else {
                StutterRiskLevel::Smooth
            };
            *lock = Some((now, current_raw));
            return (hard_faults_per_sec, risk);
        }
    } else {
        *lock = Some((now, current_raw));
    }

    (0, StutterRiskLevel::Smooth)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_system_performance() {
        let perf = query_system_performance();
        assert!(perf.is_ok(), "Failed to query system performance: {:?}", perf);
        let p = perf.unwrap();
        assert!(p.page_fault_count > 0);
        assert!(p.page_read_count > 0);
    }

    #[test]
    fn test_stutter_risk_levels() {
        assert_eq!(StutterRiskLevel::Smooth.as_str(), "Smooth (Zero Disk Stutter)");
        assert_eq!(StutterRiskLevel::Severe.as_str(), "Severe (Disk Thrashing / Micro-Stutters)");
    }

    #[test]
    fn test_get_top_faulting_processes() {
        let procs = get_top_faulting_processes(3);
        assert!(!procs.is_empty(), "Expected at least one running process with memory counters");
        for p in &procs {
            assert!(p.pid > 0);
            assert!(!p.name.is_empty());
        }
    }
}
