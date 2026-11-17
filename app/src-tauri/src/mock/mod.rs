//! Mock 服务实现：全量假数据，形状与原型一致。
//! M0 期间作为唯一实现注入 Tauri state，前端经 invoke 读取。

use pixu_app_service::*;

// ─── 工具：构造字符串与可选字段 ─────────────────────────────

fn s(v: &str) -> String { v.to_string() }
fn some(v: &str) -> Option<String> { Some(v.to_string()) }

// ─── CleanGroup 数据 ────────────────────────────────────────

fn mock_clean_groups() -> Vec<CleanGroup> {
    vec![
        CleanGroup {
            id: s("system"), name: s("系统数据"), hue: s("#9DB2D8"), icon: s("drive"),
            desc: s("系统缓存、日志与本地快照，均为可再生数据"),
            items: vec![
                CleanItem { id: s("s1"), name: s("系统缓存文件"), path: s("/Library/Caches（系统级）"), size: 4_820_000_000, selected: true, note: some("应用可自动重建"), risk: None },
                CleanItem { id: s("s2"), name: s("系统日志归档"), path: s("/private/var/log"), size: 1_240_000_000, selected: true, note: some("仅清理 14 天前日志"), risk: None },
                CleanItem { id: s("s3"), name: s("Time Machine 本地快照"), path: s("/Volumes/com.apple.TimeMachine"), size: 12_600_000_000, selected: false, note: some("删除后无法回滚到旧快照"), risk: some("warn") },
            ],
        },
        CleanGroup {
            id: s("user"), name: s("用户缓存与日志"), hue: s("#4AE3C4"), icon: s("folder"),
            desc: s("当前用户目录下的应用缓存，安全可重建"),
            items: vec![
                CleanItem { id: s("u1"), name: s("用户应用缓存"), path: s("~/Library/Caches"), size: 9_360_000_000, selected: true, note: some("运行中的应用已自动跳过"), risk: None },
                CleanItem { id: s("u2"), name: s("应用日志文件"), path: s("~/Library/Logs"), size: 780_000_000, selected: true, note: None, risk: None },
                CleanItem { id: s("u3"), name: s("临时下载残留"), path: s("~/Library/Application Support/.../tmp"), size: 1_520_000_000, selected: true, note: some("下载中的文件受保护"), risk: None },
            ],
        },
        CleanGroup {
            id: s("browser"), name: s("浏览器数据"), hue: s("#E8B36A"), icon: s("globe"),
            desc: s("仅清理缓存，不动书签、密码与登录态"),
            items: vec![
                CleanItem { id: s("b1"), name: s("Safari 浏览器缓存"), path: s("~/Library/Safari/Library/Caches"), size: 2_140_000_000, selected: true, note: None, risk: None },
                CleanItem { id: s("b2"), name: s("Chrome 浏览器缓存"), path: s("~/Library/Google/Chrome/.../Cache"), size: 3_480_000_000, selected: true, note: None, risk: None },
                CleanItem { id: s("b3"), name: s("Arc 浏览器缓存"), path: s("~/Library/Arc/.../Cache"), size: 1_260_000_000, selected: true, note: None, risk: None },
            ],
        },
        CleanGroup {
            id: s("dev"), name: s("开发者工具"), hue: s("#F2785C"), icon: s("terminal"),
            desc: s("包管理器、Xcode 与容器缓存，清理后首次构建会变慢"),
            items: vec![
                CleanItem { id: s("d1"), name: s("Xcode DerivedData"), path: s("~/Library/Developer/Xcode/DerivedData"), size: 18_400_000_000, selected: true, note: some("下次编译自动重建"), risk: None },
                CleanItem { id: s("d2"), name: s("npm / pnpm 全局缓存"), path: s("~/.npm、~/Library/pnpm/store"), size: 6_720_000_000, selected: true, note: None, risk: None },
                CleanItem { id: s("d3"), name: s("Homebrew 下载缓存"), path: s("~/Library/Caches/Homebrew"), size: 3_180_000_000, selected: true, note: None, risk: None },
                CleanItem { id: s("d4"), name: s("Docker 悬空镜像层"), path: s("docker system df（dangling）"), size: 8_940_000_000, selected: false, risk: some("warn"), note: some("正在运行的容器不受影响") },
            ],
        },
        CleanGroup {
            id: s("trash"), name: s("废纸篓"), hue: s("#8A93A8"), icon: s("trash"),
            desc: s("当前启动卷废纸篓，外接卷独立处理"),
            items: vec![
                CleanItem { id: s("t1"), name: s("废纸篓中的文件（42 项）"), path: s("~/.Trash"), size: 5_260_000_000, selected: true, note: some("永久删除，不可恢复"), risk: some("danger") },
            ],
        },
    ]
}

