use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
use crate::core::ntdll;

#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    pub total_phys_mb: u64,
    pub avail_phys_mb: u64,
    pub used_phys_mb: u64,
    pub used_percent: u32,
}

pub fn get_memory_stats() -> Result<MemoryStats, String> {
    unsafe {
        let mut status: MEMORYSTATUSEX = std::mem::zeroed();
        status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;

        if GlobalMemoryStatusEx(&mut status) == 0 {
            return Err(format!("GlobalMemoryStatusEx failed: {}", GetLastError()));
        }

        let total_mb = status.ullTotalPhys / (1024 * 1024);
        let avail_mb = status.ullAvailPhys / (1024 * 1024);
        let used_mb = total_mb.saturating_sub(avail_mb);
        let percent = status.dwMemoryLoad;

        Ok(MemoryStats {
            total_phys_mb: total_mb,
            avail_phys_mb: avail_mb,
            used_phys_mb: used_mb,
            used_percent: percent,
        })
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ExtendedMemoryStats {
    pub total_phys_mb: u64,
    pub used_phys_mb: u64,
    pub free_phys_mb: u64,
    pub zeroed_phys_mb: u64,
    pub standby_total_mb: u64,
    pub standby_by_priority_mb: [u64; 8],
    pub modified_mb: u64,
    pub used_percent: u32,
}

pub fn get_extended_memory_stats() -> Result<ExtendedMemoryStats, String> {
    let base = get_memory_stats()?;
    let nt_list = ntdll::query_system_memory_list()?;

    let page_kb = 4u64;
    let free_mb = (nt_list.free_page_count as u64 * page_kb) / 1024;
    let zeroed_mb = (nt_list.zero_page_count as u64 * page_kb) / 1024;
    let modified_mb = (nt_list.modified_page_count as u64 * page_kb) / 1024;

    let mut standby_total_pages = 0usize;
    let mut standby_by_priority_mb = [0u64; 8];
    for (i, &count) in nt_list.page_count_by_priority.iter().enumerate() {
        standby_total_pages += count;
        standby_by_priority_mb[i] = (count as u64 * page_kb) / 1024;
    }
    let standby_total_mb = (standby_total_pages as u64 * page_kb) / 1024;

    Ok(ExtendedMemoryStats {
        total_phys_mb: base.total_phys_mb,
        used_phys_mb: base.used_phys_mb,
        free_phys_mb: free_mb,
        zeroed_phys_mb: zeroed_mb,
        standby_total_mb,
        standby_by_priority_mb,
        modified_mb,
        used_percent: base.used_percent,
    })
}