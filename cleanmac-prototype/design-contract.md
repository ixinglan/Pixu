# 清曜 原型 Design Contract（冻结版，所有页面唯一视觉依据）

> 概念已定稿（2026-11-14）：弃用太阳系命名，采用中国上古神话人物体系；产品名「清曜」，代码内部标识符（OrbitShell 等）保持不变。替换只涉及文案与模块意象，token/组件/布局不动。

## Style Tier & Aesthetic Direction
- style: tech-dark（国潮深空暗色）
- aesthetic: 上古神话 + 深空观测站 —— 深靛蓝夜空 + 星点纹理 + 极光氛围光，情感核心为 CSS 灵兽珠（玉质体积 + 灵雾旋涡 + 玉核高光，非写实图片），克制、安静、可信赖
- 六模块神话映射：貔貅（清理，吞秽）· 仓颉（AI 数据，理卷）· 钟馗（软件，送客）· 千里眼（分析，洞察）· 顺风耳（状态，谛听）· 女娲（设置，补天立极）
- tone keywords: 冷静 / 克制 / 高信息密度 / 东方意象微文案（学 Mole 的气质，不抄 Mole 的视觉）
- 与 Mole 的差异化：Mole = 顶部悬浮胶囊导航 + 写实太阳系行星；清曜 = 左侧灵珠导航 + 模块色灵兽珠 + 国潮玉质质感

## Design Tokens
- 双主题：默认深色（深空观测站）；浅色（破晓观测站）通过 `:root[data-theme="light"]` 覆盖同名 token，组件结构零改动
- 主题持久化 key：`orbit.prototype.theme`（light/dark）；index.html 内联脚本首帧前应用，无闪屏；标题栏右侧 sun/moon 切换按钮
- color.bg（深）: #0A0F1E；color.bg（浅）: #EDF1F8
- 浅色主色加深为 #0C9E84 以保证对比度；浅色下隐去星点纹理、极光改晨光、灵兽珠减弱外辉光
- color.bg: #0A0F1E（深靛夜空）
- color.bg-2: #0E1528（次层）
- color.surface: rgba(255,255,255,0.045)；surface-2: rgba(255,255,255,0.07)
- color.border: rgba(255,255,255,0.09)；border-strong: rgba(255,255,255,0.16)
- color.text: #E8EDF7；text-sub: #9AA7C0；text-muted: #5E6B85
- color.primary(貔貅/清理): #4AE3C4 玉青；hover #6FEBD2
- 模块色：仓颉 银蓝 #9DB2D8｜钟馗 朱砂珊瑚 #F2785C｜千里眼 琥珀 #E8B36A｜顺风耳 赤金 #F5C451｜女娲 黛石灰蓝 #8A93A8
- color.success #4AE3C4 / warning #F5A623 / danger #FF6B6B
- font: -apple-system, "SF Pro Display", "SF Pro Text", "PingFang SC", "Helvetica Neue", sans-serif
- font.mono: "SF Mono", Menlo, Consolas, monospace
- font.scale: 12 / 13 / 14 / 16 / 20 / 24 / 32 / 44
- radius: sm 8 / md 12 / lg 16 / xl 24 / pill 999
- shadow: sm 0 2px 8px rgba(0,0,0,.25)；md 0 8px 28px rgba(0,0,0,.35)；glow 0 0 24px 模块色@35%
- spacing: 4px 基数（4/8/12/16/24/32/48）
- layout: mac 窗口 1280×832 设计稿；侧栏宽 216px；内容区内边距 32px；标题区高 72px
- icon: 内联 Lucide SVG，stroke-width 1.75，currentColor；尺寸 16/18/20/24
- motion: 页面载入 stagger 上浮（.stagger > *，delay 60ms 递增）；hover 180ms ease；漫画形象 5s 上下浮动（mascotBob）+ 4.6s 眨眼（eyeBlink）
- bg-texture: 星点（radial-gradient 多层）+ 两团极光模糊光斑 + 细网格线，禁纯平涂
- 灵兽珠规范（仅状态页保留）：.spirit-orb，变量 --orb-hue；::before 灵雾 conic 旋涡、::after 玉核高光

