//! 真实服务实现（M1 起逐步接入）。
//! 未实现的方法回退到 Mock，保证全端可用；对应阶段完成后替换为真实逻辑。

use std::sync::{Arc, Mutex, OnceLock};

use pixu_app_service::*;
use pixu_domain_scan;
use pixu_fs_ops::{FsOps, FsOutcome};
use pixu_safety_gate::SafetyGate;
use pixu_safety_guard::SafetyGuard;

use crate::mock::MockScanService;

// ─── RealScanService ────────────────────────────────────────
// M1: scan_clean_groups 真实扫描；其余方法 M2/M3 实现前回退 Mock。

pub struct RealScanService {
    // 预留：后续若需按方法回退 Mock 时启用；当前四项扫描均已真实实现。
    #[allow(dead_code)]
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
    fn scan_ai_tools(&self) -> AppResult<Vec<AiTool>> {
        // M3：真实扫描 AI 工具数据，凭证目录硬隔离（只读）
        Ok(pixu_domain_scan::scan_ai_tools())
    }
    fn list_apps(&self) -> AppResult<Vec<AppEntry>> {
        // M2：真实读取 /Applications 元数据与残留（只读）
        Ok(pixu_domain_uninstall::scan_apps())
    }
    fn analyze_disk(&self) -> AppResult<DiskNode> {
        // M2：真实磁盘树（以个人目录为根）+ 真实容量/可用空间（只读）
        let home = std::env::var_os("HOME")
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "/Users".into());
        let mut home_tree = pixu_domain_scan::build_disk_tree(&home);
        home_tree.name = "个人目录".into();

        let (total, avail) = disk_space();
        Ok(DiskNode {
            name: "Macintosh HD".into(),
            path: Some("/".into()),
            size: total,
            children: Some(vec![
                home_tree,
                DiskNode {
                    name: "可用空间".into(), path: None, size: avail,
                    children: None, protected: None, free: Some(true),
                },
            ]),
            protected: None, free: None,
        })
    }
}

/// 读取启动卷（Data 卷）的总容量与可用空间（只读）。
fn disk_space() -> (u64, u64) {
    use sysinfo::{Disk, Disks};
    let disks = Disks::new_with_refreshed_list();
    // 优先 /System/Volumes/Data，其次 /
    let pick = |mp: &str| disks.list().iter().find(|d| d.mount_point() == std::path::Path::new(mp));
    let disk: Option<&Disk> = pick("/System/Volumes/Data").or_else(|| pick("/"));
    if let Some(d) = disk {
        (d.total_space(), d.available_space())
    } else {
        (0, 0)
    }
}

// ─── RealCleanService ───────────────────────────────────────
// M1: 完整删除管线（安全门 → 护栏 → 文件操作），写操作受全局开关控制。
// SafetyGuard 惰性初始化：System::new_all() 较慢，避免阻塞应用启动。

pub struct RealCleanService {
    gate: SafetyGate,
    guard: OnceLock<SafetyGuard>,
    fs: FsOps,
    audit: Arc<AuditStore>,
}

impl RealCleanService {
    pub fn new(audit: Arc<AuditStore>) -> Self {
        Self {
            gate: SafetyGate::new(),
            guard: OnceLock::new(),
            fs: FsOps::new(),
            audit,
        }
    }

    fn guard(&self) -> &SafetyGuard {
        self.guard.get_or_init(SafetyGuard::new)
    }
}

impl Default for RealCleanService {
    fn default() -> Self { Self::new(Arc::new(AuditStore::default())) }
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

        // 4. 审计：写入内存审计日志（写盘跳过，仅内存，M5 再落盘持久化）
        let total: u64 = items.iter().map(|i| i.size).sum();
        self.audit.record_clean(items.len(), total, permanent, performed);

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

// ─── RealUninstallService ───────────────────────────────────
// M2: 卸载/废纸篓/FDA 引导接入真实校验骨架，写操作开发期全部跳过。

pub struct RealUninstallService {
    gate: SafetyGate,
}

impl RealUninstallService {
    pub fn new() -> Self { Self { gate: SafetyGate::new() } }
}

impl Default for RealUninstallService {
    fn default() -> Self { Self::new() }
}

impl UninstallService for RealUninstallService {
    fn uninstall_app(&self, app_id: String, keep_data: bool) -> AppResult<UninstallResult> {
        // 卸载是写操作（移动 .app 至废纸篓 + 清理残留）→ 开发期跳过，记录待审
        // 真实卸载管线（解析 bundle 路径 → 安全门 → fs-ops）在 WRITE_OPERATIONS_ENABLED 后接入
        let _ = (&app_id, keep_data, &self.gate);
        Ok(UninstallResult { ok: true, performed: false, message: Some(skip_message()) })
    }

