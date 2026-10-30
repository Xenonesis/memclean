use crate::config::AppConfig;
use crate::core::{ntdll, process};
use crate::engine::guard;

#[derive(Default, Debug, Clone, Copy)]
pub struct CleanResult {
    pub standby_purged: bool,
    pub modified_flushed: bool,
    pub file_cache_trimmed: bool,
    pub processes_trimmed: usize,
    pub pages_combined: usize,
    pub mb_combined: f32,
    pub vram_processes_trimmed: usize,
    pub vram_mb_trimmed: f32,
}

pub fn perform_clean(config: &AppConfig, force_aggressive: bool) -> CleanResult {
    let mut result = CleanResult::default();
    let fg_ctx = guard::get_foreground_context();

    // If game guard is on and in fullscreen, skip aggressive process trimming
    let restrict_to_standby = config.policy.game_guard_enabled
        && fg_ctx.as_ref().map_or(false, |ctx| ctx.is_fullscreen)
        && !force_aggressive;

    // 1. Purge Standby List
    if config.policy.purge_standby || force_aggressive {
        if ntdll::purge_standby_list().is_ok() {
            result.standby_purged = true;
        }
    } else if config.policy.purge_low_priority_standby {
        if ntdll::purge_low_priority_standby_list().is_ok() {
            result.standby_purged = true;
        }
    }

    if restrict_to_standby {
        return result;
    }

    // 2. Flush Modified List
    if config.policy.flush_modified || force_aggressive {
        if ntdll::flush_modified_list().is_ok() {
            result.modified_flushed = true;
        }
    }

    // 3. Trim System File Cache
    if config.policy.trim_file_cache || force_aggressive {
        if ntdll::trim_system_file_cache().is_ok() {
            result.file_cache_trimmed = true;
        }
    }

    // 4. Trim Working Sets
    if config.policy.trim_working_sets || force_aggressive {
        if let Ok(procs) = process::enumerate_processes() {
            let fg_pid = if config.policy.game_guard_enabled && !force_aggressive {
                fg_ctx.map(|ctx| ctx.pid)
            } else {
                None
            };

            for p in procs {
                if !guard::is_protected_process(&p.name, p.pid, fg_pid, &config.exclusions) {
                    if process::trim_process_working_set(p.pid).is_ok() {
                        result.processes_trimmed += 1;
                    }
                }
            }
        }
    }

    // 5. Kernel Page Combining (de-duplication)
    if config.tuning.combine_pages_on_clean || force_aggressive {
        if let Ok(c) = crate::core::combine::combine_physical_memory() {
            result.pages_combined = c.pages_combined;
            result.mb_combined = c.mb_saved;
        }
    }

    // 6. GPU VRAM Cache Decommitment
    if config.tuning.vram_trim_on_clean || force_aggressive {
        let vram_report = crate::core::vram::trim_all_vram(&config.exclusions);
        result.vram_processes_trimmed = vram_report.processes_trimmed;
        result.vram_mb_trimmed = vram_report.mb_decommitted as f32;
    }

    result
}

pub fn purge_standby_only() -> bool {
    ntdll::purge_standby_list().is_ok()
}

pub fn combine_pages_only() -> Result<crate::core::combine::CombineResult, String> {
    crate::core::combine::combine_physical_memory()
}

pub fn trim_working_sets_only(config: &AppConfig) -> usize {
    let fg_ctx = guard::get_foreground_context();
    let mut count = 0;
    if let Ok(procs) = process::enumerate_processes() {
        let fg_pid = if config.policy.game_guard_enabled {
            fg_ctx.map(|ctx| ctx.pid)
        } else {
            None
        };
        for p in procs {
            if !guard::is_protected_process(&p.name, p.pid, fg_pid, &config.exclusions) {
                if process::trim_process_working_set(p.pid).is_ok() {
                    count += 1;
                }
            }
        }
    }
    count
}

pub fn trim_vram_only(config: &AppConfig) -> crate::core::vram::VramTrimReport {
    crate::core::vram::trim_all_vram(&config.exclusions)
}
