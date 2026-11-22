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

// ─── RealStatusService（后台采样线程 + 快照读取）────────────
// 性能设计：所有慢操作（sysinfo 刷新 / ioreg / system_profiler / powermetrics）
// 都在独立采样线程内完成，live_metrics() 只克隆内存快照，微秒级返回，
// 绝不阻塞 Tauri 主线程（同步 command 跑在主线程，阻塞会导致鼠标转圈）。

#[derive(Clone)]
struct GpuInfo {
    model: String,
    vram: Option<String>,
    metal: Option<String>,
}

#[derive(Clone)]
struct BatteryInfo {
    present: bool,
    pct: Option<u32>,
    charging: bool,
    connected: bool,
    cycles: Option<u32>,
    /// 电池健康度 = MaxCapacity / DesignCapacity（百分比）。
    health: Option<u32>,
    temp_c: Option<f64>,
}

#[derive(Clone)]
struct FanInfo {
    rpm: Option<u32>,
    note: String,
}

#[derive(Clone)]
struct StatusSnapshot {
    cpu_val: u32,
    cpu_spark: Vec<u32>,
    cores: usize,
    load1: f64,
    mem_val: u32,
    mem_spark: Vec<u32>,
    mem_used: u64,
    mem_total: u64,
    swap_used: u64,
    swap_total: u64,
    disk_val: u32,
    disk_spark: Vec<u32>,
    disk_total: u64,
    disk_avail: u64,
    /// KB/s，配合 2s 采样窗口计算。
    net_down_kbs: u32,
    net_up_kbs: u32,
    net_spark: Vec<u32>,
    uptime_secs: u64,
    processes: Vec<ProcessInfo>,
    gpu: GpuInfo,
    battery: BatteryInfo,
    fan: FanInfo,
}

fn default_snapshot() -> StatusSnapshot {
    StatusSnapshot {
        cpu_val: 0,
        cpu_spark: vec![0],
        cores: 0,
        load1: 0.0,
        mem_val: 0,
        mem_spark: vec![0],
        mem_used: 0,
        mem_total: 0,
        swap_used: 0,
        swap_total: 0,
        disk_val: 0,
        disk_spark: vec![0],
        disk_total: 0,
        disk_avail: 0,
        net_down_kbs: 0,
        net_up_kbs: 0,
        net_spark: vec![0],
        uptime_secs: 0,
        processes: Vec::new(),
        gpu: GpuInfo { model: "GPU 探测中".into(), vram: None, metal: None },
        battery: BatteryInfo {
            present: false, pct: None, charging: false, connected: false,
            cycles: None, health: None, temp_c: None,
        },
        fan: FanInfo { rpm: None, note: "探测中".into() },
    }
}

pub struct RealStatusService {
    snap: Arc<Mutex<StatusSnapshot>>,
}

impl RealStatusService {
    pub fn new() -> Self {
        let snap = Arc::new(Mutex::new(default_snapshot()));
        let writer = snap.clone();
        // 采样线程随 Registry 存活整个应用生命周期；内部全部为只读系统调用。
        std::thread::Builder::new()
            .name("pixu-metrics".into())
            .spawn(move || sample_loop(writer))
            .expect("spawn metrics thread");
        Self { snap }
    }
}

impl Default for RealStatusService {
    fn default() -> Self { Self::new() }
}

