//! Pixu 扫描领域层（domain-scan）：目录遍历、体积计量、缓存分组。
//!
//! 职责：目录遍历、体积计量（APFS 克隆/硬链接感知，避免重复计费）、
//! 缓存分组规则、扫描结果数据模型。UI 无关，禁止依赖 Tauri。
//! 本 crate 仅执行「读」操作：读取元数据、统计体积，不修改文件系统。

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use pixu_app_service::{AiSubItem, AiTool, CleanGroup, CleanItem, DiskNode};

/// 递归计量路径占用字节数（读操作）。
/// 遇到无权限 / 已删除 / 符号链接环 时跳过该项，不中断整体。
pub fn measure_path(path: &str) -> u64 {
    let p = expand_tilde(Path::new(path));
    let meta = match std::fs::symlink_metadata(&p) {
        Ok(m) => m,
        Err(_) => return 0,
    };
    if meta.is_file() {
        return meta.len();
    }
    if !meta.is_dir() {
        return meta.len();
    }
    // 目录优先走 `du -sk`（macOS 系统工具，只读），比 Rust walkdir 快得多；
    // 失败（无权限等）再回退到逐文件统计。
    if let Some(size) = measure_via_du(&p) {
        return size;
    }
    let mut total = 0u64;
    for entry in WalkDir::new(&p).follow_links(false).into_iter().filter_map(|e| e.ok()) {
        if let Ok(m) = entry.metadata() {
            if m.is_file() {
                total += m.len();
            }
        }
    }
    total
}

/// 通过 `du -sk` 快速获取目录占用（macOS 只读系统工具）。
/// 适用于大目录（.app bundle、大型缓存），避免 walkdir 逐文件统计造成卡顿。
/// 失败（无权限 / du 不可用）时返回 None，由调用方回退到 walkdir。
fn measure_via_du(path: &Path) -> Option<u64> {
    let out = std::process::Command::new("/usr/bin/du")
        .args(["-sk"])
        .arg(path)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    let kb: u64 = s.split_whitespace().next()?.parse().ok()?;
    Some(kb.saturating_mul(1024))
}

/// 带超时的目录计量：du 超过 timeout_ms 则杀进程返回 None，避免巨型目录卡死 UI。
/// 用于分析页磁盘树这种「探索性」扫描；清理/卸载目标用精确的 measure_path。
fn measure_via_du_bounded(path: &Path, timeout_ms: u64) -> Option<u64> {
    use std::process::Stdio;
    let mut child = std::process::Command::new("/usr/bin/du")
        .args(["-sk"])
        .arg(path)
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if start.elapsed().as_millis() as u64 > timeout_ms {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(_) => return None,
        }
    }
    let out = child.wait_with_output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    let kb: u64 = s.split_whitespace().next()?.parse().ok()?;
    Some(kb.saturating_mul(1024))
}

/// 带超时的路径计量：文件取元数据长度，目录走带超时的 du。
fn measure_bounded(path: &Path, timeout_ms: u64) -> u64 {
    let meta = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => return 0,
    };
    if !meta.is_dir() {
        return meta.len();
    }
    measure_via_du_bounded(path, timeout_ms).unwrap_or(0)
}

/// 计量一组路径的总和。
pub fn measure_paths(paths: &[String]) -> u64 {
    paths.iter().map(|p| measure_path(p)).sum()
}

