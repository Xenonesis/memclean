# Phase 5 Implementation Plan: Zero-Injection Minimalist Game HUD Overlay

- **Date:** 2026-10-31
- **Status:** In Progress
- **Feature:** Zero-Injection, Anti-Cheat Safe (100% Ban-Proof), Click-Through Minimalist Gaming HUD Micro-Overlay (`WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE`) with `Ctrl+Shift+O` Global Hotkey Toggle.

## Objectives
1. Implement `memclean/src/ui/hud.rs`:
   - Win32 click-through, always-on-top, borderless layered window (`WS_POPUP`, `WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE`).
   - Double-buffered GDI rendering with 88% alpha frosted matte black canvas (`#111111`) and hairline `#EAEAEA` border.
   - Real-time telemetry indicators:
     * Active Application & PID
     * RAM Usage & progress bar
     * VRAM Usage (Dedicated GPU memory)
     * Active Scheduler Timer Resolution (`0.500 ms`)
     * Kernel Hard Page Fault rate & Stutter Risk status (`Smooth` / `Stutter`)
2. Register Global Hotkey `Ctrl+Shift+O` (`HOTKEY_HUD_ID = 133`) in `tray.rs` to toggle HUD visibility on/off anytime during gameplay.
3. Integrate HUD toggle into `AppConfig` (`GeneralConfig.gaming_hud_enabled`) and Settings Dialog.
4. Expose CLI command `memclean.exe --hud`.
5. Verify with automated unit tests, screenshot visual capture, and release build.
