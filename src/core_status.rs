//! 日志行类型与更新状态查询输出。
use super::reg::{open_read, read_value, PAUSE_VALUES, REG_PATH};
use super::sys::{is_admin, os_info};
use super::time::{
    group_thousands, parse_iso, pretty_time, stamp_seconds, utc_now_seconds, TIME_START,
};
use std::collections::HashMap;
use windows_sys::Win32::System::Registry::HKEY_LOCAL_MACHINE;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Info,
    Ok,
    Warn,
    Err,
    Cmd,
    Dim,
}

impl Kind {
    /// 返回 0xRRGGBB 颜色值，由 GUI 层转换为 COLORREF。
    pub fn color(self) -> u32 {
        match self {
            Kind::Info => 0xD4_D4_D4,
            Kind::Ok => 0x4E_C9_B0,
            Kind::Warn => 0xDC_DC_AA,
            Kind::Err => 0xF4_87_71,
            Kind::Cmd => 0x56_9C_D6,
            Kind::Dim => 0x7A_7A_7A,
        }
    }
}

#[derive(Clone)]
pub struct LogLine {
    pub text: String,
    pub kind: Kind,
}

pub fn line(text: impl Into<String>, kind: Kind) -> LogLine {
    LogLine {
        text: text.into(),
        kind,
    }
}

pub fn status_lines() -> Vec<LogLine> {
    let key = match open_read(HKEY_LOCAL_MACHINE, REG_PATH) {
        Some(k) => k,
        None => return vec![line("当前状态: 自动更新正常运行。", Kind::Warn)],
    };

    let mut values: HashMap<&str, String> = HashMap::new();
    for name in PAUSE_VALUES {
        if let Some(v) = read_value(&key, name) {
            values.insert(name, v);
        }
    }
    drop(key);

    if values.is_empty() {
        return vec![line("当前状态: 自动更新正常运行。", Kind::Warn)];
    }

    let expiry = values
        .get("PauseUpdatesExpiryTime")
        .and_then(|v| parse_iso(v));
    let start = values
        .get("PauseUpdatesStartTime")
        .cloned()
        .unwrap_or_else(|| TIME_START.to_string());

    match expiry {
        Some(e) if stamp_seconds(e) > utc_now_seconds() => {
            let days = (stamp_seconds(e) - utc_now_seconds()) / 86_400;
            let start_str = pretty_time(&start);
            let expiry_str = pretty_time(values.get("PauseUpdatesExpiryTime").unwrap());
            vec![
                line("当前状态: 自动更新已被暂停。", Kind::Ok),
                line(format!("暂停开始时间 : {}", start_str), Kind::Ok),
                line(format!("暂停到期时间 : {}", expiry_str), Kind::Ok),
                line(
                    format!("剩余时间     : 约 {} 天", group_thousands(days)),
                    Kind::Ok,
                ),
                line(
                    "  (到期时间为 9999-12-31，等同于永久暂停)",
                    Kind::Info,
                ),
            ]
        }
        _ => vec![line("当前状态: 自动更新正常运行。", Kind::Warn)],
    }
}

pub fn startup_lines() -> Vec<LogLine> {
    let cfg = crate::config::AppConfig::load();
    let mut out = vec![
        line(cfg.banner(), Kind::Cmd),
        line(os_info(), Kind::Info),
        line(
            format!("程序权限：{}", if is_admin() { "管理员" } else { "普通用户" }),
            Kind::Info,
        ),
    ];
    out.extend(status_lines());
    out
}

pub fn refresh_lines() -> Vec<LogLine> {
    let mut out = vec![line("> 手动刷新当前状态", Kind::Cmd)];
    out.extend(startup_lines());
    out
}
