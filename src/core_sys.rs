//! 系统信息：管理员权限判断与操作系统版本描述。
use super::reg::{open_read, read_value};
use windows_sys::Win32::System::Registry::HKEY_LOCAL_MACHINE;
use windows_sys::Win32::System::SystemInformation::OSVERSIONINFOW;
use windows_sys::Win32::UI::Shell::IsUserAnAdmin;
use windows_sys::Wdk::System::SystemServices::RtlGetVersion;

pub fn is_admin() -> bool {
    unsafe { IsUserAnAdmin() != 0 }
}

fn nt_version() -> (u32, u32, u32) {
    unsafe {
        let mut v: OSVERSIONINFOW = std::mem::zeroed();
        v.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOW>() as u32;
        if RtlGetVersion(&mut v) == 0 {
            (v.dwMajorVersion, v.dwMinorVersion, v.dwBuildNumber)
        } else {
            (0, 0, 0)
        }
    }
}

pub fn os_info() -> String {
    let (major, minor, build) = nt_version();
    let mut product_name = String::new();
    let mut display_version = String::new();

    if let Some(key) = open_read(
        HKEY_LOCAL_MACHINE,
        r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
    ) {
        if let Some(v) = read_value(&key, "ProductName") {
            product_name = v;
        }
        for name in ["DisplayVersion", "ReleaseId"] {
            if let Some(v) = read_value(&key, name) {
                display_version = v;
                break;
            }
        }
    }

    // Windows 10 与 Windows 11 共用主版本号 10，需以 build 号区分。
    if major == 10 && build >= 22_000 {
        product_name = "Windows 11".to_string();
    } else if major == 10 && product_name.is_empty() {
        product_name = "Windows 10".to_string();
    }
    if product_name.is_empty() {
        product_name = format!("Windows NT {}.{}", major, minor);
    }

    let mut parts = vec![product_name];
    if !display_version.is_empty() {
        parts.push(display_version);
    }
    if build != 0 {
        parts.push(format!("Build {}", build));
    }
    parts.join(" ")
}
