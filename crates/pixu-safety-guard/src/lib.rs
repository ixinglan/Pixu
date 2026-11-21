//! Pixu 安全护栏（safety-guard）：删除前的「占用/活跃」检测。
//!
//! 职责：目标进程存活检测、活动 SQLite（-wal/-shm 占用）识别、
//! 下载中文件保护、句柄占用检查。与 safety-gate 共同构成删除前防线。
//! 本 crate 只做「读 + 判断」，不执行任何写操作。

use std::path::Path;

use sysinfo::{Process, ProcessesToUpdate, System};

pub struct SafetyGuard {
    sys: System,
}

impl SafetyGuard {
    pub fn new() -> Self {
        let sys = System::new_all();
        Self { sys }
    }

    /// 重新拉取进程列表（读操作）。
    pub fn refresh(&mut self) {
        self.sys.refresh_processes(ProcessesToUpdate::All, true);
    }

    /// 判断给定名称的进程是否正在运行（模糊匹配进程名）。
    pub fn is_process_running(&self, name: &str) -> bool {
        let needle = name.to_lowercase();
        self.sys.processes().values().any(|p| proc_name(p).to_lowercase().contains(&needle))
    }

    /// 返回所有匹配关键词的进程名（去重），用于 UI 展示「哪些进程占用」。
    pub fn matching_processes(&self, keywords: &[&str]) -> Vec<String> {
        let mut out: Vec<String> = keywords
            .iter()
            .flat_map(|k| {
                let needle = k.to_lowercase();
                self.sys
                    .processes()
                    .values()
                    .filter(move |p| proc_name(p).to_lowercase().contains(&needle))
                    .map(proc_name)
            })
            .collect();
        out.sort();
        out.dedup();
        out
    }

    /// 判断路径是否有活动 SQLite（存在 -wal 或 -shm 伴随文件，通常表示被占用）。
    pub fn has_active_sqlite(&self, path: &str) -> bool {
        let p = Path::new(path);
        let wal = p.with_extension("db-wal");
        let shm = p.with_extension("db-shm");
        let wal2: std::path::PathBuf = format!("{}-wal", path).into();
        let shm2: std::path::PathBuf = format!("{}-shm", path).into();
        wal.exists() || shm.exists() || wal2.exists() || shm2.exists()
    }

    /// 判断文件是否「下载中」：常见临时扩展名。
    pub fn is_downloading(&self, path: &str) -> bool {
        let lower = path.to_lowercase();
        lower.ends_with(".crdownload")
            || lower.ends_with(".download")
            || lower.ends_with(".part")
            || lower.ends_with(".tmp")
    }

    /// 综合判断：目标是否「安全可删」（无占用、非下载中）。
    /// 返回 (是否安全, 原因列表)。
    pub fn assess(&self, path: &str, related_process_keywords: &[&str]) -> (bool, Vec<String>) {
        let mut reasons = Vec::new();
        let procs = self.matching_processes(related_process_keywords);
        if !procs.is_empty() {
            reasons.push(format!("相关进程正在运行：{}", procs.join(", ")));
        }
        if self.has_active_sqlite(path) {
            reasons.push("检测到活动 SQLite 数据库（-wal/-shm）".into());
        }
        if self.is_downloading(path) {
            reasons.push("文件似乎正在下载中".into());
        }
        (reasons.is_empty(), reasons)
    }
}

impl Default for SafetyGuard {
    fn default() -> Self { Self::new() }
}

fn proc_name(p: &Process) -> String {
    p.name().to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_download_ext() {
        let g = SafetyGuard::new();
        assert!(g.is_downloading("/tmp/x.crdownload"));
        assert!(g.is_downloading("/tmp/y.part"));
        assert!(!g.is_downloading("/tmp/z.txt"));
    }

    #[test]
    fn can_list_processes() {
        let g = SafetyGuard::new();
        // 不应 panic，且至少能拿到一些进程
        let kernel = g.is_process_running("kernel");
        // kernel_task 一定在跑（macOS）
        assert!(kernel || g.is_process_running("kernel_task"));
    }
}
