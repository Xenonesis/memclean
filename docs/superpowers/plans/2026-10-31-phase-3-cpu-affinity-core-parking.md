# Phase 3 Implementation Plan: CPU Core Parking & P-Core/E-Core Affinity Optimizer

- **Date:** 2026-10-31
- **Status:** In Progress
- **Feature:** Intel Alder/Raptor Lake & AMD Zen Hybrid Architecture CPU Topology Detection, P-Core Foreground Game Priority Lock, E-Core Background Clamping, and 100% Core Unparking Engine.

## Objectives
1. Implement `memclean/src/core/cpu.rs`:
   - Enumerate logical and physical processor cores via `GetLogicalProcessorInformationEx(RelationProcessorCore)`.
   - Detect `EfficiencyClass` (0 = E-Core, 1+ = P-Core) and SMT flags to calculate exact 64-bit affinity masks (`all_cores_mask`, `p_cores_mask`, `e_cores_mask`).
   - Implement `set_process_affinity(pid, mask)` and `get_process_affinity(pid)`.
   - Implement `optimize_hybrid_affinity(game_pid, exclusions)` and `restore_all_process_affinity(exclusions)`.
   - Implement Core Parking Unpark (`CPMINCORES = 100`) and Restore (`CPMINCORES = 0`) via Windows Power Management.
2. Integrate with `AppConfig` (`TuningConfig`):
   - `hybrid_cpu_optimizer_enabled`
   - `unpark_cores_enabled`
3. Expose CLI flags in `main.rs`:
   - `memclean.exe --cpu-topology`: Display hardware cores, P-Cores, E-Cores, and masks.
   - `memclean.exe --cpu-unpark`: Unpark 100% CPU cores for zero-latency burst performance.
   - `memclean.exe --cpu-restore-parking`: Reset core parking to Windows default.
4. Integrate into Settings Dialog Tab 2 (`Kernel & Gaming`).
5. Verify with automated unit tests and release compilation.
