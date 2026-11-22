# Pixu

> macOS 系统清理工具 · Rust + Tauri 2 · 纯图形界面 · 本地运行，零遥测

Pixu 是一款只面向 macOS 的清理工具：扫描缓存与日志、卸载应用及其残留、分析磁盘占用、清理 AI 工具产生的本地数据，并提供实时系统状态面板。所有扫描默认只读，删除动作经过多层安全校验，且优先移入废纸篓而非直接清除。

## 功能

- **缓存清理**：按分组扫描系统与应用缓存、日志，删除前可逐项预览
- **应用卸载**：读取 `/Applications` 中的应用元数据，对照内置残留规则列出支持文件与数据目录，并区分可删残留与个人数据
- **磁盘分析**：按目录树统计占用，定位大文件，支持将目标移入废纸篓
- **AI 数据清理**：识别主流 AI 工具的缓存、历史会话与闲置 CLI；`~/.ssh`、钥匙串等凭证目录在扫描层硬隔离，不会被触碰
- **状态监控**：CPU、内存、GPU、磁盘、网络、电池、风扇、运行时间与综合健康度，后台采样并绘制迷你趋势图，附进程占用 Top 列表
- **权限与白名单**：Full Disk Access 状态检测与快捷跳转，支持自定义保护路径

## 系统要求

- macOS 26.0 或更高版本
- Apple 芯片（arm64）
- 完整的扫描能力需要 Full Disk Access 授权，应用内可直接跳转到系统设置页

## 技术栈

| 层 | 选型 |
|---|---|
| 桌面壳 | Tauri 2（WKWebView） |
| 前端 | Vanilla JS + Vite，无前端框架 |
| 后端 | Rust 2021 edition，workspace 多 crate |
| 通信 | Tauri IPC + serde JSON 契约 |

## 快速开始

```bash
# 前置：Rust stable、Node 22+、pnpm、Xcode Command Line Tools

cd app
pnpm install
pnpm tauri dev
```

常用命令：

```bash
cargo check --workspace   # Rust 编译检查
cargo test --workspace    # 运行测试
cd app && pnpm build      # 前端构建
```

## 项目结构

```
.
├── app/
│   ├── public/            # 前端：路由、六个页面视图、内联 SVG 图标、双主题样式
│   └── src-tauri/         # Tauri 装配层：command、服务实现、权限声明
├── crates/                # Rust 领域 crate
│   ├── pixu-app-service/     # 服务 trait 与跨层 DTO
│   ├── pixu-domain-scan/     # 目录扫描与体积计量
│   ├── pixu-domain-uninstall/ # 应用卸载与残留规则
│   ├── pixu-safety-gate/     # 删除前路径安全门
│   ├── pixu-safety-guard/    # 占用检测护栏
│   ├── pixu-fs-ops/          # 废纸篓与删除操作
│   ├── pixu-platform-mac/    # macOS 平台封装
│   └── pixu-infra/           # 错误模型与审计
└── cleanmac-prototype/    # 早期高保真原型，保留作设计参考
```

后端按职责拆分 crate，依赖方向由外向内。command 只依赖服务 trait，真实实现与 mock 实现在装配处互换，前端无需改动。

## 安全设计

清理工具的核心风险是误删，Pixu 把一次删除拆成多道关卡：

1. **安全门（safety-gate）**：路径规范化、符号链接解析、系统卷与 SIP 边界检查、允许/拒绝名单；白名单读取失败时 fail-closed，默认拒绝
2. **护栏（safety-guard）**：删除前检测目标是否被进程占用、是否为活动中的 SQLite 数据库、是否正在下载
3. **废纸篓优先**：默认通过 NSFileManager 移入废纸篓，支持 Finder 的"放回"；永久删除需要独立开关与二次确认
4. **凭证隔离**：`~/.ssh`、钥匙串、token/secret 等路径在扫描层硬隔离
5. **最小权限**：Tauri capabilities 仅声明必要能力，应用不含任何遥测与统计上报

此外，所有写操作受一个总开关控制，可在开发或审计场景下整体禁用。

## 当前状态

项目处于 Beta 阶段：扫描、分析、状态监控等只读能力可正常使用；删除与卸载链路已实现并通过测试，在安全开关保护下运行。签名公证、自动更新与正式发布仍在进行中。

## 许可证

MIT License
