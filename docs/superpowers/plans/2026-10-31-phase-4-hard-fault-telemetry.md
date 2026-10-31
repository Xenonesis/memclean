# Phase 4 Implementation Plan: Driver & ETW Hard Page Fault Telemetry Engine

- **Date:** 2026-10-31
- **Status:** In Progress
- **Feature:** Real-Time Hard Page Fault & Kernel Disk I/O Micro-Stutter Telemetry Engine (`NtQuerySystemInformation` Class 2 + Per-Process Fault Ranking).

## Objectives
1. Implement `memclean/src/core/telemetry.rs`:
   - Bind `NtQuerySystemInformation` with `SystemPerformanceInformation` (Class 2, 312 bytes).
   - Track cumulative and per-second rates:
     * `total_page_faults_per_sec`
     * `hard_faults_per_sec` (PageReadCount delta - direct disk-page eviction stutters)
     * `hard_fault_io_count` (PageReadIoCount)
     * `context_switches_per_sec`
   - Calculate `StutterRisk` classification:
     * `Smooth` (< 100 hard faults/sec)
     * `Moderate` (100 - 500 hard faults/sec)
     * `ElevatedStutterRisk` (500 - 2,000 hard faults/sec)
     * `SevereDiskThrashing` (> 2,000 hard faults/sec)
   - Sample and rank top processes generating page faults.
2. CLI Integration (`main.rs`):
   - `memclean.exe --telemetry`: 1-second sample snapshot of kernel memory faults and stutter risk.
   - `memclean.exe --telemetry --json`: Structured JSON for game streaming overlays / monitoring.
3. Settings Dialog & Dashboard Integration:
   - Live telemetry status in dashboard and tray.
4. Verify with automated unit tests and release compilation.
