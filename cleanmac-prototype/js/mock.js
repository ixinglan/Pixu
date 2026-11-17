/* Orbit 单一 mock 数据源（原型假数据，业务化文案，禁止散落硬编码） */
const OrbitDB = (() => {
  const cleanGroups = [
    {
      id: 'system', name: '系统数据', hue: '#9DB2D8', icon: 'drive',
      desc: '系统缓存、日志与本地快照，均为可再生数据',
      items: [
        { id: 's1', name: '系统缓存文件', path: '/Library/Caches（系统级）', size: 4_820_000_000, selected: true, note: '应用可自动重建' },
        { id: 's2', name: '系统日志归档', path: '/private/var/log', size: 1_240_000_000, selected: true, note: '仅清理 14 天前日志' },
        { id: 's3', name: 'Time Machine 本地快照', path: '/Volumes/com.apple.TimeMachine', size: 12_600_000_000, selected: false, note: '删除后无法回滚到旧快照', risk: 'warn' }
      ]
    },
    {
      id: 'user', name: '用户缓存与日志', hue: '#4AE3C4', icon: 'folder',
      desc: '当前用户目录下的应用缓存，安全可重建',
      items: [
        { id: 'u1', name: '用户应用缓存', path: '~/Library/Caches', size: 9_360_000_000, selected: true, note: '运行中的应用已自动跳过' },
        { id: 'u2', name: '应用日志文件', path: '~/Library/Logs', size: 780_000_000, selected: true },
        { id: 'u3', name: '临时下载残留', path: '~/Library/Application Support/.../tmp', size: 1_520_000_000, selected: true, note: '下载中的文件受保护' }
      ]
    },
    {
      id: 'browser', name: '浏览器数据', hue: '#E8B36A', icon: 'globe',
      desc: '仅清理缓存，不动书签、密码与登录态',
      items: [
        { id: 'b1', name: 'Safari 浏览器缓存', path: '~/Library/Safari/Library/Caches', size: 2_140_000_000, selected: true },
        { id: 'b2', name: 'Chrome 浏览器缓存', path: '~/Library/Google/Chrome/.../Cache', size: 3_480_000_000, selected: true },
        { id: 'b3', name: 'Arc 浏览器缓存', path: '~/Library/Arc/.../Cache', size: 1_260_000_000, selected: true }
      ]
    },
    {
      id: 'dev', name: '开发者工具', hue: '#F2785C', icon: 'terminal',
      desc: '包管理器、Xcode 与容器缓存，清理后首次构建会变慢',
      items: [
        { id: 'd1', name: 'Xcode DerivedData', path: '~/Library/Developer/Xcode/DerivedData', size: 18_400_000_000, selected: true, note: '下次编译自动重建' },
        { id: 'd2', name: 'npm / pnpm 全局缓存', path: '~/.npm、~/Library/pnpm/store', size: 6_720_000_000, selected: true },
        { id: 'd3', name: 'Homebrew 下载缓存', path: '~/Library/Caches/Homebrew', size: 3_180_000_000, selected: true },
        { id: 'd4', name: 'Docker 悬空镜像层', path: 'docker system df（dangling）', size: 8_940_000_000, selected: false, risk: 'warn', note: '正在运行的容器不受影响' }
      ]
    },
    {
      id: 'trash', name: '废纸篓', hue: '#8A93A8', icon: 'trash',
      desc: '当前启动卷废纸篓，外接卷独立处理',
      items: [
        { id: 't1', name: '废纸篓中的文件（42 项）', path: '~/.Trash', size: 5_260_000_000, selected: true, note: '永久删除，不可恢复', risk: 'danger' }
      ]
    }
  ];

  const aiTools = [
    {
      id: 'claude', name: 'Claude Code', hue: '#D9A27B', icon: 'terminal',
      caches: [
        { id: 'c1', name: '项目索引与缓存', path: '~/.claude/cache', size: 1_840_000_000, selected: true },
        { id: 'c2', name: '旧版本运行时', path: '~/.claude/versions', size: 620_000_000, selected: true }
      ],
      sessions: [
        { id: 'c3', name: '90 天未打开的会话（12 个）', path: '~/.claude/projects/...', size: 480_000_000, selected: false, risk: 'warn', note: '可能包含你想保留的上下文' }
      ],
      clis: []
    },
    {
      id: 'codex', name: 'Codex CLI', hue: '#9DB2D8', icon: 'terminal',
      caches: [
        { id: 'x1', name: '会话缓存与日志', path: '~/.codex/cache', size: 960_000_000, selected: true }
      ],
      sessions: [
        { id: 'x2', name: '旧会话记录（8 个）', path: '~/.codex/sessions', size: 210_000_000, selected: false, risk: 'warn' },
        { id: 'x3', name: '可清理的 git worktree（3 个）', path: '~/code/.worktrees', size: 1_340_000_000, selected: false, risk: 'warn', note: '含未提交改动时自动跳过' }
      ],
      clis: [
        { id: 'x4', name: '闲置 CLI：旧版 gemini-cli', path: '/opt/homebrew/lib/node_modules', size: 180_000_000, selected: false, risk: 'danger', note: '将连同配置一并卸载' }
      ]
    },
    {
      id: 'cursor', name: 'Cursor', hue: '#8FE3D8', icon: 'box',
      caches: [
        { id: 'u1c', name: '编辑器与扩展缓存', path: '~/Library/Application Support/Cursor/Cache', size: 2_260_000_000, selected: true },
        { id: 'u2c', name: 'AI 补全索引缓存', path: '~/Library/Application Support/Cursor/.../index', size: 1_580_000_000, selected: true }
      ],
      sessions: [], clis: []
    }
  ];

  const apps = [
    { id: 'a1', name: 'Figma', developer: 'Figma Inc.', size: 684_000_000, lastUsed: '今天 10:24', hue: '#F2785C', initial: 'F',
      residuals: [
        { name: '应用容器', path: '/Applications/Figma.app', size: 512_000_000, type: 'app' },
        { name: '缓存目录', path: '~/Library/Caches/Figma', size: 138_000_000, type: 'cache' },
        { name: '偏好设置', path: '~/Library/Preferences/com.figma.Desktop.plist', size: 240_000, type: 'pref' },
        { name: 'Application Support', path: '~/Library/Application Support/Figma', size: 33_800_000, type: 'data', keep: true }
      ] },
    { id: 'a2', name: 'Docker Desktop', developer: 'Docker Inc.', size: 2_140_000_000, lastUsed: '昨天 21:08', hue: '#7DD3FC', initial: 'D',
      residuals: [
        { name: '应用容器', path: '/Applications/Docker.app', size: 1_180_000_000, type: 'app' },
        { name: '镜像与卷数据', path: '~/Library/Containers/com.docker.docker', size: 880_000_000, type: 'data', keep: true, note: '含你的容器卷，默认保留' },
        { name: '缓存与日志', path: '~/Library/Caches/com.docker.docker', size: 80_000_000, type: 'cache' }
      ] },
    { id: 'a3', name: 'Notion', developer: 'Notion Labs', size: 426_000_000, lastUsed: '2 天前', hue: '#E8E3DA', initial: 'N',
      residuals: [
        { name: '应用容器', path: '/Applications/Notion.app', size: 318_000_000, type: 'app' },
        { name: '离线数据', path: '~/Library/Application Support/Notion', size: 92_000_000, type: 'data', keep: true },
        { name: '缓存', path: '~/Library/Caches/notion.id', size: 16_000_000, type: 'cache' }
      ] },
    { id: 'a4', name: 'Slack', developer: 'Salesforce', size: 512_000_000, lastUsed: '3 天前', hue: '#C48BD6', initial: 'S',
      residuals: [
        { name: '应用容器', path: '/Applications/Slack.app', size: 388_000_000, type: 'app' },
        { name: '工作区缓存', path: '~/Library/Containers/com.tinyspeck.slackmacgap', size: 118_000_000, type: 'cache' },
        { name: '偏好设置', path: '~/Library/Preferences/com.tinyspeck.slackmacgap.plist', size: 180_000, type: 'pref' }
      ] },
    { id: 'a5', name: 'Ollama', developer: 'Ollama Inc.', size: 4_860_000_000, lastUsed: '5 天前', hue: '#E8E3DA', initial: 'O',
      residuals: [
        { name: '应用容器', path: '/Applications/Ollama.app', size: 240_000_000, type: 'app' },
        { name: '本地模型文件', path: '~/.ollama/models', size: 4_580_000_000, type: 'data', keep: true, note: '模型体积大，默认保留' },
        { name: '运行日志', path: '~/.ollama/logs', size: 40_000_000, type: 'cache' }
      ] },
    { id: 'a6', name: 'Zoom', developer: 'Zoom Video', size: 366_000_000, lastUsed: '2 周前', hue: '#6AA9FF', initial: 'Z',
      residuals: [
        { name: '应用容器', path: '/Applications/zoom.us.app', size: 282_000_000, type: 'app' },
        { name: '会议缓存', path: '~/Library/Caches/us.zoom.xos', size: 74_000_000, type: 'cache' },
        { name: '后台守护项', path: '~/Library/LaunchAgents/us.zoom.xos.plist', size: 12_000, type: 'daemon' }
      ] },
    { id: 'a7', name: 'Spotify', developer: 'Spotify AB', size: 318_000_000, lastUsed: '1 个月前', hue: '#5DD39E', initial: 'S',
      residuals: [
        { name: '应用容器', path: '/Applications/Spotify.app', size: 244_000_000, type: 'app' },
        { name: '流媒体缓存', path: '~/Library/Caches/com.spotify.client', size: 66_000_000, type: 'cache' },
        { name: '偏好设置', path: '~/Library/Preferences/com.spotify.client.plist', size: 96_000, type: 'pref' }
      ] },
    { id: 'a8', name: 'LM Studio', developer: 'LM Studio', size: 1_240_000_000, lastUsed: '2 个月前', hue: '#F5C451', initial: 'L',
      residuals: [
        { name: '应用容器', path: '/Applications/LM Studio.app', size: 486_000_000, type: 'app' },
        { name: '模型目录', path: '~/.cache/lm-studio/models', size: 720_000_000, type: 'data', keep: true },
        { name: '推理缓存', path: '~/Library/Caches/ai.lmstudio', size: 34_000_000, type: 'cache' }
      ] }
  ];

  const diskTree = {
    name: 'Macintosh HD', path: '/', size: 412_000_000_000,
    children: [
      { name: '用户目录', path: '/Users/ixiaoqiang', size: 268_000_000_000, children: [
        { name: '资源库', path: '~/Library', size: 124_000_000_000, children: [
          { name: 'Developer', size: 48_600_000_000 }, { name: 'Caches', size: 31_200_000_000 },
          { name: 'Containers', size: 22_800_000_000 }, { name: 'Application Support', size: 18_400_000_000 },
          { name: '其他', size: 3_000_000_000 }
        ]},
        { name: '项目代码', path: '~/Documents/TreaProject', size: 62_400_000_000, children: [
          { name: 'node_modules 集合', size: 28_900_000_000 }, { name: 'target 构建产物', size: 14_600_000_000 },
          { name: '源码与资源', size: 18_900_000_000 }
        ]},
        { name: '下载', path: '~/Downloads', size: 41_800_000_000, children: [
          { name: '安装包（12 个）', size: 18_200_000_000 }, { name: '视频素材', size: 15_600_000_000 }, { name: '其他文件', size: 8_000_000_000 }
        ]},
        { name: '影片', size: 24_600_000_000 }, { name: '图片', size: 15_200_000_000 }
      ]},
      { name: '应用程序', path: '/Applications', size: 78_400_000_000, children: [
        { name: 'Xcode', size: 32_800_000_000 }, { name: '其他应用（64 个）', size: 45_600_000_000 }
      ]},
      { name: '系统与库', path: '/System', size: 48_200_000_000, protected: true },
      { name: '可用空间', size: 17_400_000_000, free: true }
    ]
  };

  const spark = (base, n = 28, jitter = 12) =>
    Array.from({ length: n }, (_, i) => Math.max(4, Math.round(base + Math.sin(i / 2.4) * jitter + (i % 5) * 2)));

  const metrics = {
    cpu: { label: 'CPU', value: 24, unit: '%', spark: spark(24), cores: 'M3 Pro · 11 核' },
    mem: { label: '内存', value: 68, unit: '%', spark: spark(64, 28, 8), detail: '14.2 / 18 GB' },
    gpu: { label: 'GPU', value: 12, unit: '%', spark: spark(14, 28, 10) },
    disk: { label: '磁盘', value: 91, unit: '%', spark: spark(88, 28, 3), detail: '已用 412 GB / 460 GB', danger: true },
    net: { label: '网络', down: '8.4', up: '1.2', unit: 'MB/s', spark: spark(30, 28, 22) },
    battery: { label: '电池', value: 76, unit: '%', detail: '已接电源 · 循环 84 次' }
  };

  const processes = [
    { name: 'WindowServer', cpu: 18.2, mem: 980 },
    { name: 'Google Chrome 辅助进程', cpu: 12.6, mem: 2140 },
    { name: 'Xcode', cpu: 9.8, mem: 1680 },
    { name: 'Cursor', cpu: 7.4, mem: 1240 },
    { name: 'node（vite dev）', cpu: 6.1, mem: 620 },
    { name: 'Docker Desktop', cpu: 4.2, mem: 1480 },
    { name: 'Slack', cpu: 2.8, mem: 720 },
    { name: 'mds_stores（Spotlight 索引）', cpu: 1.9, mem: 240 }
  ];

  const history = [
    { id: 'h1', time: '2026-11-12 21:14', type: '深度清理', freed: 14_820_000_000, detail: '用户缓存、浏览器缓存、Homebrew 缓存' },
    { id: 'h2', time: '2026-11-09 09:02', type: '应用卸载', freed: 4_860_000_000, detail: 'Ollama（保留本地模型）' },
    { id: 'h3', time: '2026-11-05 17:40', type: 'AI 数据清理', freed: 2_340_000_000, detail: 'Claude Code 缓存、Codex 旧版本' },
    { id: 'h4', time: '2026-10-28 11:26', type: '深度清理', freed: 9_640_000_000, detail: 'Xcode DerivedData、npm 缓存' }
  ];

  const whitelist = [
    { id: 'w1', name: '1Password 数据目录', path: '~/Library/Group Containers/2BUA8C4S2C.com.1password' },
    { id: 'w2', name: '微信聊天记录', path: '~/Library/Containers/com.tencent.xinWeChat' },
    { id: 'w3', name: 'JetBrains 配置', path: '~/Library/Application Support/JetBrains' }
  ];

  const settings = {
    fdaGranted: false,
    trashFirst: true,
    protectRunning: true,
    zeroTelemetry: true,
    updateCheck: true,
    aiEntry: true,
    externalVolumes: false
  };

  return { cleanGroups, aiTools, apps, diskTree, metrics, processes, history, whitelist, settings };
})();
