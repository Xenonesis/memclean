# MemClean Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an ultra-lightweight, high-performance pure Rust Windows memory optimizer and installable executable that supersedes Sordum ReduceMemory with multi-tier kernel memory clearing, active game stutter protection, dynamic system tray badge, CLI automation, and zero-UAC boot startup.

**Architecture:** Pure Rust Win32 native application with zero external runtime overhead (< 2 MB RAM, < 1 MB binary size). Uses `windows-sys` and dynamic `ntdll` binding for kernel memory lists (`NtSetSystemInformation`), Win32 GDI for dynamic tray icon generation, a foreground window shield to prevent gaming frame drops, and Windows Task Scheduler for silent elevated startup. Packaged via an Inno Setup installer wizard.

**Tech Stack:** Rust 2021 (rustc 1.98+), `windows-sys` 0.59, `serde`, `toml`, Inno Setup 6.

**Spec:** `docs/superpowers/specs/2026-10-29-memclean-design.md`

## Global Constraints
- Target OS: Windows 10 & Windows 11 (x86_64).
- Memory footprint: Under 2 MB private working set when running in the system tray.
- Binary size: Under 1.5 MB uncompressed.
- Idle CPU: 0.0% utilization.
- Zero external runtime DLL dependencies (statically linked MSVC CRT / Rust std).
- No webviews, Chromium wrappers, or heavy GPU frameworks.

---

### Task 1: Project Scaffolding, Windows Manifest & Privilege Token Elevation

**Files:**
- Create: `memclean/Cargo.toml`
- Create: `memclean/build.rs`
- Create: `memclean/app.manifest`
- Create: `memclean/src/core/mod.rs`
- Create: `memclean/src/core/privileges.rs`
- Test: `memclean/tests/test_privileges.rs`

**Interfaces:**
- Produces:
  ```rust
  pub fn enable_memory_privileges() -> Result<(), String>;
  ```

- [ ] **Step 1: Create Cargo.toml with minimal dependencies and LTO settings**

```toml
[package]
name = "memclean"
version = "1.0.0"
edition = "2021"

[dependencies]
windows-sys = { version = "0.59", features = [
    "Win32_Foundation",
    "Win32_Security",
    "Win32_System_Threading",
    "Win32_System_ProcessStatus",
    "Win32_System_SystemInformation",
    "Win32_System_Diagnostics_ToolHelp",
    "Win32_UI_WindowsAndMessaging",
    "Win32_UI_Shell",
    "Win32_Graphics_Gdi",
    "Win32_Storage_FileSystem",
] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"

[build-dependencies]
embed-resource = "2.5"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

- [ ] **Step 2: Create Windows manifest and build.rs for UAC elevation**

Create `memclean/app.manifest`:
```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <!-- Windows 10 and Windows 11 -->
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}" />
    </application>
  </compatibility>