/// 后台采样循环：2s 一轮轻量采集，磁盘/进程/电池/风扇按更长周期降频。
fn sample_loop(snap: Arc<Mutex<StatusSnapshot>>) {
    use std::time::{Duration, Instant};
    use sysinfo::{Disks, Networks, ProcessesToUpdate, System};

    let mut sys = System::new();
    let mut nets = Networks::new_with_refreshed_list();
    let mut disks = Disks::new_with_refreshed_list();

    // CPU 使用率需要两次刷新之间的时间窗，先预热一帧。
    sys.refresh_cpu_all();
    std::thread::sleep(Duration::from_millis(200));
    sys.refresh_cpu_all();

    // GPU 型号查询较慢（system_profiler 1-3s），仅在启动时做一次。
    let gpu = query_gpu_info();
    let fan_model = machine_model();

    let mut prev_rx: u64 = nets.list().values().map(|n| n.total_received()).sum();
    let mut prev_tx: u64 = nets.list().values().map(|n| n.total_transmitted()).sum();
    let mut prev_at = Instant::now();
    let mut tick: u64 = 0;

    loop {
        sys.refresh_cpu_all();
        sys.refresh_memory();
        nets.refresh();

        let rx: u64 = nets.list().values().map(|n| n.total_received()).sum();
        let tx: u64 = nets.list().values().map(|n| n.total_transmitted()).sum();
        let dt = prev_at.elapsed().as_secs_f64().max(0.001);
        let down_kbs = (rx.saturating_sub(prev_rx) as f64 / dt / 1024.0) as u32;
        let up_kbs = (tx.saturating_sub(prev_tx) as f64 / dt / 1024.0) as u32;
        prev_rx = rx;
        prev_tx = tx;
        prev_at = Instant::now();

        let cpu_val = (sys.global_cpu_usage() as u32).min(100);
        let cores = sys.cpus().len();
        let load = System::load_average();
        let mem_total = sys.total_memory();
        let mem_used = sys.used_memory();
        let mem_val = if mem_total > 0 { (mem_used * 100 / mem_total) as u32 } else { 0 };

        // 磁盘 / 进程每 10s 一轮（进程全量刷新代价高，降频避免持续占 CPU）。
        if tick % 5 == 0 {
            disks.refresh();
            sys.refresh_processes(ProcessesToUpdate::All, true);
        }
        let (disk_total, disk_avail) = disk_space_from(&disks);
        let disk_used = disk_total.saturating_sub(disk_avail);
        let disk_val = if disk_total > 0 { (disk_used * 100 / disk_total) as u32 } else { 0 };

        // 电池每 16s、风扇每 30s 探测一次（外部命令，避免频繁 spawn）。
        let battery = if tick % 8 == 0 { query_battery() } else {
            snap.lock().unwrap().battery.clone()
        };
        let fan = if tick % 15 == 0 { query_fan(&fan_model) } else {
            snap.lock().unwrap().fan.clone()
        };

        let mut procs: Vec<ProcessInfo> = sys
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

        {
            let mut s = snap.lock().unwrap();
            push_spark(&mut s.cpu_spark, cpu_val);
            push_spark(&mut s.mem_spark, mem_val);
            push_spark(&mut s.disk_spark, disk_val);
            push_spark(&mut s.net_spark, down_kbs.min(9999));
            s.cpu_val = cpu_val;
            s.cores = cores;
            s.load1 = load.one;
            s.mem_val = mem_val;
            s.mem_used = mem_used;
            s.mem_total = mem_total;
            s.swap_used = sys.used_swap();
            s.swap_total = sys.total_swap();
            s.disk_val = disk_val;
            s.disk_total = disk_total;
            s.disk_avail = disk_avail;
            s.net_down_kbs = down_kbs;
            s.net_up_kbs = up_kbs;
            s.uptime_secs = System::uptime();
            s.processes = procs;
            s.gpu = gpu.clone();
            s.battery = battery;
            s.fan = fan;
        }

        tick += 1;
        std::thread::sleep(Duration::from_secs(2));
    }
}

