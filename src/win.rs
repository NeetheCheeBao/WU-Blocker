//! 共享 Win32 辅助层：UTF-16 与 COLORREF 转换以及窗口过程常量。
#![allow(dead_code)]

use windows_sys::Win32::Foundation::COLORREF;

/// 将 Rust UTF-8 字符串转换为以 NUL 结尾的 UTF-16 缓冲区。
pub fn wide(s: &str) -> Vec<u16> {
    let mut v: Vec<u16> = s.encode_utf16().collect();
    v.push(0);
    v
}

/// 解码 UTF-16 缓冲区（遇到第一个 NUL 即停止）。
pub fn from_wide(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

/// 将 0xRRGGBB 颜色值转换为 Win32 COLORREF（0x00BBGGRR）。
pub const fn rgb(hex: u32) -> COLORREF {
    ((hex & 0x0000_00FF) << 16) | (hex & 0x0000_FF00) | ((hex >> 16) & 0x0000_00FF)
}

// 窗口消息
pub const WM_CREATE: u32 = 0x0001;
pub const WM_DESTROY: u32 = 0x0002;
pub const WM_SIZE: u32 = 0x0005;
pub const WM_PAINT: u32 = 0x000F;
pub const WM_CLOSE: u32 = 0x0010;
pub const WM_ERASEBKGND: u32 = 0x0014;
pub const WM_DRAWITEM: u32 = 0x002B;
pub const WM_SETICON: u32 = 0x0080;
pub const WM_NCHITTEST: u32 = 0x0084;
pub const WM_KEYDOWN: u32 = 0x0100;
pub const WM_COMMAND: u32 = 0x0111;
pub const WM_TIMER: u32 = 0x0113;

// 窗口与窗口类样式
pub const WS_POPUP: u32 = 0x8000_0000;
pub const WS_VISIBLE: u32 = 0x1000_0000;
pub const WS_CHILD: u32 = 0x4000_0000;
pub const WS_CLIPSIBLINGS: u32 = 0x0400_0000;
pub const WS_CLIPCHILDREN: u32 = 0x0200_0000;
pub const WS_TABSTOP: u32 = 0x0001_0000;
pub const WS_EX_APPWINDOW: u32 = 0x0004_0000;
pub const CS_VREDRAW: u32 = 0x0001;
pub const CS_HREDRAW: u32 = 0x0002;
pub const CS_DROPSHADOW: u32 = 0x0002_0000;
pub const BS_OWNERDRAW: u32 = 0x0000_000B;
pub const HTCAPTION: isize = 2;
pub const GWLP_USERDATA: i32 = -21;
pub const SWP_NOSIZE: u32 = 0x0001;
pub const SWP_NOMOVE: u32 = 0x0002;
pub const SWP_NOZORDER: u32 = 0x0004;
pub const SWP_NOACTIVATE: u32 = 0x0010;
pub const SW_SHOWNORMAL: i32 = 1;
pub const SM_CXSCREEN: i32 = 0;
pub const SM_CYSCREEN: i32 = 1;
pub const VK_ESCAPE: usize = 0x1B;

// 绘制与菜单标志
pub const DT_LEFT: u32 = 0x0000;
pub const DT_CENTER: u32 = 0x0001;
pub const DT_VCENTER: u32 = 0x0004;
pub const DT_WORDBREAK: u32 = 0x0010;
pub const DT_SINGLELINE: u32 = 0x0020;
pub const DT_CALCRECT: u32 = 0x0400;
pub const DT_NOPREFIX: u32 = 0x0800;
pub const TRANSPARENT: i32 = 1;
pub const ODS_SELECTED: u32 = 0x0001;
pub const ODS_FOCUS: u32 = 0x0010;
pub const FW_NORMAL: i32 = 400;
pub const DEFAULT_CHARSET: u32 = 1;
pub const OUT_DEFAULT_PRECIS: u32 = 0;
pub const CLIP_DEFAULT_PRECIS: u32 = 0;
pub const CLEARTYPE_QUALITY: u32 = 5;
pub const DEFAULT_PITCH: u32 = 0;
pub const MF_STRING: u32 = 0x0000;
pub const TPM_RIGHTALIGN: u32 = 0x0008;
pub const TPM_RETURNCMD: u32 = 0x0100;
pub const TPM_LEFTBUTTON: u32 = 0x0000;
pub const ICON_SMALL: usize = 0;
pub const ICON_BIG: usize = 1;

// 配色（与旧版 Tkinter 主题一致）
pub const C_BG: u32 = 0xF0_F0_F0;
pub const C_TERM_BG: u32 = 0x0C_0C_0C;
pub const C_TERM_BORDER: u32 = 0x3C_3C_3C;
pub const C_SEP: u32 = 0x84_83_80;
pub const C_SEP_LIGHT: u32 = 0xFF_FF_FF;
pub const C_BTN: u32 = 0xF5_F5_F5;
pub const C_BTN_HOVER: u32 = 0xE0_EE_F9;
pub const C_BTN_PRESS: u32 = 0xC7_E2_F5;
pub const C_BTN_BORDER: u32 = 0x9E_9A_91;
pub const C_BTN_INNER: u32 = 0xEE_EB_E7;
pub const C_BTN_BORDER_HOVER: u32 = 0x1D_86_D7;
