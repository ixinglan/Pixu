/* Orbit 中央状态：设置持久化到 localStorage，视图只读/通过 action 修改 */
const OrbitState = (() => {
  const KEY = 'orbit.prototype.settings.v1';
  let settings = JSON.parse(JSON.stringify(OrbitDB.settings));
  try {
    const saved = JSON.parse(localStorage.getItem(KEY) || 'null');
    if (saved) settings = { ...settings, ...saved };
  } catch (e) { /* 隐私模式等场景忽略 */ }

  const persist = () => localStorage.setItem(KEY, JSON.stringify(settings));

  return {
    settings,
    setSetting(key, val) { settings[key] = val; persist(); },
    // 字节格式化：B/KB/MB/GB
    fmtBytes(n) {
      if (n == null) return '—';
      const units = ['B', 'KB', 'MB', 'GB', 'TB'];
      let v = n, i = 0;
      while (v >= 1024 && i < units.length - 1) { v /= 1024; i++; }
      return `${v >= 100 || i === 0 ? v.toFixed(0) : v.toFixed(1)} ${units[i]}`;
    },
    // 选择态工具
    groupState(items) {
      const sel = items.filter((i) => i.selected).length;
      return sel === 0 ? 'none' : sel === items.length ? 'all' : 'partial';
    },
    sumSelected(items) { return items.filter((i) => i.selected).reduce((s, i) => s + i.size, 0); },
    sumAll(items) { return items.reduce((s, i) => s + i.size, 0); }
  };
})();
