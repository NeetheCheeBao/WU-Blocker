//! UTC 时间换算：公历转天数、ISO 字符串解析与本地时间前缀，不依赖外部 crate。
use std::time::{SystemTime, UNIX_EPOCH};
use windows_sys::Win32::Foundation::SYSTEMTIME;
use windows_sys::Win32::System::SystemInformation::GetLocalTime;

pub(crate) const TIME_START: &str = "2000-01-01T01:00:00Z";
pub(crate) const TIME_END: &str = "9999-12-31T01:00:00Z";

#[derive(Clone, Copy)]
pub(crate) struct Stamp {
    pub(crate) y: i64,
    pub(crate) m: i64,
    pub(crate) d: i64,
    pub(crate) hh: i64,
    pub(crate) mm: i64,
    pub(crate) ss: i64,
}

pub(crate) fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let mp = m + if m > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

pub(crate) fn stamp_seconds(s: Stamp) -> i64 {
    days_from_civil(s.y, s.m, s.d) * 86_400 + s.hh * 3_600 + s.mm * 60 + s.ss
}

pub(crate) fn parse_iso(value: &str) -> Option<Stamp> {
    let b = value.as_bytes();
    if b.len() != 20 || b[4] != b'-' || b[7] != b'-' || b[10] != b'T'
        || b[13] != b':' || b[16] != b':' || b[19] != b'Z'
    {
        return None;
    }
    let num = |a: usize, z: usize| -> Option<i64> { value.get(a..z)?.parse::<i64>().ok() };
    Some(Stamp {
        y: num(0, 4)?,
        m: num(5, 7)?,
        d: num(8, 10)?,
        hh: num(11, 13)?,
        mm: num(14, 16)?,
        ss: num(17, 19)?,
    })
}

pub(crate) fn pretty_time(value: &str) -> String {
    match parse_iso(value) {
        Some(s) => format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
            s.y, s.m, s.d, s.hh, s.mm, s.ss
        ),
        None => value.to_string(),
    }
}

pub(crate) fn group_thousands(n: i64) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

pub(crate) fn utc_now_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// 生成 `[HH:MM:SS]  ` 前缀，与旧版 `strftime("[%H:%M:%S]  ")` 一致。
pub fn local_timestamp() -> String {
    unsafe {
        let mut t: SYSTEMTIME = std::mem::zeroed();
        GetLocalTime(&mut t);
        format!(
            "[{:02}:{:02}:{:02}]  ",
            t.wHour, t.wMinute, t.wSecond
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 公历日期换算与时间戳计算正确() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2000, 1, 1), 10_957);
        let s = parse_iso("2000-01-01T01:00:00Z").unwrap();
        assert_eq!(stamp_seconds(s), 10_957 * 86_400 + 3_600);
    }

    #[test]
    fn 时间字符串解析与可读化正确() {
        assert_eq!(
            pretty_time("9999-12-31T01:00:00Z"),
            "9999-12-31 01:00:00 UTC"
        );
        assert_eq!(pretty_time("invalid"), "invalid");
        assert!(parse_iso("2000-01-01").is_none());
        assert!(parse_iso("2000-01-01T01:00:00").is_none());
    }

    #[test]
    fn 剩余天数按千分位格式化() {
        assert_eq!(group_thousands(2_912_173), "2,912,173");
        assert_eq!(group_thousands(365), "365");
    }
}