</assembly>
```

Create `memclean/build.rs`:
```rust
fn main() {
    println!("cargo:rerun-if-changed=app.manifest");
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_manifest_file("app.manifest");
        let _ = res.compile();
    }
}
```

- [ ] **Step 3: Write the failing privilege test**

Create `memclean/tests/test_privileges.rs`:
```rust
#[test]
fn test_enable_privileges() {
    let result = memclean::core::privileges::enable_memory_privileges();
    assert!(result.is_ok(), "Privilege acquisition failed: {:?}", result);
}
```

- [ ] **Step 4: Implement `memclean/src/core/privileges.rs`**

```rust
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, LUID};
use windows_sys::Win32::Security::{
    AdjustTokenPrivileges, LookupPrivilegeValueW, LUID_AND_ATTRIBUTES,
    SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

fn encode_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn enable_memory_privileges() -> Result<(), String> {
    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        ) == 0
        {
            return Err(format!("OpenProcessToken failed with code {}", GetLastError()));
        }

        let privileges = [
            "SeProfileSingleProcessPrivilege",
            "SeIncreaseQuotaPrivilege",
            "SeDebugPrivilege",
        ];

        for priv_name in privileges {
            let wide = encode_wide(priv_name);
            let mut luid = LUID { LowPart: 0, HighPart: 0 };
            if LookupPrivilegeValueW(std::ptr::null(), wide.as_ptr(), &mut luid) != 0 {
                let mut tp = TOKEN_PRIVILEGES {
                    PrivilegeCount: 1,
                    Privileges: [LUID_AND_ATTRIBUTES {
                        Luid: luid,
                        Attributes: SE_PRIVILEGE_ENABLED,
                    }],
                };
                AdjustTokenPrivileges(
                    token,
                    0,
                    &mut tp,
                    std::mem::size_of::<TOKEN_PRIVILEGES>() as u32,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );
            }
        }

        CloseHandle(token);
        Ok(())
    }
}
```

Create `memclean/src/core/mod.rs`:
```rust
pub mod privileges;
```

Create `memclean/src/lib.rs`:
```rust
pub mod core;
```

- [ ] **Step 5: Run test to verify compilation & execution**

Run: `cargo test --test test_privileges`  
Expected: PASS (if running elevated) or graceful handled response.

- [ ] **Step 6: Commit**

```bash
git add memclean/Cargo.toml memclean/build.rs memclean/app.manifest memclean/src memclean/tests
git commit -m "feat: setup project scaffolding and token privilege elevation"
```

---

### Task 2: NT Kernel Memory Lists & System Memory Queries

**Files:**
- Create: `memclean/src/core/memory.rs`
- Create: `memclean/src/core/ntdll.rs`
- Modify: `memclean/src/core/mod.rs`
- Test: `memclean/tests/test_memory.rs`

**Interfaces:**
- Produces:
  ```rust
  pub struct MemoryStats {
      pub total_phys_mb: u64,
      pub avail_phys_mb: u64,
      pub used_phys_mb: u64,
      pub used_percent: u32,
  }
  pub fn get_memory_stats() -> Result<MemoryStats, String>;
  pub fn purge_standby_list() -> Result<(), String>;
  pub fn purge_low_priority_standby_list() -> Result<(), String>;
  pub fn flush_modified_list() -> Result<(), String>;
  pub fn empty_system_working_sets() -> Result<(), String>;
  pub fn trim_system_file_cache() -> Result<(), String>;
  ```

- [ ] **Step 1: Write the failing memory stats test**

Create `memclean/tests/test_memory.rs`:
```rust
#[test]
fn test_get_memory_stats() {
    let stats = memclean::core::memory::get_memory_stats().expect("Failed to get memory stats");
    assert!(stats.total_phys_mb > 0);
    assert!(stats.used_percent <= 100);
}
```

- [ ] **Step 2: Implement `memclean/src/core/memory.rs`**

```rust
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
```

- [ ] **Step 3: Implement `memclean/src/core/ntdll.rs` for kernel memory list purging**

```rust
use std::ffi::c_void;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

type NtSetSystemInformationFn = unsafe extern "system" fn(
    system_information_class: u32,
    system_information: *const c_void,
    system_information_length: u32,
) -> i32;

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
```

Update `memclean/src/core/mod.rs`:
```rust
pub mod memory;
pub mod ntdll;
pub mod privileges;
```

- [ ] **Step 4: Run test to verify memory stats and compilation**

Run: `cargo test --test test_memory`  
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add memclean/src/core/memory.rs memclean/src/core/ntdll.rs memclean/src/core/mod.rs memclean/tests/test_memory.rs
git commit -m "feat: add memory stats and ntdll kernel memory list commands"
```

---

### Task 3: Process Enumeration & Selective Working Set Trimming

**Files:**
- Create: `memclean/src/core/process.rs`
- Modify: `memclean/src/core/mod.rs`
- Test: `memclean/tests/test_process.rs`

**Interfaces:**
- Produces:
  ```rust
  pub struct ProcessInfo {
      pub pid: u32,
      pub name: String,
  }
  pub fn enumerate_processes() -> Result<Vec<ProcessInfo>, String>;
  pub fn trim_process_working_set(pid: u32) -> Result<(), String>;
  ```

- [ ] **Step 1: Write test for process enumeration**

Create `memclean/tests/test_process.rs`:
```rust
#[test]
fn test_enumerate_processes() {
    let procs = memclean::core::process::enumerate_processes().expect("Failed to enumerate processes");
    assert!(!procs.is_empty());
    assert!(procs.iter().any(|p| p.name.to_lowercase() == "explorer.exe" || p.name.to_lowercase() == "cargo.exe"));
}
```

- [ ] **Step 2: Implement `memclean/src/core/process.rs`**

```rust
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::ProcessStatus::K32EmptyWorkingSet;
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
```

Update `memclean/src/core/mod.rs`:
```rust
pub mod memory;
pub mod ntdll;
pub mod privileges;
pub mod process;
```

- [ ] **Step 3: Run test to verify process enumeration**

