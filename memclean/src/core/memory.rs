use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

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