    fn move_to_trash(&self, path: String) -> AppResult<TrashResult> {
        // 先过安全门（只读校验），再跳过写操作
        let _ = self.gate.check(&path);
        Ok(TrashResult { ok: true, performed: false, message: Some(skip_message()) })
    }

    fn open_fda_settings(&self) -> AppResult<FdaResult> {
        // 打开「完全磁盘访问权限」设置面板属于系统交互，开发期跳过，记录待审
        Ok(FdaResult { opened: true, performed: false, message: Some(skip_message()) })
    }
}

// ─── AuditStore（内存审计日志，写盘跳过）────────────────────

pub struct AuditStore {
    inner: Mutex<Vec<HistoryEntry>>,
}

impl Default for AuditStore {
    fn default() -> Self { Self { inner: Mutex::new(Vec::new()) } }
}

impl AuditStore {
    pub fn record_clean(&self, count: usize, total: u64, permanent: bool, performed: bool) {
        let mut v = self.inner.lock().unwrap();
        let time = now_label();
        let kind = if permanent { "永久清理" } else { "深度清理" };
        let state = if performed { "已执行" } else { "已跳过·待审" };
        let detail = format!("{} 项 · 预计释放 {}（{}）", count, fmt_bytes(total), state);
        let id = format!("h{}", v.len());
        v.insert(0, HistoryEntry { id, time, kind: kind.into(), freed: total, detail });
        if v.len() > 200 {
            v.truncate(200);
        }
    }

    pub fn list(&self) -> Vec<HistoryEntry> {
        self.inner.lock().unwrap().clone()
    }
}

fn now_label() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86400;
    let rem = secs % 86400;
    let h = rem / 3600;
    let m = (rem % 3600) / 60;
    let (y, mo, d) = civil_from_days(days as i64);
    format!("{:04}-{:02}-{:02} {:02}:{:02}", y, mo, d, h, m)
}

// Howard Hinnant 算法：days since 1970-01-01 → (年, 月, 日)
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    (y + if m <= 2 { 1 } else { 0 }, m, d)
}

fn fmt_bytes(b: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut v = b as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 { format!("{} B", b) } else { format!("{:.1} {}", v, UNITS[i]) }
}

// ─── RealStatusService（只读系统指标）─────────────────────
// 全部为读操作：CPU / 内存 / 磁盘 / 网络 / 电池 / 进程列表。

pub struct RealStatusService {
    sys: OnceLock<Mutex<sysinfo::System>>,
    // 火花线历史：[cpu, mem, disk]
    spark: Mutex<Vec<Vec<u32>>>,
}

impl RealStatusService {
    pub fn new() -> Self {
        Self {
            sys: OnceLock::new(),
            spark: Mutex::new(vec![Vec::new(), Vec::new(), Vec::new()]),
        }
    }
    fn sys(&self) -> &Mutex<sysinfo::System> {
        self.sys.get_or_init(|| Mutex::new(sysinfo::System::new()))
    }
}

impl Default for RealStatusService {
    fn default() -> Self { Self::new() }
}

