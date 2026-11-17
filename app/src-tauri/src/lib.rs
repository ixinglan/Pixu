//! Pixu 桌面壳入口（M0）：装配 Tauri 运行时 + Mock 服务。
//! 业务逻辑全部位于 workspace crates，本文件保持薄层。

mod commands;
mod mock;

use commands::*;
use mock::{MockCleanService, MockScanService, MockSettingsService, MockStatusService, MockUninstallService};

pub fn run() {
    tauri::Builder::default()
        .manage(MockScanService)
        .manage(MockStatusService)
        .manage(MockSettingsService)
        .manage(MockCleanService)
        .manage(MockUninstallService)
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running Pixu");
}
