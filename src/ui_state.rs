//! 界面共享状态：控件 ID、定时器、按钮表与 AppState。
use super::theme::Palette;
use std::collections::VecDeque;
use windows_sys::Win32::Foundation::{HWND, RECT};
use windows_sys::Win32::Graphics::Gdi::HFONT;

pub(crate) const ID_REFRESH: u32 = 101;
pub(crate) const ID_PAUSE: u32 = 102;
pub(crate) const ID_RESTORE: u32 = 103;
pub(crate) const ID_EXIT: u32 = 104;
pub(crate) const ID_MENU: u32 = 105;
pub(crate) const ID_TERM: u32 = 106;
pub(crate) const ID_REGEDIT: u32 = 201;
pub(crate) const ID_GITHUB: u32 = 202;

pub(crate) const TIMER_HOVER: usize = 1;
pub(crate) const TIMER_DRAIN: usize = 2;
pub(crate) const TIMER_STARTUP: usize = 3;

pub(crate) const BTN_LABELS: [&str; 5] = ["刷新", "永久暂停更新", "恢复自动更新", "退出", "≡"];
pub(crate) const BTN_IDS: [u32; 5] = [ID_REFRESH, ID_PAUSE, ID_RESTORE, ID_EXIT, ID_MENU];

#[derive(Clone, Copy)]
pub(crate) struct Btn {
    pub(crate) hwnd: HWND,
    pub(crate) rect: RECT,
    pub(crate) label: &'static str,
}

pub(crate) struct RenderLine {
    pub(crate) text: String,
    pub(crate) color: u32,
}

pub(crate) struct AppState {
    pub(crate) hwnd: HWND,
    pub(crate) hwnd_term: HWND,
    pub(crate) buttons: [Btn; 5],
    pub(crate) font_term: HFONT,
    pub(crate) font_ui: HFONT,
    pub(crate) pal: Palette,
    pub(crate) term: RECT,  // 终端外框（含 1px 边框）
    pub(crate) inner: RECT, // 终端内部的文字区域
    pub(crate) sep: RECT,   // 水平分隔线
    pub(crate) scale: f64,
    pub(crate) queue: VecDeque<RenderLine>,
    pub(crate) draining: bool,
    pub(crate) hover: Option<usize>,
}
