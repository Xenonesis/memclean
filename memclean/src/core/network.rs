use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegEnumKeyExW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    HKEY, HKEY_LOCAL_MACHINE, KEY_ALL_ACCESS, KEY_READ, REG_DWORD,
};

const SYSTEM_PROFILE_PATH: &str = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile";
const TCP_INTERFACES_PATH: &str = "SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkBackupState {
    pub timestamp: u64,
    pub network_throttling_index: Option<u32>,
    pub system_responsiveness: Option<u32>,
    pub interfaces: Vec<InterfaceBackup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceBackup {
    pub guid: String,
    pub tcp_ack_frequency: Option<u32>,
    pub tcp_no_delay: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub network_throttling_index: Option<u32>,
    pub system_responsiveness: Option<u32>,
    pub is_throttling_disabled: bool,
    pub is_responsiveness_maximized: bool,
    pub active_interfaces_count: usize,
    pub optimized_interfaces_count: usize,
    pub interfaces: Vec<InterfaceStatus>,
    pub backup_exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceStatus {
    pub guid: String,
    pub ip_address: Option<String>,
    pub tcp_ack_frequency: Option<u32>,
    pub tcp_no_delay: Option<u32>,
    pub is_optimized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkOptimizationReport {
    pub throttling_updated: bool,
    pub responsiveness_updated: bool,
    pub interfaces_updated: usize,
    pub backup_saved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRestoreReport {
    pub throttling_restored: bool,
    pub responsiveness_restored: bool,
    pub interfaces_restored: usize,
    pub used_backup: bool,
}

pub fn get_backup_path() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata).join("MemClean").join("network_backup.toml")
    } else {
        PathBuf::from("network_backup.toml")
    }
}

// Helper: Open a registry key under HKLM
unsafe fn open_key(sub_path: &str, access: u32) -> Option<HKEY> {
    let wide_path: Vec<u16> = sub_path.encode_utf16().chain(std::iter::once(0)).collect();
    let mut hkey: HKEY = std::mem::zeroed();
    let res = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        wide_path.as_ptr(),
        0,
        access,
        &mut hkey,
    );
    if res == 0 && hkey != 0 as _ {
        Some(hkey)
    } else {
        None
    }
}

// Helper: Read a DWORD value
unsafe fn read_dword(hkey: HKEY, value_name: &str) -> Option<u32> {
    let wide_name: Vec<u16> = value_name.encode_utf16().chain(std::iter::once(0)).collect();
    let mut val: u32 = 0;
    let mut val_type: u32 = 0;
    let mut val_size: u32 = std::mem::size_of::<u32>() as u32;

    let res = RegQueryValueExW(
        hkey,
        wide_name.as_ptr(),
        std::ptr::null_mut(),
        &mut val_type,
        &mut val as *mut u32 as _,
        &mut val_size,
    );
    if res == 0 && val_type == REG_DWORD {
        Some(val)
    } else {
        None
    }
}

// Helper: Read a string value
unsafe fn read_string(hkey: HKEY, value_name: &str) -> Option<String> {
    let wide_name: Vec<u16> = value_name.encode_utf16().chain(std::iter::once(0)).collect();
    let mut buf = [0u16; 256];
    let mut val_type: u32 = 0;
    let mut val_size: u32 = (buf.len() * 2) as u32;

    let res = RegQueryValueExW(
        hkey,
        wide_name.as_ptr(),
        std::ptr::null_mut(),
        &mut val_type,
        buf.as_mut_ptr() as _,
        &mut val_size,
    );
    if res == 0 && (val_type == 1 || val_type == 2) {
        let len = (val_size as usize / 2).saturating_sub(1);
        let s = String::from_utf16_lossy(&buf[..len]);
        if !s.trim().is_empty() {
            return Some(s.trim().to_string());
        }
    }
    None
}

// Helper: Write a DWORD value
unsafe fn write_dword(hkey: HKEY, value_name: &str, val: u32) -> bool {
    let wide_name: Vec<u16> = value_name.encode_utf16().chain(std::iter::once(0)).collect();
    let res = RegSetValueExW(
        hkey,
        wide_name.as_ptr(),
        0,
        REG_DWORD,
        &val as *const u32 as _,
        std::mem::size_of::<u32>() as u32,
    );
    res == 0
}

// Helper: Delete a value
unsafe fn delete_value(hkey: HKEY, value_name: &str) -> bool {
    let wide_name: Vec<u16> = value_name.encode_utf16().chain(std::iter::once(0)).collect();
    let res = RegDeleteValueW(hkey, wide_name.as_ptr());
    res == 0
}

