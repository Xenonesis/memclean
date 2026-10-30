# Phase 1 Implementation Plan: Network & Ping Latency Tweaker

- **Date:** 2026-10-31
- **Status:** In Progress
- **Feature:** Competitive Gaming Network Stack Optimization (TCP NoDelay, TcpAckFrequency, SystemResponsiveness, NetworkThrottlingIndex) with Zero-Loss Rollback Engine.

## Objectives
1. Identify all active network adapters (Ethernet / Wi-Fi) on the machine.
2. Safely back up existing registry values to an atomic backup file (`network_backup.toml`).
3. Apply competitive low-latency tweaks:
   - `TcpAckFrequency = 1`
   - `TCPNoDelay = 1`
   - `NetworkThrottlingIndex = 0xFFFFFFFF`
   - `SystemResponsiveness = 0`
4. Provide a 1-click / 1-command complete restore to original Windows default values.
5. Expose CLI flags (`--network-optimize`, `--network-restore`, `--network-status`).
6. Integrate UI controls into the "Kernel & Gaming" tab of the Settings Dialog.
7. Verify all changes with unit tests and live execution.
