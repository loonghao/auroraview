//! MiniProgram Builders
//!
//! Each mini-program platform has its own builder due to different:
//! - Project structure
//! - API/SDK
//! - CLI tools
//! - Upload/publish process

pub mod alipay;
pub mod bytedance;
pub mod wechat;

pub use alipay::AlipayBuilder;
pub use bytedance::ByteDanceBuilder;
pub use wechat::WeChatBuilder;
