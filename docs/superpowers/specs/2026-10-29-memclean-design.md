# MemClean: Ultra-Lightweight Pure Rust Windows Memory Optimizer
**Document:** System Architecture & Design Specification  
**Date:** 2026-10-29  
**Status:** Approved for Implementation  

---

## 1. Executive Summary & Objective

`MemClean` is a modern, pure Rust replacement for legacy Windows memory cleaners like Sordum ReduceMemory. 
While legacy tools rely on compiled AutoIt scripts (~25 MB runtime RAM) and naive `EmptyWorkingSet` sweeps that cause severe process paging churn and game frame drops, `MemClean` delivers:

- **Sysinternals-Grade Native Rust Footprint:** Single standalone binary (< 1 MB disk, < 2 MB RAM, 0.0% idle CPU).
- **Multi-Tiered Memory Purging Engine:** Standby List, Low-Priority Standby, Modified Page List, System File Cache, and Process Working Sets.
- **Smart Game & Foreground Guard:** Automatically exempts the active foreground process / fullscreen game from eviction, eliminating micro-stutters.
- **Zero-UAC Boot Startup:** Uses the Windows Task Scheduler Elevation Pattern (`TASK_RUNLEVEL_HIGHEST`) to start silently at user login with full administrator privileges without recurring UAC popups.
- **Dual-Mode Operation:** Runs seamlessly as an interactive System Tray application with dynamic GDI percentage badges, or headlessly via CLI flags for scripting and Task Scheduler automation.
- **Installable Distribution:** Complete Inno Setup installer wizard (`MemClean_Setup.exe`) alongside standalone portable distribution.

---

## 2. Architecture & Module Decomposition

```
memclean/
├── Cargo.toml
├── build.rs                  # Embeds UAC manifest (requireAdministrator) & application icon
├── app.manifest              # Windows application manifest (DPI-aware, OS compatibility)
├── src/
│   ├── main.rs               # Mode switcher: parses CLI args; dispatches CLI task vs Tray daemon
│   ├── core/
│   │   ├── mod.rs
│   │   ├── privileges.rs     # Token elevation: SeProfileSingleProcess, SeIncreaseQuota
│   │   ├── ntdll.rs          # Dynamic binding to NtSetSystemInformation (Memory List commands)
│   │   ├── memory.rs         # GlobalMemoryStatusEx, GetPerformanceInfo query wrappers
│   │   └── process.rs        # Process enumeration & selective working set trimming
│   ├── engine/
│   │   ├── mod.rs
│   │   ├── cleaner.rs        # Orchestrates the 4 cleanup tiers according to policy
│   │   └── guard.rs          # Foreground window & fullscreen/game detection shield
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── tray.rs           # Shell_NotifyIconW, dynamic GDI percentage icon, context menu
│   │   └── dialog.rs         # Native Win32 settings & status dialog
│   └── config.rs             # Configuration schema, defaults, and %APPDATA% persistence
└── installer/
    └── setup.iss             # Inno Setup compilation script
```

---

## 3. Kernel Memory Engine Specification

### 3.1 Token Privilege Escalation (`privileges.rs`)
Windows restricts kernel memory list commands to tokens possessing specific privileges:
1. `OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut h_token)`.
2. Look up `SE_PROFILE_SINGLE_PROCESS_NAME` (`SeProfileSingleProcessPrivilege`) and `SE_INCREASE_QUOTA_NAME` (`SeIncreaseQuotaPrivilege`).
3. Apply `AdjustTokenPrivileges` enabling both attributes with `SE_PRIVILEGE_ENABLED`.

### 3.2 NT Kernel Memory List Commands (`ntdll.rs`)
Calls undocumented kernel API exported by `ntdll.dll`:
```rust
type NtSetSystemInformationFn = unsafe extern "system" fn(
    system_information_class: u32, // SystemMemoryListInformation = 80 (0x50)
    system_information: *const c_void,
    system_information_length: u32,
) -> i32; // NTSTATUS
```

