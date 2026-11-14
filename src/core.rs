//! 平台逻辑层入口：注册表、时间换算、系统信息、状态查询与用户动作。
//! 各子模块文件以 `core_` 前缀命名，此处用 `#[path]` 挂载，保持 `core::` 命名空间不变。

#[path = "core_reg.rs"]
pub mod reg;
#[path = "core_time.rs"]
pub mod time;
#[path = "core_sys.rs"]
pub mod sys;
#[path = "core_status.rs"]
pub mod status;
#[path = "core_action.rs"]
pub mod action;

pub use action::{open_github, open_regedit, pause_updates, restore_updates};
pub use status::{refresh_lines, startup_lines, LogLine};
pub use time::local_timestamp;
