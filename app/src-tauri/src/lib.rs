//! Pixu 桌面壳入口：装配 Tauri 运行时 + 服务注册表。
//! 业务逻辑全部位于 workspace crates，本文件保持薄层。

mod commands;
// mock 服务保留用于回退切换；当前 Registry 装配 Real，故存在未引用的辅助函数。
#[allow(dead_code)]
mod mock;
mod services;

use commands::*;
use services::ServiceRegistry;

pub fn run() {
    tauri::Builder::default()
        .manage(ServiceRegistry::new())
        // M4：自动更新骨架。endpoint / pubkey 未部署前，check_update 会优雅降级为
        // 「无可用更新」；install_update 受写开关控制，开发期一律跳过。
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            ping,
            scan_clean_groups,
            scan_ai_tools,
            list_apps,
            analyze_disk,
            live_metrics,
            list_processes,
            get_history,
            get_whitelist,
            get_settings,
            preview_delete,
            execute_clean,
            uninstall_app,
            move_to_trash,
            open_fda_settings,
            get_app_info,
            check_update,
            install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Pixu");
}
