# MemClean: Advanced Kernel Memory Optimization, Gaming Latency Engine & Diagnostic Telemetry
**Document:** System Architecture & Design Specification (Power Upgrades)  
**Date:** 2026-10-30  
**Status:** Approved for Implementation  

---

## 1. Executive Summary & Objective

This specification details the next-generation architectural upgrades to `MemClean`, expanding it from a high-efficiency memory cleaner into a comprehensive, gamer-optimized, and diagnostic-grade Windows memory subsystem manager.

### Key Pillars:
1. **The Gaming & Latency Engine (ISLC Killer):**
   - **Intelligent Dual-Condition Standby Purge:** Cleans the standby list only when `Standby MB >= Threshold` AND `Free RAM <= Threshold`, preventing unnecessary disk churn and eliminating in-game micro-stutters.
   - **Microsecond Scheduler Timer Resolution:** Requests and locks Windows global timer resolution to `0.500 ms` or `1.000 ms` (`NtSetTimerResolution` / `timeBeginPeriod`), stabilizing frame pacing and minimizing input latency.
2. **Deep Kernel Memory De-Duplication & Diagnostics (Sysinternals Rival):**
   - **Kernel Page Combining:** Invokes `NtSetSystemInformation` with `SystemCombinePhysicalMemoryInformation = 130` to de-duplicate identical physical memory pages (across Electron runtimes, browser tabs, DLLs) into Copy-On-Write single pages without disk paging.
   - **Real-Time Memory List Telemetry:** Queries `NtQuerySystemInformation(80)` to break down physical memory into Standby Priorities 0–7, Modified, Free, and Zeroed page pools.
   - **Segmented GDI Memory Distribution Bar:** Visualizes the complete physical RAM layout cleanly within the native Win32 dashboard.
3. **Smart Event-Driven Automation & Process EcoQoS:**
   - **Inactivity / AFK Deep Clean:** Automatically sweeps working sets, modified pages, and runs page combining after a configurable period of user inactivity (`GetLastInputInfo`).
   - **Post-Game / App-Exit Optimization:** Detects when a fullscreen game or heavy application exits and performs a gentle sweep to reclaim working sets.
   - **Process EcoQoS:** Allows toggling Windows 11 Efficiency Mode (`PROCESS_POWER_THROTTLING_EXECUTION_SPEED`) on heavy background processes.
   - **Headless CLI Scripting & JSON Output:** Provides `--stats --json`, `--combine`, `--standby`, and `--timer-res` for automation, Rainmeter skins, and PowerShell scripts.

---

## 2. Architecture & Module Decomposition

```
memclean/
├── Cargo.toml
├── build.rs
├── app.manifest
├── src/
│   ├── main.rs                  # CLI parser & mode switcher (GUI vs CLI flags)
│   ├── lib.rs
│   ├── config.rs                # AppConfig with expanded triggers, tuning, and policy
│   ├── core/
│   │   ├── mod.rs
│   │   ├── privileges.rs        # SeProfileSingleProcess, SeIncreaseQuota, SeDebugPrivilege
│   │   ├── ntdll.rs             # Dynamic bindings: NtSet/QuerySystemInformation, NtSetTimerResolution
│   │   ├── memory.rs            # GlobalMemoryStatusEx, NtQuerySystemInformation(80) breakdown
│   │   ├── process.rs           # Process enumeration, working set trim, EcoQoS power throttling
│   │   ├── timer_res.rs         # High-resolution timer engine (0.5ms / 1.0ms) with Drop guard
│   │   └── combine.rs           # Kernel physical page combining (class 130)
│   ├── engine/
│   │   ├── mod.rs
│   │   ├── cleaner.rs           # Orchestrator for multi-tier cleans, de-dup, and selective purges
│   │   ├── guard.rs             # Foreground window & fullscreen/game detection shield
│   │   ├── islc.rs              # Dual-condition standby list evaluator
│   │   └── idle.rs              # User inactivity / AFK tracker (GetLastInputInfo)
│   └── ui/
│       ├── mod.rs
│       ├── tray.rs              # Dynamic percentage tray icon, tooltip, context menu, dashboard loop
│       └── dialog.rs            # Settings dialog with multi-tab/grouped trigger & tuning controls
```

---

## 3. Low-Level Kernel & Subsystem Specifications