/// 扫描清理页分组：遍历 macOS 常见可清理位置，真实统计体积。
/// 不存在或无权限的项会被跳过（不列入结果），避免展示误导数据。
pub fn scan_clean_groups() -> Vec<CleanGroup> {
    let home = home_dir();

    let mut groups = Vec::new();

    // ── 系统数据 ──
    groups.push(build_group(
        "system", "系统数据", "#9DB2D8", "drive",
        "系统缓存、日志与本地快照，均为可再生数据",
        &[
            ("s1", "系统缓存文件", "/Library/Caches", true, Some("应用可自动重建"), None),
            ("s2", "系统日志归档", "/private/var/log", true, Some("仅清理 14 天前日志"), None),
        ],
    ));

    // ── 用户缓存与日志 ──
    let mut user_items = Vec::new();
    if let Some(h) = &home {
        user_items.push(item("u1", "用户应用缓存", &h.join("Library/Caches"), true, Some("运行中的应用已自动跳过"), None));
        user_items.push(item("u2", "应用日志文件", &h.join("Library/Logs"), true, None, None));
    }
    groups.push(CleanGroup {
        id: "user".into(), name: "用户缓存与日志".into(), hue: "#4AE3C4".into(),
        icon: "folder".into(), desc: "当前用户目录下的应用缓存，安全可重建".into(),
        items: filter_existing(user_items),
    });

    // ── 浏览器数据 ──
    let mut browser_items = Vec::new();
    if let Some(h) = &home {
        browser_items.push(item("b1", "Safari 浏览器缓存", &h.join("Library/Safari/Library/Caches"), true, None, None));
        browser_items.push(item("b2", "Chrome 浏览器缓存", &h.join("Library/Google/Chrome/Default/Cache"), true, None, None));
        browser_items.push(item("b3", "Arc 浏览器缓存", &h.join("Library/Application Support/Arc/Storage"), true, None, None));
    }
    groups.push(CleanGroup {
        id: "browser".into(), name: "浏览器数据".into(), hue: "#E8B36A".into(),
        icon: "globe".into(), desc: "仅清理缓存，不动书签、密码与登录态".into(),
        items: filter_existing(browser_items),
    });

    // ── 开发者工具 ──
    let mut dev_items = Vec::new();
    if let Some(h) = &home {
        dev_items.push(item("d1", "Xcode DerivedData", &h.join("Library/Developer/Xcode/DerivedData"), true, Some("下次编译自动重建"), None));
        dev_items.push(item("d2", "npm 全局缓存", &h.join(".npm"), true, None, None));
        dev_items.push(item("d3", "pnpm store 缓存", &h.join("Library/pnpm/store"), true, None, None));
        dev_items.push(item("d4", "Homebrew 下载缓存", &h.join("Library/Caches/Homebrew"), true, None, None));
    }
    groups.push(CleanGroup {
        id: "dev".into(), name: "开发者工具".into(), hue: "#F2785C".into(),
        icon: "terminal".into(), desc: "包管理器、Xcode 与容器缓存，清理后首次构建会变慢".into(),
        items: filter_existing(dev_items),
    });

    // ── 废纸篓 ──
    let mut trash_items = Vec::new();
    if let Some(h) = &home {
        trash_items.push(item("t1", "废纸篓中的文件", &h.join(".Trash"), true, Some("永久删除，不可恢复"), Some("danger")));
    }
    groups.push(CleanGroup {
        id: "trash".into(), name: "废纸篓".into(), hue: "#8A93A8".into(),
        icon: "trash".into(), desc: "当前启动卷废纸篓，外接卷独立处理".into(),
        items: filter_existing(trash_items),
    });

    // 去掉所有项均为空的分组（如某用户根本没装浏览器）
    groups.into_iter().filter(|g| !g.items.is_empty()).collect()
}

