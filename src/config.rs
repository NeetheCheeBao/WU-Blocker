//! 应用名称写在代码中，版本号由编译期内嵌的 version.ini 控制。
const APP_NAME: &str = "WU-Blocker";
const APP_DISPLAY_NAME: &str = "Windows Update Blocker";

#[derive(Clone, Debug)]
pub struct AppConfig {
    /// 语义化版本号，不含 `v` 前缀。
    pub version: String,
}

impl AppConfig {
    /// 从编译期内嵌的 version.ini 读取版本号。
    pub fn load() -> Self {
        Self {
            version: parse_version(include_str!("../version.ini")),
        }
    }

    /// 窗口标题，例如 `WU-Blocker v2.0.0`。
    pub fn window_title(&self) -> String {
        format!("{} v{}", APP_NAME, self.version)
    }

    /// 终端启动横幅，例如 `Windows Update Blocker v2.0.0`。
    pub fn banner(&self) -> String {
        format!("{} v{}", APP_DISPLAY_NAME, self.version)
    }
}

/// 逐行解析 INI，只取 Version 字段。
fn parse_version(text: &str) -> String {
    let mut version = String::new();
    for raw in text.lines() {
        let line = raw.trim().trim_start_matches('\u{feff}').trim();
        if line.is_empty()
            || line.starts_with(';')
            || line.starts_with('#')
            || line.starts_with('[')
        {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim().eq_ignore_ascii_case("version") {
            let value = value.trim().trim_matches('"').trim_matches('\'').to_string();
            if !value.is_empty() {
                version = value;
            }
        }
    }
    version
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 名称固定且版本号来自配置() {
        let cfg = AppConfig::load();
        assert!(!cfg.version.is_empty());
        assert_eq!(cfg.window_title(), format!("WU-Blocker v{}", cfg.version));
        assert_eq!(
            cfg.banner(),
            format!("Windows Update Blocker v{}", cfg.version)
        );
    }

    #[test]
    fn 解析忽略注释与无关字段() {
        assert_eq!(
            parse_version("; 注释\n# 注释\n[App]\nName = Other\nVersion = 9.9.9\n"),
            "9.9.9"
        );
    }
}
