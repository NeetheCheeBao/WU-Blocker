//! 终端输出：RichEdit 增量写入、按中英文分段设置字体，以及输出队列的定时消费。
use super::state::{AppState, RenderLine, TIMER_DRAIN};
use super::theme::{FACE_CJK, FACE_MONO};
use crate::core;
use crate::win::*;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::Controls::{
    EM_GETLINECOUNT, EM_LINEINDEX, EM_REPLACESEL, EM_SCROLLCARET, EM_SETSEL,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetWindowTextLengthW, KillTimer, SendMessageW, SetTimer, SetWindowTextW,
};

pub(crate) const EM_SETBKGNDCOLOR: u32 = 0x0443;
const EM_SETCHARFORMAT: u32 = 0x0444;
const SCF_SELECTION: usize = 0x0001;
const CFM_COLOR: u32 = 0x4000_0000;
const CFM_FACE: u32 = 0x2000_0000;

#[repr(C)]
struct CharFormatW {
    cb_size: u32,
    dw_mask: u32,
    dw_effects: u32,
    y_height: i32,
    y_offset: i32,
    cr_text_color: u32,
    b_char_set: u8,
    b_pitch_and_family: u8,
    sz_face_name: [u16; 32],
}

impl CharFormatW {
    /// 同时指定文字颜色与字体，避免 RichEdit 自行回退时撑宽中文。
    fn new(color: u32, face: &str) -> Self {
        let mut cf: Self = unsafe { std::mem::zeroed() };
        cf.cb_size = std::mem::size_of::<Self>() as u32;
        cf.dw_mask = CFM_COLOR | CFM_FACE;
        cf.cr_text_color = color;
        for (slot, unit) in cf.sz_face_name.iter_mut().zip(wide(face)) {
            *slot = unit;
        }
        cf
    }
}

/// 判断字符是否需要交给中文字体渲染。
fn is_wide(c: char) -> bool {
    matches!(
        c as u32,
        0x2010..=0x205E | 0x2200..=0x22FF | 0x2E80..=0x9FFF | 0xF900..=0xFAFF
            | 0xFE30..=0xFE4F | 0xFF00..=0xFFEF
    )
}

/// 用指定字体插入一段纯中文或纯英文文本。
unsafe fn push_run(term: HWND, text: &str, wide_run: bool, color: u32) {
    let face = if wide_run { FACE_CJK } else { FACE_MONO };
    let cf = CharFormatW::new(rgb(color), face);
    SendMessageW(term, EM_SETCHARFORMAT, SCF_SELECTION, &cf as *const _ as isize);
    let payload = wide(text);
    SendMessageW(term, EM_REPLACESEL, 0, payload.as_ptr() as isize);
}

/// 按中英文切分成若干段后逐段插入，保证中文不被回退字体撑宽。
unsafe fn term_write(term: HWND, text: &str, color: u32) {
    let mut run = String::new();
    let mut run_wide = false;
    for ch in text.chars() {
        let wide_char = is_wide(ch);
        if !run.is_empty() && wide_char != run_wide {
            push_run(term, &run, run_wide, color);
            run.clear();
        }
        run_wide = wide_char;
        run.push(ch);
    }
    if !run.is_empty() {
        push_run(term, &run, run_wide, color);
    }
}

/// 向 RichEdit 追加一行带颜色的文本，并把行数限制在 200 行。
unsafe fn term_append(term: HWND, text: &str, color: u32) {
    let end = GetWindowTextLengthW(term);
    SendMessageW(term, EM_SETSEL, end as usize, end as isize);
    term_write(term, &format!("{}\r\n", text), color);
    let count = SendMessageW(term, EM_GETLINECOUNT, 0, 0) as i32;
    if count > 200 {
        let idx = SendMessageW(term, EM_LINEINDEX, (count - 200) as usize, 0) as isize;
        if idx > 0 {
            SendMessageW(term, EM_SETSEL, 0, idx);
            let empty = wide("");
            SendMessageW(term, EM_REPLACESEL, 0, empty.as_ptr() as isize);
        }
    }
    SendMessageW(term, EM_SCROLLCARET, 0, 0);
}

/// 把待输出的日志行加入队列，并启动消费定时器。
pub(crate) unsafe fn enqueue(ptr: *mut AppState, lines: Vec<core::LogLine>) {
    let hwnd = (*ptr).hwnd;
    let mut start_timer = false;
    {
        let st = &mut *ptr;
        for l in lines {
            let text = format!("{}{}", core::local_timestamp(), l.text);
            st.queue.push_back(RenderLine {
                text,
                color: l.kind.color(),
            });
        }
        if !st.draining && !st.queue.is_empty() {
            st.draining = true;
            start_timer = true;
        }
    }
    if start_timer {
        SetTimer(hwnd, TIMER_DRAIN, 15, None);
    }
}

/// 清空队列与终端正文，用于刷新前的复位。
pub(crate) unsafe fn clear_output(ptr: *mut AppState) {
    let hwnd = (*ptr).hwnd;
    let term = (*ptr).hwnd_term;
    let was_draining = {
        let st = &mut *ptr;
        st.queue.clear();
        let d = st.draining;
        st.draining = false;
        d
    };
    if was_draining {
        KillTimer(hwnd, TIMER_DRAIN);
    }
    let empty = wide("");
    SetWindowTextW(term, empty.as_ptr());
}

/// 定时器回调：每次取一行写入终端。
pub(crate) unsafe fn drain_one(ptr: *mut AppState) {
    let hwnd = (*ptr).hwnd;
    let term = (*ptr).hwnd_term;
    let item = {
        let st = &mut *ptr;
        let item = st.queue.pop_front();
        if st.queue.is_empty() {
            st.draining = false;
            KillTimer(hwnd, TIMER_DRAIN);
        }
        item
    };
    if let Some(l) = item {
        term_append(term, &l.text, l.color);
    }
}
