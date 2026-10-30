# MemClean Power Upgrades Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement advanced kernel memory optimizations, microsecond gaming timer resolution (ISLC-grade), kernel physical page de-duplication, deep NT memory diagnostics with segmented GDI visualization, smart event triggers (AFK/idle & ISLC dual condition), and machine-readable CLI JSON automation.

**Architecture:** Pure Win32 Rust (`windows-sys` 0.59 + dynamic `ntdll` binding), modularly partitioned into kernel querying (`ntdll.rs`, `memory.rs`), high-resolution timer (`timer_res.rs`), page combining (`combine.rs`), smart engines (`islc.rs`, `idle.rs`), extended config (`config.rs`), owner-drawn GDI telemetry (`tray.rs`), and structured CLI execution (`main.rs`).

**Tech Stack:** Rust 2021, `windows-sys` 0.59, `serde` / `serde_json` / `toml`, Win32 GDI, `ntdll.dll`, Inno Setup 6.

**Spec:** `docs/superpowers/specs/2026-10-30-memclean-power-upgrades-design.md`

## Global Constraints
- Target OS: Windows 10 & 11 (x86_64); Rust 2021 edition.
- Binary size < 1 MB; RAM footprint < 2 MB; 0.0% idle CPU; zero external DLL dependencies.
- Minimalist UI Protocol: warm monochrome `#FBFBFA`/`#FFFFFF`, off-black text `#111111`, hairline borders `#EAEAEA`, spot pastels, zero emojis.
- Backwards compatibility: legacy `memclean.toml` files must deserialize cleanly with safe defaults.
- All kernel timer and privilege state must cleanly revert to system defaults on exit (RAII guards).

---

### Task 1: Deep NT Memory Query & Telemetry Model

**Files:**
- Modify: `memclean/src/core/ntdll.rs`
- Modify: `memclean/src/core/memory.rs`
- Modify: `memclean/Cargo.toml` (add `serde_json` if needed for serialization)
- Test: `memclean/tests/test_memory_breakdown.rs`

**Interfaces:**
- Consumes: `ntdll::NtQuerySystemInformation`
- Produces:
  ```rust
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

  pub fn get_extended_memory_stats() -> Result<ExtendedMemoryStats, String>;
  ```

- [ ] **Step 1: Write the failing integration test**

Create `memclean/tests/test_memory_breakdown.rs`:
```rust
use memclean::core::memory;

#[test]
fn test_extended_memory_stats_query() {
    let stats = memory::get_extended_memory_stats();
    assert!(stats.is_ok(), "Extended memory query should succeed on Windows");
    let s = stats.unwrap();
    assert!(s.total_phys_mb > 0);
    assert!(s.used_phys_mb > 0);
    assert!(s.total_phys_mb >= s.used_phys_mb);
    assert!(s.standby_total_mb <= s.total_phys_mb);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test test_memory_breakdown`
Expected: FAIL with `unresolved import` or `no function get_extended_memory_stats`.

- [ ] **Step 3: Implement `SystemMemoryListInformation` and `get_extended_memory_stats`**

In `memclean/src/core/ntdll.rs`, add dynamic `NtQuerySystemInformation` binding and `query_system_memory_list`:
```rust
type NtQuerySystemInformationFn = unsafe extern "system" fn(
    system_information_class: u32,
    system_information: *mut c_void,
    system_information_length: u32,
    return_length: *mut u32,
) -> i32;

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
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

pub fn query_system_memory_list() -> Result<SystemMemoryListInformation, String> {
    unsafe {
        let h_ntdll = GetModuleHandleA(b"ntdll.dll\0".as_ptr());
        if h_ntdll.is_null() {
            return Err("Failed to obtain ntdll.dll module handle".to_string());
        }

        let proc = GetProcAddress(h_ntdll, b"NtQuerySystemInformation\0".as_ptr());
        if proc.is_none() {
            return Err("Failed to resolve NtQuerySystemInformation in ntdll".to_string());
        }

        let nt_query_sys_info: NtQuerySystemInformationFn = std::mem::transmute(proc);
        let mut info = SystemMemoryListInformation::default();
        let mut ret_len = 0u32;
        let status = nt_query_sys_info(
            SYSTEM_MEMORY_LIST_INFORMATION,
            &mut info as *mut _ as *mut c_void,
            std::mem::size_of::<SystemMemoryListInformation>() as u32,
            &mut ret_len,
        );

        if status >= 0 {
            Ok(info)
        } else {
            Err(format!("NtQuerySystemInformation failed with NTSTATUS: {:#010X}", status))
        }
    }
}
```

