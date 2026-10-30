use serde::{Deserialize, Serialize};
use std::ffi::c_void;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};

type InternetOpenAFn = unsafe extern "system" fn(
    *const u8,
    u32,
    *const u8,
    *const u8,
    u32,
) -> *mut c_void;

type InternetCloseHandleFn = unsafe extern "system" fn(*mut c_void) -> i32;

type InternetOpenUrlAFn = unsafe extern "system" fn(
    *mut c_void,
    *const u8,
    *const u8,
    u32,
    u32,
    usize,
) -> *mut c_void;

type InternetReadFileFn = unsafe extern "system" fn(
    *mut c_void,
    *mut u8,
    u32,
    *mut u32,
) -> i32;

type InternetSetOptionAFn = unsafe extern "system" fn(
    *mut c_void,
    u32,
    *const c_void,
    u32,
) -> i32;

const INTERNET_OPEN_TYPE_DIRECT: u32 = 1;
const INTERNET_FLAG_RELOAD: u32 = 0x80000000;
const INTERNET_FLAG_SECURE: u32 = 0x00800000;
const INTERNET_OPTION_CONNECT_TIMEOUT: u32 = 2;
const INTERNET_OPTION_RECEIVE_TIMEOUT: u32 = 6;

const GITHUB_LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/Xenonesis/memclean/releases/latest";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub html_url: String,
    pub setup_download_url: Option<String>,
    pub portable_download_url: Option<String>,
    pub release_notes: String,
}

struct WinINet {
    open: InternetOpenAFn,
    close: InternetCloseHandleFn,
    open_url: InternetOpenUrlAFn,
    read_file: InternetReadFileFn,
    set_option: InternetSetOptionAFn,
}

impl WinINet {
    fn load() -> Result<Self, String> {
        unsafe {
            let h_mod = LoadLibraryA(b"wininet.dll\0".as_ptr());
            if h_mod.is_null() {
                return Err("Failed to load wininet.dll".to_string());
            }

            let p_open = GetProcAddress(h_mod, b"InternetOpenA\0".as_ptr())
                .ok_or("Failed to resolve InternetOpenA")?;
            let p_close = GetProcAddress(h_mod, b"InternetCloseHandle\0".as_ptr())
                .ok_or("Failed to resolve InternetCloseHandle")?;
            let p_open_url = GetProcAddress(h_mod, b"InternetOpenUrlA\0".as_ptr())
                .ok_or("Failed to resolve InternetOpenUrlA")?;
            let p_read_file = GetProcAddress(h_mod, b"InternetReadFile\0".as_ptr())
                .ok_or("Failed to resolve InternetReadFile")?;
            let p_set_option = GetProcAddress(h_mod, b"InternetSetOptionA\0".as_ptr())
                .ok_or("Failed to resolve InternetSetOptionA")?;

            Ok(Self {
                open: std::mem::transmute(p_open),
                close: std::mem::transmute(p_close),
                open_url: std::mem::transmute(p_open_url),
                read_file: std::mem::transmute(p_read_file),
                set_option: std::mem::transmute(p_set_option),
            })
        }
    }
}

pub fn parse_semver(tag: &str) -> (u32, u32, u32) {
    let clean = tag.trim().trim_start_matches(|c| c == 'v' || c == 'V');
    let parts: Vec<&str> = clean.split('.').collect();
    let major = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}

pub fn is_newer_version(current: &str, latest: &str) -> bool {
    let curr = parse_semver(current);
    let late = parse_semver(latest);
    late > curr
}

pub fn parse_github_release_json(json_str: &str) -> Result<ReleaseInfo, String> {
    let val: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))?;

    let tag_name = val["tag_name"]
        .as_str()
        .ok_or("Missing tag_name")?
        .to_string();
    let html_url = val["html_url"].as_str().unwrap_or("").to_string();
    let release_notes = val["body"].as_str().unwrap_or("").to_string();

    let mut setup_download_url = None;
    let mut portable_download_url = None;

    if let Some(assets) = val["assets"].as_array() {
        for asset in assets {
            if let (Some(name), Some(url)) = (asset["name"].as_str(), asset["browser_download_url"].as_str()) {
                if name.eq_ignore_ascii_case("MemClean_Setup.exe") {
                    setup_download_url = Some(url.to_string());
                } else if name.eq_ignore_ascii_case("memclean.exe") {
                    portable_download_url = Some(url.to_string());
                }
            }
        }
    }

    Ok(ReleaseInfo {
        tag_name,
        html_url,
        setup_download_url,
        portable_download_url,
        release_notes,
    })
}

