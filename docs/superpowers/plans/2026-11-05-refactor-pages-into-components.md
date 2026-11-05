# Refactor Pages into Focused Components Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor monolithic UI and dashboard pages across the Rust native GUI (`memclean/src/ui/dialog.rs` and `memclean/src/ui/tray.rs`) and website frontend (`website/app.js`) into modular, single-responsibility components with zero regression in functionality, and clean up all unused imports and dead files.

**Architecture:** 
1. The 915-line Win32 Settings Dialog is decomposed into four focused tab components (`tab_system.rs`, `tab_triggers.rs`, `tab_kernel.rs`, `tab_hardware.rs`) coordinated by a clean `dialog/mod.rs` window host with dedicated lifecycle traits (`create_controls`, `load_config`, `save_config`).
2. The 1,500-line `tray.rs` is modularized into `icon.rs` (GDI dynamic percentage icon generation), `dashboard.rs` (custom memory distribution bar and Top 5 processes custom draw), and `tray/mod.rs` (window procedure, message loop, auto-clean orchestration).
3. The 547-line monolithic `website/app.js` is refactored into focused ES modules (`simulator.js`, `terminal.js`, `interactions.js`) with clean imports, eliminating dead code.

**Tech Stack:** Rust (2021 Edition, `windows-sys` 0.59 Win32 API), GDI, Vanilla ES6 JavaScript Modules.

**Spec:** `design-system/memclean/MASTER.md` & `docs/superpowers/specs/2026-10-30-memclean-power-upgrades-design.md`

## Global Constraints
- Target OS: Windows 10 & 11 (x86_64).
- No external runtime DLL dependencies.
- Zero functional regression: All 4 dialog tabs, controls (IDs 101-297), telemetry bar, Top 5 RAM consumers, hotkey, HUD, CLI commands, and web terminal simulator must work identically.
- Zero unused imports or dead variables: Codebase must compile cleanly under `cargo check` and `cargo test`.
- Clean file cutovers: Delete replaced monolithic files after modular cutover.

---

### Task 1: Decompose Settings Dialog Tab 0 & Tab 1 into Focused Components

**Files:**
- Create: `memclean/src/ui/dialog/tab_system.rs`
- Create: `memclean/src/ui/dialog/tab_triggers.rs`
- Create: `memclean/src/ui/dialog/common.rs`
- Test: `memclean/tests/test_config.rs`

**Interfaces:**
- `common.rs`: Exports shared helpers `set_checked`, `is_checked`, font handles (`D_FONT_UI`, `D_FONT_BOLD`, `D_FONT_SUBHEADER`, `D_FONT_DESC`), brush handles, and `create_chk`, `create_radio`.
- `tab_system.rs`:
  - `pub const CTRLS: &[i32] = &[101, 102, 103, 104, 105, 201, 202, 204, 215, 216, 203, 214];`
  - `pub unsafe fn create(hwnd: HWND, hinstance: isize)`
  - `pub unsafe fn load(hwnd: HWND, cfg: &AppConfig)`
  - `pub unsafe fn save(hwnd: HWND, cfg: &mut AppConfig)`
- `tab_triggers.rs`:
  - `pub const CTRLS: &[i32] = &[205, 206, 207, 208, 209, 210, 211, 212, 213, 220, 221, 222, 223, 224, 225, 226, 230, 231, 232, 233];`
  - `pub unsafe fn create(hwnd: HWND, hinstance: isize)`
  - `pub unsafe fn load(hwnd: HWND, cfg: &AppConfig)`
  - `pub unsafe fn save(hwnd: HWND, cfg: &mut AppConfig)`

- [ ] **Step 1: Create `dialog/common.rs` with shared Win32 control creation and state helpers**
- [ ] **Step 2: Extract Tab 0 (System Behavior, Tray Style, Hotkey, HUD, Startup) into `dialog/tab_system.rs`**
- [ ] **Step 3: Extract Tab 1 (RAM Threshold, Periodic Timer, ISLC Standby, Idle AFK) into `dialog/tab_triggers.rs`**
- [ ] **Step 4: Verify syntax and compile readiness with `cargo check`**
- [ ] **Step 5: Commit Tab 0 and Tab 1 component extractions**