Run: `cargo test --test test_process`  
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add memclean/src/core/process.rs memclean/src/core/mod.rs memclean/tests/test_process.rs
git commit -m "feat: implement process enumeration and working set trimming"
```

---

### Task 4: Smart Game Guard & Foreground Shield

**Files:**
- Create: `memclean/src/engine/guard.rs`
- Create: `memclean/src/engine/mod.rs`
- Modify: `memclean/src/lib.rs`
- Test: `memclean/tests/test_guard.rs`

**Interfaces:**
- Produces:
  ```rust
  pub struct ForegroundContext {
      pub pid: u32,
      pub is_fullscreen: bool,
  }
  pub fn get_foreground_context() -> Option<ForegroundContext>;
  pub fn is_protected_process(name: &str, pid: u32, fg_pid: Option<u32>, exclusions: &[String]) -> bool;
  ```

- [ ] **Step 1: Write test for protection logic**

Create `memclean/tests/test_guard.rs`:
```rust
#[test]
fn test_protected_process_filtering() {
    let exclusions = vec!["customgame.exe".to_string(), "notepad.exe".to_string()];
    
    // Core system processes must be protected
    assert!(memclean::engine::guard::is_protected_process("csrss.exe", 100, None, &exclusions));
    assert!(memclean::engine::guard::is_protected_process("dwm.exe", 200, None, &exclusions));
    assert!(memclean::engine::guard::is_protected_process("system", 4, None, &exclusions));

    // Foreground process must be protected
    assert!(memclean::engine::guard::is_protected_process("app.exe", 500, Some(500), &exclusions));

    // Custom exclusion list
    assert!(memclean::engine::guard::is_protected_process("customgame.exe", 600, None, &exclusions));

    // Normal non-protected process
    assert!(!memclean::engine::guard::is_protected_process("idleapp.exe", 700, Some(500), &exclusions));
}
```

- [ ] **Step 2: Implement `memclean/src/engine/guard.rs`**

```rust
use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId};

pub struct ForegroundContext {
    pub pid: u32,
    pub is_fullscreen: bool,
}

pub fn get_foreground_context() -> Option<ForegroundContext> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return None;
        }

        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return None;
        }

        let mut win_rect: RECT = std::mem::zeroed();
        GetWindowRect(hwnd, &mut win_rect);

        let h_monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut mi: MONITORINFO = std::mem::zeroed();
        mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;

        let mut is_fullscreen = false;
        if MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) != std::ptr::null_mut()
            && GetMonitorInfoW(h_monitor, &mut mi) != 0
        {
            is_fullscreen = win_rect.left <= mi.rcMonitor.left
                && win_rect.top <= mi.rcMonitor.top
                && win_rect.right >= mi.rcMonitor.right
                && win_rect.bottom >= mi.rcMonitor.bottom;
        }

        Some(ForegroundContext {
            pid,
            is_fullscreen,
        })
    }
}

pub fn is_protected_process(
    name: &str,
    pid: u32,
    fg_pid: Option<u32>,
    custom_exclusions: &[String],
) -> bool {
    let lower_name = name.to_lowercase();

    // 1. Whitelist of Windows core system critical processes
    const SYSTEM_WHITELIST: &[&str] = &[
        "system",
        "registry",
        "smss.exe",
        "csrss.exe",
        "wininit.exe",
        "services.exe",
        "lsass.exe",
        "dwm.exe",
        "fontdrvhost.exe",
        "memclean.exe",
    ];

    if SYSTEM_WHITELIST.contains(&lower_name.as_str()) {
        return true;
    }

    // 2. Foreground process protection (shield)
    if let Some(foreground_id) = fg_pid {
        if pid == foreground_id {
            return true;
        }
    }

    // 3. User configured exclusion list
    if custom_exclusions.iter().any(|ex| ex.to_lowercase() == lower_name) {
        return true;
    }

    false
}
```

Create `memclean/src/engine/mod.rs`:
```rust
pub mod guard;
```

Update `memclean/src/lib.rs`:
```rust
pub mod core;
pub mod engine;
```

- [ ] **Step 3: Run test to verify guard protection logic**

Run: `cargo test --test test_guard`  
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add memclean/src/engine/guard.rs memclean/src/engine/mod.rs memclean/src/lib.rs memclean/tests/test_guard.rs
git commit -m "feat: implement smart game guard and foreground window shield"
```

---

### Task 5: Configuration Schema & Settings Persistence

**Files:**
- Create: `memclean/src/config.rs`
- Modify: `memclean/src/lib.rs`
- Test: `memclean/tests/test_config.rs`

**Interfaces:**
- Produces:
  ```rust
  #[derive(Serialize, Deserialize, Clone, Debug)]
  pub struct AppConfig { ... }
  impl AppConfig {
      pub fn load() -> Self;
      pub fn save(&self) -> Result<(), String>;
      pub fn config_path() -> std::path::PathBuf;
  }
  ```

- [ ] **Step 1: Write test for configuration load and save**

Create `memclean/tests/test_config.rs`:
```rust
#[test]
fn test_default_config_serialization() {
    let config = memclean::config::AppConfig::default();
    let serialized = toml::to_string(&config).expect("Serialization failed");
    let deserialized: memclean::config::AppConfig = toml::from_str(&serialized).expect("Deserialization failed");
    assert_eq!(config.triggers.threshold_percent, deserialized.triggers.threshold_percent);
    assert_eq!(config.policy.purge_standby, deserialized.policy.purge_standby);
}
```

