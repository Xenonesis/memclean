use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, GetDriveTypeW, OPEN_EXISTING,
};
use windows_sys::Win32::System::IO::DeviceIoControl;
use windows_sys::Win32::System::Ioctl::{
    STORAGE_PROPERTY_QUERY, StorageDeviceSeekPenaltyProperty,
};

/// IOCTL_STORAGE_QUERY_PROPERTY is gated behind a Cargo feature in windows-sys 0.59,
/// so we define it locally rather than enabling the whole module.
const IOCTL_STORAGE_QUERY_PROPERTY: u32 = 0x002D1400;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DriveType {
    Ssd,
    Hdd,
    Unknown,
}

/// Detects whether the system drive (C:) is an SSD or HDD.
///
/// Uses `GetDriveTypeW` to confirm a fixed disk, then
/// `IOCTL_STORAGE_QUERY_PROPERTY` with `StorageDeviceSeekPenaltyDescriptor`.
/// `SeekPenalty = 1` means mechanical (HDD); `0` means SSD (no seek penalty).
/// Falls back to `Unknown` on any error (e.g. no admin handle).
pub fn detect_system_drive_type() -> DriveType {
    unsafe {
        let root = windows_sys::w!("C:\\");
        let dtype = GetDriveTypeW(root);
        if dtype != 3 {
            // DRIVE_FIXED = 3
            return DriveType::Unknown;
        }

        let handle = CreateFileW(
            windows_sys::w!("\\\\.\\C:"),
            0, // No data access needed; IOCTL query works with metadata-only handle.
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        );

        if handle == INVALID_HANDLE_VALUE {
            return DriveType::Unknown;
        }

        let mut query: STORAGE_PROPERTY_QUERY = std::mem::zeroed();
        query.PropertyId = StorageDeviceSeekPenaltyProperty as i32;
        query.QueryType = 0; // STORAGE_QUERY_TYPE::PropertyStandardQuery

        // STORAGE_DEVICE_SEEK_PENALTY_DESCRIPTOR: Version(4) + Size(4) + SeekPenalty(1) + ...
        let mut out: [u8; 16] = [0; 16];
        let mut returned: u32 = 0;

        let ok = DeviceIoControl(
            handle,
            IOCTL_STORAGE_QUERY_PROPERTY,
            &query as *const _ as *const _,
            std::mem::size_of::<STORAGE_PROPERTY_QUERY>() as u32,
            out.as_mut_ptr() as *mut _,
            out.len() as u32,
            &mut returned,
            std::ptr::null_mut(),
        );

        CloseHandle(handle);

        if ok != 0 && returned >= 9 {
            // Offset 8 is the SeekPenalty BOOLEAN byte.
            let seek_penalty = out[8] != 0;
            if seek_penalty {
                DriveType::Hdd
            } else {
                DriveType::Ssd
            }
        } else {
            DriveType::Unknown
        }
    }
}

pub fn drive_type_label(dt: DriveType) -> &'static str {
    match dt {
        DriveType::Ssd => "SSD",
        DriveType::Hdd => "HDD",
        DriveType::Unknown => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_system_drive_without_panicking() {
        let _ = detect_system_drive_type();
    }
}