```bash
git add memclean/src/ui/dialog/
git commit -m "refactor(dialog): extract Tab 0 system and Tab 1 triggers into focused components"
```

---

### Task 2: Decompose Settings Dialog Tab 2 & Tab 3 into Focused Components

**Files:**
- Create: `memclean/src/ui/dialog/tab_kernel.rs`
- Create: `memclean/src/ui/dialog/tab_hardware.rs`
- Test: `memclean/tests/test_config.rs`

**Interfaces:**
- `tab_kernel.rs`:
  - `pub const CTRLS: &[i32] = &[240, 241, 242, 243, 251, 250, 270, 275, 260, 261, 262];`
  - `pub unsafe fn create(hwnd: HWND, hinstance: isize)`
  - `pub unsafe fn load(hwnd: HWND, cfg: &AppConfig)`
  - `pub unsafe fn save(hwnd: HWND, cfg: &mut AppConfig)`
- `tab_hardware.rs`:
  - `pub const CTRLS: &[i32] = &[280, 281, 282, 283, 290, 291, 292, 295, 296, 297];`
  - `pub unsafe fn create(hwnd: HWND, hinstance: isize)`
  - `pub unsafe fn load(hwnd: HWND, cfg: &AppConfig)`
  - `pub unsafe fn save(hwnd: HWND, cfg: &mut AppConfig)`

- [ ] **Step 1: Extract Tab 2 (Kernel Memory Policy, Game Guard, Page Combining, EcoQoS) into `dialog/tab_kernel.rs`**
- [ ] **Step 2: Extract Tab 3 (Network TCP latency, VRAM Decommit, CPU Core Unpark) into `dialog/tab_hardware.rs`**
- [ ] **Step 3: Verify syntax and compile readiness with `cargo check`**
- [ ] **Step 4: Commit Tab 2 and Tab 3 component extractions**

```bash
git add memclean/src/ui/dialog/
git commit -m "refactor(dialog): extract Tab 2 kernel and Tab 3 hardware into focused components"
```

---

### Task 3: Assemble `dialog/mod.rs`, Cutover, and Delete Legacy Monolithic `dialog.rs`

**Files:**
- Create: `memclean/src/ui/dialog/mod.rs`
- Delete: `memclean/src/ui/dialog.rs`
- Modify: `memclean/src/ui/mod.rs`
- Test: `memclean/tests/test_config.rs`

**Interfaces:**
- `dialog/mod.rs`: Exports `pub fn show_settings_dialog(hparent: HWND)` identically to previous interface.
- Dispatches tab switching to `tab_system::CTRLS`, `tab_triggers::CTRLS`, `tab_kernel::CTRLS`, `tab_hardware::CTRLS`.
- Coordinates `load_config` and `save_config` across all 4 tab components.
- Manages GDI brushes/fonts setup and teardown (`WM_DESTROY`).

- [ ] **Step 1: Implement `dialog/mod.rs` integrating all 4 tab components**
- [ ] **Step 2: Remove obsolete `memclean/src/ui/dialog.rs` and update `memclean/src/ui/mod.rs`**
- [ ] **Step 3: Run `cargo check` and `cargo test` to verify zero compile errors and zero unused imports**
- [ ] **Step 4: Commit Settings Dialog component refactoring**

```bash
git rm memclean/src/ui/dialog.rs
git add memclean/src/ui/dialog/ memclean/src/ui/mod.rs
git commit -m "refactor(dialog): cutover settings dialog to modular tab architecture and remove monolithic file"
```

---

### Task 4: Modularize Main Dashboard & Tray Subsystems (`memclean/src/ui/tray/`)

**Files:**
- Create: `memclean/src/ui/tray/icon.rs`
- Create: `memclean/src/ui/tray/dashboard.rs`
- Create: `memclean/src/ui/tray/mod.rs`
- Delete: `memclean/src/ui/tray.rs`
- Modify: `memclean/src/ui/mod.rs`
- Test: `memclean/tests/test_memory_breakdown.rs`

