# Pixu（清曜）

> 面向 macOS 的轻量清理软件 · Rust + Tauri 2 · 免费开源（MIT）

Pixu 是一款聚焦 macOS 体验的系统清理工具：清理缓存、卸载软件、分析磁盘占用、清理 AI 工具数据。不做杀毒、不做 CLI，纯图形界面。

- 仅支持 **macOS 26+ / arm64（Apple 芯片）**
- 分发方式：GitHub + 官网，**不上 App Store**
- 开发策略：先搭可运行的产品壳子，真实功能逐个接入（详见 `product-rd-plan-v2.md`）

---

## 技术栈

| 层 | 技术 |
|---|---|
| 前端 | Vanilla JS（IIFE）+ Vite 8 · 无框架、无构建时类型系统 |
| 桌面壳 | Tauri 2.12 · WKWebView 渲染 |
| 后端 | Rust 2021 edition · workspace 多 crate |
| 数据传输 | serde（Rust↔JS 的 JSON 契约） |

---

## 快速开始

```bash
# 前置：Rust 1.98+ / Node 22+（nvm）/ pnpm（corepack）/ Xcode CLT

# 安装前端依赖
cd app && pnpm install

# 开发模式（热更新 + 开窗）
pnpm tauri dev

# Rust 编译检查（整个 workspace）
cargo check --workspace

# 前端构建
pnpm build
```

> `tauri dev` 需要向 `~/Library/WebKit/pixu/` 写入数据，在沙箱环境启动时需豁免沙箱。

---

## 项目结构

```
CleanMacos/
├── app/                          # Tauri 应用（前端 + 桌面壳）
│   ├── index.html                # 入口 HTML（含主题初始化、宿主检测）
│   ├── vite.config.js            # Vite 配置（端口 1420）
│   ├── public/                   # 前端静态资源
│   │   ├── js/
│   │   │   ├── views/            # 六页视图渲染逻辑
│   │   │   ├── api.js            # 数据层：全部走 Tauri invoke
│   │   │   ├── state.js          # 全局状态（设置、格式化工具）
│   │   │   ├── shell.js          # 路由 + 外壳布局 + 主题切换
│   │   │   ├── icons.js          # 内联 SVG 图标库
│   │   │   └── mascots.js        # 漫画形象（貔貅/仓颉）
│   │   └── styles.css            # 全站样式（深浅双主题 token）
│   └── src-tauri/                # Tauri Rust 端
│       ├── tauri.conf.json       # 应用配置（包名、权限、窗口）
│       ├── capabilities/         # Tauri 能力声明（最小权限）
│       ├── icons/                # 应用图标（icns 等）
│       └── src/
│           ├── main.rs           # 二进制入口
│           ├── lib.rs            # 运行时装配：注册 command + 注入服务
│           ├── commands/         # Tauri command 薄层（仅转发）
│           └── mock/             # Mock 服务实现（M0 全量假数据）
├── crates/                       # Rust 领域 crate（业务逻辑）
│   ├── pixu-app-service/         # 服务 trait + 跨层 DTO（serde）
│   ├── pixu-domain-scan/         # 扫描：目录遍历、体积计量
│   ├── pixu-domain-uninstall/    # 卸载：bundle 元数据、残留规则
│   ├── pixu-safety-gate/         # 安全门：路径校验、SIP 边界
│   ├── pixu-safety-guard/        # 安全护栏：进程占用检测
│   ├── pixu-fs-ops/              # 文件操作：废纸篓、永久删除
│   ├── pixu-platform-mac/        # macOS 平台：FDA、系统框架绑定
│   └── pixu-infra/               # 基础设施：错误模型、审计日志
├── cleanmac-prototype/           # 高保真原型（浏览器预览用，已平移进 app）
├── product-rd-plan-v2.md         # 研发计划
└── pixu-arch-flow_v2.html        # 调用链路图
```

---

## 模块说明

### 前端层（`app/public/`）

纯 Vanilla JS，无框架。各文件职责单一，通过全局命名空间（`Orbit*`）协作：

| 文件 | 职责 |
|---|---|
| `shell.js` | 应用外壳：hash 路由（`#/clean` 等）、侧边栏、顶栏、深浅主题切换、宿主检测 |
| `state.js` | 全局状态：用户设置（从 Rust 加载默认值，本地持久化到 localStorage）、字节格式化等工具函数 |
| `api.js` | 数据访问层：**所有数据经 `window.__TAURI__.core.invoke()` 从 Rust 获取**，无本地假数据 |
| `views/*.js` | 六个页面的渲染与交互逻辑（清理 / AI 数据 / 软件 / 分析 / 状态 / 设置） |
| `icons.js` | 内联 SVG 图标，按名取图 |
| `mascots.js` | 漫画形象 SVG：貔貅（三态）、仓颉（两态） |

### Tauri 壳层（`app/src-tauri/`）

桌面应用的装配层，保持薄层：

