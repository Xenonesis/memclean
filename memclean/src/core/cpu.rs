use serde::{Deserialize, Serialize};
use std::process::Command;
use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::SystemInformation::GetLogicalProcessorInformationEx;
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentProcessId, GetProcessAffinityMask, OpenProcess,
    SetProcessAffinityMask, PROCESS_ALL_ACCESS, PROCESS_QUERY_INFORMATION,
    PROCESS_SET_INFORMATION,
};

const RELATION_PROCESSOR_CORE: i32 = 0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreInfo {
    pub core_index: usize,
    pub is_performance: bool,
    pub efficiency_class: u8,
    pub smt_enabled: bool,
    pub affinity_mask: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuTopology {
    pub total_physical_cores: usize,
    pub total_logical_processors: usize,
    pub p_cores_count: usize,
    pub e_cores_count: usize,
    pub is_hybrid: bool,
    pub all_cores_mask: usize,
    pub p_cores_mask: usize,
    pub e_cores_mask: usize,
    pub cores: Vec<CoreInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HybridAffinityReport {
    pub game_pid: u32,
    pub game_locked_to_p_cores: bool,
    pub background_processes_clamped: usize,
    pub p_cores_mask: usize,
    pub e_cores_mask: usize,
}

/// Query hardware CPU core topology and compute P-Core and E-Core affinity masks
pub fn get_cpu_topology() -> CpuTopology {
    let mut length: u32 = 0;
    unsafe {
        GetLogicalProcessorInformationEx(RELATION_PROCESSOR_CORE, std::ptr::null_mut(), &mut length);
    }

    if length == 0 {
        return fallback_topology();
    }

    let mut buffer: Vec<u8> = vec![0; length as usize];
    let success = unsafe {
        GetLogicalProcessorInformationEx(
            RELATION_PROCESSOR_CORE,
            buffer.as_mut_ptr() as _,
            &mut length,
        )
    };

    if success == 0 {
        return fallback_topology();
    }

    let mut cores = Vec::new();
    let mut offset = 0;
    let mut p_count = 0;
    let mut e_count = 0;
    let mut all_mask: usize = 0;
    let mut p_mask: usize = 0;
    let mut e_mask: usize = 0;
    let mut logical_proc_count = 0;

    while offset + 8 <= buffer.len() {
        let rel = i32::from_ne_bytes(buffer[offset..offset + 4].try_into().unwrap_or([0; 4]));
        let size = u32::from_ne_bytes(buffer[offset + 4..offset + 8].try_into().unwrap_or([0; 4])) as usize;

        if size == 0 || offset + size > buffer.len() {
            break;
        }

        if rel == RELATION_PROCESSOR_CORE && size >= 48 {
            let flags = buffer[offset + 8];
            let eff_class = buffer[offset + 9];
            let smt = (flags & 1) != 0;

            // GROUP_AFFINITY starts at offset 32: Mask is KAFFINITY (usize / 8 bytes)
            let mask_bytes = &buffer[offset + 32..offset + 40];
            let mask = usize::from_ne_bytes(mask_bytes.try_into().unwrap_or([0; 8]));

            let is_p = eff_class > 0;
            if is_p {
                p_count += 1;
                p_mask |= mask;
            } else {
                e_count += 1;
                e_mask |= mask;
            }

            all_mask |= mask;
            logical_proc_count += mask.count_ones() as usize;

            cores.push(CoreInfo {
                core_index: cores.len(),
                is_performance: is_p,
                efficiency_class: eff_class,
                smt_enabled: smt,
                affinity_mask: mask,
            });
        }

        offset += size;
    }

    let is_hybrid = p_count > 0 && e_count > 0;

    // If non-hybrid (e.g. traditional monolithic CPU), all cores are performance cores
    if !is_hybrid {
        p_mask = all_mask;
        e_mask = all_mask;
    }

    CpuTopology {
        total_physical_cores: cores.len(),
        total_logical_processors: logical_proc_count,
        p_cores_count: p_count,
        e_cores_count: e_count,
        is_hybrid,
        all_cores_mask: all_mask,
        p_cores_mask: p_mask,
        e_cores_mask: e_mask,
        cores,
    }
}

fn fallback_topology() -> CpuTopology {
    let mut num_cores = 1;
    unsafe {
        let mut sys_info: windows_sys::Win32::System::SystemInformation::SYSTEM_INFO = std::mem::zeroed();
        windows_sys::Win32::System::SystemInformation::GetSystemInfo(&mut sys_info);
        if sys_info.dwNumberOfProcessors > 0 {
            num_cores = sys_info.dwNumberOfProcessors as usize;
        }
    }

    let mask = if num_cores >= 64 {
        usize::MAX
    } else {
        (1usize << num_cores) - 1
    };

    CpuTopology {
        total_physical_cores: num_cores,
        total_logical_processors: num_cores,
        p_cores_count: num_cores,
        e_cores_count: 0,
        is_hybrid: false,
        all_cores_mask: mask,
        p_cores_mask: mask,
        e_cores_mask: mask,
        cores: Vec::new(),
    }
}

/// Set process affinity mask for a given PID
pub fn set_process_affinity(pid: u32, mask: usize) -> Result<(), String> {
    unsafe {
        let h_proc = if pid == GetCurrentProcessId() {
            GetCurrentProcess()
        } else {
            let h = OpenProcess(PROCESS_SET_INFORMATION | PROCESS_QUERY_INFORMATION, 0, pid);
            if h.is_null() {
                OpenProcess(PROCESS_ALL_ACCESS, 0, pid)
            } else {
                h
            }
        };

        if h_proc.is_null() {
            return Err(format!("Failed to open process PID {} for affinity assignment", pid));
        }

        let res = SetProcessAffinityMask(h_proc, mask);

        if pid != GetCurrentProcessId() && !h_proc.is_null() {
            CloseHandle(h_proc);
        }

        if res != 0 {
            Ok(())
        } else {
            Err(format!("SetProcessAffinityMask failed for PID {}", pid))
        }
    }
}

/// Get process and system affinity masks for a given PID
pub fn get_process_affinity(pid: u32) -> Result<(usize, usize), String> {
    unsafe {
        let h_proc = if pid == GetCurrentProcessId() {
            GetCurrentProcess()
        } else {
            let h = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
            if h.is_null() {
                OpenProcess(PROCESS_ALL_ACCESS, 0, pid)
            } else {
                h
            }
        };

        if h_proc.is_null() {
            return Err(format!("Failed to open process PID {} for affinity query", pid));
        }

        let mut proc_mask: usize = 0;
        let mut sys_mask: usize = 0;
        let res = GetProcessAffinityMask(h_proc, &mut proc_mask as *mut usize as _, &mut sys_mask as *mut usize as _);

        if pid != GetCurrentProcessId() && !h_proc.is_null() {
            CloseHandle(h_proc);
        }

        if res != 0 {
            Ok((proc_mask, sys_mask))
        } else {
            Err(format!("GetProcessAffinityMask failed for PID {}", pid))
        }
    }
}

/// Lock foreground game to Performance Cores (P-Cores) and clamp background bloat to Efficiency Cores (E-Cores)
pub fn optimize_hybrid_affinity(game_pid: u32, exclusions: &[String]) -> HybridAffinityReport {
    let mut report = HybridAffinityReport::default();
    let topo = get_cpu_topology();

    report.game_pid = game_pid;
    report.p_cores_mask = topo.p_cores_mask;
    report.e_cores_mask = topo.e_cores_mask;

    // 1. Lock game to P-Cores
    if topo.p_cores_mask != 0 {
        if set_process_affinity(game_pid, topo.p_cores_mask).is_ok() {
            report.game_locked_to_p_cores = true;
        }
    }

    // 2. If hybrid architecture exists, clamp non-critical background processes to E-Cores
    if topo.is_hybrid && topo.e_cores_mask != 0 {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot != INVALID_HANDLE_VALUE {
                let mut entry: PROCESSENTRY32W = std::mem::zeroed();
                entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

                if Process32FirstW(snapshot, &mut entry) != 0 {
                    loop {
                        let pid = entry.th32ProcessID;
                        if pid > 4 && pid != game_pid && pid != GetCurrentProcessId() {
                            let len = entry
                                .szExeFile
                                .iter()
                                .position(|&c| c == 0)
                                .unwrap_or(entry.szExeFile.len());
                            let name = String::from_utf16_lossy(&entry.szExeFile[..len]).to_lowercase();

                            let is_excluded = exclusions.iter().any(|ex| ex.to_lowercase() == name);
                            if !is_excluded {
                                if set_process_affinity(pid, topo.e_cores_mask).is_ok() {
                                    report.background_processes_clamped += 1;
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
        }
    }

    report
}

/// Restore all non-protected processes back to the default all-cores affinity mask
pub fn restore_all_process_affinity(exclusions: &[String]) -> usize {
    let topo = get_cpu_topology();
    let mask = topo.all_cores_mask;
    let mut restored = 0;

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot != INVALID_HANDLE_VALUE {
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
                            if set_process_affinity(pid, mask).is_ok() {
                                restored += 1;
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
    }

    restored
}

/// Unpark 100% of CPU cores (sets CPMINCORES to 100%) for zero latency latency spikes
pub fn unpark_all_cores() -> Result<(), String> {
    let status1 = Command::new("powercfg")
        .args(&["-setacvalueindex", "scheme_current", "sub_processor", "CPMINCORES", "100"])
        .status()
        .map_err(|e| format!("powercfg command failed: {}", e))?;

    let status2 = Command::new("powercfg")
        .args(&["-setactive", "scheme_current"])
        .status()
        .map_err(|e| format!("powercfg setactive failed: {}", e))?;

    if status1.success() && status2.success() {
        Ok(())
    } else {
        Err("Failed to set core parking value to 100%".to_string())
    }
}

/// Restore default Windows core parking behavior (sets CPMINCORES to 0%)
pub fn restore_core_parking_default() -> Result<(), String> {
    let status1 = Command::new("powercfg")
        .args(&["-setacvalueindex", "scheme_current", "sub_processor", "CPMINCORES", "0"])
        .status()
        .map_err(|e| format!("powercfg command failed: {}", e))?;

    let status2 = Command::new("powercfg")
        .args(&["-setactive", "scheme_current"])
        .status()
        .map_err(|e| format!("powercfg setactive failed: {}", e))?;

    if status1.success() && status2.success() {
        Ok(())
    } else {
        Err("Failed to restore core parking value".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_topology_query() {
        let topo = get_cpu_topology();
        assert!(topo.total_physical_cores >= 1);
        assert!(topo.total_logical_processors >= topo.total_physical_cores);
        assert!(topo.all_cores_mask > 0);
        if topo.is_hybrid {
            assert!(topo.p_cores_count > 0);
            assert!(topo.e_cores_count > 0);
            assert_ne!(topo.p_cores_mask, topo.e_cores_mask);
            assert_eq!(topo.all_cores_mask, topo.p_cores_mask | topo.e_cores_mask);
        }
    }

    #[test]
    fn test_self_process_affinity_roundtrip() {
        let pid = std::process::id();
        let (initial_mask, sys_mask) = get_process_affinity(pid).expect("Failed to get process affinity");
        assert!(initial_mask > 0);
        assert!(sys_mask > 0);

        let topo = get_cpu_topology();
        let target_mask = if topo.p_cores_mask != 0 {
            topo.p_cores_mask
        } else {
            initial_mask
        };

        // Set to target mask
        let set_res = set_process_affinity(pid, target_mask);
        assert!(set_res.is_ok(), "Setting affinity failed: {:?}", set_res);

        let (new_mask, _) = get_process_affinity(pid).expect("Failed to get updated affinity");
        assert_eq!(new_mask, target_mask);

        // Restore to initial mask
        let restore_res = set_process_affinity(pid, initial_mask);
        assert!(restore_res.is_ok(), "Restoring affinity failed: {:?}", restore_res);
    }
}