// ─── AiTool 数据 ────────────────────────────────────────────

fn ai_sub(id: &str, name: &str, path: &str, size: u64, selected: bool, note: Option<String>, risk: Option<String>) -> AiSubItem {
    AiSubItem { id: s(id), name: s(name), path: s(path), size, selected, note, risk }
}

fn mock_ai_tools() -> Vec<AiTool> {
    vec![
        AiTool {
            id: s("claude"), name: s("Claude Code"), hue: s("#D9A27B"), icon: s("terminal"),
            caches: vec![
                ai_sub("c1", "项目索引与缓存", "~/.claude/cache", 1_840_000_000, true, None, None),
                ai_sub("c2", "旧版本运行时", "~/.claude/versions", 620_000_000, true, None, None),
            ],
            sessions: vec![
                ai_sub("c3", "90 天未打开的会话（12 个）", "~/.claude/projects/...", 480_000_000, false, some("可能包含你想保留的上下文"), some("warn")),
            ],
            clis: vec![],
        },
        AiTool {
            id: s("codex"), name: s("Codex CLI"), hue: s("#9DB2D8"), icon: s("terminal"),
            caches: vec![
                ai_sub("x1", "会话缓存与日志", "~/.codex/cache", 960_000_000, true, None, None),
            ],
            sessions: vec![
                ai_sub("x2", "旧会话记录（8 个）", "~/.codex/sessions", 210_000_000, false, None, some("warn")),
                ai_sub("x3", "可清理的 git worktree（3 个）", "~/code/.worktrees", 1_340_000_000, false, some("含未提交改动时自动跳过"), some("warn")),
            ],
            clis: vec![
                ai_sub("x4", "闲置 CLI：旧版 gemini-cli", "/opt/homebrew/lib/node_modules", 180_000_000, false, some("将连同配置一并卸载"), some("danger")),
            ],
        },
        AiTool {
            id: s("cursor"), name: s("Cursor"), hue: s("#8FE3D8"), icon: s("box"),
            caches: vec![
                ai_sub("u1c", "编辑器与扩展缓存", "~/Library/Application Support/Cursor/Cache", 2_260_000_000, true, None, None),
                ai_sub("u2c", "AI 补全索引缓存", "~/Library/Application Support/Cursor/.../index", 1_580_000_000, true, None, None),
            ],
            sessions: vec![],
            clis: vec![],
        },
    ]
}

// ─── AppEntry 数据 ──────────────────────────────────────────

fn res(name: &str, path: &str, size: u64, kind: &str, keep: Option<bool>, note: Option<String>) -> Residual {
    Residual { name: s(name), path: s(path), size, kind: s(kind), keep, note }
}