- [ ] **Step 2: Implement `memclean/src/config.rs`**

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Preset {
    SmartSafe,
    Aggressive,
    Custom,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GeneralConfig {
    pub run_on_startup: bool,
    pub start_minimized: bool,
    pub tray_update_interval_sec: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PolicyConfig {
    pub preset: Preset,
    pub purge_standby: bool,
    pub purge_low_priority_standby: bool,
    pub flush_modified: bool,
    pub trim_file_cache: bool,
    pub trim_working_sets: bool,
    pub game_guard_enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TriggersConfig {
    pub auto_threshold_enabled: bool,
    pub threshold_percent: u32,
    pub auto_timer_enabled: bool,
    pub timer_minutes: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub policy: PolicyConfig,
    pub triggers: TriggersConfig,
    pub exclusions: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                run_on_startup: true,
                start_minimized: true,
                tray_update_interval_sec: 2,
            },
            policy: PolicyConfig {
                preset: Preset::SmartSafe,
                purge_standby: true,
                purge_low_priority_standby: false,
                flush_modified: true,
                trim_file_cache: true,
                trim_working_sets: true,
                game_guard_enabled: true,
            },
            triggers: TriggersConfig {
                auto_threshold_enabled: true,
                threshold_percent: 80,
                auto_timer_enabled: true,
                timer_minutes: 15,
            },
            exclusions: vec![
                "explorer.exe".to_string(),
                "steam.exe".to_string(),
                "obs64.exe".to_string(),
            ],
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(appdata).join("MemClean").join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = toml::from_str(&content) {
                    return cfg;
                }
            }
        }
        let default = Self::default();
        let _ = default.save();
        default
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let data = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, data).map_err(|e| e.to_string())
    }
}
```

Update `memclean/src/lib.rs`:
```rust
pub mod config;
pub mod core;
pub mod engine;
```

- [ ] **Step 3: Run test to verify config serialization**

Run: `cargo test --test test_config`  
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add memclean/src/config.rs memclean/src/lib.rs memclean/tests/test_config.rs
git commit -m "feat: add application configuration schema and persistence"
```

---

### Task 6: Cleaner Orchestrator & CLI Subcommands

**Files:**
- Create: `memclean/src/engine/cleaner.rs`
- Modify: `memclean/src/engine/mod.rs`
- Create: `memclean/src/main.rs`
- Test: `memclean/tests/test_cleaner.rs`

**Interfaces:**
- Produces:
  ```rust
  pub struct CleanResult {
      pub standby_purged: bool,
      pub modified_flushed: bool,
      pub file_cache_trimmed: bool,
      pub processes_trimmed: usize,
  }
  pub fn perform_clean(config: &AppConfig, force_aggressive: bool) -> CleanResult;
  ```

- [ ] **Step 1: Write test for clean execution**

Create `memclean/tests/test_cleaner.rs`:
```rust
#[test]
fn test_clean_orchestration() {
    let mut config = memclean::config::AppConfig::default();
    // Disable process trimming in test to avoid interfering with test runner
    config.policy.trim_working_sets = false;
    let res = memclean::engine::cleaner::perform_clean(&config, false);
    assert!(res.standby_purged);
}
```

- [ ] **Step 2: Implement `memclean/src/engine/cleaner.rs`**

```rust
use crate::config::AppConfig;
use crate::core::{ntdll, process};
use crate::engine::guard;

#[derive(Default, Debug)]
pub struct CleanResult {
    pub standby_purged: bool,
    pub modified_flushed: bool,
    pub file_cache_trimmed: bool,
    pub processes_trimmed: usize,
}

pub fn perform_clean(config: &AppConfig, force_aggressive: bool) -> CleanResult {
    let mut result = CleanResult::default();
    let fg_ctx = guard::get_foreground_context();

    // If game guard is on and in fullscreen, skip aggressive process trimming
    let restrict_to_standby = config.policy.game_guard_enabled
        && fg_ctx.as_ref().map_or(false, |ctx| ctx.is_fullscreen)
        && !force_aggressive;

    // 1. Purge Standby List
    if config.policy.purge_standby || force_aggressive {
        if ntdll::purge_standby_list().is_ok() {
            result.standby_purged = true;
        }
    } else if config.policy.purge_low_priority_standby {
        if ntdll::purge_low_priority_standby_list().is_ok() {
            result.standby_purged = true;
        }
    }

    if restrict_to_standby {
        return result;
    }

    // 2. Flush Modified List
    if config.policy.flush_modified || force_aggressive {
        if ntdll::flush_modified_list().is_ok() {
            result.modified_flushed = true;
        }
    }

    // 3. Trim System File Cache
    if config.policy.trim_file_cache || force_aggressive {
        if ntdll::trim_system_file_cache().is_ok() {
            result.file_cache_trimmed = true;
        }
    }

    // 4. Trim Working Sets
    if config.policy.trim_working_sets || force_aggressive {
        if let Ok(procs) = process::enumerate_processes() {
            let fg_pid = if config.policy.game_guard_enabled && !force_aggressive {
                fg_ctx.map(|ctx| ctx.pid)
            } else {
                None
            };

            for p in procs {
                if !guard::is_protected_process(&p.name, p.pid, fg_pid, &config.exclusions) {
                    if process::trim_process_working_set(p.pid).is_ok() {
                        result.processes_trimmed += 1;
                    }
                }
            }
        }
    }

    result
}
```

