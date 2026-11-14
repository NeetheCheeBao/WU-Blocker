//! 手写 Win32 自绘界面，复刻旧版无边框置顶窗口与内嵌日志终端。
//! 各子模块文件以 `ui_` 前缀命名，此处用 `#[path]` 挂载，保持 `ui::run` 入口不变。

#[path = "ui_theme.rs"]
pub mod theme;
#[path = "ui_state.rs"]
pub mod state;
#[path = "ui_layout.rs"]
pub mod layout;
#[path = "ui_term.rs"]
pub mod term;
#[path = "ui_paint.rs"]
pub mod paint;
#[path = "ui_menu.rs"]
pub mod menu;
#[path = "ui_window.rs"]
pub mod window;

pub use window::run;
