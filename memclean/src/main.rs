#![windows_subsystem = "windows"]

use memclean::config::AppConfig;
use memclean::core::{memory, privileges};
use memclean::engine::cleaner;
use std::env;
#[cfg(windows)]
use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};

fn print_line(msg: &str) {
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

fn print_help() {
    print_line("MemClean - Ultra-Lightweight Pure Rust Memory Optimizer");
    print_line("Usage:");
    print_line("  memclean.exe              Launch system tray daemon");
    print_line("  memclean.exe --status     Print current physical RAM statistics");
    print_line("  memclean.exe --clean      Perform memory optimization according to config");
    print_line("  memclean.exe --aggressive Force aggressive purge of all memory layers");
    print_line("  memclean.exe --standby    Purge standby list only");
    print_line("  memclean.exe --autostart  Register elevated auto-start on Windows login");
    print_line("  memclean.exe --no-autostart Remove elevated auto-start on Windows login");
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
        print_line(&format!("Registered elevated task scheduler autostart: {:?}", status));
    } else {
        let status = std::process::Command::new("schtasks")
            .args(&["/delete", "/tn", "MemCleanAutoStart", "/f"])
            .status();
        print_line(&format!("Removed task scheduler autostart: {:?}", status));
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
        let config = AppConfig::load();
        match args[1].as_str() {
            "--status" => {
                if let Ok(stats) = memory::get_memory_stats() {
                    print_line(&format!("RAM Used:   {} MB ({}%)", stats.used_phys_mb, stats.used_percent));
                    print_line(&format!("RAM Avail:  {} MB", stats.avail_phys_mb));
                    print_line(&format!("RAM Total:  {} MB", stats.total_phys_mb));
                }
            }
            "--clean" => {
                let res = cleaner::perform_clean(&config, false);
                print_line(&format!("Optimization completed: {:?}", res));
            }
            "--aggressive" => {
                let res = cleaner::perform_clean(&config, true);
                print_line(&format!("Aggressive clean completed: {:?}", res));
            }
            "--standby" => {
                let _ = memclean::core::ntdll::purge_standby_list();
                print_line("Standby list purged.");
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