### 3.1 High-Resolution Timer Engine (`timer_res.rs`)
* **API Signature:**
  ```rust
  type NtSetTimerResolutionFn = unsafe extern "system" fn(
      desired_resolution: u32,
      set_resolution: u8,
      current_resolution: *mut u32,
  ) -> i32;

  type NtQueryTimerResolutionFn = unsafe extern "system" fn(
      minimum_resolution: *mut u32,
      maximum_resolution: *mut u32,
      current_resolution: *mut u32,
  ) -> i32;
  ```
* **Resolution Units:** 100-nanosecond units ($1\text{ ms} = 10,000\text{ units}$, $0.5\text{ ms} = 5,000\text{ units}$).
* **Modes:**
  - `Default`: System unmanaged resolution (typically 15.625 ms / 64 Hz).
  - `1.0 ms`: Standard multimedia resolution (10,000 units).
  - `0.5 ms`: Maximum competitive gaming resolution (5,000 units).
* **Guarantees:**
  - RAII guard pattern: restores default resolution on thread termination or application exit.
  - Fallback to `timeBeginPeriod(1)` via `winmm.dll` if NT functions fail.

### 3.2 Kernel Physical Page Combining (`combine.rs`)
* **API Class:** `SystemCombinePhysicalMemoryInformation = 130` ($0\text{x}82$).
* **Structure:**
  ```rust
  #[repr(C)]
  pub struct MemoryCombineInformationEx {
      pub handle: *mut std::ffi::c_void,
      pub pages_combined: usize,
      pub flags: u32,
  }
  ```
* **Privilege:** Requires `SeProfileSingleProcessPrivilege` (enabled during startup in `privileges.rs`).
* **Return Value:** Number of combined pages ($4\text{ KB}$ per page), giving total MB reclaimed without disk paging.

### 3.3 Deep NT Memory Breakdown (`ntdll.rs` & `memory.rs`)
* **API Class:** `SystemMemoryListInformation = 80` ($0\text{x}50$).
* **Structure:**
  ```rust
  #[repr(C)]
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
  ```
* **Metrics Derived:**
  - $\text{Standby Total MB} = \left(\sum_{i=0}^7 \text{PageCountByPriority}[i] \times 4096\right) / (1024 \times 1024)$
  - $\text{Free Total MB} = (\text{FreePageCount} \times 4096) / (1024 \times 1024)$
  - $\text{Zeroed Total MB} = (\text{ZeroPageCount} \times 4096) / (1024 \times 1024)$
  - $\text{Modified Total MB} = (\text{ModifiedPageCount} \times 4096) / (1024 \times 1024)$

### 3.4 Process EcoQoS / Efficiency Mode (`process.rs`)
* **Mechanism:**
  ```rust
  const ProcessPowerThrottling: u32 = 0x24;
  const PROCESS_POWER_THROTTLING_EXECUTION_SPEED: u32 = 0x1;

  #[repr(C)]
  struct ProcessPowerThrottlingState {
      version: u32,
      control_mask: u32,
      state_mask: u32,
  }
  ```
* Dynamically sets execution speed throttling on background processes without lowering base priority class or causing UI stalls.

---

## 4. Configuration Schema (`memclean.toml`)

Backwards-compatible TOML configuration:

```toml
[general]
auto_elevate = true
minimize_to_tray = true
start_minimized = false
global_hotkey = true

[triggers]
threshold_percent = 80          # 0 = disabled, 10..=95
timer_seconds = 15              # 0 = continuous, 1..=300

# ISLC Dual-Condition Engine
islc_enabled = false
islc_standby_mb = 1024          # Purge standby when >= 1024 MB
islc_free_mb = 1024             # ...AND free RAM is <= 1024 MB

# Inactivity / AFK Engine
idle_clean_enabled = false
idle_seconds = 300              # Clean when user is idle for 5 mins

[tuning]
timer_resolution_enabled = false
timer_resolution_mode = "0.5ms" # "0.5ms", "1.0ms", or "default"
combine_pages_on_clean = false  # Kernel page de-duplication

[policy]
purge_standby = true
purge_low_priority_standby = false
flush_modified = true
trim_file_cache = true
trim_working_sets = true
game_guard_enabled = true

[exclusions]
processes = [
    "csrss.exe",
    "dwm.exe",
    "audiodg.exe",
    "explorer.exe",
    "services.exe",
    "smss.exe",
    "lsass.exe"
]
```