## 漫画形象规范（mascots.js，冻结）
- 形式：Q 版内联 SVG（viewBox 220），统一 3–3.6px 圆角描边（stroke-linecap/linejoin=round），禁 emoji、禁外链位图
- 配色：主色取模块 token + 固定奶油色 #F4ECDC + 形象专属深色描边；双主题共用，浅色下仅降饱和（saturate .92）
- 貔貅（清理，玉青 #4AE3C4/深 #14544A/描边 #0B2E28）：独角、鬃毛、祥云尾；三态 idle（待机眨眼）/ working（张口吸入缓存碎屑 .m-suck）/ happy（弯眼+灵光星花 .m-spark）
- 仓颉（AI，银蓝 #9DB2D8/深 #3A4C6E/描边 #22304B）：四目、冠帽、宽袖持书卷；两态 idle / working（瞳孔上移+字符灵光上浮 .m-glyphs+头光 .m-halo）
- 承载：.mascot-stage（208px 容器，底部模块色径向呼吸光），替代清理/AI 页旧法阵灵珠；视图只允许选 mood，不得改写 SVG 结构
- 其余人物（钟馗/千里眼/顺风耳/女娲）漫画暂未制作，需要时按本规范同风格扩展，不得另起画风

## Component Spec（只允许复用，不得新造视觉）
- .btn（sm/md/lg；primary 青底深字 / ghost 描边 / danger 描边红 / disabled 40% 透明）
- .card（surface 底 + border + radius lg + hover 提亮 border）
- .tag（sm pill，语义色 12% 透明底 + 同色文字）
- .switch（圆角滑块，开启=模块色）；.checkbox（圆角方框，选中青色对勾）
- .progress（8px 轨道 + 青色填充）；.skeleton（流光灰块）
- .nav-item（左栏项：灵珠光点 + 图标 + 标签；.active 左侧 3px 模块色光条 + surface-2 底）
- .modal（居中卡片 md 阴影 + 40% 黑色遮罩）；.drawer（右侧 400px 滑入）
- .bento（状态页网格卡）；.sunburst/.sb-arc/.inspector-row（分析页旭日环与排名清单）
- .toast（底部居中 pill 提示）；.banner（FDA 权限引导，琥珀描边）
- 状态齐备：hover/active/focus-visible/disabled/empty/loading

## App Shell + Canonical Nav（冻结，逐字节复用）
- body 内：.mac-window > header.titlebar（三灯 + 居中标题） + .app-body > aside.app-nav + main.app-content
- nav items（顺序冻结，标题格式「功能 · 神话人物」）：
  1. 清理 clean（貔貅）| droplet 图标 | #/clean | 玉青
  2. AI 数据 ai（仓颉）| sparkles | #/ai | 银蓝
  3. 软件 software（钟馗）| box | #/software | 朱砂珊瑚
  4. 分析 analyze（千里眼）| grid | #/analyze | 琥珀
  5. 状态 status（顺风耳）| activity | #/status | 赤金
  6. 设置 settings（女娲）| settings-2 | #/settings | 黛石灰蓝
- active 规则：hash 匹配的 nav-item 加 .active（shell.js 统一处理，视图不得手写）
- mount: shell.js 同步注入，幂等；SPA hash 路由，仅一个外壳实例

## Page List
| 页面 | 职责 | 关键组件 |
|---|---|---|
| clean 清理 · 貔貅 | 扫描模拟→分组结果→勾选→预览确认→完成态（大数字+貔貅三态漫画） | 分组卡、checkbox、progress、modal、toast、mascot-stage |
| ai AI 数据 · 仓颉 | AI 工具缓存/旧会话/闲置 CLI 分组，缓存默认选、会话默认不选；仓颉两态漫画 | 同 clean 组件 + 风险说明 tag + mascot-stage |
| software 软件 | 应用列表（图标/大小/最后使用）+ 残留 drawer + 卸载确认 | 表行、drawer、开关、modal |
| analyze 分析 · 千里眼 | 旭日环（内环本级/外环下一层，角度=占比）+ 右侧按大小排名清单（占比条/标签/显式按钮）；扇区与清单行均可下钻，中心圆盘与面包屑返回；保护只读、可用空间不可删 | sunburst、sb-arc、inspector-row、prop-bar、breadcrumb、modal |
| status 状态 | bento 指标 + SVG 折线/仪表 + 进程表 | bento、sparkline、gauge、table |
| settings 设置 | FDA 状态 banner、白名单、隐私开关、历史记录、关于 | banner、switch、list、空状态 |

## Mock Schema（mock.js 单一数据源）
- cleanGroups[]{id,name,hue,icon,items[]{id,name,path,size,selected,note,risk}}
- aiTools[]{id,name,icon,caches[]{...},sessions[]{...},clis[]{...}}
- apps[]{id,name,developer,size,lastUsed,iconHue,residuals[]{name,path,size,type}}
- diskTree{name,path,size,children[]}；metrics{cpu,mem,gpu,disk,net,battery,spark[]}；processes[]{name,cpu,mem}
- history[]{id,time,type,freed,detail}；whitelist[]；settings{fdaGranted,telemetry:false,trashFirst:true,...}
- api.js 全部 async + delay()，标注 // TODO: replace with Tauri command
