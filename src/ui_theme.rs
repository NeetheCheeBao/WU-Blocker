//! 界面外观：配色画刷、字体、DPI 缩放与内嵌图标。
use crate::win::*;
use std::ptr::null_mut;
use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::Graphics::Gdi::{
    CreateFontW, CreateSolidBrush, DeleteObject, HBRUSH, HFONT, HGDIOBJ,
};
use windows_sys::Win32::UI::HiDpi::GetDpiForSystem;
use windows_sys::Win32::UI::WindowsAndMessaging::{CreateIconFromResourceEx, HICON};

pub(crate) const UI_CHARSET: u32 = 134;
pub(crate) const FACE_MONO: &str = "Consolas";
pub(crate) const FACE_CJK: &str = "SimSun";

/// 整套配色对应的画刷，随窗口一起创建与释放。
#[derive(Clone, Copy)]
pub(crate) struct Palette {
    pub(crate) bg: HBRUSH,
    pub(crate) term: HBRUSH,
    pub(crate) term_border: HBRUSH,
    pub(crate) sep: HBRUSH,
    pub(crate) sep_light: HBRUSH,
    pub(crate) btn: HBRUSH,
    pub(crate) btn_hover: HBRUSH,
    pub(crate) btn_press: HBRUSH,
    pub(crate) btn_border: HBRUSH,
    pub(crate) btn_inner: HBRUSH,
    pub(crate) btn_border_hover: HBRUSH,
}

impl Palette {
    pub(crate) unsafe fn create() -> Self {
        Self {
            bg: CreateSolidBrush(rgb(C_BG)),
            term: CreateSolidBrush(rgb(C_TERM_BG)),
            term_border: CreateSolidBrush(rgb(C_TERM_BORDER)),
            sep: CreateSolidBrush(rgb(C_SEP)),
            sep_light: CreateSolidBrush(rgb(C_SEP_LIGHT)),
            btn: CreateSolidBrush(rgb(C_BTN)),
            btn_hover: CreateSolidBrush(rgb(C_BTN_HOVER)),
            btn_press: CreateSolidBrush(rgb(C_BTN_PRESS)),
            btn_border: CreateSolidBrush(rgb(C_BTN_BORDER)),
            btn_inner: CreateSolidBrush(rgb(C_BTN_INNER)),
            btn_border_hover: CreateSolidBrush(rgb(C_BTN_BORDER_HOVER)),
        }
    }

    pub(crate) unsafe fn destroy(self) {
        for b in [
            self.bg, self.term, self.term_border, self.sep, self.sep_light, self.btn,
            self.btn_hover, self.btn_press, self.btn_border, self.btn_inner,
            self.btn_border_hover,
        ] {
            DeleteObject(b as HGDIOBJ);
        }
    }
}

pub(crate) fn sc(value: f64, scale: f64) -> i32 {
    (value * scale).round().max(1.0) as i32
}

pub(crate) fn inset(mut r: RECT, n: i32) -> RECT {
    r.left += n;
    r.top += n;
    r.right -= n;
    r.bottom -= n;
    r
}

pub(crate) unsafe fn make_font(name: &str, px: i32, charset: u32) -> HFONT {
    let n = wide(name);
    CreateFontW(
        -px,
        0,
        0,
        0,
        FW_NORMAL,
        0,
        0,
        0,
        charset,
        OUT_DEFAULT_PRECIS,
        CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY,
        DEFAULT_PITCH,
        n.as_ptr(),
    )
}

/// 从内嵌 ICO 提取图标，优先 DIB，失败时回退 PNG 条目。
pub(crate) unsafe fn load_icon(preferred: &[u32]) -> HICON {
    let data: &[u8] = include_bytes!("../assets/icon.ico");
    if data.len() < 22 {
        return null_mut();
    }
    let count = u16::from_le_bytes([data[4], data[5]]) as usize;

    let try_entry = |index: usize| -> HICON {
        let e = 6 + 16 * index;
        if e + 16 > data.len() {
            return null_mut();
        }
        let w = data[e] as u32;
        let h = data[e + 1] as u32;
        let size = u32::from_le_bytes([
            data[e + 8],
            data[e + 9],
            data[e + 10],
            data[e + 11],
        ]) as usize;
        let off = u32::from_le_bytes([
            data[e + 12],
            data[e + 13],
            data[e + 14],
            data[e + 15],
        ]) as usize;
        if off.checked_add(size).map_or(true, |end| end > data.len()) {
            return null_mut();
        }
        let (cx, cy) = if w == 0 { (256, 256) } else { (w as i32, h as i32) };
        CreateIconFromResourceEx(
            data[off..].as_ptr(),
            size as u32,
            1,
            0x0003_0000,
            cx,
            cy,
            0,
        )
    };

    for want in preferred {
        for i in 1..count {
            let e = 6 + 16 * i;
            if e + 16 <= data.len() && data[e] as u32 == *want {
                let icon = try_entry(i);
                if !icon.is_null() {
                    return icon;
                }
            }
        }
    }
    if count > 0 {
        let icon = try_entry(0);
        if !icon.is_null() {
            return icon;
        }
    }
    null_mut()
}

pub(crate) unsafe fn dpi_scale() -> f64 {
    let dpi = GetDpiForSystem();
    if dpi == 0 {
        1.0
    } else {
        (dpi as f64 / 96.0).max(1.0)
    }
}