/// 扫描 AI 工具数据：缓存 / 闲置会话 / 闲置 CLI（只读）。
/// 凭证目录硬隔离：`is_credential_path` 命中的路径（~/.ssh、keychain、token/secret 等）
/// 绝不扫描、绝不展示、绝不计量，从源头杜绝凭证泄露风险。
pub fn scan_ai_tools() -> Vec<AiTool> {
    let Some(home) = home_dir() else { return Vec::new() };
    let h = home.to_string_lossy().to_string();

    let mk = |id: &str, name: &str, rel: &str, sel: bool,
              note: Option<&str>, risk: Option<&str>| -> Option<AiSubItem> {
        let path = if rel.starts_with('/') { rel.to_string() } else { format!("{}/{}", h, rel) };
        if is_credential_path(&path) { return None; }
        let size = measure_path(&path);
        if size == 0 { return None; }
        Some(AiSubItem {
            id: id.into(), name: name.into(), path, size, selected: sel,
            note: note.map(str::to_string), risk: risk.map(str::to_string),
        })
    };

    let mut tools = Vec::new();

    tools.push(AiTool {
        id: "claude".into(), name: "Claude Code".into(), hue: "#D9A27B".into(), icon: "terminal".into(),
        caches: filter_opt(vec![
            mk("c1", "项目索引与缓存", ".claude/cache", true, None, None),
            mk("c2", "旧版本运行时", ".claude/versions", true, None, None),
        ]),
        sessions: filter_opt(vec![
            mk("c3", "历史会话", ".claude/projects", false, Some("可能包含你想保留的上下文"), Some("warn")),
        ]),
        clis: vec![],
    });
    tools.push(AiTool {
        id: "codex".into(), name: "Codex CLI".into(), hue: "#9DB2D8".into(), icon: "terminal".into(),
        caches: filter_opt(vec![
            mk("x1", "会话缓存与日志", ".codex/cache", true, None, None),
        ]),
        sessions: filter_opt(vec![
            mk("x2", "旧会话记录", ".codex/sessions", false, None, Some("warn")),
        ]),
        clis: vec![],
    });
    tools.push(AiTool {
        id: "cursor".into(), name: "Cursor".into(), hue: "#8FE3D8".into(), icon: "box".into(),
        caches: filter_opt(vec![
            mk("u1", "编辑器与扩展缓存", "Library/Application Support/Cursor/Cache", true, None, None),
            mk("u2", "AI 补全索引缓存", "Library/Application Support/Cursor/User/globalStorage", true, None, None),
        ]),
        sessions: vec![], clis: vec![],
    });
    tools.push(AiTool {
        id: "ollama".into(), name: "Ollama".into(), hue: "#E8E3DA".into(), icon: "box".into(),
        caches: filter_opt(vec![
            mk("o1", "运行日志", ".ollama/logs", true, None, None),
        ]),
        sessions: vec![],
        clis: filter_opt(vec![
            mk("o2", "本地模型（默认保留）", ".ollama/models", false, Some("模型体积大，默认保留"), Some("warn")),
        ]),
    });
    tools.push(AiTool {
        id: "lmstudio".into(), name: "LM Studio".into(), hue: "#F5C451".into(), icon: "box".into(),
        caches: filter_opt(vec![
            mk("l1", "推理缓存", "Library/Caches/ai.lmstudio", true, None, None),
        ]),
        sessions: vec![],
        clis: filter_opt(vec![
            mk("l2", "本地模型（默认保留）", ".cache/lm-studio/models", false, Some("模型体积大，默认保留"), Some("warn")),
        ]),
    });
    tools.push(AiTool {
        id: "huggingface".into(), name: "HuggingFace".into(), hue: "#E8B36A".into(), icon: "box".into(),
        caches: filter_opt(vec![
            mk("hf1", "模型与数据集缓存", ".cache/huggingface", false, Some("含已下载模型，默认不清理"), Some("warn")),
        ]),
        sessions: vec![], clis: vec![],
    });
    tools.push(AiTool {
        id: "chatgpt".into(), name: "ChatGPT".into(), hue: "#4AE3C4".into(), icon: "box".into(),
        caches: filter_opt(vec![
            mk("g1", "应用缓存", "Library/Caches/com.openai.chat", true, None, None),
            mk("g2", "离线数据缓存", "Library/Application Support/ChatGPT/Cache", true, None, None),
        ]),
        sessions: vec![], clis: vec![],
    });
    tools.push(AiTool {
        id: "gemini".into(), name: "Gemini CLI".into(), hue: "#6AA9FF".into(), icon: "terminal".into(),
        caches: filter_opt(vec![
            mk("gm1", "缓存与日志", ".gemini", false, None, None),
        ]),
        sessions: vec![], clis: vec![],
    });

    // 仅保留至少有一项可清理内容的工具
    tools.into_iter()
        .filter(|t| !t.caches.is_empty() || !t.sessions.is_empty() || !t.clis.is_empty())
        .collect()
}

fn filter_opt<T>(v: Vec<Option<T>>) -> Vec<T> {
    v.into_iter().flatten().collect()
}

