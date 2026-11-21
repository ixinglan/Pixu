//! 服务装配：持有所有服务的 trait object，commands 经此取用。
//! 替换 Mock → Real 实现时，仅修改 `Registry::new()`，commands 与前端零改动。

pub mod real;

use std::sync::Arc;

use pixu_app_service::*;

use crate::mock::{MockSettingsService, MockStatusService, MockUninstallService};
use crate::services::real::{RealCleanService, RealScanService};

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
            // M1: 清理页扫描与清理走真实实现；其余暂用 Mock
            scan: Arc::new(RealScanService::new()),
            status: Arc::new(MockStatusService),
            settings: Arc::new(MockSettingsService),
            clean: Arc::new(RealCleanService::new()),
            uninstall: Arc::new(MockUninstallService),
        }
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self { Self::new() }
}
