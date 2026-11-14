//! 窗口创建、消息分发与命令处理。
use super::layout::layout;
use super::menu::{check_hover, show_menu};
use super::paint::{on_draw_item, on_paint};
use super::state::{
    AppState, Btn, BTN_IDS, BTN_LABELS, ID_EXIT, ID_GITHUB, ID_MENU, ID_PAUSE, ID_REFRESH,
    ID_REGEDIT, ID_RESTORE, ID_TERM, TIMER_DRAIN, TIMER_HOVER, TIMER_STARTUP,
};
use super::term::{clear_output, drain_one, enqueue, EM_SETBKGNDCOLOR};
use super::theme::{dpi_scale, load_icon, make_font, sc, Palette, UI_CHARSET};
use crate::core;
use crate::win::*;
use std::collections::VecDeque;
use std::ffi::c_void;
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{DeleteObject, UpdateWindow, HGDIOBJ};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, LoadLibraryW};
use windows_sys::Win32::UI::Controls::EM_SETMARGINS;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, EC_LEFTMARGIN, EC_RIGHTMARGIN,
    ES_AUTOVSCROLL, ES_MULTILINE, ES_NOHIDESEL, ES_READONLY, GetMessageW, GetSystemMetrics,
    GetWindowLongPtrW, GetWindowRect, IDC_ARROW, KillTimer, LoadCursorW, PostQuitMessage,
    RegisterClassExW, SendMessageW, SetTimer, SetWindowLongPtrW, SetWindowPos, ShowWindow,
    TranslateMessage, WM_SETFONT, MSG, WNDCLASSEXW,
};