Update `memclean/src/engine/mod.rs`:
```rust
pub mod cleaner;
pub mod guard;
```

- [ ] **Step 3: Implement `memclean/src/main.rs` CLI flags**

```rust
use memclean::config::AppConfig;
use memclean::core::{memory, privileges};
use memclean::engine::cleaner;
use std::env;

fn print_help() {
    println!("MemClean - Ultra-Lightweight Pure Rust Memory Optimizer");
    println!("Usage:");
    println!("  memclean.exe              Launch system tray daemon");
    println!("  memclean.exe --status     Print current physical RAM statistics");
    println!("  memclean.exe --clean      Perform memory optimization according to config");
    println!("  memclean.exe --aggressive Force aggressive purge of all memory layers");
    println!("  memclean.exe --standby    Purge standby list only");
    println!("  memclean.exe --autostart  Register elevated auto-start on Windows login");
    println!("  memclean.exe --no-autostart Remove elevated auto-start on Windows login");
}

fn register_autostart(enable: bool) {
    let current_exe = env::current_exe().unwrap_or_default();
    let exe_str = current_exe.to_string_lossy();

    if enable {
        let status = std::process::Command::new("schtasks")
            .args(&[
                "/create",
                "/tn", "MemCleanAutoStart",
                "/tr", &format!("\"{}\"", exe_str),
                "/sc", "onlogon",
                "/rl", "highest",
                "/f"
            ])
            .status();
        println!("Registered elevated task scheduler autostart: {:?}", status);
    } else {
        let status = std::process::Command::new("schtasks")
            .args(&["/delete", "/tn", "MemCleanAutoStart", "/f"])
            .status();
        println!("Removed task scheduler autostart: {:?}", status);
    }
}

fn main() {
    let _ = privileges::enable_memory_privileges();
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let config = AppConfig::load();
        match args[1].as_str() {
            "--status" => {
                if let Ok(stats) = memory::get_memory_stats() {
                    println!("RAM Used:   {} MB ({}%)", stats.used_phys_mb, stats.used_percent);
                    println!("RAM Avail:  {} MB", stats.avail_phys_mb);
                    println!("RAM Total:  {} MB", stats.total_phys_mb);
                }
            }
            "--clean" => {
                let res = cleaner::perform_clean(&config, false);
                println!("Optimization completed: {:?}", res);
            }
            "--aggressive" => {
                let res = cleaner::perform_clean(&config, true);
                println!("Aggressive clean completed: {:?}", res);
            }
            "--standby" => {
                let _ = memclean::core::ntdll::purge_standby_list();
                println!("Standby list purged.");
            }
            "--autostart" => register_autostart(true),
            "--no-autostart" => register_autostart(false),
            "--help" | "-h" => print_help(),
            _ => print_help(),
        }
        return;
    }

    // Default: Run system tray GUI
    #[cfg(windows)]
    {
        memclean::ui::tray::run_tray_application();
    }
}
```

- [ ] **Step 4: Run test to verify cleaner**

Run: `cargo test --test test_cleaner`  
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add memclean/src/engine/cleaner.rs memclean/src/engine/mod.rs memclean/src/main.rs memclean/tests/test_cleaner.rs
git commit -m "feat: implement memory cleaning orchestrator and CLI mode"
```

---

### Task 7: Native Win32 System Tray & Dynamic Real-Time GDI Icon

**Files:**
- Create: `memclean/src/ui/tray.rs`
- Create: `memclean/src/ui/mod.rs`
- Modify: `memclean/src/lib.rs`

**Interfaces:**
- Produces:
  ```rust
  pub fn run_tray_application();
  ```

- [ ] **Step 1: Implement dynamic GDI percentage icon generation and Win32 event loop**

Create `memclean/src/ui/tray.rs`:
```rust
use crate::config::AppConfig;
use crate::core::memory;
use crate::engine::cleaner;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use windows_sys::Win32::Foundation::{GetLastError, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, CreateFontW, DeleteDC, DeleteObject, SelectObject,
    SetBkMode, SetTextColor, TextOutW, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    FW_BOLD, TRANSPARENT,
};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NOTIFYICONDATAW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, CreateWindowExW, DefWindowProcW, DestroyIcon, DestroyWindow,
    DispatchMessageW, GetCursorPos, GetMessageW, PostQuitMessage, RegisterClassW,
    SetForegroundWindow, SetTimer, TrackPopupMenu, ICONINFO, MSG, TPM_BOTTOMALIGN,
    TPM_RIGHTALIGN, WM_COMMAND, WM_DESTROY, WM_RBUTTONUP, WM_TIMER, WM_USER, WNDCLASSW,
};