impl StatusService for RealStatusService {
    fn live_metrics(&self) -> AppResult<Metrics> {
        use sysinfo::Networks;
        let mut s = self.sys().lock().unwrap();

        // CPU 双采样取使用率
        s.refresh_cpu_all();
        std::thread::sleep(std::time::Duration::from_millis(180));
        s.refresh_cpu_all();
        let cpu_val = (s.global_cpu_usage() as u32).min(100);
        let cores = s.cpus().len();

        s.refresh_memory();
        let mem_total = s.total_memory();
        let mem_used = s.used_memory();
        let mem_val = if mem_total > 0 { (mem_used * 100 / mem_total) as u32 } else { 0 };

        // 磁盘使用率
        let (total, avail) = disk_space();
        let used = total.saturating_sub(avail);
        let disk_val = if total > 0 { (used * 100 / total) as u32 } else { 0 };

        // 网络速率：双采样
        let mut nets = Networks::new_with_refreshed_list();
        let r1: u64 = nets.list().values().map(|n| n.total_received()).sum();
        let t1: u64 = nets.list().values().map(|n| n.total_transmitted()).sum();
        std::thread::sleep(std::time::Duration::from_millis(180));
        nets.refresh();
        let r2: u64 = nets.list().values().map(|n| n.total_received()).sum();
        let t2: u64 = nets.list().values().map(|n| n.total_transmitted()).sum();
        let down_mbps = ((r2.saturating_sub(r1)) as f64 / 0.18 / 1024.0 / 1024.0) as u32;
        let up_mbps = ((t2.saturating_sub(t1)) as f64 / 0.18 / 1024.0 / 1024.0) as u32;

        let (bat_pct, charging) = battery_info();

        // 火花线历史
        let mut sp = self.spark.lock().unwrap();
        push_spark(&mut sp[0], cpu_val);
        push_spark(&mut sp[1], mem_val);
        push_spark(&mut sp[2], disk_val);

        let gb = |b: u64| format!("{:.1} GB", b as f64 / 1024.0 / 1024.0 / 1024.0);

        let disk_danger = disk_val > 90;
        Ok(Metrics {
            cpu: MetricItem {
                label: "CPU".into(), value: Some(cpu_val), unit: "%".into(),
                spark: sp[0].clone(), cores: Some(format!("{} 核", cores)),
                detail: None, down: None, up: None, danger: None,
            },
            mem: MetricItem {
                label: "内存".into(), value: Some(mem_val), unit: "%".into(),
                spark: sp[1].clone(), cores: None,
                detail: Some(format!("{} / {}", gb(mem_used), gb(mem_total))),
                down: None, up: None, danger: None,
            },
            gpu: MetricItem {
                label: "GPU".into(), value: None, unit: "%".into(), spark: vec![],
                cores: None, detail: Some("未提供 GPU 使用率".into()),
                down: None, up: None, danger: None,
            },
            disk: MetricItem {
                label: "磁盘".into(), value: Some(disk_val), unit: "%".into(),
                spark: sp[2].clone(), cores: None,
                detail: Some(format!("已用 {} / {}", gb(used), gb(total))),
                down: None, up: None, danger: Some(disk_danger),
            },
            net: MetricItem {
                label: "网络".into(), value: None, unit: "MB/s".into(), spark: vec![],
                cores: None, detail: None,
                down: Some(down_mbps.to_string()), up: Some(up_mbps.to_string()), danger: None,
            },
            battery: MetricItem {
                label: "电池".into(), value: bat_pct, unit: "%".into(), spark: vec![],
                cores: None, detail: Some(battery_detail(bat_pct, charging)),
                down: None, up: None, danger: None,
            },
        })
    }

    fn list_processes(&self) -> AppResult<Vec<ProcessInfo>> {
        use sysinfo::ProcessesToUpdate;
        let mut s = self.sys().lock().unwrap();
        // 进程全量刷新较慢（首次约数十秒），但属用户主动请求的只读动作
        s.refresh_processes(ProcessesToUpdate::All, true);
        let mut procs: Vec<ProcessInfo> = s
            .processes()
            .iter()
            .map(|(_, p)| ProcessInfo {
                name: p.name().to_string_lossy().into_owned(),
                cpu: p.cpu_usage() as f64,
                mem: (p.memory() / 1024 / 1024) as u32,
            })
            .collect();
        procs.sort_by(|a, b| b.mem.cmp(&a.mem));
        procs.truncate(20);
        Ok(procs)
    }
}

fn push_spark(v: &mut Vec<u32>, val: u32) {
    v.push(val);
    if v.len() > 28 {
        let _ = v.remove(0);
    }
}

fn battery_info() -> (Option<u32>, bool) {
    let out = match std::process::Command::new("pmset").args(["-g", "batt"]).output() {
        Ok(o) => o,
        Err(_) => return (None, false),
    };
    let s = String::from_utf8_lossy(&out.stdout);
    let pct = s.lines().find_map(|l| {
        let n = l.find('%')?;
        l[..n].split_whitespace().last()?.parse::<u32>().ok()
    });
    let charging = s.contains("AC Power") || s.contains("charging");
    (pct, charging)
}

fn battery_detail(pct: Option<u32>, charging: bool) -> String {
    match pct {
        None => "未检测到电池".into(),
        Some(p) => format!("{} · {}%", if charging { "充电中" } else { "电池供电" }, p),
    }
}

// ─── RealSettingsService（FDA 探测 + 内存白名单/审计）──────

pub struct RealSettingsService {
    audit: Arc<AuditStore>,
    whitelist: Mutex<Vec<WhitelistEntry>>,
}

impl RealSettingsService {
    pub fn new(audit: Arc<AuditStore>) -> Self {
        Self { audit, whitelist: Mutex::new(Vec::new()) }
    }
}

