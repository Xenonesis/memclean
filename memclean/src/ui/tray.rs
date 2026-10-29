use crate::config::AppConfig;
use crate::core::memory;
use crate::engine::cleaner;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// NOTE: This is a stub placeholder for Task 7 (System Tray & Dynamic GDI Icon).
// The real Win32 message loop and dynamic icon generation will replace this.
static IS_RUNNING: AtomicBool = AtomicBool::new(true);

pub fn run_tray_application() {
    let config = AppConfig::load();
    let running = Arc::new(AtomicBool::new(true));

    while running.load(Ordering::SeqCst) {
        let _ = cleaner::perform_clean(&config, false);
        if let Ok(stats) = memory::get_memory_stats() {
            eprintln!(
                "MemClean running... Used: {} MB ({}%)",
                stats.used_phys_mb, stats.used_percent
            );
        }
        std::thread::sleep(std::time::Duration::from_secs(
            config.general.tray_update_interval_sec as u64,
        ));
    }

    IS_RUNNING.store(false, Ordering::SeqCst);
}