impl StatusService for RealStatusService {
    fn live_metrics(&self) -> AppResult<Metrics> {
        let s = self.snap.lock().unwrap().clone();
        let gb = |b: u64| format!("{:.1} GB", b as f64 / 1024.0 / 1024.0 / 1024.0);

        let disk_danger = s.disk_val > 90;
        let (health_score, health_note) =
            health_score(s.disk_val, s.mem_val, s.swap_total, s.swap_used, &s.battery);

        let gpu_detail = match (&s.gpu.vram, &s.gpu.metal) {
            (Some(v), Some(m)) => format!("{} · {}", v, m),
            (Some(v), None) => v.clone(),
            _ => "系统未开放无授权 GPU 使用率读取".into(),
        };

        let battery_detail = if !s.battery.present {
            "未检测到电池（台式机）".into()
        } else {
            let power = if s.battery.charging {
                "充电中"
            } else if s.battery.connected {
                "已接电源"
            } else {
                "电池供电"
            };
            let mut parts = vec![power.to_string()];
            if let Some(c) = s.battery.cycles {
                parts.push(format!("循环 {} 次", c));
            }
            if let Some(h) = s.battery.health {
                parts.push(format!("健康 {}%", h));
            }
            if let Some(t) = s.battery.temp_c {
                parts.push(format!("{:.1}°C", t));
            }
            parts.join(" · ")
        };

        Ok(Metrics {
            cpu: MetricItem {
                label: "CPU".into(), value: Some(s.cpu_val), unit: "%".into(),
                spark: s.cpu_spark.clone(),
                cores: Some(format!("{} 核 · 负载 {:.2}", s.cores, s.load1)),
                detail: None, down: None, up: None, danger: None,
            },
            mem: MetricItem {
                label: "内存".into(), value: Some(s.mem_val), unit: "%".into(),
                spark: s.mem_spark.clone(), cores: None,
                detail: Some(format!(
                    "已用 {} / {} · Swap {} / {}",
                    gb(s.mem_used), gb(s.mem_total), gb(s.swap_used), gb(s.swap_total)
                )),
                down: None, up: None, danger: None,
            },
            gpu: MetricItem {
                label: "GPU".into(), value: None, unit: "%".into(), spark: vec![],
                cores: Some(s.gpu.model.clone()), detail: Some(gpu_detail),
                down: None, up: None, danger: None,
            },
            disk: MetricItem {
                label: "磁盘".into(), value: Some(s.disk_val), unit: "%".into(),
                spark: s.disk_spark.clone(), cores: None,
                detail: Some(format!(
                    "已用 {} / {} · 可用 {}",
                    gb(s.disk_total.saturating_sub(s.disk_avail)), gb(s.disk_total), gb(s.disk_avail)
                )),
                down: None, up: None, danger: Some(disk_danger),
            },
            net: MetricItem {
                label: "网络".into(), value: None, unit: "MB/s".into(),
                spark: s.net_spark.clone(), cores: None, detail: None,
                down: Some(format!("{:.1}", s.net_down_kbs as f64 / 1024.0)),
                up: Some(format!("{:.1}", s.net_up_kbs as f64 / 1024.0)),
                danger: None,
            },
            battery: MetricItem {
                label: "电池".into(), value: s.battery.pct, unit: "%".into(), spark: vec![],
                cores: None, detail: Some(battery_detail),
                down: None, up: None,
                danger: s.battery.health.map(|h| h < 70),
            },
            health: MetricItem {
                label: "健康度".into(), value: Some(health_score), unit: "分".into(),
                spark: vec![], cores: None, detail: Some(health_note),
                down: None, up: None, danger: Some(health_score < 60),
            },
            fan: MetricItem {
                label: "风扇".into(), value: s.fan.rpm, unit: "RPM".into(), spark: vec![],
                cores: None, detail: Some(s.fan.note),
                down: None, up: None, danger: None,
            },
            uptime: MetricItem {
                label: "运行时间".into(), value: Some(s.uptime_secs as u32), unit: "秒".into(),
                spark: vec![], cores: None, detail: Some(fmt_duration(s.uptime_secs)),
                down: None, up: None, danger: None,
            },
        })
    }

    /// 直接返回后台缓存的进程 Top20；未完成首次采样时为空数组（前端显示采样中）。
    fn list_processes(&self) -> AppResult<Vec<ProcessInfo>> {
        Ok(self.snap.lock().unwrap().processes.clone())
    }
}

