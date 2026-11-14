//! 注册表读写封装：键句柄 RAII，以及字符串、DWORD 的取值与写值。
use crate::win::{from_wide, wide};
use std::ptr::{null, null_mut};
use windows_sys::Win32::System::Registry::*;

pub(crate) const REG_PATH: &str = r"SOFTWARE\Microsoft\WindowsUpdate\UX\Settings";
pub(crate) const REG_LASTKEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Applets\Regedit";

pub(crate) const PAUSE_VALUES: [&str; 7] = [
    "FlightSettingsMaxPauseDays",
    "PauseFeatureUpdatesStartTime",
    "PauseFeatureUpdatesEndTime",
    "PauseQualityUpdatesStartTime",
    "PauseQualityUpdatesEndTime",
    "PauseUpdatesStartTime",
    "PauseUpdatesExpiryTime",
];

/// 注册表键句柄，离开作用域时自动关闭。
pub(crate) struct Key(HKEY);

impl Drop for Key {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.0);
        }
    }
}

pub(crate) fn open_read(root: HKEY, path: &str) -> Option<Key> {
    unsafe {
        let p = wide(path);
        let mut h: HKEY = null_mut();
        if RegOpenKeyExW(root, p.as_ptr(), 0, KEY_READ, &mut h) == 0 {
            Some(Key(h))
        } else {
            None
        }
    }
}

pub(crate) fn open_write(root: HKEY, path: &str) -> Option<Key> {
    unsafe {
        let p = wide(path);
        let mut h: HKEY = null_mut();
        if RegOpenKeyExW(root, p.as_ptr(), 0, KEY_WRITE, &mut h) == 0 {
            Some(Key(h))
        } else {
            None
        }
    }
}

pub(crate) fn create_write(root: HKEY, path: &str) -> Result<Key, u32> {
    unsafe {
        let p = wide(path);
        let mut h: HKEY = null_mut();
        let rc = RegCreateKeyExW(
            root,
            p.as_ptr(),
            0,
            null(),
            0,
            KEY_WRITE,
            null(),
            &mut h,
            null_mut(),
        );
        if rc == 0 {
            Ok(Key(h))
        } else {
            Err(rc)
        }
    }
}

pub(crate) fn read_value(key: &Key, name: &str) -> Option<String> {
    unsafe {
        let n = wide(name);
        let mut size: u32 = 0;
        let mut ty: u32 = 0;
        if RegQueryValueExW(key.0, n.as_ptr(), null(), &mut ty, null_mut(), &mut size) != 0 {
            return None;
        }
        if size == 0 {
            return Some(String::new());
        }
        let mut buf = vec![0u8; size as usize + 2];
        let mut got = size;
        if RegQueryValueExW(
            key.0,
            n.as_ptr(),
            null(),
            &mut ty,
            buf.as_mut_ptr(),
            &mut got,
        ) != 0
        {
            return None;
        }
        if ty == REG_DWORD {
            if got < 4 {
                return None;
            }
            let v = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
            return Some(v.to_string());
        }
        let words = std::slice::from_raw_parts(buf.as_ptr() as *const u16, got as usize / 2);
        Some(from_wide(words))
    }
}

pub(crate) fn set_sz(key: &Key, name: &str, value: &str) -> Result<(), u32> {
    unsafe {
        let n = wide(name);
        let v = wide(value);
        let rc = RegSetValueExW(
            key.0,
            n.as_ptr(),
            0,
            REG_SZ,
            v.as_ptr() as *const u8,
            (v.len() * 2) as u32,
        );
        if rc == 0 { Ok(()) } else { Err(rc) }
    }
}

pub(crate) fn set_dword(key: &Key, name: &str, value: u32) -> Result<(), u32> {
    unsafe {
        let n = wide(name);
        let rc = RegSetValueExW(
            key.0,
            n.as_ptr(),
            0,
            REG_DWORD,
            &value as *const u32 as *const u8,
            4,
        );
        if rc == 0 { Ok(()) } else { Err(rc) }
    }
}

pub(crate) fn delete_value(key: &Key, name: &str) -> bool {
    unsafe {
        let n = wide(name);
        RegDeleteValueW(key.0, n.as_ptr()) == 0
    }
}

/// 把 Win32 错误码转换成可读文本。
pub(crate) fn err_text(code: u32) -> String {
    std::io::Error::from_raw_os_error(code as i32).to_string()
}
