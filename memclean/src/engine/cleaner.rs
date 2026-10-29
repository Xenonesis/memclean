use crate::config::AppConfig;
use crate::core::{ntdll, process};
use crate::engine::guard;

#[derive(Default, Debug)]
pub struct CleanResult {
    pub standby_purged: bool,
    pub modified_flushed: bool,
    pub file_cache_trimmed: bool,
    pub processes_trimmed: usize,
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

    result
}