Command codes for `SYSTEM_MEMORY_LIST_COMMAND` (passed as `u32` payload):
- `MemoryEmptyWorkingSets = 2`: Empties all process working sets at kernel level.
- `MemoryFlushModifiedList = 3`: Writes modified/dirty pages to pagefile/disk, turning them into standby pages.
- `MemoryPurgeStandbyList = 4`: Purges all pages from the standby list to the free page list (zero disk I/O, zero application penalty).
- `MemoryPurgeLowPriorityStandbyList = 5`: Purges only priority-0 standby pages, preserving hot cache.

### 3.3 System File Cache Trimming (`memory.rs`)
Calls `SetSystemFileCacheSize((SIZE_T)-1, (SIZE_T)-1, 0)` from `kernel32.dll` to purge bloated file system cache buffers.

### 3.4 Process Working Set Trimming (`process.rs`)
- Uses `CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)`.
- Iterates process entries via `Process32FirstW` / `Process32NextW`.
- Skips process if PID == 0, PID == 4 (System), PID matches active foreground PID, or process name matches the exclusion list.
- Opens candidate process with `OpenProcess(PROCESS_SET_QUOTA | PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid)`.
- Calls `K32EmptyWorkingSet(h_process)` or `SetProcessWorkingSetSize(h_process, usize::MAX, usize::MAX)`.
- Closes handles immediately to prevent handle leaks.

---

## 4. Smart Game & Foreground Guard (`guard.rs`)

### 4.1 Foreground Shield
- Queries `GetForegroundWindow()`.
- Obtains owner thread & process ID: `GetWindowThreadProcessId(hwnd, &mut pid)`.
- Resolves executable base name using `QueryFullProcessImageNameW`.
- Guarantees the foreground process is **never** subjected to working set trimming.

### 4.2 Fullscreen Detection
- Retrieves window rect via `GetWindowRect(hwnd, &mut rect)`.
- Retrieves monitor rect for the window via `MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST)` and `GetMonitorInfoW`.
- If window rect spans or exceeds the monitor rect, fullscreen mode is flagged.
- When fullscreen/game mode is active:
  - Background notifications are silenced.
  - Periodic auto-clean is downgraded to **Standby List Purge only**, strictly avoiding any process page fault latency.

### 4.3 Default Safety Exclusions
Hardcoded whitelist:
`System`, `Registry`, `smss.exe`, `csrss.exe`, `wininit.exe`, `services.exe`, `lsass.exe`, `dwm.exe`, `fontdrvhost.exe`.

---

## 5. UI, System Tray & Configuration Specification

### 5.1 Dynamic Tray Icon Generation (`tray.rs`)
- Polls memory every 2 seconds via `GlobalMemoryStatusEx(&mut mem_status)`.
- Calculates percentage: `used_percent = (total - avail) * 100 / total`.
- Creates 16×16 / 32×32 32-bpp GDI DIB section (`CreateDIBSection`).
- Uses high-contrast color scheme:
  - Green (`#4CAF50`): RAM used < 70%
  - Orange (`#FF9800`): RAM used 70% – 85%
  - Red (`#F44336`): RAM used > 85%
- Draws two-digit percentage text using native Windows font rasterization.
- Converts bitmap to `HICON` via `CreateIconIndirect`.
- Dispatches `Shell_NotifyIconW(NIM_MODIFY, &nid)` to update the taskbar tray.
- Destroys previous icon handle via `DestroyIcon` to avoid GDI handle leaks.

### 5.2 Tray Menu & Interactions
- **Left Double-Click:** Performs Quick Optimize according to configured policy.
- **Right Click Context Menu:**
  - `⚡ Quick Optimize` (Bold)
  - `🧹 Purge Standby List`
  - `📦 Empty Working Sets`
  - `---`
  - `⚙️ Settings...`
  - `🚀 Start with Windows` (Toggle checkmark)
  - `❌ Exit`

