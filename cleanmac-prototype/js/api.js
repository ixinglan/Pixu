/* Orbit API 桩层：签名即未来 Tauri command 形状，当前返回 mock，延迟模拟真实加载 */
const OrbitAPI = (() => {
  const delay = (ms) => new Promise((r) => setTimeout(r, ms));

  return {
    // TODO: Tauri command `scan_clean_groups` -> invoke('scan_clean_groups')
    async scanCleanGroups(onStage) {
      const stages = ['正在建立文件索引…', '探测运行中进程与打开句柄…', '计量系统缓存与日志…',
        '扫描浏览器与开发者工具…', '校验保护名单与路径安全…', '汇总可清理项…'];
      for (const s of stages) { await delay(420); onStage?.(s); }
      return JSON.parse(JSON.stringify(OrbitDB.cleanGroups));
    },
    // TODO: Tauri command `scan_ai_tools`
    async scanAiTools(onStage) {
      const stages = ['定位 AI 工具数据目录…', '区分缓存 / 会话 / 闲置 CLI…', '检查 worktree 未提交改动…'];
      for (const s of stages) { await delay(380); onStage?.(s); }
      return JSON.parse(JSON.stringify(OrbitDB.aiTools));
    },
    // TODO: Tauri command `list_apps`
    async listApps() { await delay(500); return JSON.parse(JSON.stringify(OrbitDB.apps)); },
    // TODO: Tauri command `analyze_disk(path)`
    async analyzeDisk() { await delay(700); return JSON.parse(JSON.stringify(OrbitDB.diskTree)); },
    // TODO: Tauri command `live_metrics`（未来走 Tauri event 推送）
    async liveMetrics() { await delay(300); return JSON.parse(JSON.stringify(OrbitDB.metrics)); },
    async listProcesses() { await delay(260); return JSON.parse(JSON.stringify(OrbitDB.processes)); },
    // TODO: Tauri command `get_history`
    async getHistory() { await delay(240); return JSON.parse(JSON.stringify(OrbitDB.history)); },
    // TODO: Tauri command `get_whitelist` / `save_whitelist`
    async getWhitelist() { await delay(200); return JSON.parse(JSON.stringify(OrbitDB.whitelist)); },
    // TODO: Tauri command `preview_delete(items)` —— dry-run 与实删共用决策路径
    async previewDelete(items) { await delay(600); return { count: items.length, total: items.reduce((s, i) => s + (i.size || 0), 0) }; },
    // TODO: Tauri command `execute_clean(items, { permanent })`
    async executeClean(items, opts = {}) {
      await delay(900);
      return { freed: items.reduce((s, i) => s + (i.size || 0), 0), permanent: !!opts.permanent, skipped: 0 };
    },
    // TODO: Tauri command `uninstall_app(appId, { keepData })`
    async uninstallApp() { await delay(1000); return { ok: true }; },
    // TODO: Tauri command `move_to_trash(path)`
    async moveToTrash() { await delay(500); return { ok: true }; },
    // TODO: Tauri command `open_system_settings(pane)` —— 跳转完整磁盘访问设置
    async openFdaSettings() { await delay(300); return { opened: true }; }
  };
})();