unsafe fn on_command(ptr: *mut AppState, hwnd: HWND, id: u32) {
    match id {
        ID_REFRESH => {
            clear_output(ptr);
            enqueue(ptr, core::refresh_lines());
        }
        ID_PAUSE => enqueue(ptr, core::pause_updates()),
        ID_RESTORE => enqueue(ptr, core::restore_updates()),
        ID_EXIT => {
            DestroyWindow(hwnd);
        }
        ID_MENU => {
            let mut r: RECT = std::mem::zeroed();
            GetWindowRect((*ptr).buttons[4].hwnd, &mut r);
            let cmd = show_menu(hwnd, POINT { x: r.right, y: r.bottom });
            match cmd {
                ID_REGEDIT => enqueue(ptr, core::open_regedit()),
                ID_GITHUB => core::open_github(),
                _ => {}
            }
        }
        ID_REGEDIT => enqueue(ptr, core::open_regedit()),
        ID_GITHUB => core::open_github(),
        _ => {}
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState;
    if ptr.is_null() {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    match msg {
        WM_PAINT => {
            on_paint(ptr);
            0
        }
        WM_ERASEBKGND => 1,
        WM_NCHITTEST => HTCAPTION, // 允许在客户区拖动无边框窗口
        WM_DRAWITEM => {
            on_draw_item(ptr, lparam);
            1
        }
        WM_COMMAND => {
            on_command(ptr, hwnd, (wparam & 0xFFFF) as u32);
            0
        }
        WM_TIMER => {
            match wparam {
                TIMER_HOVER => check_hover(ptr),
                TIMER_DRAIN => drain_one(ptr),
                TIMER_STARTUP => {
                    KillTimer(hwnd, TIMER_STARTUP);
                    enqueue(ptr, core::startup_lines());
                }
                _ => {}
            }
            0
        }
        WM_SIZE => {
            layout(ptr);
            0
        }
        WM_DESTROY => {
            let state = &*ptr;
            DeleteObject(state.font_term as HGDIOBJ);
            DeleteObject(state.font_ui as HGDIOBJ);
            state.pal.destroy();
            KillTimer(hwnd, TIMER_HOVER);
            KillTimer(hwnd, TIMER_DRAIN);
            KillTimer(hwnd, TIMER_STARTUP);
            PostQuitMessage(0);
            0
        }
        WM_CLOSE => {
            DestroyWindow(hwnd);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

pub fn run() {
    unsafe {
        let scale = dpi_scale();
        let w = sc(460.0, scale);
        let h = sc(300.0, scale);
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let x = (screen_w - w) / 2;
        let y = (screen_h - h) / 2;

        let instance: HINSTANCE = GetModuleHandleW(null());
        let pal = Palette::create();
        let font_ui = make_font("Microsoft YaHei", sc(13.0, scale), UI_CHARSET);
        let font_term = make_font("Consolas", sc(12.0, scale), DEFAULT_CHARSET);
        let icon_big = load_icon(&[48, 64, 32]);
        let icon_small = load_icon(&[16, 20, 24, 32]);

        let class_name = wide("WU_Blocker_Rust_Window");
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW | CS_DROPSHADOW,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: icon_big,
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: pal.bg,
            lpszMenuName: null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: icon_small,
        };
        if RegisterClassExW(&wc) == 0 {
            return;
        }

        // 窗口标题由代码中的名称与 version.ini 的版本号组合而成。
        let title = wide(&crate::config::AppConfig::load().window_title());
        let hwnd = CreateWindowExW(
            WS_EX_APPWINDOW,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_POPUP | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
            x,
            y,
            w,
            h,
            null_mut(),
            null_mut(),
            instance,
            null(),
        );
        if hwnd.is_null() {
            return;
        }

        let msftedit = wide("msftedit.dll");
        LoadLibraryW(msftedit.as_ptr());
        let term_class = wide("RICHEDIT50W");
        let empty_text = wide("");
        let term_style = WS_CHILD
            | WS_VISIBLE
            | (ES_MULTILINE as u32)
            | (ES_READONLY as u32)
            | (ES_AUTOVSCROLL as u32)
            | (ES_NOHIDESEL as u32);
        let hwnd_term = CreateWindowExW(
            0,
            term_class.as_ptr(),
            empty_text.as_ptr(),
            term_style,
            0,
            0,
            10,
            10,
            hwnd,
            ID_TERM as usize as *mut c_void,
            instance,
            null(),
        );
        if hwnd_term.is_null() {
            return;
        }
        SendMessageW(hwnd_term, EM_SETBKGNDCOLOR, 0, rgb(C_TERM_BG) as isize);
        SendMessageW(hwnd_term, WM_SETFONT, font_term as usize, 1);
        SendMessageW(
            hwnd_term,
            EM_SETMARGINS,
            (EC_LEFTMARGIN | EC_RIGHTMARGIN) as usize,
            0,
        );

        let button_class = wide("BUTTON");
        let mut buttons = [Btn {
            hwnd: null_mut(),
            rect: RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            label: "",
        }; 5];
        for (i, (&label, &id)) in BTN_LABELS.iter().zip(BTN_IDS.iter()).enumerate() {
            let text = wide(label);
            buttons[i] = Btn {
                hwnd: CreateWindowExW(
                    0,
                    button_class.as_ptr(),
                    text.as_ptr(),
                    WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_OWNERDRAW,
                    0,
                    0,
                    10,
                    10,
                    hwnd,
                    id as usize as *mut c_void,
                    instance,
                    null(),
                ),
                rect: RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                },
                label,
            };
        }

        let state = Box::new(AppState {
            hwnd,
            hwnd_term,
            buttons,
            font_term,
            font_ui,
            pal,
            term: RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            inner: RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            sep: RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            scale,
            queue: VecDeque::new(),
            draining: false,
            hover: None,
        });
        let ptr = Box::into_raw(state);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr as isize);

        SendMessageW(hwnd, WM_SETICON, ICON_SMALL, icon_small as isize);
        SendMessageW(hwnd, WM_SETICON, ICON_BIG, icon_big as isize);

        layout(ptr);
        // WS_EX_APPWINDOW 已让窗口显示在任务栏；此处再设为置顶，与旧版一致。
        SetWindowPos(
            hwnd,
            (-1isize) as HWND,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
        ShowWindow(hwnd, SW_SHOWNORMAL);
        UpdateWindow(hwnd);

        SetTimer(hwnd, TIMER_HOVER, 60, None);
        SetTimer(hwnd, TIMER_STARTUP, 120, None);

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            if msg.message == WM_KEYDOWN && msg.wParam == VK_ESCAPE {
                DestroyWindow(hwnd);
                continue;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // GDI 对象已在 WM_DESTROY 中释放，此处仅回收状态内存。
        drop(Box::from_raw(ptr));
    }
}