// Enumerate interface GUIDs under TCP_INTERFACES_PATH
unsafe fn list_interface_guids() -> Vec<String> {
    let mut guids = Vec::new();
    if let Some(hkey) = open_key(TCP_INTERFACES_PATH, KEY_READ) {
        let mut index = 0;
        let mut name_buf = [0u16; 256];
        loop {
            let mut name_len = name_buf.len() as u32;
            let ret = RegEnumKeyExW(
                hkey,
                index,
                name_buf.as_mut_ptr(),
                &mut name_len,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            if ret != 0 {
                break;
            }
            let guid_str = String::from_utf16_lossy(&name_buf[..name_len as usize]);
            guids.push(guid_str);
            index += 1;
        }
        RegCloseKey(hkey);
    }
    guids
}

/// Query current system network settings
pub fn get_network_status() -> NetworkStatus {
    unsafe {
        let mut throttling = None;
        let mut responsiveness = None;

        if let Some(hkey) = open_key(SYSTEM_PROFILE_PATH, KEY_READ) {
            throttling = read_dword(hkey, "NetworkThrottlingIndex");
            responsiveness = read_dword(hkey, "SystemResponsiveness");
            RegCloseKey(hkey);
        }

        let is_throttling_disabled = throttling == Some(0xFFFFFFFF);
        let is_responsiveness_maximized = responsiveness == Some(0);

        let guids = list_interface_guids();
        let mut interfaces = Vec::new();
        let mut active_count = 0;
        let mut optimized_count = 0;

        for guid in guids {
            let sub_path = format!("{}\\{}", TCP_INTERFACES_PATH, guid);
            if let Some(hkey) = open_key(&sub_path, KEY_READ) {
                let ip = read_string(hkey, "DhcpIPAddress")
                    .or_else(|| read_string(hkey, "IPAddress"));
                let ack = read_dword(hkey, "TcpAckFrequency");
                let nagle = read_dword(hkey, "TCPNoDelay");

                let has_ip = ip.as_ref().map(|s| s != "0.0.0.0" && !s.is_empty()).unwrap_or(false);
                if has_ip {
                    active_count += 1;
                }

                let is_opt = ack == Some(1) && nagle == Some(1);
                if is_opt {
                    optimized_count += 1;
                }

                interfaces.push(InterfaceStatus {
                    guid,
                    ip_address: ip,
                    tcp_ack_frequency: ack,
                    tcp_no_delay: nagle,
                    is_optimized: is_opt,
                });
                RegCloseKey(hkey);
            }
        }

        let backup_exists = get_backup_path().exists();

        NetworkStatus {
            network_throttling_index: throttling,
            system_responsiveness: responsiveness,
            is_throttling_disabled,
            is_responsiveness_maximized,
            active_interfaces_count: active_count,
            optimized_interfaces_count: optimized_count,
            interfaces,
            backup_exists,
        }
    }
}

/// Optimize network settings for ultra-low latency & gaming
pub fn optimize_network() -> Result<NetworkOptimizationReport, String> {
    unsafe {
        // 1. Create backup if not already present
        let backup_path = get_backup_path();
        let mut backup_saved = false;
        if !backup_path.exists() {
            let current = get_network_status();
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let backup_state = NetworkBackupState {
                timestamp,
                network_throttling_index: current.network_throttling_index,
                system_responsiveness: current.system_responsiveness,
                interfaces: current
                    .interfaces
                    .iter()
                    .map(|iface| InterfaceBackup {
                        guid: iface.guid.clone(),
                        tcp_ack_frequency: iface.tcp_ack_frequency,
                        tcp_no_delay: iface.tcp_no_delay,
                    })
                    .collect(),
            };

            if let Some(parent) = backup_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(toml_str) = toml::to_string_pretty(&backup_state) {
                let _ = std::fs::write(&backup_path, toml_str);
                backup_saved = true;
            }
        }

        // 2. Apply Multimedia SystemProfile tweaks
        let (throttling_updated, responsiveness_updated) = if let Some(hkey) = open_key(SYSTEM_PROFILE_PATH, KEY_ALL_ACCESS) {
            let t = write_dword(hkey, "NetworkThrottlingIndex", 0xFFFFFFFF);
            let r = write_dword(hkey, "SystemResponsiveness", 0);
            RegCloseKey(hkey);
            (t, r)
        } else {
            return Err("Failed to open SystemProfile registry key for writing (Administrator privileges required)".to_string());
        };

        // 3. Apply TCP stack tweaks to all active / valid interfaces
        let guids = list_interface_guids();
        let mut interfaces_updated = 0;

        for guid in guids {
            let sub_path = format!("{}\\{}", TCP_INTERFACES_PATH, guid);
            if let Some(hkey) = open_key(&sub_path, KEY_ALL_ACCESS) {
                let ack_ok = write_dword(hkey, "TcpAckFrequency", 1);
                let nagle_ok = write_dword(hkey, "TCPNoDelay", 1);
                if ack_ok && nagle_ok {
                    interfaces_updated += 1;
                }
                RegCloseKey(hkey);
            }
        }

        Ok(NetworkOptimizationReport {
            throttling_updated,
            responsiveness_updated,
            interfaces_updated,
            backup_saved,
        })
    }
}

/// Restore original Windows network defaults
pub fn restore_network_defaults() -> Result<NetworkRestoreReport, String> {
    unsafe {
        let backup_path = get_backup_path();
        let backup_opt = if backup_path.exists() {
            std::fs::read_to_string(&backup_path)
                .ok()
                .and_then(|s| toml::from_str::<NetworkBackupState>(&s).ok())
        } else {
            None
        };

        // 1. Restore SystemProfile
        let (throttling_restored, responsiveness_restored) = if let Some(hkey) = open_key(SYSTEM_PROFILE_PATH, KEY_ALL_ACCESS) {
            let target_throttle = backup_opt
                .as_ref()
                .and_then(|b| b.network_throttling_index)
                .unwrap_or(10); // Standard Windows default is 10
            let target_resp = backup_opt
                .as_ref()
                .and_then(|b| b.system_responsiveness)
                .unwrap_or(20); // Standard Windows default is 20

            let t = write_dword(hkey, "NetworkThrottlingIndex", target_throttle);
            let r = write_dword(hkey, "SystemResponsiveness", target_resp);
            RegCloseKey(hkey);
            (t, r)
        } else {
            return Err("Failed to open SystemProfile registry key for writing (Administrator privileges required)".to_string());
        };

        // 2. Restore Interfaces
        let mut interfaces_restored = 0;
        let guids = list_interface_guids();

        for guid in guids {
            let sub_path = format!("{}\\{}", TCP_INTERFACES_PATH, guid);
            if let Some(hkey) = open_key(&sub_path, KEY_ALL_ACCESS) {
                let matched_backup = backup_opt
                    .as_ref()
                    .and_then(|b| b.interfaces.iter().find(|i| i.guid == guid));

                if let Some(ib) = matched_backup {
                    if let Some(ack) = ib.tcp_ack_frequency {
                        let _ = write_dword(hkey, "TcpAckFrequency", ack);
                    } else {
                        let _ = delete_value(hkey, "TcpAckFrequency");
                    }

                    if let Some(nagle) = ib.tcp_no_delay {
                        let _ = write_dword(hkey, "TCPNoDelay", nagle);
                    } else {
                        let _ = delete_value(hkey, "TCPNoDelay");
                    }
                } else {
                    // Default fallback: remove the tweaks so Windows uses default behavior
                    let _ = delete_value(hkey, "TcpAckFrequency");
                    let _ = delete_value(hkey, "TCPNoDelay");
                }
                interfaces_restored += 1;
                RegCloseKey(hkey);
            }
        }

        let used_backup = backup_opt.is_some();
        if backup_path.exists() {
            let _ = std::fs::remove_file(backup_path);
        }

        Ok(NetworkRestoreReport {
            throttling_restored,
            responsiveness_restored,
            interfaces_restored,
            used_backup,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_backup_roundtrip() {
        let backup = NetworkBackupState {
            timestamp: 123456789,
            network_throttling_index: Some(10),
            system_responsiveness: Some(20),
            interfaces: vec![InterfaceBackup {
                guid: "{TEST-GUID-1234}".to_string(),
                tcp_ack_frequency: None,
                tcp_no_delay: Some(1),
            }],
        };

        let serialized = toml::to_string(&backup).expect("Serialization failed");
        let deserialized: NetworkBackupState =
            toml::from_str(&serialized).expect("Deserialization failed");

        assert_eq!(deserialized.timestamp, 123456789);
        assert_eq!(deserialized.network_throttling_index, Some(10));
        assert_eq!(deserialized.system_responsiveness, Some(20));
        assert_eq!(deserialized.interfaces.len(), 1);
        assert_eq!(deserialized.interfaces[0].guid, "{TEST-GUID-1234}");
        assert_eq!(deserialized.interfaces[0].tcp_ack_frequency, None);
        assert_eq!(deserialized.interfaces[0].tcp_no_delay, Some(1));
    }

    #[test]
    fn test_network_status_query() {
        // Read-only query should not fail or panic
        let status = get_network_status();
        assert!(status.active_interfaces_count <= status.interfaces.len());
    }
}
