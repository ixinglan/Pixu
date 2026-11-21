//! Pixu 卸载领域层（domain-uninstall）：App bundle 元数据读取与残留发现。
//!
//! 职责：/Applications bundle 元数据读取、应用残留规则库（首版精选 50 款）、
//! 个人数据与可删残留的分类模型。
//! 本 crate 仅执行「读」操作：读取 plist、统计体积、探测残留路径是否存在，
//! 不执行卸载/删除（卸载动作由 fs-ops 执行，受写开关控制）。

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use pixu_app_service::{AppEntry, Residual};
use pixu_domain_scan::measure_path;

const PALETTE: &[&str] = &[
    "#F2785C", "#7DD3FC", "#E8E3DA", "#C48BD6", "#6AA9FF",
    "#5DD39E", "#F5C451", "#D9A27B", "#4AE3C4", "#9DB2D8",
    "#8FE3D8", "#E8B36A", "#8A93A8", "#f472b6", "#a78bfa",
];

/// 扫描 /Applications 下所有 .app，解析元数据并发现残留（只读）。
pub fn scan_apps() -> Vec<AppEntry> {
    let mut paths: Vec<PathBuf> = Vec::new();

    let collect = |dir: &Path, into: &mut Vec<PathBuf>| {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for entry in rd.flatten() {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) == Some("app") {
                    into.push(p);
                }
            }
        }
    };
    collect(Path::new("/Applications"), &mut paths);
    if let Some(home) = home_dir() {
        collect(&home.join("Applications"), &mut paths);
    }

    // 并行构建：每个 .app 的 plist 解析 + du 计量彼此独立，并行可大幅缩短总耗时
    let mut apps: Vec<AppEntry> = Vec::with_capacity(paths.len());
    std::thread::scope(|s| {
        let mut handles = Vec::with_capacity(paths.len());
        for p in paths {
            handles.push(s.spawn(move || build_app_entry(&p)));
        }
        for h in handles {
            if let Ok(Some(app)) = h.join() {
                apps.push(app);
            }
        }
    });

    apps.sort_by(|a, b| b.size.cmp(&a.size));
    apps
}

fn build_app_entry(app_path: &Path) -> Option<AppEntry> {
    let info = read_info_plist(app_path)?;
    let name = info.name;
    let bundle_id = info.bundle_id.clone();
    let size = measure_path(&app_path.to_string_lossy());
    if size == 0 {
        return None;
    }
    let last_used = last_used_label(app_path);
    let initial = name.chars().next().unwrap_or('?').to_string();
    let hue = hue_for(&name);

    let mut residuals = Vec::new();
    // 应用容器本身
    residuals.push(Residual {
        name: "应用容器".into(),
        path: app_path.to_string_lossy().to_string(),
        size, kind: "app".into(), keep: None, note: None,
    });
    // 通用派生残留
    residuals.extend(derive_residuals(&name, &bundle_id));
    // 精选规则残留
    residuals.extend(curated_residuals(&name, &bundle_id));
    // 过滤不存在/0 体积的残留
    residuals = residuals.into_iter().filter(|r| r.size > 0 || r.kind == "app").collect();

    Some(AppEntry {
        id: bundle_id.clone(),
        name, developer: info.developer, size, last_used, hue, initial, residuals,
    })
}

struct PlistInfo {
    name: String,
    bundle_id: String,
    developer: String,
}

fn read_info_plist(app_path: &Path) -> Option<PlistInfo> {
    let plist_path = app_path.join("Contents/Info.plist");
    let value: plist::Value = plist::from_file(&plist_path).ok()?;
    let dict = value.as_dictionary()?;
    let get = |k: &str| -> Option<String> {
        dict.get(k).and_then(|v| v.as_string()).map(|s| s.to_string())
    };
    let name = get("CFBundleName")
        .or_else(|| get("CFBundleDisplayName"))
        .or_else(|| app_path.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string()))
        .unwrap_or_else(|| "未知应用".into());
    let bundle_id = get("CFBundleIdentifier").unwrap_or_else(|| format!("unknown.{}", name));
    let developer = get("CFBundleGetInfoString")
        .or_else(|| get("NSHumanReadableCopyright"))
        .unwrap_or_else(|| "未知开发者".into());
    Some(PlistInfo { name, bundle_id, developer })
}

