//! Pixu 应用服务层：用例编排 trait 与跨层 DTO。
//!
//! M0：仅定义 trait 与 serde 结构，Mock 实现位于 app/src-tauri/src/mock。
//! M1+：各 domain crate 提供 Real 实现，装配层替换即可，前端零改动。

use serde::{Deserialize, Serialize};

// ─── 通用错误 ───────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
}

impl AppError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self { code: code.into(), message: message.into() }
    }
    pub fn io(message: impl Into<String>) -> Self {
        Self { code: s("io"), message: message.into() }
    }
    pub fn safety(message: impl Into<String>) -> Self {
        Self { code: s("safety"), message: message.into() }
    }
    pub fn not_found(message: impl Into<String>) -> Self {
        Self { code: s("not_found"), message: message.into() }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::io(e.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

fn s(v: &str) -> String { v.to_string() }

// ─── 清理页 DTO ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanItem {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size: u64,
    pub selected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanGroup {
    pub id: String,
    pub name: String,
    pub hue: String,
    pub icon: String,
    pub desc: String,
    pub items: Vec<CleanItem>,
}

// ─── AI 数据页 DTO ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSubItem {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size: u64,
    pub selected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiTool {
    pub id: String,
    pub name: String,
    pub hue: String,
    pub icon: String,
    pub caches: Vec<AiSubItem>,
    pub sessions: Vec<AiSubItem>,
    pub clis: Vec<AiSubItem>,
}

// ─── 软件页 DTO ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Residual {
    pub name: String,
    pub path: String,
    pub size: u64,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub developer: String,
    pub size: u64,
    pub last_used: String,
    pub hue: String,
    pub initial: String,
    pub residuals: Vec<Residual>,
}

// ─── 分析页 DTO ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskNode {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<DiskNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protected: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub free: Option<bool>,
}

// ─── 状态页 DTO ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricItem {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<u32>,
    pub unit: String,
    pub spark: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cores: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub down: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub up: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub danger: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub cpu: MetricItem,
    pub mem: MetricItem,
    pub gpu: MetricItem,
    pub disk: MetricItem,
    pub net: MetricItem,
    pub battery: MetricItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub name: String,
    pub cpu: f64,
    pub mem: u32,
}

// ─── 设置页 DTO ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub time: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub freed: u64,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistEntry {
    pub id: String,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub fda_granted: bool,
    pub trash_first: bool,
    pub protect_running: bool,
    pub zero_telemetry: bool,
    pub update_check: bool,
    pub ai_entry: bool,
    pub external_volumes: bool,
}

// ─── 操作结果 DTO ───────────────────────────────────────────
// performed: 是否真实执行了写操作（开发期写操作一律跳过 → false）
// message:   跳过原因或执行说明

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewResult {
    pub count: usize,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanResult {
    pub freed: u64,
    pub permanent: bool,
    pub skipped: u32,
    pub performed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallResult {
    pub ok: bool,
    pub performed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrashResult {
    pub ok: bool,
    pub performed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FdaResult {
    pub opened: bool,
    pub performed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

// ─── 应用信息 / 更新（M4） ──────────────────────────────────

/// 应用版本号（取自 crate 版本，与 tauri.conf.json 保持一致）。
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 发布渠道：构建/运行时通过环境变量 PIXU_CHANNEL 切换（stable / beta）。
/// 未设置时默认 stable；Beta 包构建时注入 PIXU_CHANNEL=beta 即可分流。
pub fn app_channel() -> String {
    std::env::var("PIXU_CHANNEL").unwrap_or_else(|_| "stable".to_string())
}

/// 构建 profile（debug / release），用于「关于」页展示。
pub fn build_profile() -> &'static str {
    if cfg!(debug_assertions) { "debug" } else { "release" }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub channel: String,
    pub build: String,
    pub write_enabled: bool,
}

pub fn app_info() -> AppInfo {
    AppInfo {
        version: APP_VERSION.to_string(),
        channel: app_channel(),
        build: build_profile().to_string(),
        write_enabled: WRITE_OPERATIONS_ENABLED,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub available: bool,
    pub current_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// 安装动作是否真实执行：开发期一律 false（写操作跳过）。
    pub performed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl UpdateInfo {
    pub fn not_available(current_version: String, message: Option<String>) -> Self {
        Self {
            available: false,
            current_version,
            latest_version: None,
            notes: None,
            url: None,
            performed: false,
            message,
        }
    }
}

/// 开发期全局开关：写操作是否真实执行。
/// false 时所有写操作返回「跳过」结果，不改动用户文件。
pub const WRITE_OPERATIONS_ENABLED: bool = false;

pub fn skip_message() -> String {
    s("写操作已跳过（开发期 WRITE_OPERATIONS_ENABLED=false），请在 docs/pending-write-operations.md 审阅后手动执行")
}

// ─── 服务 trait ─────────────────────────────────────────────

pub trait ScanService: Send + Sync {
    fn scan_clean_groups(&self) -> AppResult<Vec<CleanGroup>>;
    fn scan_ai_tools(&self) -> AppResult<Vec<AiTool>>;
    fn list_apps(&self) -> AppResult<Vec<AppEntry>>;
    fn analyze_disk(&self) -> AppResult<DiskNode>;
}

pub trait StatusService: Send + Sync {
    fn live_metrics(&self) -> AppResult<Metrics>;
    fn list_processes(&self) -> AppResult<Vec<ProcessInfo>>;
}

pub trait SettingsService: Send + Sync {
    fn get_history(&self) -> AppResult<Vec<HistoryEntry>>;
    fn get_whitelist(&self) -> AppResult<Vec<WhitelistEntry>>;
    fn get_settings(&self) -> AppResult<Settings>;
}

pub trait CleanService: Send + Sync {
    fn preview_delete(&self, items: Vec<CleanItem>) -> AppResult<PreviewResult>;
    fn execute_clean(&self, items: Vec<CleanItem>, permanent: bool) -> AppResult<CleanResult>;
}

pub trait UninstallService: Send + Sync {
    fn uninstall_app(&self, app_id: String, keep_data: bool) -> AppResult<UninstallResult>;
    fn move_to_trash(&self, path: String) -> AppResult<TrashResult>;
    fn open_fda_settings(&self) -> AppResult<FdaResult>;
}