### 5.3 Settings Dialog (`dialog.rs`)
Native Win32 Dialog created via code or dialog resource:
- **Presets:** `Smart Safe Mode (Default)`, `Aggressive Purge`, `Custom`.
- **Custom Toggles:**
  - `[x] Purge Standby List`
  - `[x] Low-Priority Standby Purge`
  - `[x] Flush Modified Pages`
  - `[x] Trim System File Cache`
  - `[x] Trim Background Working Sets`
  - `[x] Enable Smart Game / Foreground Shield`
- **Automation Triggers:**
  - `[x] Auto-optimize when RAM exceeds [ 80 ] %`
  - `[x] Auto-optimize every [ 15 ] minutes`
- **Exclusion List:** Multi-line text box / list view for excluded `.exe` process names.

### 5.4 Configuration Schema (`config.rs`)
Stored in `%APPDATA%\MemClean\config.toml` (or `.ini`):
```toml
[general]
run_on_startup = true
start_minimized = true
tray_update_interval_sec = 2

[policy]
preset = "smart_safe" # "smart_safe" | "aggressive" | "custom"
purge_standby = true
purge_low_priority_standby = false
flush_modified = true
trim_file_cache = true
trim_working_sets = true
game_guard_enabled = true

[triggers]
auto_threshold_enabled = true
threshold_percent = 80
auto_timer_enabled = true
timer_minutes = 15

[exclusions]
processes = ["explorer.exe", "steam.exe", "obs64.exe"]
```

---

## 6. CLI Automation Specification

When invoked with command-line flags, `MemClean` operates headlessly without creating a GUI or tray icon:

- `memclean.exe --status`: Prints current RAM statistics in JSON or human-readable format:
  ```
  Physical Memory: 16384 MB Total | 11468 MB Used (70%) | 4916 MB Free
  Standby Cache:   3200 MB
  Modified List:   128 MB
  ```
- `memclean.exe --clean [all|standby|modified|filecache|workingsets]`
- `memclean.exe --preset [safe|aggressive]`
- `memclean.exe --quiet`: Suppresses stdout, returns exit code 0 on success.
- `memclean.exe --install-autostart`: Registers the elevated Scheduled Task.
- `memclean.exe --remove-autostart`: Unregisters the elevated Scheduled Task.

---

## 7. Packaging, UAC Elevation & Installer Specification

### 7.1 Zero-UAC Elevation Pattern
To run with administrative privileges without triggering a UAC prompt on every Windows restart:
- Creates a Windows Scheduled Task via `schtasks.exe`:
  ```cmd
  schtasks /create /tn "MemCleanAutoStart" /tr "\"C:\Program Files\MemClean\memclean.exe\"" /sc onlogon /rl highest /f
  ```
- Configures power settings so task runs on battery power.
- Deletion on uninstall:
  ```cmd
  schtasks /delete /tn "MemCleanAutoStart" /f
  ```

### 7.2 Inno Setup Configuration (`installer/setup.iss`)
- Target: `C:\Program Files\MemClean\memclean.exe`.
- Creates Desktop & Start Menu shortcuts.
- Closes running `memclean.exe` instances gracefully before overwrite (`CloseApplications=yes`).
- Registers uninstaller in Windows "Installed Apps" / Control Panel.
- Executes `schtasks` registration automatically if "Start with Windows" is checked.

### 7.3 Release Artifacts
1. `MemClean_Setup.exe`: Full Windows installer (~1.2 MB).
2. `MemClean_Portable.zip`: Standalone executable + default config template.

---

## 8. Verification & Acceptance Criteria

1. **Memory Reduction Verification:**
   - Standby cache drops by gigabytes upon `PurgeStandbyList`.
   - File cache drops upon `SetSystemFileCacheSize`.
   - Background working sets trimmed without touching the active foreground process.
2. **Game Stutter Prevention:**
   - Running full-screen game while auto-clean triggers produces **0** drop in FPS.
3. **Resource Footprint:**
   - `memclean.exe` uses < 2 MB private working set in Task Manager.
   - CPU utilization stays at 0.0% when idle in system tray.
4. **Lifecycle & Persistence:**
   - Survives Windows reboot and starts elevated without UAC prompt.
   - Settings persist accurately across restarts.