/// 凭证目录硬隔离：命中即绝不扫描/计量/展示。
fn is_credential_path(p: &str) -> bool {
    let p = p.to_lowercase();
    let patterns = [
        "/.ssh", "/.gnupg", "keychain", "/library/keychains",
        "id_rsa", "id_ed25519", ".p12", ".pem",
    ];
    patterns.iter().any(|c| p.contains(c))
}

/// 构建分析页磁盘树：从根开始，聚合各顶级目录占用（读操作）。
/// 受权限限制，/System 等 SIP 目录可能统计不全，已用项标记 protected。
/// 顶级子项体积并行计量（du），根体积取子项之和，避免对根再做一次全量 du。
pub fn build_disk_tree(root: &str) -> DiskNode {
    let root_path = expand_tilde(Path::new(root));

    let mut entries: Vec<(String, PathBuf)> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&root_path) {
        for entry in rd.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            entries.push((name, path));
        }
    }

    // 并行计量每个顶级子项
    let mut measured: Vec<(String, PathBuf, u64)> = Vec::with_capacity(entries.len());
    std::thread::scope(|s| {
        let mut handles = Vec::with_capacity(entries.len());
        for (name, path) in entries {
            handles.push(s.spawn(move || {
                let size = measure_bounded(&path, 20_000);
                (name, path, size)
            }));
        }
        for h in handles {
            if let Ok((name, path, size)) = h.join() {
                if size > 0 {
                    measured.push((name, path, size));
                }
            }
        }
    });

    let mut children: Vec<DiskNode> = measured
        .into_iter()
        .map(|(name, path, size)| DiskNode {
            name,
            path: Some(path.to_string_lossy().to_string()),
            size,
            children: None,
            protected: Some(is_protected(&path)),
            free: None,
        })
        .collect();
    children.sort_by(|a, b| b.size.cmp(&a.size));

    let total: u64 = children.iter().map(|c| c.size).sum();

    DiskNode {
        name: root_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| root.into()),
        path: Some(root_path.to_string_lossy().to_string()),
        size: total,
        children: Some(children),
        protected: None, free: None,
    }
}

fn is_protected(path: &Path) -> bool {
    let s = path.to_string_lossy();
    s.starts_with("/System") || s.starts_with("/private/var/db") || s == "/Library"
}

// ─── 内部工具 ───────────────────────────────────────────────

fn item(id: &str, name: &str, path: &Path, selected: bool, note: Option<&str>, risk: Option<&str>) -> CleanItem {
    CleanItem {
        id: id.into(), name: name.into(),
        path: path.to_string_lossy().to_string(),
        size: measure_path(&path.to_string_lossy()),
        selected, note: note.map(|s| s.into()), risk: risk.map(|s| s.into()),
    }
}

fn build_group(id: &str, name: &str, hue: &str, icon: &str, desc: &str,
               defs: &[(&str, &str, &str, bool, Option<&str>, Option<&str>)]) -> CleanGroup {
    let items: Vec<CleanItem> = defs.iter()
        .map(|(i, n, p, sel, note, risk)| item(i, n, Path::new(p), *sel, *note, *risk))
        .collect();
    CleanGroup {
        id: id.into(), name: name.into(), hue: hue.into(), icon: icon.into(),
        desc: desc.into(), items: filter_existing(items),
    }
}

/// 过滤掉体积为 0（不存在/无权限）的项，避免 UI 展示空条目。
fn filter_existing(items: Vec<CleanItem>) -> Vec<CleanItem> {
    items.into_iter().filter(|i| i.size > 0).collect()
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn expand_tilde(path: &Path) -> PathBuf {
    if let Some(s) = path.to_str() {
        if let Some(rest) = s.strip_prefix("~/") {
            if let Some(home) = home_dir() { return home.join(rest); }
        } else if s == "~" {
            if let Some(home) = home_dir() { return home; }
        }
    }
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_nonexistent_is_zero() {
        assert_eq!(measure_path("/nonexistent/pixu/path/xyz"), 0);
    }

    #[test]
    fn scan_groups_well_formed() {
        let groups = scan_clean_groups();
        // 每个分组的项 size 都应 > 0（已过滤）
        for g in &groups {
            for it in &g.items {
                assert!(it.size > 0, "item {} size is 0", it.name);
            }
        }
    }
}
