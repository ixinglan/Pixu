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
}

pub type AppResult<T> = Result<T, AppError>;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallResult {
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrashResult {
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FdaResult {
    pub opened: bool,
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
