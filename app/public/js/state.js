/* Orbit 中央状态：设置从 Rust 加载默认值，用户修改持久化到 localStorage */
const OrbitState = (() => {
  const KEY = 'orbit.prototype.settings.v1';
  const settings = {};

  // 从 Rust 加载默认设置，再叠加用户本地持久化
  const ready = OrbitAPI.getSettings().then((s) => {
    if (!s) return;
    try {
      const saved = JSON.parse(localStorage.getItem(KEY) || 'null');
      Object.assign(settings, s, saved || {});
    } catch (e) {
      Object.assign(settings, s);
    }
  }).catch(() => {});

  const persist = () => {
    try { localStorage.setItem(KEY, JSON.stringify(settings)); } catch (e) {}
  };

  return {
    settings,
    ready,
    setSetting(key, val) { settings[key] = val; persist(); },
    // 字节格式化：B/KB/MB/GB
    fmtBytes(n) {
      if (n == null) return '—';
      const units = ['B', 'KB', 'MB', 'GB', 'TB'];
      let v = n, i = 0;
      while (v >= 1024 && i < units.length - 1) { v /= 1024; i++; }
      return `${v >= 100 || i === 0 ? v.toFixed(0) : v.toFixed(1)} ${units[i]}`;
    },
    groupState(items) {
      const sel = items.filter((i) => i.selected).length;
      return sel === 0 ? 'none' : sel === items.length ? 'all' : 'partial';
    },
    sumSelected(items) { return items.filter((i) => i.selected).reduce((s, i) => s + i.size, 0); },
    sumAll(items) { return items.reduce((s, i) => s + i.size, 0); }
  };
})();
