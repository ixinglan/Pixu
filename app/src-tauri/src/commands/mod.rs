//! Tauri command 薄层：仅做参数转发，业务逻辑在 app-service trait。
//! 经 ServiceRegistry 取用服务，不绑定具体实现（Mock/Real 可互换）。
//!
//! 性能约定：凡是可能阻塞的命令（文件扫描 / sysinfo 刷新 / 外部命令 /
//! SafetyGuard 初始化）一律声明为 `async fn` 并经 `spawn_blocking`
//! 放到工作线程执行——同步 command 会占用 Tauri 主线程，阻塞 WebKit
//! 导致菜单切换时鼠标转圈。纯内存命令（ping / get_app_info）保持同步。

use pixu_app_service::*;
use tauri::State;

use crate::services::ServiceRegistry;

/// 把阻塞式服务调用移到工作线程，避免冻结 UI 主线程。
async fn offload<F, T>(f: F) -> AppResult<T>
where
    F: FnOnce() -> AppResult<T> + Send + 'static,
    T: Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| AppError::new("join", format!("后台任务执行失败：{}", e)))?
}

// ─── 自检 ───────────────────────────────────────────────────

#[tauri::command]
pub fn ping() -> &'static str { "pong" }

// ─── 扫描 / 数据（均为重型只读，工作线程执行）────────────────

#[tauri::command]
pub async fn scan_clean_groups(reg: State<'_, ServiceRegistry>) -> AppResult<Vec<CleanGroup>> {
    let svc = reg.scan.clone();
    offload(move || svc.scan_clean_groups()).await
}

#[tauri::command]
pub async fn scan_ai_tools(reg: State<'_, ServiceRegistry>) -> AppResult<Vec<AiTool>> {
    let svc = reg.scan.clone();
    offload(move || svc.scan_ai_tools()).await
}

#[tauri::command]
pub async fn list_apps(reg: State<'_, ServiceRegistry>) -> AppResult<Vec<AppEntry>> {
    let svc = reg.scan.clone();
    offload(move || svc.list_apps()).await
}

#[tauri::command]
pub async fn analyze_disk(reg: State<'_, ServiceRegistry>) -> AppResult<DiskNode> {
    let svc = reg.scan.clone();
    offload(move || svc.analyze_disk()).await
}

// ─── 状态（读后台采样快照，本身极快；仍异步化以统一线程模型）──

#[tauri::command]
pub async fn live_metrics(reg: State<'_, ServiceRegistry>) -> AppResult<Metrics> {
    let svc = reg.status.clone();
    offload(move || svc.live_metrics()).await
}

#[tauri::command]
pub async fn list_processes(reg: State<'_, ServiceRegistry>) -> AppResult<Vec<ProcessInfo>> {
    let svc = reg.status.clone();
    offload(move || svc.list_processes()).await
}

// ─── 设置 ───────────────────────────────────────────────────

#[tauri::command]
pub async fn get_history(reg: State<'_, ServiceRegistry>) -> AppResult<Vec<HistoryEntry>> {
    let svc = reg.settings.clone();
    offload(move || svc.get_history()).await
}

#[tauri::command]
pub async fn get_whitelist(reg: State<'_, ServiceRegistry>) -> AppResult<Vec<WhitelistEntry>> {
    let svc = reg.settings.clone();
    offload(move || svc.get_whitelist()).await
}

#[tauri::command]
pub async fn get_settings(reg: State<'_, ServiceRegistry>) -> AppResult<Settings> {
    let svc = reg.settings.clone();
    offload(move || svc.get_settings()).await
}

// ─── 清理 / 卸载操作（写操作开发期跳过；SafetyGuard 初始化很重，仍需工作线程）

#[tauri::command]
pub async fn preview_delete(
    reg: State<'_, ServiceRegistry>,
    items: Vec<CleanItem>,
) -> AppResult<PreviewResult> {
    let svc = reg.clean.clone();
    offload(move || svc.preview_delete(items)).await
}

#[tauri::command]
pub async fn execute_clean(
    reg: State<'_, ServiceRegistry>,
    items: Vec<CleanItem>,
    permanent: bool,
) -> AppResult<CleanResult> {
    let svc = reg.clean.clone();
    offload(move || svc.execute_clean(items, permanent)).await
}

#[tauri::command]
pub async fn uninstall_app(
    reg: State<'_, ServiceRegistry>,
    app_id: String,
    keep_data: bool,
) -> AppResult<UninstallResult> {
    let svc = reg.uninstall.clone();
    offload(move || svc.uninstall_app(app_id, keep_data)).await
}

#[tauri::command]
pub async fn move_to_trash(
    reg: State<'_, ServiceRegistry>,
    path: String,
) -> AppResult<TrashResult> {
    let svc = reg.uninstall.clone();
    offload(move || svc.move_to_trash(path)).await
}

#[tauri::command]
pub async fn open_fda_settings(reg: State<'_, ServiceRegistry>) -> AppResult<FdaResult> {
    let svc = reg.uninstall.clone();
    offload(move || svc.open_fda_settings()).await
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
