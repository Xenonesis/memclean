#![windows_subsystem = "windows"]

use memclean::config::AppConfig;
use memclean::core::{memory, privileges};
use memclean::engine::cleaner;
use std::env;
#[cfg(windows)]
use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};

fn print_help() {
    println!("MemClean - Ultra-Lightweight Pure Rust Memory Optimizer");
    println!("Usage:");
    println!("  memclean.exe              Launch system tray daemon");
    println!("  memclean.exe --status     Print current physical RAM statistics");
    println!("  memclean.exe --clean      Perform memory optimization according to config");
    println!("  memclean.exe --aggressive Force aggressive purge of all memory layers");
    println!("  memclean.exe --standby    Purge standby list only");
    println!("  memclean.exe --autostart  Register elevated auto-start on Windows login");
    println!("  memclean.exe --no-autostart Remove elevated auto-start on Windows login");
}

fn register_autostart(enable: bool) {
    let current_exe = env::current_exe().unwrap_or_default();
    let exe_str = current_exe.to_string_lossy();

    if enable {
        let status = std::process::Command::new("schtasks")
            .args(&[
                "/create",
                "/tn", "MemCleanAutoStart",
                "/tr", &format!("\"{}\" --minimized", exe_str),
                "/sc", "onlogon",
                "/rl", "highest",
                "/f"
            ])
            .status();
        println!("Registered elevated task scheduler autostart: {:?}", status);
    } else {
        let status = std::process::Command::new("schtasks")
            .args(&["/delete", "/tn", "MemCleanAutoStart", "/f"])
            .status();
        println!("Removed task scheduler autostart: {:?}", status);
    }
}

fn main() {
    let _ = privileges::enable_memory_privileges();
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        if args[1] == "--minimized" {
            #[cfg(windows)]
            {
                memclean::ui::tray::run_application(true);
            }
            return;
        }

        #[cfg(windows)]
        unsafe {
            AttachConsole(ATTACH_PARENT_PROCESS);
        }
        let config = AppConfig::load();
        match args[1].as_str() {
            "--status" => {
                if let Ok(stats) = memory::get_memory_stats() {
                    println!("RAM Used:   {} MB ({}%)", stats.used_phys_mb, stats.used_percent);
                    println!("RAM Avail:  {} MB", stats.avail_phys_mb);
                    println!("RAM Total:  {} MB", stats.total_phys_mb);
                }
            }
            "--clean" => {
                let res = cleaner::perform_clean(&config, false);
                println!("Optimization completed: {:?}", res);
            }
            "--aggressive" => {
                let res = cleaner::perform_clean(&config, true);
                println!("Aggressive clean completed: {:?}", res);
            }
            "--standby" => {
                let _ = memclean::core::ntdll::purge_standby_list();
                println!("Standby list purged.");
            }
            "--autostart" => register_autostart(true),
            "--no-autostart" => register_autostart(false),
            "--help" | "-h" => print_help(),
            _ => print_help(),
        }
        return;
    }

    // Default: Run full GUI window
    #[cfg(windows)]
    {
        memclean::ui::tray::run_application(false);
    }
}
