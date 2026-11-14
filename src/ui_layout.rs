//! 窗口布局：按 DPI 缩放计算终端、分隔线与五个按钮的位置。
use super::state::AppState;
use super::theme::{inset, sc};
use crate::win::*;
use std::ptr::null_mut;
use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::UI::WindowsAndMessaging::{GetClientRect, SetWindowPos};

pub(crate) unsafe fn layout(ptr: *mut AppState) {
    let mut rc: RECT = std::mem::zeroed();
    GetClientRect((*ptr).hwnd, &mut rc);
    let w = rc.right;
    let h = rc.bottom;
    let s = (*ptr).scale;

    let pad_l = sc(4.0, s);
    let pad_r = sc(4.0, s);
    let pad_t = sc(4.0, s);
    let pad_b = sc(6.0, s);
    let btn_h = sc(29.0, s);
    let sep_h = sc(2.0, s);

    let btn_bottom = h - pad_b;
    let btn_top = btn_bottom - btn_h;
    let sep_top = btn_top - sc(12.0, s) - sep_h;
    let term_bottom = sep_top - sc(27.0, s);

    (*ptr).term = RECT {
        left: pad_l,
        top: pad_t,
        right: w - pad_r,
        bottom: term_bottom,
    };
    (*ptr).sep = RECT {
        left: pad_l,
        top: sep_top,
        right: w - pad_r,
        bottom: sep_top + sep_h,
    };

    let border = sc(1.0, s);
    let pad = sc(10.0, s);
    (*ptr).inner = inset((*ptr).term, border);
    (*ptr).inner.left += pad - border;
    (*ptr).inner.top += pad - border;
    (*ptr).inner.right -= pad - border;
    (*ptr).inner.bottom -= pad - border;

    let gap = sc(8.0, s);
    let small = sc(58.0, s);
    let menu = sc(28.0, s);
    let flex = (w - pad_l - pad_r - small * 2 - menu - gap * 4) / 2;
    let widths = [small, flex, flex, small, menu];

    let mut x = pad_l;
    let mut rects = [RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    }; 5];
    for (i, width) in widths.iter().enumerate() {
        rects[i] = RECT {
            left: x,
            top: btn_top,
            right: x + *width,
            bottom: btn_bottom,
        };
        x += *width + gap;
    }
    for i in 0..5 {
        (*ptr).buttons[i].rect = rects[i];
        let r = rects[i];
        SetWindowPos(
            (*ptr).buttons[i].hwnd,
            null_mut(),
            r.left,
            r.top,
            r.right - r.left,
            r.bottom - r.top,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
    let t = (*ptr).inner;
    SetWindowPos(
        (*ptr).hwnd_term,
        null_mut(),
        t.left,
        t.top,
        t.right - t.left,
        t.bottom - t.top,
        SWP_NOZORDER | SWP_NOACTIVATE,
    );
}