fn mock_apps() -> Vec<AppEntry> {
    vec![
        AppEntry { id: s("a1"), name: s("Figma"), developer: s("Figma Inc."), size: 684_000_000, last_used: s("今天 10:24"), hue: s("#F2785C"), initial: s("F"),
            residuals: vec![
                res("应用容器", "/Applications/Figma.app", 512_000_000, "app", None, None),
                res("缓存目录", "~/Library/Caches/Figma", 138_000_000, "cache", None, None),
                res("偏好设置", "~/Library/Preferences/com.figma.Desktop.plist", 240_000, "pref", None, None),
                res("Application Support", "~/Library/Application Support/Figma", 33_800_000, "data", Some(true), None),
            ] },
        AppEntry { id: s("a2"), name: s("Docker Desktop"), developer: s("Docker Inc."), size: 2_140_000_000, last_used: s("昨天 21:08"), hue: s("#7DD3FC"), initial: s("D"),
            residuals: vec![
                res("应用容器", "/Applications/Docker.app", 1_180_000_000, "app", None, None),
                res("镜像与卷数据", "~/Library/Containers/com.docker.docker", 880_000_000, "data", Some(true), some("含你的容器卷，默认保留")),
                res("缓存与日志", "~/Library/Caches/com.docker.docker", 80_000_000, "cache", None, None),
            ] },
        AppEntry { id: s("a3"), name: s("Notion"), developer: s("Notion Labs"), size: 426_000_000, last_used: s("2 天前"), hue: s("#E8E3DA"), initial: s("N"),
            residuals: vec![
                res("应用容器", "/Applications/Notion.app", 318_000_000, "app", None, None),
                res("离线数据", "~/Library/Application Support/Notion", 92_000_000, "data", Some(true), None),
                res("缓存", "~/Library/Caches/notion.id", 16_000_000, "cache", None, None),
            ] },
        AppEntry { id: s("a4"), name: s("Slack"), developer: s("Salesforce"), size: 512_000_000, last_used: s("3 天前"), hue: s("#C48BD6"), initial: s("S"),
            residuals: vec![
                res("应用容器", "/Applications/Slack.app", 388_000_000, "app", None, None),
                res("工作区缓存", "~/Library/Containers/com.tinyspeck.slackmacgap", 118_000_000, "cache", None, None),
                res("偏好设置", "~/Library/Preferences/com.tinyspeck.slackmacgap.plist", 180_000, "pref", None, None),
            ] },
        AppEntry { id: s("a5"), name: s("Ollama"), developer: s("Ollama Inc."), size: 4_860_000_000, last_used: s("5 天前"), hue: s("#E8E3DA"), initial: s("O"),
            residuals: vec![
                res("应用容器", "/Applications/Ollama.app", 240_000_000, "app", None, None),
                res("本地模型文件", "~/.ollama/models", 4_580_000_000, "data", Some(true), some("模型体积大，默认保留")),
                res("运行日志", "~/.ollama/logs", 40_000_000, "cache", None, None),
            ] },
        AppEntry { id: s("a6"), name: s("Zoom"), developer: s("Zoom Video"), size: 366_000_000, last_used: s("2 周前"), hue: s("#6AA9FF"), initial: s("Z"),
            residuals: vec![
                res("应用容器", "/Applications/zoom.us.app", 282_000_000, "app", None, None),
                res("会议缓存", "~/Library/Caches/us.zoom.xos", 74_000_000, "cache", None, None),
                res("后台守护项", "~/Library/LaunchAgents/us.zoom.xos.plist", 12_000, "daemon", None, None),
            ] },
        AppEntry { id: s("a7"), name: s("Spotify"), developer: s("Spotify AB"), size: 318_000_000, last_used: s("1 个月前"), hue: s("#5DD39E"), initial: s("S"),
            residuals: vec![
                res("应用容器", "/Applications/Spotify.app", 244_000_000, "app", None, None),
                res("流媒体缓存", "~/Library/Caches/com.spotify.client", 66_000_000, "cache", None, None),
                res("偏好设置", "~/Library/Preferences/com.spotify.client.plist", 96_000, "pref", None, None),
            ] },
        AppEntry { id: s("a8"), name: s("LM Studio"), developer: s("LM Studio"), size: 1_240_000_000, last_used: s("2 个月前"), hue: s("#F5C451"), initial: s("L"),
            residuals: vec![
                res("应用容器", "/Applications/LM Studio.app", 486_000_000, "app", None, None),
                res("模型目录", "~/.cache/lm-studio/models", 720_000_000, "data", Some(true), None),
                res("推理缓存", "~/Library/Caches/ai.lmstudio", 34_000_000, "cache", None, None),
            ] },
    ]
}

// ─── DiskNode 数据 ──────────────────────────────────────────

fn leaf(name: &str, size: u64) -> DiskNode {
    DiskNode { name: s(name), path: None, size, children: None, protected: None, free: None }
}

fn node(name: &str, path: Option<String>, size: u64, children: Vec<DiskNode>) -> DiskNode {
    DiskNode { name: s(name), path, size, children: Some(children), protected: None, free: None }
}