In `memclean/src/core/memory.rs`, expose `ExtendedMemoryStats` and `get_extended_memory_stats()`:
```rust
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
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test test_memory_breakdown`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add memclean/src/core/ntdll.rs memclean/src/core/memory.rs memclean/tests/test_memory_breakdown.rs
git commit -m "feat(core): implement NtQuerySystemInformation(80) and ExtendedMemoryStats"
```

---

### Task 2: High-Resolution Timer Engine (`timer_res.rs`)

**Files:**
- Create: `memclean/src/core/timer_res.rs`
- Modify: `memclean/src/core/mod.rs`
- Modify: `memclean/src/core/ntdll.rs`
- Test: `memclean/tests/test_timer_res.rs`

**Interfaces:**
- Consumes: `NtSetTimerResolution`, `NtQueryTimerResolution` in `ntdll.rs`
- Produces:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
  pub enum TimerResolutionMode {
      Default,
      OneMs,
      HalfMs,
  }

  pub struct TimerResolutionGuard;
  impl TimerResolutionGuard {
      pub fn set_mode(mode: TimerResolutionMode) -> Result<Self, String>;
      pub fn get_current_resolution_ms() -> f32;
  }
  ```

- [ ] **Step 1: Write the failing unit test**

Create `memclean/tests/test_timer_res.rs`:
```rust
use memclean::core::timer_res::{TimerResolutionGuard, TimerResolutionMode};

#[test]
fn test_timer_resolution_guard_lifecycle() {
    let current_before = TimerResolutionGuard::get_current_resolution_ms();
    assert!(current_before > 0.0);

    {
        let guard = TimerResolutionGuard::set_mode(TimerResolutionMode::OneMs);
        assert!(guard.is_ok());
        let current_during = TimerResolutionGuard::get_current_resolution_ms();
        assert!(current_during <= 1.05);
    } // Guard drops here

    let current_after = TimerResolutionGuard::get_current_resolution_ms();
    assert!(current_after >= 0.4);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test test_timer_res`
Expected: FAIL with missing module `timer_res`.

- [ ] **Step 3: Implement `timer_res.rs`**

Create `memclean/src/core/timer_res.rs`:
Bind dynamic `NtSetTimerResolution` and `NtQueryTimerResolution` with `winmm::timeBeginPeriod` fallback. Implement RAII `Drop` to call `NtSetTimerResolution(..., 0, ...)` and `timeEndPeriod`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test test_timer_res`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add memclean/src/core/timer_res.rs memclean/src/core/mod.rs memclean/src/core/ntdll.rs memclean/tests/test_timer_res.rs
git commit -m "feat(core): implement high-resolution timer engine with RAII drop guard"
```

---

### Task 3: Kernel Physical Page Combining (`combine.rs`)

**Files:**
- Create: `memclean/src/core/combine.rs`
- Modify: `memclean/src/core/mod.rs`
- Modify: `memclean/src/core/ntdll.rs`
- Test: `memclean/tests/test_combine.rs`

**Interfaces:**
- Consumes: `NtSetSystemInformation(130, ...)`
- Produces:
  ```rust
  #[derive(Debug, Clone, Copy, Default)]
  pub struct CombineResult {
      pub pages_combined: usize,
      pub mb_saved: f32,
  }

  pub fn combine_physical_memory() -> Result<CombineResult, String>;
  ```

- [ ] **Step 1: Write the failing unit test**

Create `memclean/tests/test_combine.rs`:
```rust
use memclean::core::combine;

#[test]
fn test_combine_physical_memory_call() {
    // Calling combine should either succeed (if elevated) or return Err without crashing
    let res = combine::combine_physical_memory();
    match res {
        Ok(c) => assert!(c.pages_combined >= 0),
        Err(e) => assert!(!e.is_empty()),
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test test_combine`
Expected: FAIL with missing module `combine`.

