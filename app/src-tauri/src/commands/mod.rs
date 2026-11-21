//! Tauri command 薄层：仅做参数转发，业务逻辑在 app-service trait。
//! 经 ServiceRegistry 取用服务，不绑定具体实现（Mock/Real 可互换）。

use pixu_app_service::*;
use tauri::State;

use crate::services::ServiceRegistry;

// ─── 自检 ───────────────────────────────────────────────────

#[tauri::command]
pub fn ping() -> &'static str { "pong" }

// ─── 扫描 / 数据 ────────────────────────────────────────────

#[tauri::command]
pub fn scan_clean_groups(reg: State<'_, ServiceRegistry>) -> AppResult<Vec<CleanGroup>> {
    reg.scan.scan_clean_groups()
}

#[tauri::command]
pub fn scan_ai_tools(reg: State<'_, ServiceRegistry>) -> AppResult<Vec<AiTool>> {
    reg.scan.scan_ai_tools()
}

#[tauri::command]
pub fn list_apps(reg: State<'_, ServiceRegistry>) -> AppResult<Vec<AppEntry>> {
    reg.scan.list_apps()
}

#[tauri::command]
pub fn analyze_disk(reg: State<'_, ServiceRegistry>) -> AppResult<DiskNode> {
    reg.scan.analyze_disk()
}

// ─── 状态 ───────────────────────────────────────────────────

#[tauri::command]
pub fn live_metrics(reg: State<'_, ServiceRegistry>) -> AppResult<Metrics> {
    reg.status.live_metrics()
}

#[tauri::command]
pub fn list_processes(reg: State<'_, ServiceRegistry>) -> AppResult<Vec<ProcessInfo>> {
    reg.status.list_processes()
}

// ─── 设置 ───────────────────────────────────────────────────

#[tauri::command]
pub fn get_history(reg: State<'_, ServiceRegistry>) -> AppResult<Vec<HistoryEntry>> {
    reg.settings.get_history()
}

#[tauri::command]
pub fn get_whitelist(reg: State<'_, ServiceRegistry>) -> AppResult<Vec<WhitelistEntry>> {
    reg.settings.get_whitelist()
}

#[tauri::command]
pub fn get_settings(reg: State<'_, ServiceRegistry>) -> AppResult<Settings> {
    reg.settings.get_settings()
}

// ─── 清理 / 卸载操作 ────────────────────────────────────────

#[tauri::command]
pub fn preview_delete(reg: State<'_, ServiceRegistry>, items: Vec<CleanItem>) -> AppResult<PreviewResult> {
    reg.clean.preview_delete(items)
}

#[tauri::command]
pub fn execute_clean(reg: State<'_, ServiceRegistry>, items: Vec<CleanItem>, permanent: bool) -> AppResult<CleanResult> {
    reg.clean.execute_clean(items, permanent)
}

#[tauri::command]
pub fn uninstall_app(reg: State<'_, ServiceRegistry>, app_id: String, keep_data: bool) -> AppResult<UninstallResult> {
    reg.uninstall.uninstall_app(app_id, keep_data)
}

#[tauri::command]
pub fn move_to_trash(reg: State<'_, ServiceRegistry>, path: String) -> AppResult<TrashResult> {
    reg.uninstall.move_to_trash(path)
}

#[tauri::command]
pub fn open_fda_settings(reg: State<'_, ServiceRegistry>) -> AppResult<FdaResult> {
    reg.uninstall.open_fda_settings()
}
