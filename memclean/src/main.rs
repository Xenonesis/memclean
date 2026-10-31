#![windows_subsystem = "windows"]

use memclean::config::AppConfig;
use memclean::core::{memory, privileges};
use memclean::engine::cleaner;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(windows)]
use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};

static SILENT: AtomicBool = AtomicBool::new(false);

fn print_line(msg: &str) {
    if SILENT.load(Ordering::Relaxed) {
        return;
    }
    use std::io::Write;
    let _ = writeln!(std::io::stdout(), "{}", msg);

    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::Console::{GetStdHandle, WriteConsoleW, STD_OUTPUT_HANDLE};
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if !handle.is_null() && handle != windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
            let line = format!("{}\r\n", msg);
            let wide: Vec<u16> = line.encode_utf16().collect();
            let mut written = 0;
            WriteConsoleW(
                handle,
                wide.as_ptr() as _,
                wide.len() as u32,
                &mut written,
                std::ptr::null_mut(),
            );
        }
    }
}

#[derive(serde::Serialize)]
struct JsonStatsOutput {
    timestamp: u64,
    memory: JsonMemoryStats,
    timer_resolution: JsonTimerResolution,
    top_consumers: Vec<JsonTopConsumer>,
}

#[derive(serde::Serialize)]
struct JsonMemoryStats {
    total_mb: u64,
    used_mb: u64,
    free_mb: u64,
    zeroed_mb: u64,
    standby_mb: u64,
    modified_mb: u64,
    used_percent: u32,
}

#[derive(serde::Serialize)]
struct JsonTimerResolution {
    current_ms: f64,
    minimum_ms: f64,
    maximum_ms: f64,
}

#[derive(serde::Serialize)]
struct JsonTopConsumer {
    pid: u32,
    name: String,
    working_set_mb: u64,
}

fn print_stats_json() {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let ext_stats = memory::get_extended_memory_stats();
    let base_stats = memory::get_memory_stats();

    let memory_stats = if let Ok(s) = ext_stats {
        JsonMemoryStats {
            total_mb: s.total_phys_mb,
            used_mb: s.used_phys_mb,
            free_mb: s.free_phys_mb,
            zeroed_mb: s.zeroed_phys_mb,
            standby_mb: s.standby_total_mb,
            modified_mb: s.modified_mb,
            used_percent: s.used_percent,
        }
    } else if let Ok(s) = base_stats {
        JsonMemoryStats {
            total_mb: s.total_phys_mb,
            used_mb: s.used_phys_mb,
            free_mb: s.avail_phys_mb,
            zeroed_mb: 0,
            standby_mb: 0,
            modified_mb: 0,
            used_percent: s.used_percent,
        }
    } else {
        JsonMemoryStats {
            total_mb: 0,
            used_mb: 0,
            free_mb: 0,
            zeroed_mb: 0,
            standby_mb: 0,
            modified_mb: 0,
            used_percent: 0,
        }
    };

    let round3 = |v: f64| (v * 1000.0).round() / 1000.0;
    let (min_100ns, max_100ns, cur_100ns) =
        memclean::core::ntdll::query_timer_resolution().unwrap_or((156250, 5000, 156250));

    let timer_res = JsonTimerResolution {
        current_ms: round3((cur_100ns as f64) / 10_000.0),
        minimum_ms: round3((min_100ns as f64) / 10_000.0),
        maximum_ms: round3((max_100ns as f64) / 10_000.0),
    };

    let consumers = memclean::core::process::get_top_memory_processes(10);
    let top_consumers = consumers
        .into_iter()
        .map(|c| JsonTopConsumer {
            pid: c.pid,
            name: c.name,
            working_set_mb: c.working_set_mb,
        })
        .collect();

    let output = JsonStatsOutput {
        timestamp,
        memory: memory_stats,
        timer_resolution: timer_res,
        top_consumers,
    };

    if let Ok(json_str) = serde_json::to_string_pretty(&output) {
        print_line(&json_str);
    }
}