- [ ] **Step 3: Implement `combine.rs`**

Create `memclean/src/core/combine.rs`:
```rust
use std::ffi::c_void;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

const SYSTEM_COMBINE_PHYSICAL_MEMORY_INFORMATION: u32 = 130;

#[repr(C)]
struct MemoryCombineInformationEx {
    handle: *mut c_void,
    pages_combined: usize,
    flags: u32,
}

pub fn combine_physical_memory() -> Result<CombineResult, String> {
    unsafe {
        let h_ntdll = GetModuleHandleA(b"ntdll.dll\0".as_ptr());
        if h_ntdll.is_null() {
            return Err("Failed to obtain ntdll.dll module handle".to_string());
        }

        let proc = GetProcAddress(h_ntdll, b"NtSetSystemInformation\0".as_ptr());
        if proc.is_none() {
            return Err("Failed to resolve NtSetSystemInformation in ntdll".to_string());
        }

        let nt_set_sys_info: unsafe extern "system" fn(u32, *const c_void, u32) -> i32 =
            std::mem::transmute(proc);

        let mut info = MemoryCombineInformationEx {
            handle: std::ptr::null_mut(),
            pages_combined: 0,
            flags: 0,
        };

        let status = nt_set_sys_info(
            SYSTEM_COMBINE_PHYSICAL_MEMORY_INFORMATION,
            &mut info as *mut _ as *const c_void,
            std::mem::size_of::<MemoryCombineInformationEx>() as u32,
        );

        if status >= 0 {
            let mb = (info.pages_combined as f32 * 4.0) / 1024.0;
            Ok(CombineResult {
                pages_combined: info.pages_combined,
                mb_saved: mb,
            })
        } else {
            Err(format!("NtSetSystemInformation(130) failed with NTSTATUS: {:#010X}", status))
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test test_combine`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add memclean/src/core/combine.rs memclean/src/core/mod.rs memclean/tests/test_combine.rs
git commit -m "feat(core): implement kernel physical page combining via class 130"
```

---

### Task 4: Intelligent Standby (ISLC) & Inactivity/AFK Engine

**Files:**
- Create: `memclean/src/engine/islc.rs`
- Create: `memclean/src/engine/idle.rs`
- Modify: `memclean/src/engine/mod.rs`
- Test: `memclean/tests/test_islc_and_idle.rs`

**Interfaces:**
- Consumes: `ExtendedMemoryStats`, Win32 `GetLastInputInfo`
- Produces:
  ```rust
  pub fn should_islc_purge(stats: &ExtendedMemoryStats, standby_threshold_mb: u64, free_threshold_mb: u64) -> bool;
  pub fn get_user_idle_seconds() -> u64;
  pub fn should_idle_clean(idle_seconds_threshold: u32, already_cleaned_in_session: bool) -> bool;
  ```

- [ ] **Step 1: Write the failing unit test**

Create `memclean/tests/test_islc_and_idle.rs`:
```rust
use memclean::core::memory::ExtendedMemoryStats;
use memclean::engine::islc::should_islc_purge;
use memclean::engine::idle::should_idle_clean;

