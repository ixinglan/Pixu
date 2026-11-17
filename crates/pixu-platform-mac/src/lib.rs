//! 平台层：macOS 专属系统绑定。
//!
//! 职责：TCC / Full Disk Access 状态读取与设置跳转、
//! SMAppService 封装（二期限时 helper 评审后启用）、
//! objc2 系统框架绑定、系统版本与卷信息。仅编译 aarch64-apple-darwin。