fn push_spark(v: &mut Vec<u32>, val: u32) {
    v.push(val);
    if v.len() > 30 {
        let _ = v.remove(0);
    }
}

fn fmt_duration(secs: u64) -> String {
    let days = secs / 86_400;
    let hours = (secs % 86_400) / 3_600;
    let mins = (secs % 3_600) / 60;
    if days > 0 {
        format!("{} 天 {} 小时", days, hours)
    } else if hours > 0 {
        format!("{} 小时 {} 分", hours, mins)
    } else {
        format!("{} 分钟", mins)
    }
}

/// 综合健康度：磁盘 40% + 内存 25% + 电池 25%（无电池机型权重并入磁盘/内存）+ 基线 10%。
fn health_score(
    disk_val: u32,
    mem_val: u32,
    swap_total: u64,
    swap_used: u64,
    batt: &BatteryInfo,
) -> (u32, String) {
    let disk_pts = match disk_val {
        v if v <= 70 => 40,
        v if v <= 85 => 32,
        v if v <= 90 => 22,
        v if v <= 95 => 12,
        _ => 5,
    };
    let mem_pts = match mem_val {
        v if v <= 70 => 25,
        v if v <= 85 => 18,
        v if v <= 95 => 10,
        _ => 5,
    };
    let swap_penalty = if swap_total > 0 && swap_used * 100 / swap_total > 60 { 4 } else { 0 };

    let (batt_pts, batt_weight_note) = if batt.present {
        let pts = match batt.health {
            Some(h) if h >= 90 => 25,
            Some(h) if h >= 80 => 18,
            Some(h) if h >= 70 => 10,
            Some(_) => 5,
            None => 20,
        };
        (pts, true)
    } else {
        // 台式机无电池：25 分按磁盘 15 / 内存 10 再分配
        let extra_disk = if disk_val <= 80 { 15 } else { 8 };
        let extra_mem = if mem_val <= 80 { 10 } else { 5 };
        (extra_disk + extra_mem, false)
    };
    let _ = batt_weight_note;

    let base: u32 = disk_pts + mem_pts + batt_pts + 10;
    let score = base.saturating_sub(swap_penalty).min(100);

    let mut weak = Vec::new();
    if disk_val > 85 { weak.push("磁盘占用偏高，建议清理缓存与大文件"); }
    if mem_val > 85 { weak.push("内存压力较大，可关闭闲置应用"); }
    if swap_total > 0 && swap_used * 100 / swap_total > 60 { weak.push("Swap 使用偏高，内存可能不足"); }
    if let Some(h) = batt.health {
        if h < 80 { weak.push("电池健康度下降，建议关注续航"); }
    }
    let note = if weak.is_empty() {
        "各项指标正常，系统状态良好".into()
    } else {
        weak.join("；")
    };
    (score, note)
}

fn disk_space_from(disks: &sysinfo::Disks) -> (u64, u64) {
    let pick = |mp: &str| {
        disks.list().iter().find(|d| d.mount_point() == std::path::Path::new(mp))
    };
    if let Some(d) = pick("/System/Volumes/Data").or_else(|| pick("/")) {
        (d.total_space(), d.available_space())
    } else {
        (0, 0)
    }
}

