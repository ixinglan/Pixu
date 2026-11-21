//! 服务装配：持有所有服务的 trait object，commands 经此取用。
//! 替换 Mock → Real 实现时，仅修改 `Registry::new()`，commands 与前端零改动。

pub mod real;

use std::sync::Arc;

use pixu_app_service::*;

use crate::services::real::{
    AuditStore, RealCleanService, RealScanService, RealSettingsService, RealStatusService,
    RealUninstallService,
};

pub struct ServiceRegistry {
    pub scan: Arc<dyn ScanService>,
    pub status: Arc<dyn StatusService>,
    pub settings: Arc<dyn SettingsService>,
    pub clean: Arc<dyn CleanService>,
    pub uninstall: Arc<dyn UninstallService>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        // 内存审计日志在「清理服务」与「设置服务」间共享（写盘跳过，M5 落盘）
        let audit = Arc::new(AuditStore::default());
        Self {
            // M3：全量切换真实实现；Mock 仅保留作单元测试/回退参考
            scan: Arc::new(RealScanService::new()),
            status: Arc::new(RealStatusService::new()),
            settings: Arc::new(RealSettingsService::new(audit.clone())),
            clean: Arc::new(RealCleanService::new(audit)),
            uninstall: Arc::new(RealUninstallService::new()),
        }
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self { Self::new() }
}
