# Phase 2 Implementation Plan: GPU & VRAM Cache Manager

- **Date:** 2026-10-31
- **Status:** In Progress
- **Feature:** Windows Display Driver Model (WDDM) GPU / VRAM Cache Manager & Trimmer (`D3DKMTTrimProcessCommitment` + DXGI Adapter Telemetry) with Game Shield Protection.

## Objectives
1. Implement `memclean/src/core/vram.rs` to interface with:
   - `gdi32.dll` -> `D3DKMTTrimProcessCommitment` (WDDM 2.1+ kernel video memory decommitment).
   - `dxgi.dll` -> `CreateDXGIFactory1` / `EnumAdapters1` (GPU identification, dedicated VRAM, and shared system memory).
2. Support trimming VRAM commitments across all non-protected processes, skipping foreground fullscreen games (Game Guard integration).
3. Expose CLI flags:
   - `memclean.exe --vram` / `--gpu-status`: Display GPU names, dedicated VRAM, shared RAM, and driver details.
   - `memclean.exe --vram-trim`: Decommit and purge unused VRAM allocations across running processes.
4. Add `vram_trim_on_clean` toggle in `AppConfig` (`TuningConfig`).
5. Update Settings Dialog Tab 2 (`Kernel & Gaming`) with GPU / VRAM decommitment controls.
6. Verify with automated unit tests and release compilation.