fn mock_disk_tree() -> DiskNode {
    DiskNode {
        name: s("Macintosh HD"), path: some("/"), size: 412_000_000_000,
        children: Some(vec![
            node("用户目录", some("/Users/ixiaoqiang"), 268_000_000_000, vec![
                node("资源库", some("~/Library"), 124_000_000_000, vec![
                    leaf("Developer", 48_600_000_000),
                    leaf("Caches", 31_200_000_000),
                    leaf("Containers", 22_800_000_000),
                    leaf("Application Support", 18_400_000_000),
                    leaf("其他", 3_000_000_000),
                ]),
                node("项目代码", some("~/Documents/TreaProject"), 62_400_000_000, vec![
                    leaf("node_modules 集合", 28_900_000_000),
                    leaf("target 构建产物", 14_600_000_000),
                    leaf("源码与资源", 18_900_000_000),
                ]),
                node("下载", some("~/Downloads"), 41_800_000_000, vec![
                    leaf("安装包（12 个）", 18_200_000_000),
                    leaf("视频素材", 15_600_000_000),
                    leaf("其他文件", 8_000_000_000),
                ]),
                leaf("影片", 24_600_000_000),
                leaf("图片", 15_200_000_000),
            ]),
            node("应用程序", some("/Applications"), 78_400_000_000, vec![
                leaf("Xcode", 32_800_000_000),
                leaf("其他应用（64 个）", 45_600_000_000),
            ]),
            DiskNode { name: s("系统与库"), path: some("/System"), size: 48_200_000_000, children: None, protected: Some(true), free: None },
            DiskNode { name: s("可用空间"), path: None, size: 17_400_000_000, children: None, protected: None, free: Some(true) },
        ]),
        protected: None, free: None,
    }
}

// ─── Metrics 数据 ───────────────────────────────────────────

fn spark(base: u32, n: usize, jitter: u32) -> Vec<u32> {
    (0..n).map(|i| {
        let v = base as f64 + ((i as f64) / 2.4).sin() * jitter as f64 + ((i % 5) * 2) as f64;
        (v.max(4.0)).round() as u32
    }).collect()
}

fn mock_metrics() -> Metrics {
    Metrics {
        cpu: MetricItem { label: s("CPU"), value: Some(24), unit: s("%"), spark: spark(24, 28, 12), cores: some("M3 Pro · 11 核"), detail: None, down: None, up: None, danger: None },
        mem: MetricItem { label: s("内存"), value: Some(68), unit: s("%"), spark: spark(64, 28, 8), cores: None, detail: some("14.2 / 18 GB"), down: None, up: None, danger: None },
        gpu: MetricItem { label: s("GPU"), value: Some(12), unit: s("%"), spark: spark(14, 28, 10), cores: None, detail: None, down: None, up: None, danger: None },
        disk: MetricItem { label: s("磁盘"), value: Some(91), unit: s("%"), spark: spark(88, 28, 3), cores: None, detail: some("已用 412 GB / 460 GB"), down: None, up: None, danger: Some(true) },
        net: MetricItem { label: s("网络"), value: None, unit: s("MB/s"), spark: spark(30, 28, 22), cores: None, detail: None, down: some("8.4"), up: some("1.2"), danger: None },
        battery: MetricItem { label: s("电池"), value: Some(76), unit: s("%"), spark: vec![], cores: None, detail: some("已接电源 · 循环 84 次"), down: None, up: None, danger: None },
    }
}

// ─── ProcessInfo 数据 ───────────────────────────────────────

fn mock_processes() -> Vec<ProcessInfo> {
    vec![
        ProcessInfo { name: s("WindowServer"), cpu: 18.2, mem: 980 },
        ProcessInfo { name: s("Google Chrome 辅助进程"), cpu: 12.6, mem: 2140 },
        ProcessInfo { name: s("Xcode"), cpu: 9.8, mem: 1680 },
        ProcessInfo { name: s("Cursor"), cpu: 7.4, mem: 1240 },
        ProcessInfo { name: s("node（vite dev）"), cpu: 6.1, mem: 620 },
        ProcessInfo { name: s("Docker Desktop"), cpu: 4.2, mem: 1480 },
        ProcessInfo { name: s("Slack"), cpu: 2.8, mem: 720 },
        ProcessInfo { name: s("mds_stores（Spotlight 索引）"), cpu: 1.9, mem: 240 },
    ]
}

