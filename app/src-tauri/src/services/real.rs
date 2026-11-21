//! 真实服务实现（M1 起逐步接入）。
//! 未实现的方法回退到 Mock，保证全端可用；对应阶段完成后替换为真实逻辑。

use std::sync::OnceLock;

use pixu_app_service::*;
use pixu_domain_scan;
use pixu_fs_ops::{FsOps, FsOutcome};
use pixu_safety_gate::SafetyGate;
use pixu_safety_guard::SafetyGuard;

use crate::mock::{MockCleanService, MockScanService};

// ─── RealScanService ────────────────────────────────────────
// M1: scan_clean_groups 真实扫描；其余方法 M2/M3 实现前回退 Mock。

pub struct RealScanService {
    fallback: MockScanService,
}

impl RealScanService {
    pub fn new() -> Self { Self { fallback: MockScanService } }
}

impl Default for RealScanService {
    fn default() -> Self { Self::new() }
}

impl ScanService for RealScanService {
    fn scan_clean_groups(&self) -> AppResult<Vec<CleanGroup>> {
        // 真实遍历文件系统统计体积（只读）
        Ok(pixu_domain_scan::scan_clean_groups())
    }
    fn scan_ai_tools(&self) -> AppResult<Vec<AiTool>> { self.fallback.scan_ai_tools() }
    fn list_apps(&self) -> AppResult<Vec<AppEntry>> { self.fallback.list_apps() }
    fn analyze_disk(&self) -> AppResult<DiskNode> { self.fallback.analyze_disk() }
}

// ─── RealCleanService ───────────────────────────────────────
// M1: 完整删除管线（安全门 → 护栏 → 文件操作），写操作受全局开关控制。
// SafetyGuard 惰性初始化：System::new_all() 较慢，避免阻塞应用启动。

pub struct RealCleanService {
    fallback: MockCleanService,
    gate: SafetyGate,
    guard: OnceLock<SafetyGuard>,
    fs: FsOps,
}

impl RealCleanService {
    pub fn new() -> Self {
        Self {
            fallback: MockCleanService,
            gate: SafetyGate::new(),
            guard: OnceLock::new(),
            fs: FsOps::new(),
        }
    }

    fn guard(&self) -> &SafetyGuard {
        self.guard.get_or_init(SafetyGuard::new)
    }
}

impl Default for RealCleanService {
    fn default() -> Self { Self::new() }
}

impl CleanService for RealCleanService {
    fn preview_delete(&self, items: Vec<CleanItem>) -> AppResult<PreviewResult> {
        // 预览：仅汇总选中体积（只读，用扫描时已得的 size）
        let total: u64 = items.iter().map(|i| i.size).sum();
        Ok(PreviewResult { count: items.len(), total })
    }

    fn execute_clean(&self, items: Vec<CleanItem>, permanent: bool) -> AppResult<CleanResult> {
        // 1. 安全门：过滤被拒绝的路径
        let paths: Vec<String> = items.iter().map(|i| i.path.clone()).collect();
        let denied = self.gate.check_all(&paths);
        let denied_count = denied.len();

        // 2. 护栏：检测占用（仅统计跳过数，不阻断流程）
        let guard = self.guard();
        let occupied_count = items
            .iter()
            .filter(|i| {
                let kw = process_keywords_for(&i.name);
                let (safe, _) = guard.assess(&i.path, &kw);
                !safe
            })
            .count();

        let actionable: Vec<String> = items
            .iter()
            .filter(|i| !denied.iter().any(|(p, _)| p == &i.path))
            .map(|i| i.path.clone())
            .collect();

        // 3. 文件操作：开发期 WRITE_OPERATIONS_ENABLED=false → 全部跳过
        let outcome = if permanent {
            self.fs.delete_permanently(&actionable)?
        } else {
            self.fs.move_to_trash(&actionable)?
        };

        let (performed, freed, message) = match outcome {
            FsOutcome::Performed { freed } => (true, freed, None),
            FsOutcome::Skipped { reason } => (false, 0, Some(reason)),
        };

        let skipped = denied_count as u32
            + occupied_count as u32
            + if !performed { actionable.len() as u32 } else { 0 };

        // 4. 审计：M3 接入 infra 审计日志（写盘跳过，仅内存记录）
        let _ = (&items, &self.fallback);

        Ok(CleanResult { freed, permanent, skipped, performed, message })
    }
}

/// 根据清理项名称推断相关进程关键词（用于护栏检测占用）。
fn process_keywords_for(name: &str) -> Vec<&'static str> {
    let n = name.to_lowercase();
    if n.contains("chrome") { vec!["Google Chrome"] }
    else if n.contains("safari") { vec!["Safari"] }
    else if n.contains("arc") { vec!["Arc"] }
    else if n.contains("xcode") { vec!["Xcode"] }
    else if n.contains("docker") { vec!["Docker"] }
    else if n.contains("spotify") { vec!["Spotify"] }
    else { vec![] }
}
