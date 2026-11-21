//! 集成 / fuzz 测试：跨 crate 契约，默认快速运行（不触发 sysinfo 重型初始化）。
//!
//! 覆盖：
//! - safety-gate 对随机 / 对抗输入永不 panic，且系统前缀 / 根 / 相对路径不变量成立；
//! - 应用信息契约（版本、渠道、写开关）；
//! - 写操作全局开关在开发期恒为 false（端到端跳过契约见 real.rs 的 #[ignore] 测试）。

use pixu_app_service::{app_info, APP_VERSION, WRITE_OPERATIONS_ENABLED};
use pixu_safety_gate::{GateVerdict, SafetyGate};

/// 确定性 LCG（零依赖、可复现），用于生成随机字节输入。
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self { Lcg(seed) }
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }
    fn bytes(&mut self, n: usize) -> Vec<u8> {
        (0..n).map(|_| (self.next() & 0xff) as u8).collect()
    }
}

#[test]
fn fuzz_safety_gate_never_panics() {
    let g = SafetyGate::new();
    let mut rng = Lcg::new(0xC0FFEE);
    for _ in 0..3000 {
        let len = (rng.next() % 48) as usize;
        let raw = rng.bytes(len);
        // 模拟不可信前端输入：非法 UTF-8 被替换为 U+FFFD，不得导致 panic。
        let s = String::from_utf8_lossy(&raw).into_owned();
        let _ = g.check(&s);
        let _ = g.check_all(&[s.clone(), s]);
    }

    // 对抗用例集
    let adversarial = [
        "",
        "~",
        "~/.ssh/id_rsa",
        "/",
        "/System",
        "/System/Library",
        "/usr/bin",
        "/..",
        "../../etc/passwd",
        "\0",
        "\n\t",
        "C:\\Windows",
        "/dev/null",
        "/private/var/db/.install",
        "/Library/Preferences/SystemConfiguration",
        "☃",
        "ａｂｃ",
        "/private/var/folders/xx/yy/T",
    ];
    for s in adversarial {
        let _ = g.check(s);
    }
}

#[test]
fn safety_gate_invariants() {
    let g = SafetyGate::new();

    // 系统关键前缀与根目录一律拒绝
    let denied = [
        "/",
        "/System",
        "/System/Library/CoreServices",
        "/usr",
        "/usr/bin/ls",
        "/bin/rm",
        "/sbin",
        "/private/var/db",
        "/private/var/folders/xx",
        "/.vol",
        "/Library/Preferences/SystemConfiguration/com.apple.Boot.plist",
    ];
    for p in denied {
        let v = g.check(p);
        assert!(!v.is_allowed(), "should deny {}: {:?}", p, v);
    }

    // 相对路径拒绝（normalize 后非绝对）
    assert!(!g.check("relative/path").is_allowed());
    assert!(!g.check("no/such/relative/dir").is_allowed());

    // 用户缓存区允许（存在与否 normalize 都应放行）
    if let Ok(home) = std::env::var("HOME") {
        let p = format!("{}/Library/Caches/pixu_fuzz_nonexistent", home);
        let v = g.check(&p);
        assert!(v.is_allowed(), "should allow user cache {}: {:?}", p, v);
    }
}

#[test]
fn check_all_returns_only_denied() {
    let g = SafetyGate::new();
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let paths = vec![
        "/System".to_string(),
        format!("{}/Library/Caches/x", home),
        "/usr/bin".to_string(),
    ];
    let denied = g.check_all(&paths);
    // 只有系统路径被拒，用户缓存通过 → 拒绝项恰好 2 个
    assert_eq!(denied.len(), 2);
    assert!(denied.iter().all(|(p, _)| p != &format!("{}/Library/Caches/x", home)));
}

#[test]
fn app_info_contract() {
    assert!(!WRITE_OPERATIONS_ENABLED, "dev build must keep all write operations disabled");

    let info = app_info();
    assert_eq!(info.version, APP_VERSION);
    // 渠道只允许 stable / beta 两值（环境变量未设时为 stable）
    assert!(
        matches!(info.channel.as_str(), "stable" | "beta"),
        "unexpected channel: {}",
        info.channel
    );
    assert!(matches!(info.build.as_str(), "debug" | "release"));
    assert_eq!(info.write_enabled, false);
}

#[test]
fn verdict_helpers() {
    assert!(GateVerdict::Allow.is_allowed());
    assert!(!GateVerdict::Deny("x".into()).is_allowed());
}
