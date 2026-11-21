//! 服务装配：持有所有服务的 trait object，commands 经此取用。
//! 替换 Mock → Real 实现时，仅修改 `Registry::new()`，commands 与前端零改动。

pub mod real;

use std::sync::Arc;

use pixu_app_service::*;

use crate::mock::{MockSettingsService, MockStatusService};
use crate::services::real::{RealCleanService, RealScanService, RealUninstallService};

pub struct ServiceRegistry {
    pub scan: Arc<dyn ScanService>,
    pub status: Arc<dyn StatusService>,
    pub settings: Arc<dyn SettingsService>,
    pub clean: Arc<dyn CleanService>,
    pub uninstall: Arc<dyn UninstallService>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            // M2: 清理页、软件页、分析页走真实实现；状态/设置暂用 Mock（M3 替换）
            scan: Arc::new(RealScanService::new()),
            status: Arc::new(MockStatusService),
            settings: Arc::new(MockSettingsService),
            clean: Arc::new(RealCleanService::new()),
            uninstall: Arc::new(RealUninstallService::new()),
        }
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self { Self::new() }
}
