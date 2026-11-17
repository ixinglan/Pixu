//! Tauri command 薄层：仅做参数转发，业务逻辑在 app-service trait。
//! M0 全部走 Mock 实现，M1 起替换为 Real 实现时本文件零改动。

use pixu_app_service::*;
use tauri::State;

use crate::mock::{MockCleanService, MockScanService, MockSettingsService, MockStatusService, MockUninstallService};

// ─── 自检 ───────────────────────────────────────────────────

#[tauri::command]
pub fn ping() -> &'static str { "pong" }

// ─── 扫描 / 数据 ────────────────────────────────────────────

#[tauri::command]
pub fn scan_clean_groups(svc: State<'_, MockScanService>) -> AppResult<Vec<CleanGroup>> {
    svc.scan_clean_groups()
}

#[tauri::command]
pub fn scan_ai_tools(svc: State<'_, MockScanService>) -> AppResult<Vec<AiTool>> {
    svc.scan_ai_tools()
}

#[tauri::command]
pub fn list_apps(svc: State<'_, MockScanService>) -> AppResult<Vec<AppEntry>> {
    svc.list_apps()
}

#[tauri::command]
pub fn analyze_disk(svc: State<'_, MockScanService>) -> AppResult<DiskNode> {
    svc.analyze_disk()
}

// ─── 状态 ───────────────────────────────────────────────────

#[tauri::command]
pub fn live_metrics(svc: State<'_, MockStatusService>) -> AppResult<Metrics> {
    svc.live_metrics()
}

#[tauri::command]
pub fn list_processes(svc: State<'_, MockStatusService>) -> AppResult<Vec<ProcessInfo>> {
    svc.list_processes()
}

// ─── 设置 ───────────────────────────────────────────────────

#[tauri::command]
pub fn get_history(svc: State<'_, MockSettingsService>) -> AppResult<Vec<HistoryEntry>> {
    svc.get_history()
}

#[tauri::command]
pub fn get_whitelist(svc: State<'_, MockSettingsService>) -> AppResult<Vec<WhitelistEntry>> {
    svc.get_whitelist()
}

#[tauri::command]
pub fn get_settings(svc: State<'_, MockSettingsService>) -> AppResult<Settings> {
    svc.get_settings()
}

// ─── 清理 / 卸载操作 ────────────────────────────────────────

#[tauri::command]
pub fn preview_delete(svc: State<'_, MockCleanService>, items: Vec<CleanItem>) -> AppResult<PreviewResult> {
    svc.preview_delete(items)
}

#[tauri::command]
pub fn execute_clean(svc: State<'_, MockCleanService>, items: Vec<CleanItem>, permanent: bool) -> AppResult<CleanResult> {
    svc.execute_clean(items, permanent)
}

#[tauri::command]
pub fn uninstall_app(svc: State<'_, MockUninstallService>, app_id: String, keep_data: bool) -> AppResult<UninstallResult> {
    svc.uninstall_app(app_id, keep_data)
}

#[tauri::command]
pub fn move_to_trash(svc: State<'_, MockUninstallService>, path: String) -> AppResult<TrashResult> {
    svc.move_to_trash(path)
}

#[tauri::command]
pub fn open_fda_settings(svc: State<'_, MockUninstallService>) -> AppResult<FdaResult> {
    svc.open_fda_settings()
}
