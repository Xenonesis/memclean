# Kernel Stability, UAC Elevation & Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the 5 architectural gaps and subtle bugs identified in kernel testing: auto-UAC elevation with truthful privilege reporting, Settings dialog Windows startup integration, persistent & interactive CLI timer resolution, interactive CLI Gaming HUD overlay, and non-zero CLI error exit codes.

**Architecture:** Implement token-based elevation detection via Win32 `GetTokenInformation` and auto-relaunch using `ShellExecuteW(..., "runas", ...)`; replace unchecked silent kernel calls with truthful user feedback; wire autostart registration into `core::process` and the Settings Dialog Tab 0; expand `--timer-res` to persist to TOML config and support `--wait` console hold with `SetConsoleCtrlHandler`; replace hardcoded HUD sleep with an interactive console event loop; add non-zero process exits for CLI failures.

**Tech Stack:** Rust 2021, `windows-sys` 0.59 (`Win32_Security`, `Win32_UI_Shell`, `Win32_System_Threading`), Native Win32 GDI & Registry, Inno Setup 6.

---

### Task 1: Privilege Subsystem - `is_elevated()` & Auto-UAC Elevation

**Files:**
- Modify: `memclean/src/core/privileges.rs`
- Modify: `memclean/src/main.rs`
- Test: `memclean/tests/test_privileges.rs`

**Interfaces:**
- Produces: `pub fn is_elevated() -> bool` in `memclean::core::privileges`
- Produces: `pub fn relaunch_as_admin() -> Result<(), String>` in `memclean::core::privileges`
- Consumes: `windows_sys::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION}` and `windows_sys::Win32::UI::Shell::ShellExecuteW`

- [ ] **Step 1: Write unit test for `is_elevated()`**

Create `memclean/tests/test_privileges.rs`:
```rust
use memclean::core::privileges;

#[test]
fn test_is_elevated_callable_without_panic() {
    let elevated = privileges::is_elevated();
    // On Windows, is_elevated returns a valid boolean without panicking or leaking handles
    println!("Process elevation status: {}", elevated);
}
```

- [ ] **Step 2: Run test to verify compilation failure or undefined symbol**

Run: `cargo test --test test_privileges`
Expected: FAIL or error indicating `is_elevated` does not exist.

- [ ] **Step 3: Implement `is_elevated()` and `relaunch_as_admin()` in `privileges.rs`**

In `memclean/src/core/privileges.rs`, add:
```rust
use windows_sys::Win32::Security::{
    AdjustTokenPrivileges, GetTokenInformation, LookupPrivilegeValueW, TokenElevation,
    LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_ELEVATION,
    TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

/// Check if current process token has elevated / Administrator privileges
pub fn is_elevated() -> bool {
    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut ret_len: u32 = 0;
        let res = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as _,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut ret_len,
        );

        CloseHandle(token);

        res != 0 && elevation.TokenIsElevated != 0
    }
}

/// Relaunch current executable with elevated Administrator privileges via UAC ShellExecuteW
pub fn relaunch_as_admin(args: &[String]) -> Result<(), String> {
    let current_exe = std::env::current_exe().map_err(|e| format!("Failed to get current_exe: {}", e))?;
    let exe_str = current_exe.to_str().ok_or_else(|| "Invalid exe path UTF-8".to_string())?;

    let wide_verb = encode_wide("runas");
    let wide_file = encode_wide(exe_str);
    let params_str = args.join(" ");
    let wide_params = if params_str.is_empty() {
        Vec::new()
    } else {
        encode_wide(&params_str)
    };

    unsafe {
        let h_inst = ShellExecuteW(
            std::ptr::null_mut(),
            wide_verb.as_ptr(),
            wide_file.as_ptr(),
            if wide_params.is_empty() {
                std::ptr::null()
            } else {
                wide_params.as_ptr()
            },
            std::ptr::null(),
            SW_SHOWNORMAL as i32,
        );

        if h_inst as usize > 32 {
            Ok(())
        } else {
            Err(format!("UAC elevation request failed or was dismissed (code: {})", h_inst as usize))
        }
    }
}
```

- [ ] **Step 4: Update `main.rs` to handle auto-elevation for GUI mode**

