//! 鼠标悬停跟踪与右上角弹出菜单。
use super::state::{AppState, ID_GITHUB, ID_REGEDIT};
use crate::win::*;
use std::ptr::null;
use windows_sys::Win32::Foundation::{HWND, POINT, RECT};
use windows_sys::Win32::Graphics::Gdi::InvalidateRect;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, GetWindowRect, HMENU,
    SetForegroundWindow, TrackPopupMenu,
};

/// 按光标位置刷新悬停按钮，只重绘状态发生变化的那一个。
pub(crate) unsafe fn check_hover(ptr: *mut AppState) {
    let mut p: POINT = std::mem::zeroed();
    if GetCursorPos(&mut p) == 0 {
        return;
    }
    let mut found: Option<usize> = None;
    for (i, b) in (*ptr).buttons.iter().enumerate() {
        let mut r: RECT = std::mem::zeroed();
        if GetWindowRect(b.hwnd, &mut r) != 0
            && p.x >= r.left
            && p.x < r.right
            && p.y >= r.top
            && p.y < r.bottom
        {
            found = Some(i);
            break;
        }
    }
    if found == (*ptr).hover {
        return;
    }
    let old = (*ptr).hover;
    (*ptr).hover = found;
    if let Some(i) = old {
        InvalidateRect((*ptr).buttons[i].hwnd, null(), 0);
    }
    if let Some(i) = found {
        InvalidateRect((*ptr).buttons[i].hwnd, null(), 0);
    }
}

/// 弹出菜单并返回被选中的命令 ID，未选中时返回 0。
pub(crate) unsafe fn show_menu(hwnd: HWND, anchor: POINT) -> u32 {
    let menu: HMENU = CreatePopupMenu();
    if menu.is_null() {
        return 0;
    }
    let a = wide("打开注册表");
    let b = wide("检查更新");
    AppendMenuW(menu, MF_STRING, ID_REGEDIT as usize, a.as_ptr());
    AppendMenuW(menu, MF_STRING, ID_GITHUB as usize, b.as_ptr());
    SetForegroundWindow(hwnd);
    let cmd = TrackPopupMenu(
        menu,
        TPM_RIGHTALIGN | TPM_RETURNCMD | TPM_LEFTBUTTON,
        anchor.x,
        anchor.y,
        0,
        hwnd,
        null(),
    );
    DestroyMenu(menu);
    if cmd <= 0 { 0 } else { cmd as u32 }
}