fn http_get(url: &str) -> Result<Vec<u8>, String> {
    let inet = WinINet::load()?;
    let user_agent = b"MemClean-App\0";
    let h_internet = unsafe {
        (inet.open)(
            user_agent.as_ptr(),
            INTERNET_OPEN_TYPE_DIRECT,
            std::ptr::null(),
            std::ptr::null(),
            0,
        )
    };
    if h_internet.is_null() {
        return Err("InternetOpenA failed".to_string());
    }

    // Set 5-second timeout
    let timeout_ms: u32 = 5000;
    unsafe {
        (inet.set_option)(
            h_internet,
            INTERNET_OPTION_CONNECT_TIMEOUT,
            &timeout_ms as *const _ as *const c_void,
            std::mem::size_of::<u32>() as u32,
        );
        (inet.set_option)(
            h_internet,
            INTERNET_OPTION_RECEIVE_TIMEOUT,
            &timeout_ms as *const _ as *const c_void,
            std::mem::size_of::<u32>() as u32,
        );
    }

    let url_c = std::ffi::CString::new(url).map_err(|_| "Invalid URL string")?;
    let headers = b"Accept: application/vnd.github.v3+json\r\nUser-Agent: MemClean-App\r\n\0";

    let h_url = unsafe {
        (inet.open_url)(
            h_internet,
            url_c.as_ptr() as *const u8,
            headers.as_ptr(),
            headers.len() as u32 - 1,
            INTERNET_FLAG_RELOAD | INTERNET_FLAG_SECURE,
            0,
        )
    };

    if h_url.is_null() {
        unsafe { (inet.close)(h_internet) };
        return Err("InternetOpenUrlA failed".to_string());
    }

    let mut body = Vec::new();
    let mut buffer = [0u8; 8192];
    let mut bytes_read = 0u32;

    loop {
        let res = unsafe {
            (inet.read_file)(
                h_url,
                buffer.as_mut_ptr(),
                buffer.len() as u32,
                &mut bytes_read as *mut u32,
            )
        };
        if res == 0 || bytes_read == 0 {
            break;
        }
        body.extend_from_slice(&buffer[..bytes_read as usize]);
    }

    unsafe {
        (inet.close)(h_url);
        (inet.close)(h_internet);
    }

    Ok(body)
}

pub fn check_for_update() -> Result<Option<ReleaseInfo>, String> {
    let bytes = http_get(GITHUB_LATEST_RELEASE_URL)?;
    let json_str = String::from_utf8(bytes).map_err(|e| format!("UTF8 error: {}", e))?;
    let info = parse_github_release_json(&json_str)?;

    let current_version = env!("CARGO_PKG_VERSION");
    if is_newer_version(current_version, &info.tag_name) {
        Ok(Some(info))
    } else {
        Ok(None)
    }
}

pub fn download_file(url: &str, dest_path: &Path) -> Result<(), String> {
    let bytes = http_get(url)?;
    let mut file = File::create(dest_path)
        .map_err(|e| format!("Failed to create destination file: {}", e))?;
    file.write_all(&bytes)
        .map_err(|e| format!("Failed to write downloaded bytes: {}", e))?;
    file.flush()
        .map_err(|e| format!("Failed to flush file: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_parsing() {
        assert_eq!(parse_semver("v1.0.0"), (1, 0, 0));
        assert_eq!(parse_semver("1.2.3"), (1, 2, 3));
        assert_eq!(parse_semver("V2.10.4"), (2, 10, 4));
        assert_eq!(parse_semver("v0.5"), (0, 5, 0));
        assert_eq!(parse_semver("invalid"), (0, 0, 0));
    }

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("1.0.0", "v1.0.1"));
        assert!(is_newer_version("1.0.0", "1.1.0"));
        assert!(is_newer_version("1.0.0", "v2.0.0"));
        assert!(!is_newer_version("1.0.0", "v1.0.0"));
        assert!(!is_newer_version("1.1.0", "v1.0.5"));
        assert!(!is_newer_version("2.0.0", "v1.9.9"));
    }

    #[test]
    fn test_parse_github_release_json() {
        let json = r#"{
            "tag_name": "v1.1.0",
            "html_url": "https://github.com/Xenonesis/memclean/releases/tag/v1.1.0",
            "body": "New scheduler timer and ISLC features.",
            "assets": [
                {
                    "name": "MemClean_Setup.exe",
                    "browser_download_url": "https://github.com/download/MemClean_Setup.exe"
                },
                {
                    "name": "memclean.exe",
                    "browser_download_url": "https://github.com/download/memclean.exe"
                }
            ]
        }"#;

        let res = parse_github_release_json(json).unwrap();
        assert_eq!(res.tag_name, "v1.1.0");
        assert_eq!(
            res.html_url,
            "https://github.com/Xenonesis/memclean/releases/tag/v1.1.0"
        );
        assert_eq!(
            res.setup_download_url.as_deref(),
            Some("https://github.com/download/MemClean_Setup.exe")
        );
        assert_eq!(
            res.portable_download_url.as_deref(),
            Some("https://github.com/download/memclean.exe")
        );
        assert_eq!(res.release_notes, "New scheduler timer and ISLC features.");
    }
}