fn print_stats_table() {
    let stats = memory::get_extended_memory_stats();
    let (min_100ns, max_100ns, cur_100ns) =
        memclean::core::ntdll::query_timer_resolution().unwrap_or((156250, 5000, 156250));
    let cur_ms = (cur_100ns as f64) / 10_000.0;
    let min_ms = (min_100ns as f64) / 10_000.0;
    let max_ms = (max_100ns as f64) / 10_000.0;
    let consumers = memclean::core::process::get_top_memory_processes(5);

    print_line("============================================================");
    print_line("                   MemClean System Statistics               ");
    print_line("============================================================");
    if let Ok(s) = stats {
        print_line("Physical Memory:");
        print_line(&format!("  Total:        {:>6} MB", s.total_phys_mb));
        print_line(&format!("  In-Use:       {:>6} MB ({}%)", s.used_phys_mb, s.used_percent));
        print_line(&format!("  Standby:      {:>6} MB", s.standby_total_mb));
        print_line(&format!("  Modified:     {:>6} MB", s.modified_mb));
        print_line(&format!("  Free:         {:>6} MB", s.free_phys_mb));
        print_line(&format!("  Zeroed:       {:>6} MB", s.zeroed_phys_mb));
    } else if let Ok(s) = memory::get_memory_stats() {
        print_line("Physical Memory:");
        print_line(&format!("  Total:        {:>6} MB", s.total_phys_mb));
        print_line(&format!("  In-Use:       {:>6} MB ({}%)", s.used_phys_mb, s.used_percent));
        print_line(&format!("  Available:    {:>6} MB", s.avail_phys_mb));
    }
    print_line("");
    print_line("Timer Resolution:");
    print_line(&format!("  Current:      {:.3} ms (Min: {:.3} ms, Max: {:.3} ms)", cur_ms, min_ms, max_ms));
    print_line("");
    print_line("Top Memory Consumers:");
    print_line(&format!("  {:<8} {:<30} {:>12}", "PID", "Process Name", "Working Set"));
    print_line(&format!("  {:<8} {:<30} {:>12}", "--------", "------------------------------", "------------"));
    for p in consumers {
        print_line(&format!("  {:<8} {:<30} {:>9} MB", p.pid, p.name, p.working_set_mb));
    }
    print_line("============================================================");
}

fn print_help() {
    print_line("MemClean - Ultra-Lightweight Pure Rust Memory Optimizer");
    print_line("");
    print_line("Usage:");
    print_line("  memclean.exe [OPTIONS]");
    print_line("");
    print_line("Options:");
    print_line("  -c, --clean               Perform memory optimization according to config");
    print_line("  -a, --aggressive          Force aggressive purge of all memory layers");
    print_line("      --standby             Purge standby list only");
    print_line("      --combine             Run kernel physical page combining (RAM deduplication)");
    print_line("      --working-sets        Trim working sets of all non-protected processes");
    print_line("      --timer-res <TARGET>  Set timer resolution: 0.5, 1.0, or default (ms)");
    print_line("      --stats, --status     Print current memory and system statistics");
    print_line("      --json                Output statistics in machine-readable JSON format");
    print_line("      --network-status      Print current Windows TCP and multimedia network throttling state");
    print_line("      --network-optimize    Apply competitive low-latency TCP stack (TCPNoDelay, instant ACKs)");
    print_line("      --network-restore     Restore Windows default network throttling and TCP stack settings");
    print_line("      --vram, --gpu-status  Display detected GPU adapters, dedicated VRAM, and shared RAM");
    print_line("      --vram-trim           Decommit and purge eligible GPU video memory commitments (WDDM)");
    print_line("      --cpu-topology        Display detected CPU physical cores, P-Cores, E-Cores, and SMT threads");
    print_line("      --cpu-unpark          Unpark 100% of CPU cores (CPMINCORES 100) for zero-latency burst performance");
    print_line("      --cpu-restore-parking Reset CPU core parking behavior to Windows default");
    print_line("      --telemetry           Sample 1-second kernel hard page faults, disk I/O, and stutter risk");
    print_line("      --hud                 Launch standalone zero-injection gaming HUD micro-overlay");
    print_line("  -s, --silent              Suppress console text output");
    print_line("      --autostart           Register elevated auto-start on Windows login");
    print_line("      --no-autostart        Remove elevated auto-start on Windows login");
    print_line("      --minimized           Launch directly to system tray");
    print_line("  -h, --help                Print this help information");
    print_line("");
    print_line("Examples:");
    print_line("  memclean.exe --clean");
    print_line("  memclean.exe --clean --aggressive");
    print_line("  memclean.exe --standby");
    print_line("  memclean.exe --combine");
    print_line("  memclean.exe --working-sets");
    print_line("  memclean.exe --timer-res 0.5");
    print_line("  memclean.exe --stats");
    print_line("  memclean.exe --stats --json");
}

