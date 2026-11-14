//! 用户动作：暂停与恢复自动更新、打开注册表编辑器与项目主页。
use super::reg::{
    create_write, delete_value, err_text, open_write, set_dword, set_sz, PAUSE_VALUES, REG_LASTKEY,
    REG_PATH,
};
use super::status::{line, status_lines, Kind, LogLine};
use super::time::{TIME_END, TIME_START};
use crate::win::{wide, SW_SHOWNORMAL};
use std::ptr::{null, null_mut};
use windows_sys::Win32::Globalization::GetUserDefaultUILanguage;
use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use windows_sys::Win32::UI::Shell::ShellExecuteW;

pub const PROJECT_URL: &str = "https://github.com/NeetheCheeBao/WU-Blocker";

fn tail_lines(out: &mut Vec<LogLine>) {
    out.push(line("-".repeat(40), Kind::Dim));
    out.push(line("> 刷新当前更新状态:", Kind::Cmd));
    out.extend(status_lines());
}

pub fn pause_updates() -> Vec<LogLine> {
    let mut out = vec![
        line("-".repeat(40), Kind::Dim),
        line("> 执行操作: 永久暂停更新", Kind::Cmd),
    ];

    match create_write(HKEY_LOCAL_MACHINE, REG_PATH) {
        Ok(key) => {
            let mut failed: Option<String> = None;
            if let Err(rc) = set_dword(&key, "FlightSettingsMaxPauseDays", u32::MAX) {
                failed = Some(err_text(rc));
            }
            let pairs = [
                ("PauseFeatureUpdatesStartTime", TIME_START),
                ("PauseFeatureUpdatesEndTime", TIME_END),
                ("PauseQualityUpdatesStartTime", TIME_START),
                ("PauseQualityUpdatesEndTime", TIME_END),
                ("PauseUpdatesStartTime", TIME_START),
                ("PauseUpdatesExpiryTime", TIME_END),
            ];
            for (name, value) in pairs {
                if let Err(rc) = set_sz(&key, name, value) {
                    failed = Some(err_text(rc));
                }
            }
            drop(key);

            match failed {
                None => {
                    out.push(line("[√] 注册表写入成功。", Kind::Ok));
                    out.push(line(
                        "[√] Windows 自动更新已暂停，有效期至 9999-12-31。",
                        Kind::Ok,
                    ));
                }
                Some(e) => {
                    out.push(line(format!("[错误] 无法修改注册表: {}", e), Kind::Err));
                    out.push(line("[提示] 请确保以管理员身份运行本程序。", Kind::Warn));
                }
            }
        }
        Err(rc) => {
            out.push(line(
                format!("[错误] 无法修改注册表: {}", err_text(rc)),
                Kind::Err,
            ));
            out.push(line("[提示] 请确保以管理员身份运行本程序。", Kind::Warn));
        }
    }

    tail_lines(&mut out);
    out
}

pub fn restore_updates() -> Vec<LogLine> {
    let mut out = vec![
        line("-".repeat(40), Kind::Dim),
        line("> 执行操作: 恢复自动更新", Kind::Cmd),
    ];

    match open_write(HKEY_LOCAL_MACHINE, REG_PATH) {
        None => {
            out.push(line(
                "[*] 注册表项不存在，系统已处于默认更新状态。",
                Kind::Warn,
            ));
            out.push(line("-".repeat(40), Kind::Dim));
            out.push(line("> 刷新当前更新状态:", Kind::Cmd));
            out.extend(status_lines());
        }
        Some(key) => {
            let mut removed = 0;
            for name in PAUSE_VALUES {
                if delete_value(&key, name) {
                    removed += 1;
                }
            }
            drop(key);
            out.push(line(
                format!("[√] 已清除 {} 项暂停设置。", removed),
                Kind::Ok,
            ));
            out.push(line("[√] Windows 自动更新已恢复默认设置。", Kind::Ok));
            tail_lines(&mut out);
        }
    }
    out
}

pub fn open_github() {
    unsafe {
        let op = wide("open");
        let url = wide(PROJECT_URL);
        ShellExecuteW(
            null_mut(),
            op.as_ptr(),
            url.as_ptr(),
            null(),
            null(),
            SW_SHOWNORMAL,
        );
    }
}

pub fn open_regedit() -> Vec<LogLine> {
    let mut out = vec![
        line("> 打开注册表编辑器并定位到:", Kind::Cmd),
        line(
            r"  HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings",
            Kind::Dim,
        ),
    ];

    let prefix = unsafe {
        let lang = GetUserDefaultUILanguage();
        if (lang & 0x3FF) == 0x04 {
            "计算机"
        } else {
            "Computer"
        }
    };
    let last_key = format!(r"{}\HKEY_LOCAL_MACHINE\{}", prefix, REG_PATH);

    match create_write(HKEY_CURRENT_USER, REG_LASTKEY) {
        Ok(key) => {
            if let Err(rc) = set_sz(&key, "LastKey", &last_key) {
                out.push(line(
                    format!("[提示] 无法预设注册表定位信息: {}", err_text(rc)),
                    Kind::Warn,
                ));
            }
            drop(key);
        }
        Err(rc) => out.push(line(
            format!("[提示] 无法预设注册表定位信息: {}", err_text(rc)),
            Kind::Warn,
        )),
    }

    unsafe {
        let op = wide("open");
        let file = wide("regedit.exe");
        let params = wide("/m");
        let h = ShellExecuteW(
            null_mut(),
            op.as_ptr(),
            file.as_ptr(),
            params.as_ptr(),
            null(),
            SW_SHOWNORMAL,
        );
        // ShellExecuteW 成功时返回值大于 32，该值并非真正的句柄。
        if (h as isize) > 32 {
            out.push(line("[√] 已启动注册表编辑器。", Kind::Ok));
        } else {
            out.push(line(
                format!("[错误] 无法启动注册表编辑器: {}", (h as isize)),
                Kind::Err,
            ));
        }
    }
    out
}