impl SettingsService for RealSettingsService {
    fn get_settings(&self) -> AppResult<Settings> {
        Ok(Settings {
            fda_granted: detect_fda(),
            trash_first: true,
            protect_running: true,
            zero_telemetry: true,
            update_check: true,
            ai_entry: true,
            external_volumes: false,
        })
    }

    fn get_whitelist(&self) -> AppResult<Vec<WhitelistEntry>> {
        Ok(self.whitelist.lock().unwrap().clone())
    }

    fn get_history(&self) -> AppResult<Vec<HistoryEntry>> {
        Ok(self.audit.list())
    }
}

/// 探测是否已授予完全磁盘访问权限：尝试枚举 TCC 保护目录，
/// 任一返回 PermissionDenied 即视为未授权（只读探测，不读取内容）。
fn detect_fda() -> bool {
    let Some(home) = std::env::var_os("HOME") else {
        return false;
    };
    let probes = ["Library/Mail", "Library/Safari", "Library/Accounts"];
    let mut any_denied = false;
    for p in probes {
        let path = std::path::Path::new(&home).join(p);
        if let Err(e) = std::fs::read_dir(&path) {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                any_denied = true;
            }
        }
    }
    !any_denied
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "heavy: real FS scan (apps + disk tree); run with --ignored"]
    fn real_scan_paths_do_not_panic() {
        let s = RealScanService::new();
        let apps = s.list_apps().expect("list_apps should not error");
        assert!(!apps.is_empty(), "should find at least one app");
        let disk = s.analyze_disk().expect("analyze_disk should not error");
        assert!(disk.size > 0, "disk total should be > 0");
    }

    #[test]
    #[ignore = "heavy: real sysinfo init + process refresh (~数十秒); run with --ignored"]
    fn real_m3_services_do_not_panic() {
        // AI 工具扫描：只读遍历；凭证目录必须被硬隔离，不得 panic
        let ai = RealScanService::new()
            .scan_ai_tools()
            .expect("scan_ai_tools should not error");
        for t in &ai {
            assert!(!t.name.is_empty());
        }

        // 状态页：真实只读指标（sysinfo 惰性初始化，首次约数十秒）
        let st = RealStatusService::new();
        let m = st.live_metrics().expect("live_metrics should not error");
        assert_eq!(m.cpu.label, "CPU");
        assert!(m.cpu.cores.is_some());
        let _ = st.list_processes().expect("list_processes should not error");

        // 设置页：白名单 / 审计日志内存视图（写盘全部跳过）
        let audit = Arc::new(AuditStore::default());
        let set = RealSettingsService::new(audit);
        let _ = set.get_settings().expect("get_settings should not error");
    }

    #[test]
    #[ignore = "heavy: 端到端写跳过契约（SafetyGuard 初始化 ~数十秒）；run with --ignored"]
    fn write_operations_are_skipped_e2e() {
        use pixu_app_service::WRITE_OPERATIONS_ENABLED;
        // 开发期契约：全局写开关必须为 false
        assert!(!WRITE_OPERATIONS_ENABLED, "dev build must keep writes disabled");

        use crate::services::ServiceRegistry;
        let reg = ServiceRegistry::new();
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());

        // 清理：路径通过安全门、文件不存在 → 仍应返回 performed=false，不触碰文件
        let item = CleanItem {
            id: "m4-test".into(),
            name: "tmp cache".into(),
            path: format!("{}/Library/Caches/pixu_nonexistent_m4_test", home),
            size: 1024,
            selected: true,
            note: None,
            risk: None,
        };
        let res = reg.clean.execute_clean(vec![item], false).expect("execute_clean");
        assert!(!res.performed, "clean must be skipped in dev, got {:?}", res);

        let un = reg
            .uninstall
            .uninstall_app("com.nonexistent.pixu".into(), true)
            .expect("uninstall_app");
        assert!(!un.performed, "uninstall must be skipped in dev");

        let tr = reg
            .uninstall
            .move_to_trash("/tmp/pixu_nonexistent_m4_trash".into())
            .expect("move_to_trash");
        assert!(!tr.performed, "trash must be skipped in dev");

        let fda = reg.uninstall.open_fda_settings().expect("open_fda_settings");
        assert!(!fda.performed, "open system settings must be skipped in dev");

        // 审计日志已记录（内存），证明流程走到了记录环节但未执行写
        let history = reg.settings.get_history().expect("get_history");
        assert!(!history.is_empty(), "clean should have written an in-memory audit entry");
    }
}
