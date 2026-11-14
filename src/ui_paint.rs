//! 绘制：窗口底衬、终端边框、分隔线与自绘按钮。
use super::state::{AppState, BTN_IDS};
use super::theme::inset;
use crate::win::*;
use windows_sys::Win32::Foundation::{LPARAM, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, DrawFocusRect, DrawTextW, EndPaint, FillRect, SelectObject, SetBkMode, SetTextColor,
    HGDIOBJ, PAINTSTRUCT,
};
use windows_sys::Win32::UI::Controls::DRAWITEMSTRUCT;
use windows_sys::Win32::UI::WindowsAndMessaging::GetClientRect;

pub(crate) unsafe fn on_paint(ptr: *mut AppState) {
    // 终端文字由 RichEdit 自行绘制，父窗口只负责边框与底衬。
    let (hwnd, pal, term, sep) = {
        let st = &*ptr;
        (st.hwnd, st.pal, st.term, st.sep)
    };

    let mut ps: PAINTSTRUCT = std::mem::zeroed();
    let hdc = BeginPaint(hwnd, &mut ps);

    let mut client: RECT = std::mem::zeroed();
    GetClientRect(hwnd, &mut client);
    FillRect(hdc, &client, pal.bg);

    FillRect(hdc, &term, pal.term_border);
    let body = inset(term, 1);
    FillRect(hdc, &body, pal.term);

    let mut s1 = sep;
    s1.bottom = s1.top + 1;
    FillRect(hdc, &s1, pal.sep);
    let mut s2 = sep;
    s2.top = s2.top + 1;
    FillRect(hdc, &s2, pal.sep_light);

    EndPaint(hwnd, &ps);
}

pub(crate) unsafe fn on_draw_item(ptr: *mut AppState, lparam: LPARAM) {
    let dis = &*(lparam as *const DRAWITEMSTRUCT);
    let index = BTN_IDS.iter().position(|&id| id == dis.CtlID);
    let Some(index) = index else {
        return;
    };

    let (font, pal, hovered, label) = {
        let st = &*ptr;
        (st.font_ui, st.pal, st.hover == Some(index), st.buttons[index].label)
    };
    let pressed = (dis.itemState & ODS_SELECTED) != 0;
    let focused = (dis.itemState & ODS_FOCUS) != 0;
    let hdc = dis.hDC;
    let rc = dis.rcItem;

    let border = if hovered || pressed {
        pal.btn_border_hover
    } else {
        pal.btn_border
    };
    let fill = if pressed {
        pal.btn_press
    } else if hovered {
        pal.btn_hover
    } else {
        pal.btn
    };

    FillRect(hdc, &rc, border);
    let r1 = inset(rc, 1);
    FillRect(hdc, &r1, pal.btn_inner);
    let r2 = inset(r1, 1);
    FillRect(hdc, &r2, fill);

    let old = SelectObject(hdc, font as HGDIOBJ);
    SetBkMode(hdc, TRANSPARENT);
    SetTextColor(hdc, 0x0000_0000);
    let w = wide(label);
    let mut tr = r2;
    DrawTextW(
        hdc,
        w.as_ptr(),
        -1,
        &mut tr,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
    );
    SelectObject(hdc, old);

    if focused {
        let fr = inset(rc, 4);
        DrawFocusRect(hdc, &fr);
    }
}