---

## 5. UI Dashboard & Visual Representation

### 5.1 Telemetry Header & Segmented Memory Bar
1. **Header:**
   - Left: `[M] MEMCLEAN - PHYSICAL RAM TELEMETRY`
   - Right: `Timer Res: 0.500 ms (Locked)` or `Timer Res: 15.625 ms`
2. **Segmented Memory Distribution Bar:**
   Rendered with native Win32 GDI `FillRect` using custom brush palettes conforming to Minimalist UI Protocol:
   - **In-Use:** Charcoal `#111111`
   - **Standby:** Cool Slate `#7A9AB5`
   - **Modified:** Amber `#D97706`
   - **Free/Zeroed:** Hairline `#EAEAEA`
3. **Legend Text:**
   `■ In-Use: 9,412 MB (57%)  ■ Standby: 6,948 MB (42%)  ■ Modified: 47 MB  □ Free: 1,112 MB`

### 5.2 Action Ribbon
- `[ Optimize All RAM ]` (Primary Action)
- `[ Purge Standby ]` (Instant Standby Purge)
- `[ Combine Pages ]` (Run Memory De-Duplication)
- `[ Settings ]`

### 5.3 Top 5 RAM Consumers
- Displays PID, process name, working set size (MB).
- Action buttons per process:
  - `[ Trim ]`
  - `[ EcoQoS ]` (toggles Windows 11 Efficiency Mode)

### 5.4 System Tray Integration
- Dynamic percentage icon (monochrome GDI bitmap).
- Tooltip: `MemClean | Used: 57% | Standby: 6.9 GB | Timer: 0.500ms`
- Context Menu:
  - `Optimize All RAM`
  - `Purge Standby List (Quick)`
  - `Combine Duplicate Pages`
  - `Timer Resolution -> [ Default (15.6ms) | 1.0 ms | ✓ 0.5 ms ]`
  - Separator
  - `Dashboard...`
  - `Settings...`
  - `Exit`

---

## 6. CLI Automation & Scripting Interface

Commands supported by `memclean.exe`:

```bash
# General full optimization
memclean.exe --clean

# Surgical purges
memclean.exe --standby
memclean.exe --combine
memclean.exe --working-sets

# Timer resolution configuration
memclean.exe --timer-res 0.5

# Diagnostics and machine-readable output
memclean.exe --stats
memclean.exe --stats --json

# Flags
--silent, -s      # Suppress console stdout (for silent batch/scripts)
--help, -h        # Display CLI usage manual
```

### JSON Schema Output (`--stats --json`):
```json
{
  "timestamp": 1730295840,
  "memory": {
    "total_mb": 16384,
    "used_mb": 9412,
    "free_mb": 870,
    "zeroed_mb": 242,
    "standby_mb": 6948,
    "modified_mb": 47,
    "used_percent": 57
  },
  "timer_resolution": {
    "current_ms": 0.500,
    "minimum_ms": 15.625,
    "maximum_ms": 0.500
  },
  "top_consumers": [
    { "pid": 14208, "name": "chrome.exe", "working_set_mb": 1240 },
    { "pid": 9812, "name": "Code.exe", "working_set_mb": 850 }
  ]
}
```

---

## 7. Testing & Verification Plan

1. **Unit Tests:**
   - Configuration serde roundtrip with new triggers (`islc_enabled`, `timer_resolution_mode`, etc.).
   - Clamping rules for ISLC thresholds, timer resolution, and idle seconds.
   - Parsing of `SYSTEM_MEMORY_LIST_INFORMATION` into `ExtendedMemoryStats`.
2. **Integration / Smoke Tests:**
   - Verify `NtSetTimerResolution(5000, 1, ...)` locks resolution and restores on exit via throwaway probe.
   - Verify `NtQuerySystemInformation(80)` reads non-zero page counts for standby, free, zeroed.
   - Verify `SystemCombinePhysicalMemoryInformation = 130` call returns without panic or memory corruption.
   - Test CLI flags `--stats --json`, `--standby`, and `--combine`.
3. **UI Verification:**
   - Verify segmented memory distribution bar paints without flicker.
   - Verify settings dialog controls properly persist and update live monitoring loops.
   - Verify Tray tooltip updates dynamically with memory stats and timer resolution.