In `memclean/src/main.rs`:
When no CLI command is passed (or `--minimized` is passed for GUI tray startup):
If `!privileges::is_elevated()` and `config.general.auto_elevate`:
Attempt `privileges::relaunch_as_admin(&args[1..])`. If it returns `Ok(())`, the elevated child is running, so exit parent cleanly (`return;`).
If the user clicks "No" on UAC or an error occurs, log the standard mode notice and continue in GUI mode without crashing.

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --test test_privileges`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add memclean/src/core/privileges.rs memclean/src/main.rs memclean/tests/test_privileges.rs
git commit -m "feat(privileges): implement is_elevated detection and auto-UAC elevation relaunch"
```

---

### Task 2: Truthful Feedback in Tray & Dashboard Actions

**Files:**
- Modify: `memclean/src/ui/tray.rs`
- Modify: `memclean/src/engine/cleaner.rs`

**Interfaces:**
- Modifies: `WM_COMMAND` handlers in `memclean::ui::tray` (button IDs 202, 203, 503, 504, 505) to check `Result` and set accurate status text.

- [ ] **Step 1: Inspect `cleaner.rs` and `tray.rs` error paths**

Ensure all kernel commands (`ntdll::purge_standby_list()`, `ntdll::combine_physical_memory()`, `ntdll::empty_system_working_sets()`, `ntdll::trim_system_file_cache()`) propagate errors clearly.

- [ ] **Step 2: Update `tray.rs` button & menu handlers**

In `memclean/src/ui/tray.rs`:
For ID `202 | 503` (Purge Standby):
```rust
match crate::core::ntdll::purge_standby_list() {
    Ok(()) => set_text(hwnd, 103, "Status: Standby list purged."),
    Err(e) => set_text(hwnd, 103, &format!("Status: Standby purge failed (Requires Admin)")),
}
```
For ID `504` (Empty Working Sets):
```rust
match crate::core::ntdll::empty_system_working_sets() {
    Ok(()) => set_text(hwnd, 103, "Status: Working sets emptied."),
    Err(e) => set_text(hwnd, 103, "Status: Working sets empty failed."),
}
```
For ID `505` (Trim System File Cache):
```rust
match crate::core::ntdll::trim_system_file_cache() {
    Ok(()) => set_text(hwnd, 103, "Status: System file cache trimmed."),
    Err(_) => set_text(hwnd, 103, "Status: File cache trim failed (Requires Admin)."),
}
```

- [ ] **Step 3: Update Main Dashboard Header to reflect elevation state**

In `memclean/src/ui/tray.rs`:
In `WM_PAINT`, when drawing Card 1 header `[M] MEMCLEAN - PHYSICAL RAM TELEMETRY`:
If `!crate::core::privileges::is_elevated()`:
Draw header as `[M] MEMCLEAN - RAM TELEMETRY (STANDARD USER - LIMITED KERNEL RIGHTS)` or display an indicator badge so user immediately knows why standby/combine require admin.

- [ ] **Step 4: Run full test suite**

Run: `cargo test`
Expected: PASS (all tests pass)

- [ ] **Step 5: Commit**

```bash
git add memclean/src/ui/tray.rs memclean/src/engine/cleaner.rs
git commit -m "fix(ui): provide truthful status feedback for privileged kernel memory actions"
```

---

### Task 3: Startup Registration Subsystem & Settings Dialog Integration

**Files:**
- Modify: `memclean/src/core/process.rs`
- Modify: `memclean/src/ui/dialog.rs`
- Modify: `memclean/src/main.rs`
- Test: `memclean/tests/test_autostart.rs`

**Interfaces:**
- Produces: `pub fn register_autostart(enable: bool) -> Result<(), String>` in `memclean::core::process`
- Consumes: `windows_sys::Win32::System::Registry`

- [ ] **Step 1: Write unit test for `register_autostart`**

Create `memclean/tests/test_autostart.rs`:
```rust
use memclean::core::process;

#[test]
fn test_register_autostart_toggle() {
    // Test that toggling false (unregister) works cleanly without panic
    let res = process::register_autostart(false);
    assert!(res.is_ok());
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test --test test_autostart`
Expected: FAIL indicating `register_autostart` not found in `process`.

- [ ] **Step 3: Move `register_autostart` into `memclean/src/core/process.rs`**

