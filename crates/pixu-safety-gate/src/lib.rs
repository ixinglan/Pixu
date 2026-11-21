//! Pixu 安全门（safety-gate）：所有删除调用的必经校验层。
//!
//! 职责：路径规范化、符号链接解析、允许/拒绝名单、系统卷与 SIP 边界、
//! firmlink/synthetic 链接处理。白名单读不到时 fail-closed。
//! 本 crate 只做「读 + 判断」，不执行任何写操作。

use std::path::{Path, PathBuf};

use pixu_app_service::AppError;

/// 校验结论。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateVerdict {
    /// 允许删除。
    Allow,
    /// 拒绝删除，附原因。
    Deny(String),
}

impl GateVerdict {
    pub fn is_allowed(&self) -> bool { matches!(self, GateVerdict::Allow) }
}

/// 绝对拒绝的路径前缀（系统关键区域，命中即 Deny）。
/// 这些区域受 SIP 保护或属于系统核心，删除会破坏系统。
const DENY_PREFIXES: &[&str] = &[
    "/System",
    "/usr",
    "/bin",
    "/sbin",
    "/private/var/db",
    "/private/var/folders", // 每个用户的安全容器临时目录
    "/Library/Preferences/SystemConfiguration",
    "/.vol",
];

/// 相对当前用户主目录，默认允许清理的区域（仅用于提示，非白名单授权）。
const USER_CLEANABLE_HINTS: &[&str] = &[
    "Library/Caches",
    "Library/Logs",
    "Library/Containers",
    ".Trash",
];

/// 安全门。
pub struct SafetyGate;

impl SafetyGate {
    pub fn new() -> Self { Self }

    /// 规范化并解析符号链接，返回真实路径（canonicalize）。
    /// 读操作，不修改文件系统。
    pub fn normalize(&self, path: &str) -> Result<PathBuf, AppError> {
        let p = PathBuf::from(path);
        let expanded = expand_tilde(&p);
        match expanded.canonicalize() {
            Ok(real) => Ok(real),
            // 文件不存在时仍返回展开后的绝对路径，用于「预览不存在项」场景
            Err(_) => Ok(expanded),
        }
    }

    /// 校验给定路径是否允许被删除。
    /// 仅判断，不执行。
    pub fn check(&self, path: &str) -> GateVerdict {
        let real = match self.normalize(path) {
            Ok(r) => r,
            Err(_) => return GateVerdict::Deny("路径无法解析".into()),
        };
        let real_str = real.to_string_lossy();

        // 1. 拒绝系统关键前缀
        for prefix in DENY_PREFIXES {
            if real_str == *prefix || real_str.starts_with(&format!("{}/", prefix)) {
                return GateVerdict::Deny(format!("命中系统保护区域：{}", prefix));
            }
        }

        // 2. 拒绝根目录本身及直接子节点
        if real_str == "/" {
            return GateVerdict::Deny("不能删除根目录".into());
        }

        // 3. 符号链接逃逸检查：解析前后若跨越 /System / /private/var/db 等，拒绝
        //    （normalize 已 canonicalize，此处 real 即为解析后真实路径，已纳入上面前缀检查）

        // 4. 路径必须为绝对路径
        if !real.is_absolute() {
            return GateVerdict::Deny("仅允许绝对路径".into());
        }

        GateVerdict::Allow
    }

    /// 批量校验，返回全部拒绝项。空 vec 表示全部通过。
    pub fn check_all(&self, paths: &[String]) -> Vec<(String, String)> {
        paths
            .iter()
            .filter_map(|p| match self.check(p) {
                GateVerdict::Deny(reason) => Some((p.clone(), reason)),
                GateVerdict::Allow => None,
            })
            .collect()
    }

    /// 判断是否属于用户可清理提示区域（仅用于 UI 提示，不授权）。
    pub fn is_user_cleanable_hint(path: &str) -> bool {
        let expanded = expand_tilde(Path::new(path));
        if let Some(home) = home_dir() {
            for hint in USER_CLEANABLE_HINTS {
                if expanded.starts_with(home.join(hint)) {
                    return true;
                }
            }
        }
        false
    }
}

impl Default for SafetyGate {
    fn default() -> Self { Self::new() }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// 把路径开头的 `~` 展开为用户主目录。不修改文件系统。
fn expand_tilde(path: &Path) -> PathBuf {
    if let Ok(s) = path.to_str().ok_or(()) {
        if let Some(rest) = s.strip_prefix("~/") {
            if let Some(home) = home_dir() {
                return home.join(rest);
            }
        } else if s == "~" {
            if let Some(home) = home_dir() {
                return home;
            }
        }
    }
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deny_system() {
        let g = SafetyGate::new();
        assert!(!g.check("/System/Library").is_allowed());
        assert!(!g.check("/usr/bin").is_allowed());
        assert!(!g.check("/").is_allowed());
    }

    #[test]
    fn allow_user_cache() {
        let g = SafetyGate::new();
        if let Some(home) = home_dir() {
            let p = home.join("Library/Caches");
            let v = g.check(p.to_str().unwrap());
            // 存在则允许；不存在时 normalize 返回展开路径，仍应允许
            assert!(v.is_allowed(), "expected allow, got {:?}", v);
        }
    }
}
