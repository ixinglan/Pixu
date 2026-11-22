/* 视图：状态（status）—— 只读 bento 仪表盘，无任何破坏性入口
 * 性能设计：
 *  - 后端指标由采样线程每 2s 写入快照，本页只读取，不再触发重型系统调用
 *  - 本页每 3s 轮询一次指标、每 15s 刷新进程，均为定向节点更新（不重建 DOM，不闪烁）
 *  - 离开页面后定时器靠路由守卫自动空转，再次进入时先清旧定时器
 */
OrbitShell.views.status = (() => {
  const I = OrbitShell.icon;
  let metricTimer = null;
  let procTimer = null;

  const C = {
    cpu: '#4AE3C4', mem: '#9DB2D8', gpu: '#E8B36A', disk: '#F5C451',
    net: '#6FA8FF', health: '#4AE3C4',
  };

  const na = (v) => (v === null || v === undefined ? '—' : v);

  /* 迷你折线：空数据/单点时画水平基线，杜绝除零 NaN */
  function sparkline(data, color) {
    const d = Array.isArray(data) ? data.filter((x) => typeof x === 'number') : [];
    const gid = `sg-${color.replace('#', '')}`;
    if (d.length < 2) {
      return `<svg class="spark" viewBox="0 0 100 44" preserveAspectRatio="none">
        <line x1="0" y1="34" x2="100" y2="34" stroke="${color}" stroke-width="1.4"
          stroke-dasharray="3 4" opacity="0.45" stroke-linecap="round"/>
      </svg>`;
    }
    const max = Math.max(...d), min = Math.min(...d);
    const span = Math.max(1, max - min);
    const pts = d.map((v, i) => {
      const x = (i / (d.length - 1)) * 100;
      const y = 40 - ((v - min) / span) * 34 - 3;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    }).join(' ');
    return `<svg class="spark" viewBox="0 0 100 44" preserveAspectRatio="none">
      <defs><linearGradient id="${gid}" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stop-color="${color}" stop-opacity="0.32"/>
        <stop offset="100%" stop-color="${color}" stop-opacity="0"/>
      </linearGradient></defs>
      <polygon points="0,44 ${pts} 100,44" fill="url(#${gid})"/>
      <polyline points="${pts}" fill="none" stroke="${color}" stroke-width="1.6"
        stroke-linejoin="round" stroke-linecap="round"/>
    </svg>`;
  }

  /* 圆环仪表：value/max 归一化，支持自定义单位与尺寸 */
  function ring(value, max, unit, color, danger, size = 84) {
    const r = 38, circ = 2 * Math.PI * r;
    const pct = Math.max(0, Math.min(100, (value / max) * 100));
    const dash = (pct / 100) * circ;
    const stroke = danger ? '#FF6B6B' : color;
    return `<svg class="gauge" viewBox="0 0 92 92" style="width:${size}px;height:${size}px">
      <circle cx="46" cy="46" r="${r}" fill="none" stroke="rgba(128,140,170,0.16)" stroke-width="8"/>
      <circle cx="46" cy="46" r="${r}" fill="none" stroke="${stroke}" stroke-width="8"
        stroke-linecap="round" stroke-dasharray="${dash} ${circ}"
        transform="rotate(-90 46 46)" style="transition:stroke-dasharray .5s var(--ease)"/>
      <text x="46" y="47" text-anchor="middle" font-size="17" font-weight="700" fill="${stroke}">${value}</text>
      <text x="46" y="63" text-anchor="middle" font-size="9.5" fill="var(--text-muted)">${unit}</text>
    </svg>`;
  }

  function skeleton() {
    return `<div class="bento">
      ${Array.from({ length: 8 }, () => '<div class="card metric-card"><div class="sk-line w40"></div><div class="sk-line w60 mt-12"></div><div class="sk-spark"></div></div>').join('')}
      <div class="card span-3"><div class="sk-line w30"></div><div class="sk-line w90 mt-16"></div></div>
    </div>`;
  }

  function metricCard({ id, icon, label, value, unit, color, spark, sub, badge, danger }) {
    return `<div class="card metric-card ${danger ? 'is-danger' : ''}" data-card="${id}">
      <div class="metric-label">${I(icon, 'ico-sm')}<span>${label}</span>
        ${badge ? `<span class="metric-badge">${badge}</span>` : ''}
        ${danger ? '<span class="tag tag-danger">告急</span>' : ''}
      </div>
      <div class="metric-value"><span data-f="${id}.value">${na(value)}</span><small>${unit}</small></div>
      ${spark ? `<div data-f="${id}.spark">${sparkline(spark, color)}</div>` : '<div class="spark-placeholder"></div>'}
      <div class="metric-sub text-12 text-muted" data-f="${id}.sub">${sub || ''}</div>
    </div>`;
  }

  function layout(m, procs) {
    return `<div class="bento stagger">
      ${metricCard({
        id: 'cpu', icon: 'cpu', label: m.cpu.label, value: na(m.cpu.value), unit: '%',
        color: C.cpu, spark: m.cpu.spark, sub: m.cpu.cores || '',
      })}
      ${metricCard({
        id: 'mem', icon: 'layers', label: m.mem.label, value: na(m.mem.value), unit: '%',
        color: C.mem, spark: m.mem.spark, sub: m.mem.detail || '',
        danger: (m.mem.value || 0) > 90,
      })}
      ${metricCard({
        id: 'gpu', icon: 'gpu', label: m.gpu.label,
        value: na(m.gpu.value), unit: m.gpu.unit || '%',
        color: C.gpu, spark: m.gpu.spark,
        badge: m.gpu.cores || 'Apple GPU', sub: m.gpu.detail || '',
      })}
      ${metricCard({
        id: 'disk', icon: 'drive', label: m.disk.label, value: na(m.disk.value), unit: '%',
        color: C.disk, spark: m.disk.spark, sub: m.disk.detail || '',
        danger: m.disk.danger === true,
      })}

      <div class="card metric-card" data-card="net">
        <div class="metric-label">${I('wifi', 'ico-sm')}<span>${m.net.label}</span></div>
        <div class="net-rates">
          <div class="net-rate"><span class="net-arrow down">↓</span>
            <b data-f="net.down">${na(m.net.down)}</b><small>MB/s</small></div>
          <div class="net-rate"><span class="net-arrow up">↑</span>
            <b data-f="net.up">${na(m.net.up)}</b><small>MB/s</small></div>
        </div>
        <div data-f="net.spark">${sparkline(m.net.spark, C.net)}</div>
        <div class="metric-sub text-12 text-muted">下载速率趋势（2s 采样）</div>
      </div>

      <div class="card metric-card" data-card="battery">
        <div class="metric-label">${I('battery', 'ico-sm')}<span>${m.battery.label}</span>
          ${m.battery.danger === true ? '<span class="tag tag-danger">留意</span>' : ''}
        </div>
        <div class="metric-value"><span data-f="battery.value">${na(m.battery.value)}</span><small>%</small></div>
        <div class="spark-placeholder"></div>
        <div class="metric-sub text-12 text-muted" data-f="battery.sub">${m.battery.detail || ''}</div>
      </div>

      <div class="card metric-card" data-card="fan">
        <div class="metric-label">${I('fan', 'ico-sm')}<span>${m.fan.label}</span></div>
        <div class="metric-value"><span data-f="fan.value">${na(m.fan.value)}</span><small>${m.fan.unit || 'RPM'}</small></div>
        <div class="spark-placeholder"></div>
        <div class="metric-sub text-12 text-muted" data-f="fan.sub">${m.fan.detail || ''}</div>
      </div>

      <div class="card metric-card" data-card="uptime">
        <div class="metric-label">${I('clock', 'ico-sm')}<span>${m.uptime.label}</span></div>
        <div class="metric-value uptime-value" data-f="uptime.value" style="font-size:21px">${m.uptime.detail || '—'}</div>
        <div class="spark-placeholder"></div>
        <div class="metric-sub text-12 text-muted">本次开机至今</div>
      </div>

      <div class="card metric-card health-card" data-card="health">
        <div class="metric-label">${I('gauge', 'ico-sm')}<span>${m.health.label}</span>
          ${m.health.danger === true ? '<span class="tag tag-danger">需关注</span>' : '<span class="tag tag-ok">良好</span>'}
        </div>
        <div class="health-body">
          <div data-f="health.ring">${ring(m.health.value || 0, 100, '分', C.health, m.health.danger === true, 96)}</div>
          <p class="text-13 text-sub" data-f="health.sub" style="line-height:1.7">${m.health.detail || ''}</p>
        </div>
        <a href="#/clean" class="btn btn-primary btn-sm health-cta">${I('droplet', 'ico-sm')}前往清理</a>
      </div>

      <div class="card span-3 proc-card">
        <div class="metric-label" style="margin-bottom:10px">
          ${I('activity', 'ico-sm')}<span>进程 Top（按内存）</span>
          <span class="proc-meta text-12 text-muted">后台每 10 秒采样 · 最多 20 项</span>
        </div>
        <div class="proc-head list-row">
          <div class="row-main text-12 text-muted">进程名</div>
          <span class="proc-col text-12 text-muted">CPU</span>
          <span class="proc-col text-12 text-muted">内存</span>
        </div>
        <div data-f="procs">${procRows(procs)}</div>
      </div>
    </div>`;
  }

  function procRows(procs) {
    if (!procs || procs.length === 0) {
      return '<div class="proc-empty text-13 text-sub">首次采样中，约 10 秒后显示进程列表…</div>';
    }
    return procs.map((p) => `
      <div class="list-row" style="padding:7px 6px;cursor:default">
        <div class="row-main"><div class="row-title text-13">${p.name}</div></div>
        <span class="proc-col">${p.cpu.toFixed(1)}%</span>
        <span class="proc-col text-muted">${p.mem >= 1024 ? (p.mem / 1024).toFixed(1) + ' GB' : p.mem + ' MB'}</span>
      </div>`).join('');
  }

  /* 定向更新：只替换文本/SVG 节点，避免整页重排与闪烁 */
  function patch(slot, m, procs) {
    const setText = (key, val) => {
      const el = slot.querySelector(`[data-f="${key}"]`);
      if (el) el.textContent = val;
    };
    const setSpark = (key, data, color) => {
      const el = slot.querySelector(`[data-f="${key}"]`);
      if (el) el.innerHTML = sparkline(data, color);
    };
    setText('cpu.value', na(m.cpu.value));
    setText('mem.value', na(m.mem.value));
    setText('gpu.value', na(m.gpu.value));
    setText('disk.value', na(m.disk.value));
    setText('net.down', na(m.net.down));
    setText('net.up', na(m.net.up));
    setText('battery.value', na(m.battery.value));
    setText('fan.value', na(m.fan.value));
    setText('uptime.value', m.uptime.detail || '—');
    setText('cpu.sub', m.cpu.cores || '');
    setText('mem.sub', m.mem.detail || '');
    setText('gpu.sub', m.gpu.detail || '');
    setText('disk.sub', m.disk.detail || '');
    setText('battery.sub', m.battery.detail || '');
    setText('fan.sub', m.fan.detail || '');
    setText('health.sub', m.health.detail || '');
    setSpark('cpu.spark', m.cpu.spark, C.cpu);
    setSpark('mem.spark', m.mem.spark, C.mem);
    setSpark('gpu.spark', m.gpu.spark, C.gpu);
    setSpark('disk.spark', m.disk.spark, C.disk);
    setSpark('net.spark', m.net.spark, C.net);
    const ringEl = slot.querySelector('[data-f="health.ring"]');
    if (ringEl) ringEl.innerHTML = ring(m.health.value || 0, 100, '分', C.health, m.health.danger === true, 96);
    if (procs) {
      const pe = slot.querySelector('[data-f="procs"]');
      if (pe) pe.innerHTML = procRows(procs);
    }
  }

  async function refreshMetrics(slot) {
    try {
      const m = await OrbitAPI.liveMetrics();
      if (OrbitShell.current() !== 'status') return;
      patch(slot, m, null);
    } catch (_) { /* 轮询失败静默，下一周期重试 */ }
  }

  async function refreshProcesses(slot) {
    try {
      const procs = await OrbitAPI.listProcesses();
      if (OrbitShell.current() !== 'status') return;
      const pe = slot.querySelector('[data-f="procs"]');
      if (pe) pe.innerHTML = procRows(procs);
    } catch (_) { /* 同上 */ }
  }

  function dispose() {
    if (metricTimer) { clearInterval(metricTimer); metricTimer = null; }
    if (procTimer) { clearInterval(procTimer); procTimer = null; }
  }

  async function render(slot) {
    dispose();

    slot.innerHTML = skeleton();
    let m, procs;
    try {
      [m, procs] = await Promise.all([OrbitAPI.liveMetrics(), OrbitAPI.listProcesses()]);
    } catch (e) {
      slot.innerHTML = `<div class="card"><div class="text-13" style="color:var(--danger)">状态加载失败：${e?.message || e}</div></div>`;
      return;
    }
    if (OrbitShell.current() !== 'status') return;
    slot.innerHTML = layout(m, procs);

    metricTimer = setInterval(() => refreshMetrics(slot), 3000);
    procTimer = setInterval(() => refreshProcesses(slot), 15000);
  }

  return { render, dispose };
})();