| 部分 | 职责 |
|---|---|
| `src/main.rs` | 二进制入口，调用 `lib.rs::run()` |
| `src/lib.rs` | Tauri Builder 装配：`.manage()` 注入服务实例，`invoke_handler!` 注册全部 command |
| `src/commands/mod.rs` | **15 个 command**，每个只做一件事：从 `State` 取出服务，调用 trait 方法，返回结果。不写业务逻辑 |
| `src/mock/mod.rs` | Mock 实现：全量假数据，形状与前端预期一致。M0 期间是唯一实现，M1 起替换为 Real |
| `tauri.conf.json` | 产品名 Pixu、identifier `com.pixu.app`、`withGlobalTauri: true`（前端可用 `window.__TAURI__`）、最低 macOS 26.0 |
| `capabilities/default.json` | 权限最小化：仅 `core:default`，M0 无需其他能力 |

### Rust 领域 crate（`crates/`）

按职责拆分的 workspace 成员，依赖方向由外向内，禁止循环依赖：

| crate | 职责 | 状态 |
|---|---|---|
| `pixu-app-service` | **服务 trait 与 DTO**：定义 `ScanService` / `StatusService` / `SettingsService` / `CleanService` / `UninstallService` 五个 trait，以及跨层传输的 serde 结构（`CleanGroup`、`AppEntry`、`DiskNode` 等）。前端拿到的 JSON 字段与这里一一对应 | ✅ M0 完成 |
| `pixu-domain-scan` | 目录遍历、体积计量（APFS 克隆/硬链接感知，避免重复计费）、缓存分组规则、AI 工具扫描（凭证目录硬隔离） | ✅ M2 / M3 完成 |
| `pixu-domain-uninstall` | `/Applications` bundle 元数据读取、应用残留规则库（首版精选 50 款）、个人数据与可删残留的分类 | ✅ M2 完成 |
| `pixu-safety-gate` | **删除前必经之门**：路径规范化、符号链接解析、允许/拒绝名单、系统卷与 SIP 边界、firmlink 处理。白名单读不到时 fail-closed | ✅ M1 完成 |
| `pixu-safety-guard` | 删除前护栏：目标进程存活检测、活动 SQLite（-wal/-shm 占用）识别、下载中文件保护、句柄占用检查 | ✅ M1 完成 |
| `pixu-fs-ops` | 实际文件操作：NSFileManager 废纸篓（支持 Finder Put Back）、永久删除（仅独立开关+二次确认）、操作超时预算。不做路径决策——输入必须已过 safety-gate | ✅ M1 完成（写操作开发期跳过，见 pending 文档） |
| `pixu-platform-mac` | macOS 平台封装：TCC / Full Disk Access 状态读取与设置跳转、SMAppService、objc2 系统框架绑定。仅编译 `aarch64-apple-darwin` | ⏳ 待 M5（FDA 探测 M3 已在 services/real.rs 内联实现；objc2 封装留待发布前） |
| `pixu-infra` | 基础设施：统一错误模型、本地审计日志（tracing + `~/Library/Logs` 落盘）、配置与历史存储、零遥测约束 | ✅ M1 完成（审计日志当前内存实现，落盘留待 M5） |

---

## 调用链路

一次「清理页扫描」的完整代码路径：

```
用户点击「开始扫描」
  → views/clean.js: startScan()
  → api.js: OrbitAPI.scanCleanGroups()  →  invoke('scan_clean_groups')
  → Tauri IPC（JSON 序列化）
  → commands/mod.rs: scan_clean_groups(reg: State<ServiceRegistry>)
  → reg.scan.scan_clean_groups()   ← 调用 ScanService trait
  → services/real.rs: RealScanService → pixu_domain_scan 真实遍历文件系统（只读）
  ← 原路返回（serde → JSON → JS 对象）
前端渲染清理分组列表
```

**解耦核心**：command 依赖的是 `State<ServiceRegistry>`，调用的是 `ScanService` trait。
Registry 装配 Real 实现；如需回退 Mock，仅改 `ServiceRegistry::new()`，command 和前端**零改动**。
所有写操作受 `pixu_app_service::WRITE_OPERATIONS_ENABLED`（开发期 = false）控制，
返回 `performed: false`，不改动用户文件，详见 `docs/pending-write-operations.md`。

完整图解见 `pixu-arch-flow_v2.html`。

---

## 当前进度

| 阶段 | 内容 | 状态 |
|---|---|---|
| **M0 壳子** | 工程骨架 + 原型平移 + trait/DTO + mock 数据 + 前端全量 invoke | ✅ 完成 |
| M1 | 安全内核 + 清理页真实实现 | ✅ 完成 |
| M2 | 卸载 + 分析页真实实现 | ✅ 完成 |
| M3 | AI 数据 + 状态 + 设置真实实现 | ✅ 完成 |
| M4 | 加固公测（fuzz/集成测试 + 自动更新骨架 + Beta 渠道） | ✅ 完成 |
| M5 | 签名公证 + CI + 发布 | 🔲 |

详见 `product-rd-plan-v2.md`。
