//! 程序入口：DPI 感知、管理员提权与 GUI 主循环。
#![windows_subsystem = "windows"]

mod config;
mod core;
mod ui;
mod win;

use crate::win::{wide, SW_SHOWNORMAL};
use std::ptr::{null, null_mut};
use windows_sys::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows_sys::Win32::UI::Shell::{IsUserAnAdmin, ShellExecuteW};
use windows_sys::Win32::UI::WindowsAndMessaging::SetProcessDPIAware;

/// 启用每显示器 DPI 感知 V2；旧系统上回退到进程级 DPI 感知。
fn set_dpi_awareness() {
    unsafe {
        if SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) == 0 {
            SetProcessDPIAware();
        }
    }
}

/// 以管理员权限重新启动自身。返回 true 表示当前进程应当直接退出。
fn elevate_if_needed() -> bool {
    unsafe {
        if IsUserAnAdmin() != 0 {
            return false;
        }
        // 未提权时请求 UAC；成功后由新进程承载界面，当前进程结束。
        let exe = std::env::current_exe().unwrap_or_default();
        let exe = wide(&exe.to_string_lossy());
        let operation = wide("runas");
        let params = wide("");
        let result = ShellExecuteW(
            null_mut(),
            operation.as_ptr(),
            exe.as_ptr(),
            params.as_ptr(),
            null(),
            SW_SHOWNORMAL,
        );
        // 提权被拒绝时仍以普通权限运行，便于查看状态（写操作会给出权限提示）。
        (result as isize) > 32
    }
}

fn main() {
    set_dpi_awareness();
    if elevate_if_needed() {
        return;
    }
    ui::run();
}