#[test]
fn test_islc_trigger_conditions() {
    let mut stats = ExtendedMemoryStats {
        total_phys_mb: 16384,
        used_phys_mb: 8000,
        free_phys_mb: 800,
        zeroed_phys_mb: 100,
        standby_total_mb: 5000,
        standby_by_priority_mb: [0; 8],
        modified_mb: 50,
        used_percent: 50,
    };

    // Standby 5000 >= 1024, Free 800 <= 1024 -> Trigger!
    assert!(should_islc_purge(&stats, 1024, 1024));

    // Free memory is abundant (2000 > 1024) -> No trigger!
    stats.free_phys_mb = 2000;
    assert!(!should_islc_purge(&stats, 1024, 1024));

    // Standby memory is small (500 < 1024) -> No trigger!
    stats.free_phys_mb = 800;
    stats.standby_total_mb = 500;
    assert!(!should_islc_purge(&stats, 1024, 1024));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test test_islc_and_idle`
Expected: FAIL with missing module `islc`.

- [ ] **Step 3: Implement `islc.rs` and `idle.rs`**

Create `memclean/src/engine/islc.rs` and `memclean/src/engine/idle.rs` with `GetLastInputInfo` logic.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test test_islc_and_idle`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add memclean/src/engine/islc.rs memclean/src/engine/idle.rs memclean/src/engine/mod.rs memclean/tests/test_islc_and_idle.rs
git commit -m "feat(engine): implement ISLC dual-condition logic and user idle tracker"
```

---

### Task 5: Process EcoQoS / Efficiency Mode

**Files:**
- Modify: `memclean/src/core/process.rs`
- Test: `memclean/tests/test_process_eco.rs`

**Interfaces:**
- Consumes: Win32 `SetProcessInformation`
- Produces:
  ```rust
  pub fn set_process_eco_qos(pid: u32, enable: bool) -> Result<(), String>;
  ```

- [ ] **Step 1: Write the failing test**

Create `memclean/tests/test_process_eco.rs`:
```rust
use memclean::core::process;

#[test]
fn test_set_current_process_eco_qos() {
    let cur_pid = std::process::id();
    let res = process::set_process_eco_qos(cur_pid, true);
    // On supported Windows 11 versions this succeeds; on older systems it gracefully returns an Err
    if let Err(e) = res {
        assert!(!e.is_empty());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test test_process_eco`
Expected: FAIL with `function set_process_eco_qos not found`.

- [ ] **Step 3: Implement `set_process_eco_qos`**

In `memclean/src/core/process.rs`, implement `set_process_eco_qos` via `SetProcessInformation` using `ProcessPowerThrottling`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test test_process_eco`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add memclean/src/core/process.rs memclean/tests/test_process_eco.rs
git commit -m "feat(core): implement Windows 11 EcoQoS process power throttling"
```

---

### Task 6: Configuration Schema Expansion (`config.rs`)

**Files:**
- Modify: `memclean/src/config.rs`
- Test: `memclean/tests/test_config_expansion.rs`

**Interfaces:**
- Produces:
  - `TriggersConfig` with `islc_enabled`, `islc_standby_mb`, `islc_free_mb`, `idle_clean_enabled`, `idle_seconds`.
  - `TuningConfig` with `timer_resolution_enabled`, `timer_resolution_mode`, `combine_pages_on_clean`.
  - Clamping helpers: `clamp_islc_mb()`, `clamp_idle_seconds()`.

- [ ] **Step 1: Write the failing unit test**

Create `memclean/tests/test_config_expansion.rs`:
Test serialization, deserialization of legacy config (without new fields), and clamping.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test test_config_expansion`
Expected: FAIL with missing fields or types.

- [ ] **Step 3: Implement expanded schema in `config.rs`**

Update `memclean/src/config.rs` with `#[serde(default)]` on all new fields so that legacy configs parse without errors.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test test_config_expansion`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add memclean/src/config.rs memclean/tests/test_config_expansion.rs
git commit -m "feat(config): expand schema with ISLC triggers, idle clean, and timer tuning"
```

---

### Task 7: Cleaner Orchestration & CLI Automation Expansion

**Files:**
- Modify: `memclean/src/engine/cleaner.rs`
- Modify: `memclean/src/main.rs`
- Test: Throwaway CLI smoke test run via `cargo run -- --stats --json`

**Interfaces:**
- Produces:
  - CLI flags: `--clean`, `--standby`, `--combine`, `--working-sets`, `--timer-res <ms>`, `--stats`, `--stats --json`, `--silent`.
  - Machine-readable JSON output on `--stats --json`.

- [ ] **Step 1: Update `perform_clean` in `cleaner.rs`**

Add `combine_physical_memory()` support when `config.tuning.combine_pages_on_clean` is enabled or in aggressive mode. Record combined pages in `CleanResult`.

- [ ] **Step 2: Implement CLI flag parsing in `main.rs`**

Add support for `--standby`, `--combine`, `--timer-res`, `--stats`, and `--json`. When `--json` is requested with `--stats`, print serialized JSON schema.

- [ ] **Step 3: Smoke test CLI output**

Run: `cargo run -- --stats --json`
Verify clean JSON output with memory breakdown and timer resolution info.

- [ ] **Step 4: Commit**

```bash
git add memclean/src/engine/cleaner.rs memclean/src/main.rs
git commit -m "feat(cli): add --stats --json, --combine, --standby, and --timer-res automation"
```

---

### Task 8: Win32 Dashboard UI Upgrade (Telemetry Bar & Actions)

**Files:**
- Modify: `memclean/src/ui/tray.rs`

**Interfaces:**
- Produces:
  - Segmented GDI memory distribution bar (In-Use, Standby, Modified, Free) + live legend.
  - Header Timer Resolution indicator (`Timer Res: 0.500 ms (Locked)`).
  - Action ribbon: `[ Clean All RAM ]`, `[ Purge Standby ]`, `[ Combine Pages ]`, `[ Settings ]`.
  - Process list with `[ Trim ]` and `[ EcoQoS ]` owner-drawn buttons.
  - Dynamic tray hover tooltip & context menu additions.

- [ ] **Step 1: Add custom GDI brushes for segmented bar**

Create brushes for Charcoal (`#111111`), Slate (`#7A9AB5`), Amber (`#D97706`), and Hairline (`#EAEAEA`).

- [ ] **Step 2: Render segmented distribution bar in `WM_PAINT`**

Compute pixel widths proportionally from `ExtendedMemoryStats` (`used_phys_mb`, `standby_total_mb`, `modified_mb`, `free_phys_mb`). Paint segments consecutively using `FillRect`.

- [ ] **Step 3: Add Action ribbon buttons and handlers**

Add button IDs for `Purge Standby` (102) and `Combine Pages` (103). Handle commands in `WM_COMMAND`.

- [ ] **Step 4: Add EcoQoS button per process in Top 5 list**

Render secondary `[ Eco ]` button next to `[ Trim ]` on each process row.

- [ ] **Step 5: Verify build and smoke test visually**

Compile and run `memclean.exe` to verify window layout, bar rendering, and status messages.

- [ ] **Step 6: Commit**

```bash
git add memclean/src/ui/tray.rs
git commit -m "feat(ui): add segmented GDI memory bar, timer badge, and ribbon actions"
```

---

### Task 9: Settings Dialog Upgrade (ISLC, Timer Tuning, AFK Clean)

**Files:**
- Modify: `memclean/src/ui/dialog.rs`

**Interfaces:**
- Produces:
  - Grouped controls for ISLC Standby Threshold (MB) & Free RAM Threshold (MB).
  - Grouped controls for Timer Resolution mode (`Default`, `1.0ms`, `0.5ms`).
  - Grouped controls for Idle AFK cleanup (minutes).
  - Page combining toggle checkbox.

- [ ] **Step 1: Add control IDs and labels in `dialog.rs`**

Define control IDs for ISLC checkbox (220), Standby edit (221), Free edit (222), Timer mode dropdown (230), Idle clean checkbox (240), Idle minutes edit (241).

- [ ] **Step 2: Populate controls from `AppConfig` on dialog init**

Load values, apply validation and clamping on save button click (301).

- [ ] **Step 3: Save and apply settings dynamically**

Update running `TimerResolutionGuard` and reload monitoring parameters.

- [ ] **Step 4: Verify dialog interaction**

Launch dashboard, open Settings, modify ISLC thresholds and timer resolution, save, and confirm persistence in `memclean.toml`.

- [ ] **Step 5: Commit**

```bash
git add memclean/src/ui/dialog.rs
git commit -m "feat(dialog): add ISLC dual thresholds, timer resolution, and AFK clean controls"
```

---

### Task 10: Verification, Installer Build, and Site Sync

**Files:**
- Modify: `installer/setup.iss`
- Modify: `website/index.html`
- Modify: `README.md`

- [ ] **Step 1: Run full test suite**

Run: `cargo test`
Ensure all 11+ tests pass with zero warnings or errors.

- [ ] **Step 2: Build release binaries and Inno Setup installer**

Run:
```bash
cargo build --release
"C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer/setup.iss
```

- [ ] **Step 3: Update documentation and website**

Update `README.md` and `website/index.html` with the new capabilities (ISLC Dual Engine, 0.5ms Timer Resolution, Kernel Page Combining, Granular Telemetry).

- [ ] **Step 4: Commit and push to `master`**

```bash
git add .
git commit -m "feat: complete memclean power upgrades with ISLC engine, timer res, and page combining"
git push origin master
```
