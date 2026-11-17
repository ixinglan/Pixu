/* Orbit API 层：全部经 Tauri invoke 从 Rust 获取数据，无本地假数据。
   onStage 进度提示由前端在 invoke 返回后本地模拟（仅 UI 反馈）。 */
const OrbitAPI = (() => {
  const delay = (ms) => new Promise((r) => setTimeout(r, ms));

  function invoke(cmd, args) {
    if (window.__TAURI__?.core?.invoke) {
      return window.__TAURI__.core.invoke(cmd, args);
    }
    // 非 Tauri 环境（浏览器预览）：返回空状态，不提供假数据
    return Promise.reject(new Error('invoke not available'));
  }

  return {
    async scanCleanGroups(onStage) {
      const stages = ['正在建立文件索引…', '探测运行中进程与打开句柄…', '计量系统缓存与日志…',
        '扫描浏览器与开发者工具…', '校验保护名单与路径安全…', '汇总可清理项…'];
      const data = await invoke('scan_clean_groups');
      for (const s of stages) { await delay(280); onStage?.(s); }
      return data || [];
    },

    async scanAiTools(onStage) {
      const stages = ['定位 AI 工具数据目录…', '区分缓存 / 会话 / 闲置 CLI…', '检查 worktree 未提交改动…'];
      const data = await invoke('scan_ai_tools');
      for (const s of stages) { await delay(260); onStage?.(s); }
      return data || [];
    },

    async listApps() {
      return (await invoke('list_apps')) || [];
    },

    async analyzeDisk() {
      return (await invoke('analyze_disk')) || null;
    },

    async liveMetrics() {
      return (await invoke('live_metrics')) || null;
    },

    async listProcesses() {
      return (await invoke('list_processes')) || [];
    },

    async getHistory() {
      return (await invoke('get_history')) || [];
    },

    async getWhitelist() {
      return (await invoke('get_whitelist')) || [];
    },

    async getSettings() {
      return (await invoke('get_settings')) || null;
    },

    async previewDelete(items) {
      return invoke('preview_delete', { items });
    },

    async executeClean(items, opts = {}) {
      return invoke('execute_clean', { items, permanent: !!opts.permanent });
    },

    async uninstallApp(appId, opts = {}) {
      return invoke('uninstall_app', { appId, keepData: !!opts.keepData });
    },

    async moveToTrash(path) {
      return invoke('move_to_trash', { path });
    },

    async openFdaSettings() {
      return invoke('open_fda_settings');
    }
  };
})();