fn register_autostart(enable: bool) {
    let current_exe = env::current_exe().unwrap_or_default();
    let exe_str = current_exe.to_string_lossy();
    let value_data = format!("\"{}\" --minimized", exe_str);

    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::Registry::{
            RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY_CURRENT_USER,
            KEY_ALL_ACCESS, REG_SZ,
        };

        let subkey = windows_sys::w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        let val_name = windows_sys::w!("MemClean");
        let mut hkey = std::ptr::null_mut();

        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey, 0, KEY_ALL_ACCESS, &mut hkey) == 0 && !hkey.is_null() {
            if enable {
                let wide_data: Vec<u16> = value_data.encode_utf16().chain(std::iter::once(0)).collect();
                let res = RegSetValueExW(
                    hkey,
                    val_name,
                    0,
                    REG_SZ,
                    wide_data.as_ptr() as _,
                    (wide_data.len() * 2) as u32,
                );
                if res == 0 {
                    print_line("Registered MemClean in Windows startup (HKCU Run key).");
                } else {
                    print_line(&format!("Failed to register startup in registry (code: {})", res));
                }
            } else {
                let _ = RegDeleteValueW(hkey, val_name);
                print_line("Removed MemClean from Windows startup (HKCU Run key).");
            }
            RegCloseKey(hkey);
        } else {
            print_line("Failed to open HKCU Run registry key.");
        }
    }
}

