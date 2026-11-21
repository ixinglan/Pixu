//! Pixu 文件操作层（fs-ops）：实际的删除 / 移入废纸篓执行点。
//!
//! 职责：NSFileManager 废纸篓（支持 Finder Put Back）、永久删除（仅独立开关 +
//! 二次确认场景）、操作超时预算、结果回传。
//!
//! ⚠️ 安全约束：本 crate **不做路径决策**——输入路径必须已经过 safety-gate 校验。
//! 所有写操作受 `pixu_app_service::WRITE_OPERATIONS_ENABLED` 全局开关控制；
//! 开发期（false）一律返回 `Skipped`，不改动用户文件。

use pixu_app_service::{skip_message, AppError, AppResult, WRITE_OPERATIONS_ENABLED};
use pixu_safety_gate::SafetyGate;

/// 文件操作结果。
#[derive(Debug, Clone)]
pub enum FsOutcome {
    /// 真实执行完成，释放字节数。
    Performed { freed: u64 },
    /// 未执行（开发期写操作被跳过 / 或安全门拒绝）。
    Skipped { reason: String },
}

pub struct FsOps {
    gate: SafetyGate,
}

impl FsOps {
    pub fn new() -> Self { Self { gate: SafetyGate::new() } }

    /// 移入废纸篓（支持 Finder Put Back）。
    /// 开发期返回 Skipped，不执行。
    pub fn move_to_trash(&self, paths: &[String]) -> AppResult<FsOutcome> {
        // 安全门：拒绝路径直接报错
        let denied = self.gate.check_all(paths);
        if !denied.is_empty() {
            return Err(AppError::safety(format!(
                "安全门拒绝 {} 项：{}",
                denied.len(),
                denied.iter().map(|(p, r)| format!("{}({})", p, r)).collect::<Vec<_>>().join("; ")
            )));
        }

        if !WRITE_OPERATIONS_ENABLED {
            return Ok(FsOutcome::Skipped { reason: skip_message() });
        }

        // TODO(M5/发布前): 启用时此处调用 NSFileManager trashItemAtURL:
        //   let fm = NSFileManager::defaultManager();
        //   fm.trashItemAtURL_resultingItemURL_error_(url, None)?;
        // 当前未接 objc2，开发期不会走到这里。
        Ok(FsOutcome::Performed { freed: 0 })
    }

    /// 永久删除（绕过废纸篓，不可恢复）。
    /// 仅在 trash_first=false 且用户二次确认时调用；开发期返回 Skipped。
    pub fn delete_permanently(&self, paths: &[String]) -> AppResult<FsOutcome> {
        let denied = self.gate.check_all(paths);
        if !denied.is_empty() {
            return Err(AppError::safety(format!("安全门拒绝 {} 项", denied.len())));
        }

        if !WRITE_OPERATIONS_ENABLED {
            return Ok(FsOutcome::Skipped { reason: skip_message() });
        }

        // TODO: std::fs::remove_dir_all / remove_file，带超时预算
        Ok(FsOutcome::Performed { freed: 0 })
    }

    /// 只读：计算一组路径的总字节数（用于预览，不删除）。
    pub fn measure(&self, paths: &[String]) -> u64 {
        paths.iter().map(|p| dir_size_approx(p)).sum()
    }
}

impl Default for FsOps {
    fn default() -> Self { Self::new() }
}

/// 递归估算目录/文件大小（读操作，遇到无权限跳过）。
fn dir_size_approx(path: &str) -> u64 {
    use std::fs;
    let p = std::path::Path::new(path);
    let meta = match fs::metadata(p) { Ok(m) => m, Err(_) => return 0 };
    if meta.is_file() {
        return meta.len();
    }
    let mut total = 0u64;
    if let Ok(rd) = fs::read_dir(p) {
        for entry in rd.flatten() {
            total += dir_size_approx(&entry.path().to_string_lossy());
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trash_is_skipped_in_dev() {
        let fs = FsOps::new();
        let r = fs.move_to_trash(&["/tmp/nonexistent_pixu_test_dir".into()]).unwrap();
        // 开发期应返回 Skipped（即使路径不存在，开关先于执行）
        match r {
            FsOutcome::Skipped { .. } => {}
            FsOutcome::Performed { .. } => panic!("expected skipped in dev mode"),
        }
    }

    #[test]
    fn denies_system_paths() {
        let fs = FsOps::new();
        let r = fs.move_to_trash(&["/System".into()]);
        assert!(r.is_err());
    }
}