**Interfaces:**
- `icon.rs`:
  - `pub fn create_percent_icon(percent: u32) -> HICON`
  - `pub fn create_percent_icon_with_config(percent: u32, cfg: &TrayConfig) -> HICON`
  - `pub fn format_tray_tooltip(percent: u32, standby_mb: u64, timer_res_ms: f32) -> String`
  - `pub unsafe fn load_embedded_icon(size: i32) -> HICON`
- `dashboard.rs`:
  - `pub unsafe fn paint_dashboard(hwnd: HWND, hdc: HDC, rc_client: &RECT)`
  - `pub unsafe fn update_stats_display(hwnd: HWND)`
  - `pub unsafe fn update_top_processes(hwnd: HWND)`
- `tray/mod.rs`:
  - `pub fn run_application(start_minimized: bool)`
  - Window procedure, timer routing, and hotkey synchronization.

- [ ] **Step 1: Extract icon generation logic into `memclean/src/ui/tray/icon.rs`**
- [ ] **Step 2: Extract GDI dashboard rendering and process list into `memclean/src/ui/tray/dashboard.rs`**
- [ ] **Step 3: Assemble `memclean/src/ui/tray/mod.rs` coordinating window loop and background events**
- [ ] **Step 4: Delete obsolete `memclean/src/ui/tray.rs` and update `memclean/src/ui/mod.rs`**
- [ ] **Step 5: Run full `cargo test` across all suites to ensure zero broken tests and zero unused imports**
- [ ] **Step 6: Commit tray and dashboard modular refactor**

```bash
git rm memclean/src/ui/tray.rs
git add memclean/src/ui/tray/ memclean/src/ui/mod.rs
git commit -m "refactor(ui): decompose tray and dashboard into icon, dashboard, and event loop components"
```

---

### Task 5: Modularize Website Frontend (`website/app.js` into ES Modules)

**Files:**
- Create: `website/js/modules/simulator.js`
- Create: `website/js/modules/terminal.js`
- Create: `website/js/modules/interactions.js`
- Modify: `website/app.js`
- Modify: `website/index.html`
- Test: Local browser verification via python HTTP server

**Interfaces:**
- `simulator.js`: Exports `initMemorySimulator()` managing gauge physics, live RAM graph, and purge simulation.
- `terminal.js`: Exports `initCliTerminal()` managing `COMMAND_DATA`, CLI tab buttons, and interactive terminal emulator.
- `interactions.js`: Exports `initInteractions()` managing copy snippet buttons, smooth scrolling, and GitHub releases download tracking.
- `app.js`: Clean ES module entry point importing and bootstrapping the modules.

- [ ] **Step 1: Extract simulator logic into `website/js/modules/simulator.js`**
- [ ] **Step 2: Extract interactive terminal and command database into `website/js/modules/terminal.js`**
- [ ] **Step 3: Extract copy buttons and page interactions into `website/js/modules/interactions.js`**
- [ ] **Step 4: Refactor `website/app.js` into clean entry point and update `<script type="module" src="app.js"></script>` in `website/index.html`**
- [ ] **Step 5: Audit `website/styles.css` and delete any unused selectors or orphaned code**
- [ ] **Step 6: Test website in browser using local HTTP server**
- [ ] **Step 7: Commit website component refactoring**

```bash
git add website/
git commit -m "refactor(website): modularize app.js into simulator, terminal, and interaction components"
```

---

### Task 6: Comprehensive Verification, Dead Code Removal & Release Build

**Files:**
- Test: All 16 test suites (`cargo test`)
- Audit: Compiler warnings & unused imports (`cargo clippy`, `cargo check`)
- Build: Standalone release binary `dist/memclean.exe` and Inno Setup installer `dist/MemClean_Setup.exe`
- Scan: Windows Defender threat check

- [ ] **Step 1: Run `cargo clippy --all-targets` and remove any unused imports or dead functions**
- [ ] **Step 2: Run `cargo test` to verify all 52 unit/integration tests pass**
- [ ] **Step 3: Build release binary (`cargo build --release`) and compile Inno Setup installer (`ISCC.exe`)**
- [ ] **Step 4: Run Windows Defender scan on both binaries (0 threats required)**
- [ ] **Step 5: Run CLI smoke tests across all 23 flags**
- [ ] **Step 6: Push commits to `origin/master` and update GitHub releases**

```bash
git push origin master
```