/// 由 bundle id / 名称派生通用残留路径（Preferences / Caches / Application Support / Containers）。
fn derive_residuals(name: &str, bundle_id: &str) -> Vec<Residual> {
    let mut out = Vec::new();
    if let Some(home) = home_dir() {
        let candidates: Vec<(&str, PathBuf, &str, bool)> = vec![
            ("偏好设置", home.join("Library/Preferences").join(format!("{}.plist", bundle_id)), "pref", false),
            ("缓存目录", home.join("Library/Caches").join(bundle_id), "cache", false),
            ("HTTP 存储", home.join("Library/Caches").join(format!("{}.os", bundle_id)), "cache", false),
            ("Application Support", home.join("Library/Application Support").join(name), "data", true),
            ("容器数据", home.join("Library/Containers").join(bundle_id), "data", true),
            ("保存的状态", home.join("Library/Saved Application State").join(format!("{}.savedState", bundle_id)), "cache", false),
        ];
        for (label, path, kind, keep) in candidates {
            let size = measure_path(&path.to_string_lossy());
            if size > 0 {
                out.push(Residual {
                    name: label.into(), path: path.to_string_lossy().to_string(),
                    size, kind: kind.into(), keep: if keep { Some(true) } else { None }, note: None,
                });
            }
        }
    }
    out
}

/// 精选残留规则库（首版 50 款）：针对知名应用补充其特有的数据目录。
/// 仅当对应路径真实存在且体积 > 0 时才返回。
fn curated_residuals(name: &str, bundle_id: &str) -> Vec<Residual> {
    let n = name.to_lowercase();
    let bid = bundle_id.to_lowercase();
    let mut out = Vec::new();
    if let Some(home) = home_dir() {
        // (匹配关键词, 残留名, 路径, kind, keep, note)
        let rules: Vec<(&str, &str, PathBuf, &str, bool, Option<&str>)> = vec![
            // 浏览器
            ("chrome", "用户数据", home.join("Library/Application Support/Google/Chrome"), "data", false, None),
            ("firefox", "配置与扩展", home.join("Library/Application Support/Firefox"), "data", false, None),
            ("edge", "用户数据", home.join("Library/Application Support/Microsoft Edge"), "data", false, None),
            ("arc", "本地数据", home.join("Library/Application Support/Arc"), "data", false, None),
            // 开发工具
            ("xcode", "模拟器数据", home.join("Library/Developer/CoreSimulator"), "data", false, None),
            ("xcode", "归档与设备支持", home.join("Library/Developer/Xcode/Archives"), "data", false, None),
            ("visual studio code", "扩展与配置", home.join(".vscode"), "data", true, None),
            ("docker", "镜像与卷", home.join("Library/Containers/com.docker.docker"), "data", true, Some("含容器卷，默认保留")),
            ("docker", "镜像数据", home.join(".docker"), "data", false, None),
            ("iterm", "偏好与配置", home.join("Library/Application Support/iTerm2"), "data", true, None),
            // AI / 模型
            ("ollama", "本地模型", home.join(".ollama/models"), "data", true, Some("模型体积大，默认保留")),
            ("ollama", "运行日志", home.join(".ollama/logs"), "cache", false, None),
            ("lm studio", "模型目录", home.join(".cache/lm-studio/models"), "data", true, None),
            ("huggingface", "模型缓存", home.join(".cache/huggingface"), "data", false, None),
            ("cursor", "用户数据", home.join("Library/Application Support/Cursor/User"), "data", true, None),
            // 通讯协作
            ("slack", "工作区缓存", home.join("Library/Containers/com.tinyspeck.slackmacgap"), "cache", false, None),
            ("notion", "离线数据", home.join("Library/Application Support/Notion"), "data", true, None),
            ("discord", "缓存", home.join("Library/Application Support/discord"), "cache", false, None),
            ("telegram", "聊天数据", home.join("Library/Group Containers/*.ru.keepcoder"), "data", true, None),
            ("wechat", "聊天记录", home.join("Library/Containers/com.tencent.xinWeChat"), "data", true, Some("聊天记录，默认保留")),
            ("feishu", "本地数据", home.join("Library/Application Support/LarkShell"), "data", false, None),
            // 媒体
            ("spotify", "流媒体缓存", home.join("Library/Caches/com.spotify.client"), "cache", false, None),
            ("网易云音乐", "缓存与下载", home.join("Library/Containers/com.netease.163music"), "cache", false, None),
            ("vlc", "配置", home.join("Library/Application Support/org.videolan.vlc"), "pref", false, None),
            // 设计创意
            ("figma", "桌面数据", home.join("Library/Application Support/Figma"), "cache", false, None),
            ("photoshop", "媒体缓存", home.join("Library/Caches/Adobe/Photoshop"), "cache", false, None),
            ("sketch", "插件与缓存", home.join("Library/Application Support/com.bohemiancoding.sketch3"), "data", false, None),
            // 笔记阅读
            ("obsidian", "工作区数据", home.join(".obsidian"), "data", true, None),
            ("logseq", "图谱数据", home.join(".logseq"), "data", true, None),
            ("zotero", "文献库", home.join("Zotero"), "data", true, None),
            // 实用工具
            ("1password", "数据目录", home.join("Library/Group Containers/2BUA8C4S2C.com.1password"), "data", true, None),
            ("alfred", "工作流与缓存", home.join("Library/Application Support/Alfred"), "data", false, None),
            ("raycast", "扩展与配置", home.join("Library/Application Support/com.raycast.macos"), "data", false, None),
            ("cleanmymac", "缓存", home.join("Library/Caches/com.macpaw.CleanMyMac"), "cache", false, None),
            ("the unarchiver", "临时文件", home.join("Library/Caches/cx.c3.theunarchiver"), "cache", false, None),
            // 办公
            ("microsoft word", "恢复文档", home.join("Library/Containers/com.microsoft.Word/Data/Documents"), "data", false, None),
            ("microsoft excel", "恢复文档", home.join("Library/Containers/com.microsoft.Excel/Data"), "data", false, None),
            ("wps", "备份与缓存", home.join("Library/Containers/com.kingsoft.wpsoffice.mac"), "cache", false, None),
            // 游戏平台
            ("steam", "游戏库", home.join("Library/Application Support/Steam/steamapps"), "data", true, Some("含已安装游戏，默认保留")),
            ("epic games", "游戏缓存", home.join("Library/Caches/com.epicgames.EpicGamesLauncher"), "cache", false, None),
            // 系统扩展
            ("istat menus", "配置", home.join("Library/Application Support/iStat Menus"), "pref", false, None),
            ("bartender", "配置", home.join("Library/Application Support/Bartender"), "pref", false, None),
            ("karabiner", "配置", home.join(".config/karabiner"), "pref", true, None),
            // 开发语言运行时
            ("go", "模块缓存", home.join("go/pkg/mod"), "cache", false, None),
            ("rust", "cargo 注册表", home.join(".cargo/registry"), "cache", false, None),
            ("node", "npm 缓存", home.join(".npm"), "cache", false, None),
            ("python", "pip 缓存", home.join("Library/Caches/pip"), "cache", false, None),
            ("homebrew", "下载缓存", home.join("Library/Caches/Homebrew"), "cache", false, None),
        ];
        for (keyword, label, path, kind, keep, note) in rules {
            if !n.contains(keyword) && !bid.contains(keyword) { continue; }
            let size = measure_path(&path.to_string_lossy());
            if size > 0 {
                out.push(Residual {
                    name: label.into(), path: path.to_string_lossy().to_string(),
                    size, kind: kind.into(), keep: if keep { Some(true) } else { None },
                    note: note.map(|s| s.into()),
                });
            }
        }
    }
    out
}