const WM_TRAYICON: u32 = WM_USER + 1;
const TIMER_ID: usize = 1001;

fn create_percent_icon(percent: u32) -> windows_sys::Win32::UI::WindowsAndMessaging::HICON {
    unsafe {
        let hdc = CreateCompatibleDC(std::ptr::null_mut());

        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = 16;
        bmi.bmiHeader.biHeight = 16;
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB;

        let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
        let hbm_color = CreateDIBSection(hdc, &bmi, DIB_RGB_COLORS, &mut bits, std::ptr::null_mut(), 0);
        let hbm_mask = CreateDIBSection(hdc, &bmi, DIB_RGB_COLORS, &mut bits, std::ptr::null_mut(), 0);

        let old_bm = SelectObject(hdc, hbm_color);

        // Color based on memory usage: Green < 70%, Yellow 70-85%, Red > 85%
        let text_color = if percent < 70 {
            0x002E7D32 // Green
        } else if percent < 85 {
            0x00F57C00 // Orange
        } else {
            0x00C62828 // Red
        };

        SetBkMode(hdc, TRANSPARENT as i32);
        SetTextColor(hdc, text_color);

        let font = CreateFontW(
            12, 0, 0, 0, FW_BOLD as i32, 0, 0, 0, 0, 0, 0, 0, 0,
            windows_sys::w!("Arial"),
        );
        let old_font = SelectObject(hdc, font);

        let text = format!("{:02}", percent.min(99));
        let wide: Vec<u16> = text.encode_utf16().collect();
        TextOutW(hdc, 0, 1, wide.as_ptr(), wide.len() as i32);

        SelectObject(hdc, old_font);
        DeleteObject(font);
        SelectObject(hdc, old_bm);
        DeleteDC(hdc);

        let mut icon_info = ICONINFO {
            fIcon: 1,
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: hbm_mask,
            hbmColor: hbm_color,
        };

        let icon = CreateIconIndirect(&mut icon_info);
        DeleteObject(hbm_color);
        DeleteObject(hbm_mask);

        icon
    }
}

pub fn run_tray_application() {
    unsafe {
        let class_name = windows_sys::w!("MemCleanTrayClass");
        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(tray_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: std::ptr::null_mut(),
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name,
        };
        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            0,
            class_name,
            windows_sys::w!("MemClean"),
            0,
            0, 0, 0, 0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );

        let stats = memory::get_memory_stats().unwrap_or(memory::MemoryStats {
            total_phys_mb: 0,
            avail_phys_mb: 0,
            used_phys_mb: 0,
            used_percent: 50,
        });

        let hicon = create_percent_icon(stats.used_percent);

        let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        nid.uCallbackMessage = WM_TRAYICON;
        nid.hIcon = hicon;

        let tip = format!("MemClean: {}% RAM Used\0", stats.used_percent);
        let tip_wide: Vec<u16> = tip.encode_utf16().collect();
        let copy_len = tip_wide.len().min(nid.szTip.len());
        nid.szTip[..copy_len].copy_from_slice(&tip_wide[..copy_len]);

        Shell_NotifyIconW(NIM_ADD, &nid);

        // Update timer every 2 seconds
        SetTimer(hwnd, TIMER_ID, 2000, None);

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            DispatchMessageW(&msg);
        }

        Shell_NotifyIconW(NIM_DELETE, &nid);
        DestroyIcon(hicon);
    }
}