// ─── History / Whitelist / Settings ─────────────────────────

fn mock_history() -> Vec<HistoryEntry> {
    vec![
        HistoryEntry { id: s("h1"), time: s("2026-11-12 21:14"), kind: s("深度清理"), freed: 14_820_000_000, detail: s("用户缓存、浏览器缓存、Homebrew 缓存") },
        HistoryEntry { id: s("h2"), time: s("2026-11-09 09:02"), kind: s("应用卸载"), freed: 4_860_000_000, detail: s("Ollama（保留本地模型）") },
        HistoryEntry { id: s("h3"), time: s("2026-11-05 17:40"), kind: s("AI 数据清理"), freed: 2_340_000_000, detail: s("Claude Code 缓存、Codex 旧版本") },
        HistoryEntry { id: s("h4"), time: s("2026-10-28 11:26"), kind: s("深度清理"), freed: 9_640_000_000, detail: s("Xcode DerivedData、npm 缓存") },
    ]
}

fn mock_whitelist() -> Vec<WhitelistEntry> {
    vec![
        WhitelistEntry { id: s("w1"), name: s("1Password 数据目录"), path: s("~/Library/Group Containers/2BUA8C4S2C.com.1password") },
        WhitelistEntry { id: s("w2"), name: s("微信聊天记录"), path: s("~/Library/Containers/com.tencent.xinWeChat") },
        WhitelistEntry { id: s("w3"), name: s("JetBrains 配置"), path: s("~/Library/Application Support/JetBrains") },
    ]
}

fn mock_settings() -> Settings {
    Settings {
        fda_granted: false,
        trash_first: true,
        protect_running: true,
        zero_telemetry: true,
        update_check: true,
        ai_entry: true,
        external_volumes: false,
    }
}

// ─── Mock 服务实现 ──────────────────────────────────────────

pub struct MockScanService;
pub struct MockStatusService;
pub struct MockSettingsService;
pub struct MockCleanService;
pub struct MockUninstallService;

impl ScanService for MockScanService {
    fn scan_clean_groups(&self) -> AppResult<Vec<CleanGroup>> { Ok(mock_clean_groups()) }
    fn scan_ai_tools(&self) -> AppResult<Vec<AiTool>> { Ok(mock_ai_tools()) }
    fn list_apps(&self) -> AppResult<Vec<AppEntry>> { Ok(mock_apps()) }
    fn analyze_disk(&self) -> AppResult<DiskNode> { Ok(mock_disk_tree()) }
}

impl StatusService for MockStatusService {
    fn live_metrics(&self) -> AppResult<Metrics> { Ok(mock_metrics()) }
    fn list_processes(&self) -> AppResult<Vec<ProcessInfo>> { Ok(mock_processes()) }
}

impl SettingsService for MockSettingsService {
    fn get_history(&self) -> AppResult<Vec<HistoryEntry>> { Ok(mock_history()) }
    fn get_whitelist(&self) -> AppResult<Vec<WhitelistEntry>> { Ok(mock_whitelist()) }
    fn get_settings(&self) -> AppResult<Settings> { Ok(mock_settings()) }
}

impl CleanService for MockCleanService {
    fn preview_delete(&self, items: Vec<CleanItem>) -> AppResult<PreviewResult> {
        let total: u64 = items.iter().map(|i| i.size).sum();
        Ok(PreviewResult { count: items.len(), total })
    }
    fn execute_clean(&self, items: Vec<CleanItem>, permanent: bool) -> AppResult<CleanResult> {
        let freed: u64 = items.iter().map(|i| i.size).sum();
        Ok(CleanResult { freed, permanent, skipped: 0 })
    }
}

impl UninstallService for MockUninstallService {
    fn uninstall_app(&self, _app_id: String, _keep_data: bool) -> AppResult<UninstallResult> {
        Ok(UninstallResult { ok: true })
    }
    fn move_to_trash(&self, _path: String) -> AppResult<TrashResult> {
        Ok(TrashResult { ok: true })
    }
    fn open_fda_settings(&self) -> AppResult<FdaResult> {
        Ok(FdaResult { opened: true })
    }
}