fn main() {
    let _ = privileges::enable_memory_privileges();
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        if args.iter().any(|a| a == "--minimized") {
            #[cfg(windows)]
            {
                memclean::ui::tray::run_application(true);
            }
            return;
        }

        #[cfg(windows)]
        unsafe {
            use windows_sys::Win32::System::Console::{
                GetStdHandle, SetStdHandle, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE,
            };
            let current_out = GetStdHandle(STD_OUTPUT_HANDLE);
            let is_redirected = !current_out.is_null()
                && current_out != windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;

            if !is_redirected {
                if AttachConsole(ATTACH_PARENT_PROCESS) != 0 {
                    use windows_sys::Win32::Storage::FileSystem::{
                        CreateFileW, FILE_GENERIC_WRITE, FILE_SHARE_READ, FILE_SHARE_WRITE,
                        OPEN_EXISTING,
                    };
                    let conout = CreateFileW(
                        windows_sys::w!("CONOUT$"),
                        FILE_GENERIC_WRITE,
                        FILE_SHARE_READ | FILE_SHARE_WRITE,
                        std::ptr::null(),
                        OPEN_EXISTING,
                        0,
                        std::ptr::null_mut(),
                    );
                    if conout != windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
                        SetStdHandle(STD_OUTPUT_HANDLE, conout);
                        SetStdHandle(STD_ERROR_HANDLE, conout);
                    }
                }
            }
        }

        let silent = args.iter().any(|a| a == "--silent" || a == "-s");
        SILENT.store(silent, Ordering::Relaxed);

        let config = AppConfig::load();

        if args.iter().any(|a| a == "--help" || a == "-h") {
            print_help();
            return;
        }

        if args.iter().any(|a| a == "--autostart") {
            register_autostart(true);
            return;
        }

        if args.iter().any(|a| a == "--no-autostart") {
            register_autostart(false);
            return;
        }

        let is_json = args.iter().any(|a| a == "--json");
        let is_stats = args.iter().any(|a| a == "--stats" || a == "--status");
        let is_telemetry = args.iter().any(|a| a == "--telemetry");
        if (is_stats || is_json) && !is_telemetry {
            if is_json {
                print_stats_json();
            } else {
                print_stats_table();
            }
            return;
        }

        if args.iter().any(|a| a == "--standby") {
            if cleaner::purge_standby_only() {
                print_line("Standby list purged successfully.");
            } else {
                print_line("Failed to purge standby list (requires Administrator privileges).");
            }
            return;
        }

        if args.iter().any(|a| a == "--combine") {
            match cleaner::combine_pages_only() {
                Ok(c) => {
                    print_line(&format!(
                        "Page combining executed: {} duplicate pages merged ({:.2} MB reclaimed).",
                        c.pages_combined, c.mb_saved
                    ));
                }
                Err(e) => {
                    print_line(&format!("Page combining failed: {}", e));
                }
            }
            return;
        }

        if args.iter().any(|a| a == "--working-sets") {
            let count = cleaner::trim_working_sets_only(&config);
            print_line(&format!("Trimmed working sets for {} processes.", count));
            return;
        }
        if args.iter().any(|a| a == "--settings") {
            #[cfg(windows)]
            memclean::ui::dialog::show_settings_dialog(0 as _);
            return;
        }

        if args.iter().any(|a| a == "--network-status") {
            let status = memclean::core::network::get_network_status();
            print_line("=== MemClean Competitive Network Latency Status ===");
            print_line(&format!(
                "Network Throttling Index: {:?} (Disabled: {})",
                status.network_throttling_index, status.is_throttling_disabled
            ));
            print_line(&format!(
                "System Responsiveness   : {:?} (Gaming 100%: {})",
                status.system_responsiveness, status.is_responsiveness_maximized
            ));
            print_line(&format!(
                "Active Interfaces       : {} detected (Optimized: {}/{})",
                status.active_interfaces_count, status.optimized_interfaces_count, status.interfaces.len()
            ));
            print_line(&format!("Rollback Backup Exists  : {}", status.backup_exists));
            for iface in &status.interfaces {
                if let Some(ip) = &iface.ip_address {
                    if ip != "0.0.0.0" && !ip.is_empty() {
                        print_line(&format!(
                            "  - [{}] IP: {} | TcpAckFrequency: {:?} | TCPNoDelay: {:?} | Optimized: {}",
                            &iface.guid[..8.min(iface.guid.len())],
                            ip,
                            iface.tcp_ack_frequency,
                            iface.tcp_no_delay,
                            iface.is_optimized
                        ));
                    }
                }
            }
            return;
        }

        if args.iter().any(|a| a == "--network-optimize") {
            match memclean::core::network::optimize_network() {
                Ok(rep) => {
                    print_line("Network optimization applied successfully:");
                    print_line(&format!("  - Network Throttling Disabled : {}", rep.throttling_updated));
                    print_line(&format!("  - System Responsiveness (100%): {}", rep.responsiveness_updated));
                    print_line(&format!("  - Interfaces Optimized        : {}", rep.interfaces_updated));
                    print_line(&format!("  - Rollback Backup Saved       : {}", rep.backup_saved));
                }
                Err(e) => {
                    print_line(&format!("Error applying network optimization: {}", e));
                }
            }
            return;
        }

        if args.iter().any(|a| a == "--network-restore") {
            match memclean::core::network::restore_network_defaults() {
                Ok(rep) => {
                    print_line("Network settings restored to defaults:");
                    print_line(&format!("  - Throttling Restored  : {}", rep.throttling_restored));
                    print_line(&format!("  - Responsiveness (20%) : {}", rep.responsiveness_restored));
                    print_line(&format!("  - Interfaces Restored  : {}", rep.interfaces_restored));
                    print_line(&format!("  - Used Rollback Backup : {}", rep.used_backup));
                }
                Err(e) => {
                    print_line(&format!("Error restoring network settings: {}", e));
                }
            }
            return;
        }
        if args.iter().any(|a| a == "--vram" || a == "--gpu-status") {
            let adapters = memclean::core::vram::get_gpu_adapters();
            print_line("=== MemClean GPU & Video Memory (VRAM) Diagnostics ===");
            if adapters.is_empty() {
                print_line("No DXGI hardware adapters detected.");
            } else {
                for a in adapters {
                    print_line(&format!(
                        "[{}] {} (Vendor: 0x{:04X}, Device: 0x{:04X})",
                        a.index, a.description, a.vendor_id, a.device_id
                    ));
                    print_line(&format!(
                        "    Dedicated VRAM       : {} MB",
                        a.dedicated_vram_mb
                    ));
                    print_line(&format!(
                        "    Dedicated System RAM : {} MB",
                        a.dedicated_system_mb
                    ));
                    print_line(&format!(
                        "    Shared System RAM    : {} MB",
                        a.shared_system_mb
                    ));
                }
            }
            return;
        }

        if args.iter().any(|a| a == "--vram-trim") {
            let report = cleaner::trim_vram_only(&config);
            print_line(&format!(
                "GPU VRAM trim executed: {:.2} MB video memory decommitted across {} processes.",
                report.mb_decommitted, report.processes_trimmed
            ));
            return;
        }

        if args.iter().any(|a| a == "--cpu-topology") {
            let topo = memclean::core::cpu::get_cpu_topology();
            print_line("=== MemClean CPU Topology & Hybrid Architecture ===");
            print_line(&format!(
                "Physical Cores     : {} (P-Cores: {}, E-Cores: {})",
                topo.total_physical_cores, topo.p_cores_count, topo.e_cores_count
            ));
            print_line(&format!(
                "Logical Processors : {} threads",
                topo.total_logical_processors
            ));
            print_line(&format!(
                "Architecture Type  : {}",
                if topo.is_hybrid { "Intel/AMD Hybrid Core Architecture" } else { "Monolithic Symmetric Core Architecture" }
            ));
            print_line(&format!("All-Cores Mask     : 0x{:X}", topo.all_cores_mask));
            print_line(&format!("P-Cores Affinity   : 0x{:X}", topo.p_cores_mask));
            print_line(&format!("E-Cores Affinity   : 0x{:X}", topo.e_cores_mask));
            for c in &topo.cores {
                let kind = if c.is_performance { "P-Core" } else { "E-Core" };
                print_line(&format!(
                    "  Core [{:2}]: {} | Class: {} | SMT: {} | Mask: 0x{:X}",
                    c.core_index, kind, c.efficiency_class, c.smt_enabled, c.affinity_mask
                ));
            }
            return;
        }

        if args.iter().any(|a| a == "--cpu-unpark") {
            match memclean::core::cpu::unpark_all_cores() {
                Ok(()) => {
                    print_line("CPU core parking disabled: 100% of CPU cores unparked for maximum responsiveness.");
                }
                Err(e) => {
                    print_line(&format!("Error unparking CPU cores: {}", e));
                }
            }
            return;
        }

        if args.iter().any(|a| a == "--cpu-restore-parking") {
            match memclean::core::cpu::restore_core_parking_default() {
                Ok(()) => {
                    print_line("CPU core parking restored to Windows default scheme.");
                }
                Err(e) => {
                    print_line(&format!("Error restoring CPU core parking: {}", e));
                }
            }
            return;
        }

        if args.iter().any(|a| a == "--telemetry") {
            let is_json = args.iter().any(|a| a == "--json");
            match memclean::core::telemetry::sample_fault_telemetry(std::time::Duration::from_millis(1000)) {
                Ok(t) => {
                    if is_json {
                        if let Ok(j) = serde_json::to_string_pretty(&t) {
                            print_line(&j);
                        }
                    } else {
                        print_line("=== MemClean Live Memory Telemetry & Stutter Risk ===");
                        print_line(&format!("Sampling Duration       : {} ms", t.sample_duration_ms));
                        print_line(&format!("Hard Page Faults Rate   : {} / sec (Disk Reads)", t.hard_faults_per_sec));
                        print_line(&format!("Hard Fault I/O Ops Rate : {} / sec", t.hard_fault_io_per_sec));
                        print_line(&format!("Total Page Faults Rate  : {} / sec", t.total_page_faults_per_sec));
                        print_line(&format!("Context Switches Rate   : {} / sec", t.context_switches_per_sec));
                        print_line(&format!("Stutter Risk Level      : {}", t.stutter_risk.as_str()));
                        print_line("\nTop Page Faulting Processes (Cumulative):");
                        for p in &t.top_faulting_processes {
                            print_line(&format!("  - [{:5}] {:<24} : {:>10} faults", p.pid, p.name, p.page_fault_count));
                        }
                    }
                }
                Err(e) => {
                    print_line(&format!("Error collecting telemetry: {}", e));
                }
            }
            return;
        }

        if args.iter().any(|a| a == "--hud") {
            print_line("Launching MemClean Gaming HUD overlay (Ctrl+Shift+O to toggle)...");
            #[cfg(windows)]
            {
                memclean::ui::hud::show_hud();
                std::thread::sleep(std::time::Duration::from_secs(10));
            }
            return;
        }

        let timer_res_pos = args.iter().position(|a| a == "--timer-res");
        let timer_res_eq = args.iter().find(|a| a.starts_with("--timer-res="));
        if timer_res_pos.is_some() || timer_res_eq.is_some() {
            let target_opt = timer_res_pos
                .and_then(|idx| args.get(idx + 1).cloned())
                .or_else(|| timer_res_eq.map(|a| a["--timer-res=".len()..].to_string()));

            if let Some(target) = target_opt {
                let mode = match target.trim().to_lowercase().as_str() {
                    "0.5" | "0.5ms" | "half" => Some(memclean::core::timer_res::TimerResolutionMode::HalfMs),
                    "1.0" | "1" | "1ms" | "one" => Some(memclean::core::timer_res::TimerResolutionMode::OneMs),
                    "default" | "def" => Some(memclean::core::timer_res::TimerResolutionMode::Default),
                    _ => None,
                };

                match mode {
                    Some(m) => {
                        let mut guard = memclean::core::timer_res::TimerResolutionGuard::new();
                        match guard.set_mode(m) {
                            Ok(()) => {
                                let cur = memclean::core::timer_res::TimerResolutionGuard::get_current_resolution_ms();
                                print_line(&format!("Active timer resolution: {:.3} ms", cur));
                                std::mem::forget(guard);
                            }
                            Err(e) => {
                                print_line(&format!("Failed to set timer resolution: {}", e));
                            }
                        }
                    }
                    None => {
                        print_line(&format!("Invalid timer resolution target '{}'. Valid options: 0.5, 1.0, default", target));
                    }
                }
            } else {
                print_line("Error: --timer-res requires a target: 0.5, 1.0, or default");
            }
            return;
        }

        let is_clean = args.iter().any(|a| a == "--clean" || a == "-c");
        let is_aggressive = args.iter().any(|a| a == "--aggressive" || a == "-a");
        if is_clean || is_aggressive {
            let res = cleaner::perform_clean(&config, is_aggressive);
            let vram_msg = if res.vram_processes_trimmed > 0 {
                format!(", VRAM: {:.1} MB trimmed ({} procs)", res.vram_mb_trimmed, res.vram_processes_trimmed)
            } else {
                String::new()
            };
            print_line(&format!(
                "Clean complete: Standby: {}, Modified: {}, Cache: {}, Processes trimmed: {}, Pages combined: {} ({:.1} MB){}",
                res.standby_purged,
                res.modified_flushed,
                res.file_cache_trimmed,
                res.processes_trimmed,
                res.pages_combined,
                res.mb_combined,
                vram_msg
            ));
            return;
        }

        print_help();
        return;
    }

    // Default: Run full GUI window
    #[cfg(windows)]
    {
        memclean::ui::tray::run_application(false);
    }
}