Implement `register_autostart(enable: bool) -> Result<(), String>` in `core/process.rs`:
```rust
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
    KEY_ALL_ACCESS, REG_SZ,
};

pub fn register_autostart(enable: bool) -> Result<(), String> {
    unsafe {
        let subkey: Vec<u16> = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run\0"
            .encode_utf16()
            .collect();
        let val_name: Vec<u16> = "MemClean\0".encode_utf16().collect();
        let mut hkey: HKEY = std::ptr::null_mut();

        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_ALL_ACCESS, &mut hkey) == 0 && !hkey.is_null() {
            if enable {
                let current_exe = std::env::current_exe().map_err(|e| format!("Get current_exe failed: {}", e))?;
                let exe_str = current_exe.to_str().ok_or_else(|| "Invalid UTF-8 in exe path".to_string())?;
                let value_data = format!("\"{}\" --minimized\0", exe_str);
                let wide_data: Vec<u16> = value_data.encode_utf16().collect();

                let res = RegSetValueExW(
                    hkey,
                    val_name.as_ptr(),
                    0,
                    REG_SZ,
                    wide_data.as_ptr() as _,
                    (wide_data.len() * 2) as u32,
                );
                RegCloseKey(hkey);
                if res == 0 {
                    Ok(())
                } else {
                    Err(format!("RegSetValueExW failed with code {}", res))
                }
            } else {
                let _ = RegDeleteValueW(hkey, val_name.as_ptr());
                RegCloseKey(hkey);
                Ok(())
            }
        } else {
            Err("Failed to open HKCU Run registry key".to_string())
        }
    }
}
```

- [ ] **Step 4: Update Settings Dialog in `dialog.rs` with Startup Checkbox**

In `memclean/src/ui/dialog.rs`:
1. Add `const CHK_RUN_ON_STARTUP: i32 = 216;` to `TAB_0_CTRLS`.
2. In `create_controls()` under Card 2:
   ```rust
   create_chk(windows_sys::w!("Enable Global Hotkey (Ctrl + Alt + M)"), 30, 326, 420, 204);
   create_chk(windows_sys::w!("Enable Gaming HUD Micro-Overlay (Ctrl + Shift + O)"), 30, 344, 420, 215);
   create_chk(windows_sys::w!("Launch MemClean automatically on Windows startup"), 30, 362, 420, 216);
   create_chk(windows_sys::w!("Start Minimized to System Tray on Login"), 30, 380, 420, 203);
   create_chk(windows_sys::w!("Automatically check for updates on startup"), 30, 398, 420, 214);
   ```
3. In `load_config_to_controls()`:
   `set_checked(hwnd, 216, cfg.general.run_on_startup);`
4. In `save_config_from_controls()` (Button 301):
   ```rust
   let run_on_startup = is_checked(hwnd, 216);
   cfg.general.run_on_startup = run_on_startup;
   let _ = crate::core::process::register_autostart(run_on_startup);
   ```
5. Update `TAB_0_CTRLS` and test constants in `dialog.rs`.

- [ ] **Step 5: Run tests and verify**

Run: `cargo test`
Expected: PASS (all tests pass)

- [ ] **Step 6: Commit**

```bash
git add memclean/src/core/process.rs memclean/src/ui/dialog.rs memclean/src/main.rs memclean/tests/test_autostart.rs
git commit -m "feat(settings): integrate live Windows autostart checkbox in Settings dialog"
```

---

### Task 4: Persistent & Interactive CLI Timer Resolution Engine

**Files:**
- Modify: `memclean/src/main.rs`
- Modify: `memclean/src/core/timer_res.rs`
- Modify: `README.md`

**Interfaces:**
- Modifies: `memclean.exe --timer-res <0.5|1.0|default> [--wait|-w]`
- Persists setting into `AppConfig` and supports `--wait` mode to hold resolution until Ctrl+C.

- [ ] **Step 1: Implement console Ctrl+C handler and wait loop in `main.rs`**

When `--timer-res` is provided:
1. Parse the requested target (`"0.5"`, `"1.0"`, `"default"`).
2. Load `AppConfig::load()`, update `cfg.tuning.timer_resolution_mode`, and save `cfg.save()`.
3. If `--wait` or `-w` is passed:
   - Create `TimerResolutionGuard::new()` and `set_mode()`.
   - Print:
     ```
     [OK] High-resolution timer locked to 0.500 ms.
     Holding scheduler precision active. Press Ctrl+C to restore default and exit...
     ```
   - Register `SetConsoleCtrlHandler` to handle Ctrl+C / SIGINT cleanly.
   - Wait on an `AtomicBool` loop (`thread::sleep(Duration::from_millis(200))`).
   - On exit, drop guard to restore system resolution cleanly.
4. If `--wait` is NOT passed:
   - Print clear explanation:
     ```
     [OK] Timer resolution set to 0.500 ms in configuration.
     Background MemClean daemon will maintain this precision automatically.
     (To hold this resolution standalone in this console, pass: memclean --timer-res 0.5 --wait)
     ```