fn last_used_label(app_path: &Path) -> String {
    let meta = match std::fs::metadata(app_path) { Ok(m) => m, Err(_) => return "未知".into() };
    let mtime = meta.modified().unwrap_or(UNIX_EPOCH);
    let now = SystemTime::now();
    let days = now.duration_since(mtime)
        .map(|d| d.as_secs() / 86400)
        .unwrap_or(0);
    match days {
        0 => "今天".into(),
        1 => "昨天".into(),
        d if d < 30 => format!("{} 天前", d),
        d if d < 365 => format!("{} 个月前", d / 30),
        d => format!("{} 年前", d / 365),
    }
}

fn hue_for(name: &str) -> String {
    let mut h = DefaultHasher::new();
    name.hash(&mut h);
    let idx = (h.finish() as usize) % PALETTE.len();
    PALETTE[idx].to_string()
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hue_is_stable() {
        assert_eq!(hue_for("Figma"), hue_for("Figma"));
        assert_ne!(hue_for("Figma"), hue_for("Chrome"));
    }

    #[test]
    #[ignore = "heavy: scans all of /Applications with du; run with --ignored"]
    fn reads_installed_apps() {
        // /Applications 在 macOS 一定存在，至少能扫到系统自带 App
        let apps = scan_apps();
        // 不断言具体数量，只确保不 panic 且返回的条目结构合法
        for a in &apps {
            assert!(!a.id.is_empty());
            assert!(!a.name.is_empty());
        }
    }
}
