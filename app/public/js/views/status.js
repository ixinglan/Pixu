/* 视图：状态（status）—— 只读 bento 仪表盘，无任何破坏性入口 */
OrbitShell.views.status = (() => {
  const I = OrbitShell.icon;

  function sparkline(data, color) {
    const max = Math.max(...data), min = Math.min(...data);
    const pts = data.map((v, i) => {
      const x = (i / (data.length - 1)) * 100;
      const y = 40 - ((v - min) / Math.max(1, max - min)) * 36 - 2;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    }).join(' ');
    return `<svg class="spark" viewBox="0 0 100 44" preserveAspectRatio="none">
      <defs><linearGradient id="sg-${color.replace('#', '')}" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stop-color="${color}" stop-opacity="0.35"/><stop offset="100%" stop-color="${color}" stop-opacity="0"/>
      </linearGradient></defs>
      <polygon points="0,44 ${pts} 100,44" fill="url(#sg-${color.replace('#', '')})"/>
      <polyline points="${pts}" fill="none" stroke="${color}" stroke-width="1.6" stroke-linejoin="round" stroke-linecap="round"/>
    </svg>`;
  }

  function gauge(value, color, danger) {
    const r = 38, c = 2 * Math.PI * r;
    const dash = (value / 100) * c;
    return `<svg class="gauge" viewBox="0 0 92 92">
      <circle cx="46" cy="46" r="${r}" fill="none" stroke="rgba(255,255,255,0.08)" stroke-width="8"/>
      <circle cx="46" cy="46" r="${r}" fill="none" stroke="${danger ? '#FF6B6B' : color}" stroke-width="8"
        stroke-linecap="round" stroke-dasharray="${dash} ${c}" transform="rotate(-90 46 46)"/>
      <text x="46" y="51" text-anchor="middle" font-size="19" font-weight="700" fill="${danger ? '#FF6B6B' : '#E8EDF7'}">${value}%</text>
    </svg>`;
  }

  async function render(slot) {
    const [m, procs] = await Promise.all([OrbitAPI.liveMetrics(), OrbitAPI.listProcesses()]);
    slot.innerHTML = `
      <div class="bento stagger">
        <div class="card">
          <div class="metric-label">${I('cpu', 'ico-sm')}${m.cpu.label}</div>
          <div class="metric-value">${m.cpu.value}<small>%</small></div>
          ${sparkline(m.cpu.spark, '#4AE3C4')}
          <div class="text-12 text-muted mt-8">${m.cpu.cores}</div>
        </div>
        <div class="card">
          <div class="metric-label">${I('layers', 'ico-sm')}${m.mem.label}</div>
          <div class="metric-value">${m.mem.value}<small>%</small></div>
          ${sparkline(m.mem.spark, '#9DB2D8')}
          <div class="text-12 text-muted mt-8">${m.mem.detail}</div>
        </div>
        <div class="card">
          <div class="metric-label">${I('gpu', 'ico-sm')}${m.gpu.label}</div>
          <div class="metric-value">${m.gpu.value}<small>%</small></div>
          ${sparkline(m.gpu.spark, '#E8B36A')}
          <div class="text-12 text-muted mt-8">硬件加速 · Metal</div>
        </div>
        <div class="card flex items-center justify-between" style="gap:10px">
          <div>
            <div class="metric-label">${I('drive', 'ico-sm')}${m.disk.label}
              ${m.disk.danger ? '<span class="tag tag-danger">告急</span>' : ''}</div>
            <div class="metric-value">${m.disk.value}<small>%</small></div>
            <div class="text-12 text-muted mt-8">${m.disk.detail}</div>
          </div>
          ${gauge(m.disk.value, '#F5C451', m.disk.danger)}
        </div>
        <div class="card">
          <div class="metric-label">${I('wifi', 'ico-sm')}${m.net.label}</div>
          <div class="metric-value" style="font-size:22px">↓ ${m.net.down}<small>MB/s</small></div>
          <div class="text-13 text-sub" style="margin-top:4px">↑ ${m.net.up} MB/s</div>
          ${sparkline(m.net.spark, '#F5C451')}
        </div>
        <div class="card flex items-center justify-between" style="gap:10px">
          <div>
            <div class="metric-label">${I('battery', 'ico-sm')}${m.battery.label}</div>
            <div class="metric-value">${m.battery.value}<small>%</small></div>
            <div class="text-12 text-muted mt-8">${m.battery.detail}</div>
          </div>
          ${gauge(m.battery.value, '#4AE3C4', false)}
        </div>
        <div class="card span-2">
          <div class="metric-label mb-16" style="margin-bottom:12px">${I('activity', 'ico-sm')}进程 Top（按 CPU）</div>
          <div>
            ${procs.map((p) => `
              <div class="list-row" style="padding:8px 6px;cursor:default">
                <div class="row-main"><div class="row-title text-13">${p.name}</div></div>
                <span class="row-size" style="width:84px;text-align:right">${p.cpu.toFixed(1)}%</span>
                <span class="row-size" style="width:84px;text-align:right;color:var(--text-muted)">${p.mem >= 1024 ? (p.mem / 1024).toFixed(1) + ' GB' : p.mem + ' MB'}</span>
              </div>`).join('')}
          </div>
        </div>
        <div class="card span-2">
          <div class="metric-label" style="margin-bottom:12px">${I('shieldCheck', 'ico-sm')}健康概览</div>
          <div class="flex items-center gap-16">
            <div class="spirit-orb" style="width:86px;height:86px;--orb-hue:#F5C451;flex-shrink:0"></div>
            <div class="row-main">
              <div class="text-16" style="font-weight:700;margin-bottom:6px">磁盘占用偏高，建议尽快清理</div>
              <p class="text-sub text-13" style="line-height:1.7">CPU 与内存负载正常；磁盘已用 91%，Xcode 构建缓存与 Docker 悬空镜像层是最大可回收项。本页所有数据仅本地读取，不执行任何修改操作。</p>
              <a href="#/clean" class="btn btn-primary btn-sm mt-16" style="margin-top:14px">${I('droplet', 'ico-sm')}前往清理</a>
            </div>
          </div>
        </div>
      </div>`;
  }

  return { render };
})();
