use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, LUID};
use windows_sys::Win32::Security::{
    AdjustTokenPrivileges, GetTokenInformation, LookupPrivilegeValueW, TokenElevation,
    LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_ELEVATION,
    TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

fn encode_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn enable_memory_privileges() -> Result<(), String> {
    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        ) == 0
        {
            return Err(format!("OpenProcessToken failed with code {}", GetLastError()));
        }

        let privileges = [
            "SeProfileSingleProcessPrivilege",
            "SeIncreaseQuotaPrivilege",
            "SeDebugPrivilege",
        ];

        for priv_name in privileges {
            let wide = encode_wide(priv_name);
            let mut luid = LUID { LowPart: 0, HighPart: 0 };
            if LookupPrivilegeValueW(std::ptr::null(), wide.as_ptr(), &mut luid) != 0 {
                let mut tp = TOKEN_PRIVILEGES {
                    PrivilegeCount: 1,
                    Privileges: [LUID_AND_ATTRIBUTES {
                        Luid: luid,
                        Attributes: SE_PRIVILEGE_ENABLED,
                    }],
                };
                AdjustTokenPrivileges(
                    token,
                    0,
                    &mut tp,
                    std::mem::size_of::<TOKEN_PRIVILEGES>() as u32,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );
            }
        }

        CloseHandle(token);
        Ok(())
    }
}

/// Check if current process token has elevated / Administrator privileges
pub fn is_elevated() -> bool {
    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut ret_len: u32 = 0;
        let res = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as _,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut ret_len,
        );

        CloseHandle(token);

        res != 0 && elevation.TokenIsElevated != 0
    }
}

/// Relaunch current executable with elevated Administrator privileges via UAC ShellExecuteW
pub fn relaunch_as_admin(args: &[String]) -> Result<(), String> {
    let current_exe = std::env::current_exe().map_err(|e| format!("Failed to get current_exe: {}", e))?;
    let exe_str = current_exe.to_str().ok_or_else(|| "Invalid exe path UTF-8".to_string())?;

    let wide_verb = encode_wide("runas");
    let wide_file = encode_wide(exe_str);
    let params_str = args.join(" ");
    let wide_params = if params_str.is_empty() {
        Vec::new()
    } else {
        encode_wide(&params_str)
    };

    unsafe {
        let h_inst = ShellExecuteW(
            std::ptr::null_mut(),
            wide_verb.as_ptr(),
            wide_file.as_ptr(),
            if wide_params.is_empty() {
                std::ptr::null()
            } else {
                wide_params.as_ptr()
            },
            std::ptr::null(),
            SW_SHOWNORMAL as i32,
        );

        if h_inst as usize > 32 {
            Ok(())
        } else {
            Err(format!("UAC elevation request failed or was dismissed (code: {})", h_inst as usize))
        }
    }
}