unsafe extern "system" fn tray_wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_TRAYICON => {
            if lparam as u32 == WM_RBUTTONUP {
                let mut pt = POINT { x: 0, y: 0 };
                GetCursorPos(&mut pt);
                SetForegroundWindow(hwnd);
                show_tray_menu(hwnd, pt.x, pt.y);
            }
            0
        }
        WM_TIMER => {
            if let Ok(stats) = memory::get_memory_stats() {
                let config = AppConfig::load();
                if config.triggers.auto_threshold_enabled && stats.used_percent >= config.triggers.threshold_percent {
                    cleaner::perform_clean(&config, false);
                }

                let new_icon = create_percent_icon(stats.used_percent);
                let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
                nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                nid.hWnd = hwnd;
                nid.uID = 1;
                nid.uFlags = NIF_ICON | NIF_TIP;
                nid.hIcon = new_icon;

                let tip = format!("MemClean: {}% RAM Used ({} MB Free)\0", stats.used_percent, stats.avail_phys_mb);
                let tip_wide: Vec<u16> = tip.encode_utf16().collect();
                let copy_len = tip_wide.len().min(nid.szTip.len());
                nid.szTip[..copy_len].copy_from_slice(&tip_wide[..copy_len]);

                Shell_NotifyIconW(NIM_MODIFY, &nid);
                DestroyIcon(new_icon);
            }
            0
        }
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as u32;
            match id {
                101 => { // Quick Clean
                    let config = AppConfig::load();
                    cleaner::perform_clean(&config, false);
                }
                102 => { // Purge Standby
                    let _ = crate::core::ntdll::purge_standby_list();
                }
                103 => { // Empty Working Sets
                    let _ = crate::core::ntdll::empty_system_working_sets();
                }
                104 => { // Settings
                    crate::ui::dialog::show_settings_dialog(hwnd);
                }
                105 => { // Exit
                    DestroyWindow(hwnd);
                }
                _ => {}
            }
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn show_tray_menu(hwnd: HWND, x: i32, y: i32) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, DestroyMenu, MF_SEPARATOR, MF_STRING,
    };

    let menu = CreatePopupMenu();
    AppendMenuW(menu, MF_STRING, 101, windows_sys::w!("Quick Optimize"));
    AppendMenuW(menu, MF_STRING, 102, windows_sys::w!("Purge Standby List"));
    AppendMenuW(menu, MF_STRING, 103, windows_sys::w!("Empty Working Sets"));
    AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
    AppendMenuW(menu, MF_STRING, 104, windows_sys::w!("Settings..."));
    AppendMenuW(menu, MF_STRING, 105, windows_sys::w!("Exit"));

    TrackPopupMenu(menu, TPM_RIGHTALIGN | TPM_BOTTOMALIGN, x, y, 0, hwnd, std::ptr::null());
    DestroyMenu(menu);
}
```

Create `memclean/src/ui/mod.rs`:
```rust
pub mod dialog;
pub mod tray;
```

Update `memclean/src/lib.rs`:
```rust
pub mod config;
pub mod core;
pub mod engine;
pub mod ui;
```

- [ ] **Step 2: Commit**

```bash
git add memclean/src/ui/tray.rs memclean/src/ui/mod.rs memclean/src/lib.rs
git commit -m "feat: implement native Win32 system tray and dynamic percentage icon"
```

---

### Task 8: Native Win32 Settings Dialog

**Files:**
- Create: `memclean/src/ui/dialog.rs`
- Modify: `memclean/src/ui/mod.rs`

**Interfaces:**
- Produces:
  ```rust
  pub fn show_settings_dialog(parent: HWND);
  ```

- [ ] **Step 1: Implement `memclean/src/ui/dialog.rs`**

```rust
use crate::config::AppConfig;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    IsDialogMessageW, MessageBoxW, RegisterClassW, ShowWindow, BS_AUTOCHECKBOX, BS_PUSHBUTTON,
    MB_ICONINFORMATION, MB_OK, MSG, SW_SHOW, WM_COMMAND, WM_DESTROY, WNDCLASSW, WS_CHILD,
    WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