- [ ] **Step 2: Update README.md CLI table with `--wait` option**

Update `README.md` to document:
`memclean.exe --timer-res <0.5|1.0|default> [--wait]` - Persists timer resolution to config. Pass `--wait` to lock resolution in the current terminal until Ctrl+C.

- [ ] **Step 3: Test `--timer-res 0.5` and `--timer-res default`**

Verify that config is saved and output is clear.

- [ ] **Step 4: Commit**

```bash
git add memclean/src/main.rs memclean/src/core/timer_res.rs README.md
git commit -m "feat(cli): make timer resolution persist to config and add interactive --wait mode"
```

---

### Task 5: Interactive Gaming HUD CLI Mode & CLI Exit Codes

**Files:**
- Modify: `memclean/src/main.rs`
- Modify: `memclean/src/core/network.rs`

**Interfaces:**
- CLI flags return exit code `1` on error (`std::process::exit(1)`)
- `memclean.exe --hud` runs interactive message pump / console waiter instead of hardcoded 10s sleep.

- [ ] **Step 1: Update `main.rs` for `--hud` interactive mode**

Replace `sleep(10)` in `main.rs` with an interactive loop:
```rust
if args.iter().any(|a| a == "--hud") {
    print_line("=== MemClean Gaming HUD Micro-Overlay (Zero-Injection) ===");
    print_line("Status: Active. Press Ctrl+Shift+O anywhere to toggle visibility.");
    print_line("Press Ctrl+C in this console to close overlay and exit.");
    memclean::ui::hud::show_hud();

    // Setup Ctrl+C handler and wait until terminated
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    unsafe {
        windows_sys::Win32::System::Console::SetConsoleCtrlHandler(Some(ctrl_handler), 1);
    }
    while running.load(std::sync::atomic::Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    memclean::ui::hud::hide_hud();
    return;
}
```

- [ ] **Step 2: Update CLI error exit codes**

In `memclean/src/main.rs`:
- If `optimize_network()` returns `Err(e)`:
  `print_line(&format!("Error applying network optimization: {}", e)); std::process::exit(1);`
- If `restore_network_defaults()` returns `Err(e)`:
  `print_line(&format!("Error restoring network defaults: {}", e)); std::process::exit(1);`
- If `unpark_all_cores()` returns `Err(e)`:
  `print_line(&format!("Error unparking CPU cores: {}", e)); std::process::exit(1);`
- If `restore_core_parking_default()` returns `Err(e)`:
  `print_line(&format!("Error restoring CPU core parking: {}", e)); std::process::exit(1);`

- [ ] **Step 3: Run cargo test and manual checks**

Verify that non-elevated `--network-optimize` exits with code 1.

- [ ] **Step 4: Commit**

```bash
git add memclean/src/main.rs memclean/src/core/network.rs
git commit -m "fix(cli): run interactive Gaming HUD loop and return non-zero exit codes on errors"
```

---

### Task 6: Comprehensive Verification, Release Build & Installer Compilation

**Files:**
- Modify: `installer/setup.iss` (if version bump needed)
- Output: `target/release/memclean.exe`, `dist/memclean.exe`, `dist/MemClean_Setup.exe`

- [ ] **Step 1: Run full unit test suite**

Run: `cargo test`
Expected: All tests pass.

- [ ] **Step 2: Build optimized release binary**

Run: `cargo build --release`
Verify binary size and no compiler warnings.

- [ ] **Step 3: Copy to `dist/` and compile Inno Setup installer**

Run:
```powershell
cp target/release/memclean.exe dist/memclean.exe
& "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer/setup.iss
```
Expected: `dist/MemClean_Setup.exe` generated successfully.

- [ ] **Step 4: Live smoke test all 5 fixes**

1. Run `memclean.exe --status`: verify telemetry & elevation detection.
2. Run `memclean.exe --network-optimize`: verify exit code 1 when not elevated.
3. Run `memclean.exe --timer-res 0.5`: verify saved config output.
4. Launch GUI: verify Tab 0 has "Launch MemClean automatically on Windows startup" checkbox and saving toggles registry.
5. In GUI: click "Purge Standby": verify truthful feedback when unelevated vs elevated.

- [ ] **Step 5: Git commit and push to `master`**

```bash
git add .
git commit -m "release: kernel stability, auto-elevation, autostart integration, and CLI polish"
git push origin master
```
