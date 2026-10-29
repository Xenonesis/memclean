use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, LUID};
use windows_sys::Win32::Security::{
    AdjustTokenPrivileges, LookupPrivilegeValueW, LUID_AND_ATTRIBUTES,
    SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

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