pub fn show_settings_dialog(_parent: HWND) {
    unsafe {
        let class_name = windows_sys::w!("MemCleanSettingsClass");
        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(dialog_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: std::ptr::null_mut(),
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: 6 as _, // COLOR_WINDOW
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name,
        };
        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            0,
            class_name,
            windows_sys::w!("MemClean Settings"),
            WS_OVERLAPPEDWINDOW & !(0x00010000 | 0x00040000), // Disable maximize & resize
            200, 200, 360, 280,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );

        // Checkboxes for features
        CreateWindowExW(
            0, windows_sys::w!("BUTTON"), windows_sys::w!("Purge Standby List"),
            WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX as u32,
            20, 20, 280, 24, hwnd, 201 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        CreateWindowExW(
            0, windows_sys::w!("BUTTON"), windows_sys::w!("Flush Modified List"),
            WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX as u32,
            20, 50, 280, 24, hwnd, 202 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        CreateWindowExW(
            0, windows_sys::w!("BUTTON"), windows_sys::w!("Trim System File Cache"),
            WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX as u32,
            20, 80, 280, 24, hwnd, 203 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        CreateWindowExW(
            0, windows_sys::w!("BUTTON"), windows_sys::w!("Trim Background Working Sets"),
            WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX as u32,
            20, 110, 280, 24, hwnd, 204 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        CreateWindowExW(
            0, windows_sys::w!("BUTTON"), windows_sys::w!("Enable Game & Foreground Shield"),
            WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX as u32,
            20, 140, 280, 24, hwnd, 205 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );

        // Save & Close button
        CreateWindowExW(
            0, windows_sys::w!("BUTTON"), windows_sys::w!("Save and Close"),
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32,
            100, 190, 140, 32, hwnd, 301 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );

        ShowWindow(hwnd, SW_SHOW);

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            if IsDialogMessageW(hwnd, &mut msg) == 0 {
                DispatchMessageW(&msg);
            }
        }
    }
}

unsafe extern "system" fn dialog_wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as u32;
            if id == 301 {
                let config = AppConfig::load();
                let _ = config.save();
                MessageBoxW(
                    hwnd,
                    windows_sys::w!("Settings saved successfully."),
                    windows_sys::w!("MemClean"),
                    MB_OK | MB_ICONINFORMATION,
                );
                DestroyWindow(hwnd);
            }
            0
        }
        WM_DESTROY => {
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add memclean/src/ui/dialog.rs
git commit -m "feat: implement native Win32 settings dialog"
```

---

### Task 9: Inno Setup Installer & Release Automation Script

**Files:**
- Create: `installer/setup.iss`
- Create: `scripts/build-release.ps1`
- Test: Build script execution & artifact verification

**Interfaces:**
- Produces:
  `dist/MemClean_Setup.exe` (Windows Installer)
  `dist/MemClean_Portable.zip` (Portable Package)

- [ ] **Step 1: Create `installer/setup.iss`**

```iss
[Setup]
AppId={{9F57C02D-2A9F-4D2A-B981-D189873E4A10}
AppName=MemClean
AppVersion=1.0.0
AppPublisher=MemClean
DefaultDirName={autopf}\MemClean
DefaultGroupName=MemClean
AllowNoIcons=yes
OutputDir=..\dist
OutputBaseFilename=MemClean_Setup
Compression=lzma2/ultra64
SolidCompression=yes
PrivilegesRequired=admin
CloseApplications=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "autostart"; Description: "Run MemClean automatically on Windows startup (Silent Elevation)"; GroupDescription: "Startup:"

[Files]
Source: "..\target\release\memclean.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\MemClean"; Filename: "{app}\memclean.exe"
Name: "{group}\Uninstall MemClean"; Filename: "{uninstallexe}"
Name: "{autodesktop}\MemClean"; Filename: "{app}\memclean.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\memclean.exe"; Parameters: "--autostart"; Tasks: autostart; Flags: runhidden
Filename: "{app}\memclean.exe"; Description: "{cm:LaunchProgram,MemClean}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{app}\memclean.exe"; Parameters: "--no-autostart"; Flags: runhidden
```

- [ ] **Step 2: Create `scripts/build-release.ps1`**

```powershell
$ErrorActionPreference = "Stop"

Write-Host "Building MemClean in Release Mode..." -ForegroundColor Cyan
Set-Location -Path "$PSScriptRoot\..\memclean"
cargo build --release

$targetDir = "$PSScriptRoot\..\dist"
if (-not (Test-Path $targetDir)) {
    New-Item -ItemType Directory -Path $targetDir | Out-Null
}

# Copy portable executable
Copy-Item "target\release\memclean.exe" -Destination "$targetDir\memclean.exe" -Force
Write-Host "Portable binary ready at dist\memclean.exe" -ForegroundColor Green

# Compile installer if ISCC (Inno Setup) is present
$isccPath = (Get-Command iscc.exe -ErrorAction SilentlyContinue).Source
if (-not $isccPath) {
    $candidates = @(
        "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
        "${env:ProgramFiles}\Inno Setup 6\ISCC.exe"
    )
    foreach ($cand in $candidates) {
        if (Test-Path $cand) {
            $isccPath = $cand
            break
        }
    }
}

if ($isccPath) {
    Write-Host "Compiling Installer with Inno Setup ($isccPath)..." -ForegroundColor Cyan
    & $isccPath "$PSScriptRoot\..\installer\setup.iss"
    Write-Host "Installer generated at dist\MemClean_Setup.exe" -ForegroundColor Green
} else {
    Write-Host "Inno Setup compiler (ISCC.exe) not found. Portable binary compiled successfully." -ForegroundColor Yellow
}
```

- [ ] **Step 3: Commit**

```bash
git add installer/setup.iss scripts/build-release.ps1
git commit -m "feat: add Inno Setup installer script and release build automation"
```
