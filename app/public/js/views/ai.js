/* 视图：AI 数据（ai）—— 缓存默认选 / 会话默认不选 / 闲置 CLI 高风险 */
OrbitShell.views.ai = (() => {
  const I = OrbitShell.icon;
  let tools = [];
  let phase = 'idle';

  function sectionTag(kind) {
    if (kind === 'sessions') return `<span class="tag tag-warning">旧会话 · 默认保留</span>`;
    if (kind === 'clis') return `<span class="tag tag-danger">闲置 CLI · 卸载级操作</span>`;
    return `<span class="tag">缓存 · 可重建</span>`;
  }

  async function startScan() {
    phase = 'scanning';
    const slot = document.getElementById('viewSlot');
    slot.innerHTML = `
      <div class="scan-stage">
        <div class="mascot-stage" style="--stage-hue:#9DB2D8">${OrbitMascots.cangjie('working')}</div>
        <div class="text-16" style="font-weight:650" data-stage>正在定位 AI 工具目录…</div>
        <div class="scan-log" data-log></div>
      </div>`;
    const logEl = slot.querySelector('[data-log]');
    tools = await OrbitAPI.scanAiTools((s) => {
      slot.querySelector('[data-stage]').textContent = s;
      logEl.innerHTML += `<div><span class="ok">✓</span> ${s}</div>`;
    });
    phase = 'result';
    renderResult();
  }

  function allItems() {
    return tools.flatMap((t) => [...t.caches, ...t.sessions, ...t.clis]);
  }

  function renderResult() {
    const slot = document.getElementById('viewSlot');
    slot.innerHTML = `
      <div class="banner stagger">
        ${I('lock')}
        <div class="banner-txt">
          <b>凭证与配置目录永不触碰</b><br>
          <span>仅处理缓存、旧会话与闲置 CLI；含未提交改动的 worktree 会被自动跳过</span>
        </div>
      </div>
      <div class="stagger">
        ${tools.map((t) => `
          <div class="card mb-16" data-tool="${t.id}">
            <div class="flex items-center gap-12 mb-16">
              <span class="app-ic" style="background:${t.hue}">${I(t.icon)}</span>
              <div class="row-main">
                <div class="row-title">${t.name}</div>
                <div class="row-sub text-mono">${[...t.caches, ...t.sessions, ...t.clis].length} 个可处理项</div>
              </div>
              <span class="row-size">${OrbitShell.fmt([...t.caches, ...t.sessions, ...t.clis].reduce((s, i) => s + i.size, 0))}</span>
            </div>
            ${['caches', 'sessions', 'clis'].map((kind) => t[kind].length ? `
              <div class="mb-16">
                <div class="text-12 text-muted mb-16" style="margin:0 0 8px">${sectionTag(kind)}</div>
                ${t[kind].map((it) => `
                  <label class="list-row">
                    <span class="checkbox ${it.selected ? 'is-checked' : ''}">${I('check')}</span>
                    <div class="row-main">
                      <div class="row-title">${it.name}</div>
                      <div class="row-sub text-mono">${it.path}${it.note ? ` · ${it.note}` : ''}</div>
                    </div>
                    <span class="row-size">${OrbitShell.fmt(it.size)}</span>
                  </label>`).join('')}
              </div>` : '').join('')}
          </div>`).join('')}
        <div class="card sticky-bar flex items-center justify-between">
          <div>
            <div class="text-13 text-sub">已选择 <b data-sel-count style="color:var(--text)">0</b> 项，预计释放</div>
            <div class="num-big" style="font-size:30px;color:var(--hue-ai)" data-sel-size>0 GB</div>
          </div>
          <div class="flex gap-12">
            <button class="btn btn-ghost" data-action="rescan">${I('refresh')}重新扫描</button>
            <button class="btn btn-primary btn-lg" data-action="preview">${I('eye')}预览处理</button>
          </div>
        </div>
      </div>`;
    slot.querySelectorAll('label.list-row').forEach((row) => {
      row.addEventListener('click', (e) => {
        e.preventDefault();
        const rowPath = row.querySelector('.row-sub').textContent;
        for (const t of tools) {
          for (const kind of ['caches', 'sessions', 'clis']) {
            const hit = t[kind].find((x) => rowPath.includes(x.path));
            if (hit) { hit.selected = !hit.selected; renderResult(); return; }
          }
        }
      });
    });
    slot.querySelector('[data-action="rescan"]').addEventListener('click', startScan);
    slot.querySelector('[data-action="preview"]').addEventListener('click', openPreview);
    refreshSummary();
  }

  function refreshSummary() {
    const items = allItems().filter((i) => i.selected);
    const c = document.querySelector('[data-sel-count]');
    if (!c) return;
    c.textContent = items.length;
    document.querySelector('[data-sel-size]').textContent = OrbitShell.fmt(items.reduce((s, i) => s + i.size, 0));
    const btn = document.querySelector('[data-action="preview"]');
    btn.disabled = !items.length;
    btn.classList.toggle('is-disabled', !items.length);
  }

  function openPreview() {
    const items = allItems().filter((i) => i.selected);
    if (!items.length) return;
    const hasCli = items.some((i) => i.risk === 'danger');
    const m = OrbitShell.modal({
      title: '处理预览 · dry-run',
      body: `
        <div class="banner" style="margin-bottom:16px">
          ${I(hasCli ? 'alert' : 'info')}
          <div class="banner-txt">
            <b>${hasCli ? '包含闲置 CLI 卸载，将连同其配置一并移除' : '缓存移入废纸篓，会话类按你的勾选处理'}</b><br>
            <span>每一项都可在执行前取消，路径与保护名单已校验</span>
          </div>
        </div>
        <div class="preview-list">
          ${items.map((i) => `
            <div class="preview-item">
              <span class="pi-path">${i.path}</span>
              <span class="pi-size">${OrbitShell.fmt(i.size)}</span>
            </div>`).join('')}
        </div>
        <div class="flex items-center justify-between mt-16">
          <span class="text-sub text-13">共 ${items.length} 项</span>
          <span class="num-big" style="font-size:24px;color:var(--hue-ai)">${OrbitShell.fmt(items.reduce((s, i) => s + i.size, 0))}</span>
        </div>`,
      footer: `<button class="btn btn-ghost" data-cancel>取消</button>
        <button class="btn btn-primary" data-confirm>${I('sparkles')}确认执行</button>`
    });
    m.root.querySelector('[data-cancel]').addEventListener('click', m.close);
    m.root.querySelector('[data-confirm]').addEventListener('click', async () => {
      m.close();
      const res = await OrbitAPI.executeClean(items);
      OrbitShell.toast(`AI 数据处理完成，释放 ${OrbitShell.fmt(res.freed)}`, 'sparkles');
      phase = 'idle';
      render(document.getElementById('viewSlot'));
    });
  }

  async function render(slot) {
    if (phase === 'result' && tools.length) return renderResult();
    slot.innerHTML = `
      <div class="scan-stage stagger">
        <div class="mascot-stage" style="--stage-hue:#9DB2D8">${OrbitMascots.cangjie('idle')}</div>
        <div class="num-big" style="font-size:30px">仓颉理卷，Agent 余屑分门别类</div>
        <p class="text-sub mt-8" style="max-width:520px;margin:10px auto 0;line-height:1.7">
          识别 Claude Code、Codex、Cursor 等工具的缓存与旧会话；缓存默认预选，会话与 CLI 默认保留，决定权在你。
        </p>
        <div class="mt-24"><button class="btn btn-primary btn-lg" data-action="scan">${I('sparkles')}扫描 AI 数据</button></div>
        <p class="text-muted text-12 mt-16">${I('lock', 'ico-sm')} 不读取会话内容，仅按目录元数据计量</p>
      </div>`;
    slot.querySelector('[data-action="scan"]').addEventListener('click', startScan);
  }

  return { render };
})();