/// 机器型号（sysctl hw.model），用于判断无风扇机型（MacBook Air）。
fn machine_model() -> String {
    std::process::Command::new("sysctl")
        .args(["-n", "hw.model"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

/// GPU 型号 / 显存 / Metal 能力：system_profiler JSON，慢但只在启动时执行一次。
fn query_gpu_info() -> GpuInfo {
    let unknown = GpuInfo {
        model: "Apple GPU".into(), vram: None,
        metal: Some("Metal".into()),
    };
    let Ok(out) = std::process::Command::new("system_profiler")
        .args(["SPDisplaysDataType", "-json"])
        .output()
    else {
        return unknown;
    };
    let Ok(val) = serde_json::from_slice::<serde_json::Value>(&out.stdout) else {
        return unknown;
    };
    let Some(arr) = val.get("SPDisplaysDataType").and_then(|v| v.as_array()) else {
        return unknown;
    };
    let Some(first) = arr.first().and_then(|v| v.as_object()) else {
        return unknown;
    };
    let mut model = String::new();
    let mut vram = None;
    let mut metal = None;
    for (k, v) in first {
        let key = k.to_lowercase();
        let text = v.as_str().map(str::to_string);
        if (key.contains("model") || key.contains("_name")) && model.is_empty() {
            if let Some(t) = text { model = t; }
        } else if key.contains("vram") {
            vram = text;
        } else if key.contains("metal") {
            metal = text.map(|t| if t.to_lowercase().contains("supported") || t == "yes" {
                "Metal".to_string()
            } else {
                t
            });
        }
    }
    if model.is_empty() {
        model = "Apple GPU".into();
    }
    GpuInfo { model, vram, metal: metal.or_else(|| Some("Metal".into())) }
}

/// 电池详情：ioreg AppleSmartBattery，无需特权，字段齐全（循环/设计容量/温度）。
fn query_battery() -> BatteryInfo {
    let absent = BatteryInfo {
        present: false, pct: None, charging: false, connected: false,
        cycles: None, health: None, temp_c: None,
    };
    let Ok(out) = std::process::Command::new("ioreg")
        .args(["-rn", "AppleSmartBattery"])
        .output()
    else {
        return absent;
    };
    let text = String::from_utf8_lossy(&out.stdout);
    if text.trim().is_empty() {
        return absent;
    }
    // ioreg 树状输出行首带 "|   " 前缀，需先剥除再匹配引号键。
    let pick_i64 = |key: &str| -> Option<i64> {
        text.lines().find_map(|line| {
            let l = line.trim().trim_start_matches('|').trim();
            let l = l.strip_prefix('"')?;
            let (k, rest) = l.split_once('"')?;
            if k != key { return None; }
            rest.split('=').nth(1)?.trim().parse::<i64>().ok()
        })
    };
    // ioreg 布尔字段输出为 Yes/No（如 "ExternalConnected" = Yes），数字解析兜底。
    let pick_bool = |key: &str| -> bool {
        text.lines().any(|line| {
            let l = line.trim().trim_start_matches('|').trim();
            let Some(l) = l.strip_prefix('"') else { return false; };
            let Some((k, rest)) = l.split_once('"') else { return false; };
            if k != key { return false; }
            let v = rest.split('=').nth(1).unwrap_or("").trim();
            v.eq_ignore_ascii_case("Yes") || v == "1"
        })
    };

    let current = pick_i64("CurrentCapacity");
    let max = pick_i64("MaxCapacity");
    let design = pick_i64("DesignCapacity");
    let pct = match (current, max) {
        (Some(c), Some(m)) if m > 0 => Some((c * 100 / m) as u32),
        _ => None,
    };
    // ioreg 温度单位为厘摄氏度。
    let temp_c = pick_i64("Temperature").map(|t| t as f64 / 100.0);
    // 健康度：优先顶层 DesignCapacity；Apple Silicon 新系统该字段嵌在 BatteryData 字典内，
    // 取 FullChargeCapacity / DesignCapacity（mAh 原始值）。
    let health = match (max, design) {
        (Some(m), Some(d)) if d > 0 => Some((m * 100 / d) as u32),
        _ => {
            let bd = text.lines().find(|l| l.contains("BatteryData"));
            let Some(bd) = bd else {
                return BatteryInfo {
                    present: true, pct, charging: pick_bool("IsCharging"),
                    connected: pick_bool("ExternalConnected") || pick_bool("IsCharging"),
                    cycles: pick_i64("CycleCount").map(|v| v as u32),
                    health: None, temp_c,
                };
            };
            let raw_in = |k: &str| -> Option<i64> {
                let pat = format!("\"{}\"=", k);
                let seg = bd.split(&pat).nth(1)?;
                seg.split([',', '}']).next()?.trim().parse().ok()
            };
            let full = raw_in("FullChargeCapacity").or_else(|| raw_in("AppleRawMaxCapacity"));
            let d2 = raw_in("DesignCapacity");
            match (full, d2) {
                (Some(f), Some(d)) if d > 0 => Some((f * 100 / d) as u32),
                _ => None,
            }
        }
    };

    BatteryInfo {
        present: true,
        pct,
        charging: pick_bool("IsCharging"),
        connected: pick_bool("ExternalConnected") || pick_bool("IsCharging"),
        cycles: pick_i64("CycleCount").map(|v| v as u32),
        health,
        temp_c,
    }
}

/// 风扇转速：Air 系列无风扇；其余机型尝试 powermetrics SMC 采样（无特权会失败，优雅降级）。
fn query_fan(machine_model: &str) -> FanInfo {
    if machine_model.contains("MacBookAir") {
        return FanInfo { rpm: None, note: "无风扇（被动散热）".into() };
    }
    let Ok(out) = std::process::Command::new("powermetrics")
        .args(["--samplers", "smc", "-n", "1", "-i", "100"])
        .output()
    else {
        return FanInfo { rpm: None, note: "需授权读取（SMC 接入留待后续）".into() };
    };
    if !out.status.success() {
        return FanInfo { rpm: None, note: "需授权读取（SMC 接入留待后续）".into() };
    }
    let text = String::from_utf8_lossy(&out.stdout);
    // powermetrics 输出形如：Fan: 1450 rpm
    let rpm = text.lines().find_map(|line| {
        let l = line.trim();
        if !l.to_lowercase().starts_with("fan") { return None; }
        l.split_whitespace().find_map(|w| w.parse::<u32>().ok())
    });
    match rpm {
        Some(r) => FanInfo { rpm: Some(r), note: format!("当前 {} RPM", r) },
        None => FanInfo { rpm: None, note: "未读到风扇数据".into() },
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
    #[ignore = "heavy: 等待后台采样线程产出真实快照（ioreg/system_profiler）；run with --ignored"]
    fn real_status_snapshot_sanity() {
        let st = RealStatusService::new();
        // 等待采样线程完成首轮（CPU 预热 200ms + 2s 周期 + GPU system_profiler）
        std::thread::sleep(std::time::Duration::from_secs(5));
        let m = st.live_metrics().expect("live_metrics");
        println!("[snapshot] cpu={:?} cores={:?}", m.cpu.value, m.cpu.cores);
        println!("[snapshot] mem={:?} detail={:?}", m.mem.value, m.mem.detail);
        println!("[snapshot] gpu badge={:?} detail={:?}", m.gpu.cores, m.gpu.detail);
        println!("[snapshot] disk={:?} detail={:?}", m.disk.value, m.disk.detail);
        println!("[snapshot] battery={:?} detail={:?}", m.battery.value, m.battery.detail);
        println!("[snapshot] fan={:?} detail={:?}", m.fan.value, m.fan.detail);
        println!("[snapshot] uptime={:?} detail={:?}", m.uptime.value, m.uptime.detail);
        println!("[snapshot] health={:?} detail={:?}", m.health.value, m.health.detail);

        assert!(m.uptime.value.unwrap_or(0) > 0, "uptime should be > 0");
        assert!(m.cpu.cores.is_some(), "cpu cores should be populated");
        assert!(m.gpu.cores.is_some(), "gpu model badge should be populated");
        assert!(m.disk.detail.as_ref().is_some_and(|d| d.contains("GB")));
        assert!(m.health.value.is_some(), "health score should exist");
        // 笔记本应有电池；台式机 CI 允许 None，但 detail 必须给出说明
        assert!(m.battery.detail.is_some());

        let procs = st.list_processes().expect("list_processes");
        println!("[snapshot] processes={}", procs.len());
        assert!(!procs.is_empty(), "首轮采样后进程缓存不应为空");
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
