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

// ─── 应用信息 / 自动更新（M4） ──────────────────────────────
// 「检查更新」为只读 + 网络，允许执行；「安装更新」属写/替换操作，开发期跳过。

#[tauri::command]
pub fn get_app_info() -> AppResult<AppInfo> {
    Ok(pixu_app_service::app_info())
}

#[tauri::command]
pub async fn check_update(app: tauri::AppHandle) -> AppResult<UpdateInfo> {
    use tauri_plugin_updater::UpdaterExt;

    let channel = pixu_app_service::app_channel();
    let current = pixu_app_service::APP_VERSION.to_string();

    // 骨架阶段：endpoint 域名 / pubkey 尚未部署，这里仅把 channel 注入查询串，
    // 服务端据此分流 stable / beta。{{target}}/{{arch}}/{{current_version}}
    // 由 updater 插件在运行时替换。
    let endpoint = format!(
        "https://updates.pixu.app/{{{{target}}}}/{{{{arch}}}}/{{{{current_version}}}}?channel={}",
        channel
    );
    let url = match url::Url::parse(&endpoint) {
        Ok(u) => u,
        Err(e) => return Ok(UpdateInfo::not_available(current, Some(format!("endpoint 解析失败：{}", e)))),
    };

    let updater = match app.updater_builder().endpoints(vec![url]) {
        Ok(b) => match b.build() {
            Ok(u) => u,
            Err(e) => return Ok(UpdateInfo::not_available(
                current,
                Some(format!("updater 初始化失败（骨架阶段）：{}", e)),
            )),
        },
        Err(e) => return Ok(UpdateInfo::not_available(
            current,
            Some(format!("updater endpoint 配置失败：{}", e)),
        )),
    };

    match updater.check().await {
        Ok(Some(u)) => Ok(UpdateInfo {
            available: true,
            current_version: current,
            latest_version: Some(u.version),
            notes: u.body,
            url: Some(u.download_url.to_string()),
            // 仅告知有更新；安装动作由 install_update 单独触发并受写开关控制。
            performed: false,
            message: None,
        }),
        Ok(None) => Ok(UpdateInfo::not_available(current, None)),
        Err(e) => Ok(UpdateInfo::not_available(
            current,
            Some(format!("更新检查失败（骨架阶段，endpoint/pubkey 未部署）：{}", e)),
        )),
    }
}

#[tauri::command]
pub fn install_update() -> AppResult<UpdateInfo> {
    // 安装/替换应用是写操作：开发期 WRITE_OPERATIONS_ENABLED=false → 一律跳过。
    Ok(UpdateInfo {
        available: false,
        current_version: pixu_app_service::APP_VERSION.to_string(),
        latest_version: None,
        notes: None,
        url: None,
        performed: false,
        message: Some(pixu_app_service::skip_message()),
    })
}
