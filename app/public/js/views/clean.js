/* 视图：清理（clean）—— 扫描 → 分组勾选 → dry-run 预览 → 执行 → 完成 */
OrbitShell.views.clean = (() => {
  const I = OrbitShell.icon;
  let groups = [];
  let phase = 'idle'; // idle | scanning | result | done
  let lastFreed = 0;
  let lastResult = null;

  function hero(slot, sub, btnText) {
    slot.innerHTML = `
      <div class="scan-stage stagger">
        <div class="mascot-stage" style="--stage-hue:#4AE3C4">${OrbitMascots.pixiu('idle')}</div>
        <div class="num-big" style="font-size:30px">貔貅吞秽，清出一方天地</div>
        <p class="text-sub mt-8" style="max-width:520px;margin:10px auto 0;line-height:1.7">
          ${sub}
        </p>
        <div class="mt-24 flex items-center" style="justify-content:center;gap:12px">
          <button class="btn btn-primary btn-lg" data-action="scan">${I('refresh')}${btnText}</button>
        </div>
        <p class="text-muted text-12 mt-16">${I('shieldCheck', 'ico-sm')} 全程只读扫描，执行前可逐项预览，缓存类删除可重建</p>
      </div>`;
    slot.querySelector('[data-action="scan"]').addEventListener('click', startScan);
  }

  async function startScan() {
    phase = 'scanning';
    const slot = document.getElementById('viewSlot');
    slot.innerHTML = `
      <div class="scan-stage">
        <div class="mascot-stage" style="--stage-hue:#4AE3C4">${OrbitMascots.pixiu('working')}</div>
        <div class="text-16" style="font-weight:650" data-stage>正在启动扫描引擎…</div>
        <div class="scan-log" data-log></div>
      </div>`;
    const logEl = slot.querySelector('[data-log]');
    groups = await OrbitAPI.scanCleanGroups((stage) => {
      slot.querySelector('[data-stage]').textContent = stage;
      logEl.innerHTML += `<div><span class="ok">✓</span> ${stage}</div>`;
    });
    phase = 'result';
    renderResult();
  }

  function riskTag(item) {
    if (item.risk === 'danger') return `<span class="tag tag-danger">永久删除</span>`;
    if (item.risk === 'warn') return `<span class="tag tag-warning">需留意</span>`;
    return `<span class="tag">安全可重建</span>`;
  }

  function renderResult() {
    const slot = document.getElementById('viewSlot');
    slot.innerHTML = `
      <div class="stagger">
        ${groups.map((g) => {
          const st = OrbitState.groupState(g.items);
          return `
          <div class="card mb-16" data-group="${g.id}" style="--tag-hue:${g.hue}">
            <div class="flex items-center justify-between">
              <div class="row-check" data-toggle-group>
                <span class="checkbox ${st === 'all' ? 'is-checked' : st === 'partial' ? 'is-partial' : ''}" data-group-check>
                  ${I('check')}
                </span>
                <div>
                  <div class="card-title">${I(g.icon, 'ico-sm')}${g.name}
                    <span class="tag tag-hue">${g.items.length} 项 · ${OrbitShell.fmt(OrbitState.sumAll(g.items))}</span>
                  </div>
                  <div class="card-sub">${g.desc}</div>
                </div>
              </div>
              <button class="btn btn-ghost btn-sm" data-expand>${I('chevronDown', 'ico-sm')}明细</button>
            </div>
            <div class="divider"></div>
            <div data-item-list>
              ${g.items.map((it) => `
                <label class="list-row" style="cursor:pointer">
                  <span class="checkbox ${it.selected ? 'is-checked' : ''}" data-item-check="${it.id}">${I('check')}</span>
                  <div class="row-main">
                    <div class="row-title">${it.name} ${riskTag(it)}</div>
                    <div class="row-sub text-mono">${it.path}${it.note ? ` · ${it.note}` : ''}</div>
                  </div>
                  <span class="row-size">${OrbitShell.fmt(it.size)}</span>
                </label>`).join('')}
            </div>
          </div>`;
        }).join('')}
        <div class="card sticky-bar flex items-center justify-between">
          <div>
            <div class="text-13 text-sub">已选择 <b data-sel-count style="color:var(--text)">0</b> 项，预计释放</div>
            <div class="num-big" style="font-size:30px;color:var(--primary)" data-sel-size>0 GB</div>
          </div>
          <div class="flex gap-12">
            <button class="btn btn-ghost" data-action="rescan">${I('refresh')}重新扫描</button>
            <button class="btn btn-primary btn-lg" data-action="preview">${I('eye')}预览删除</button>
          </div>
        </div>
      </div>`;
    bindResult(slot);
    refreshSummary();
  }

  function bindResult(slot) {
    slot.querySelectorAll('[data-group]').forEach((card) => {
      const g = groups.find((x) => x.id === card.dataset.group);
      card.querySelector('[data-toggle-group]').addEventListener('click', () => {
        const st = OrbitState.groupState(g.items);
        const target = st !== 'all';
        g.items.forEach((it) => { it.selected = target; });
        renderResult();
      });
      card.querySelectorAll('[data-item-check]').forEach((box) => {
        box.closest('label').addEventListener('click', (e) => {
          e.preventDefault();
          const it = g.items.find((x) => x.id === box.dataset.itemCheck);
          it.selected = !it.selected;
          renderResult();
        });
      });
      const expandBtn = card.querySelector('[data-expand]');
      const list = card.querySelector('[data-item-list]');
      expandBtn.addEventListener('click', () => {
        const hidden = list.style.display === 'none';
        list.style.display = hidden ? '' : 'none';
        expandBtn.innerHTML = `${I(hidden ? 'chevronDown' : 'chevronRight', 'ico-sm')}明细`;
      });
    });
    slot.querySelector('[data-action="rescan"]').addEventListener('click', startScan);
    slot.querySelector('[data-action="preview"]').addEventListener('click', openPreview);
  }

  function selectedItems() {
    return groups.flatMap((g) => g.items.filter((i) => i.selected).map((i) => ({ ...i, group: g.name })));
  }

  function refreshSummary() {
    const items = selectedItems();
    const countEl = document.querySelector('[data-sel-count]');
    const sizeEl = document.querySelector('[data-sel-size]');
    if (!countEl) return;
    countEl.textContent = items.length;
    sizeEl.textContent = OrbitShell.fmt(items.reduce((s, i) => s + i.size, 0));
    const btn = document.querySelector('[data-action="preview"]');
    btn.disabled = items.length === 0;
    btn.classList.toggle('is-disabled', items.length === 0);
  }

  function openPreview() {
    const items = selectedItems();
    if (!items.length) return;
    const permanent = items.some((i) => i.risk === 'danger');
    const m = OrbitShell.modal({
      title: '删除预览 · dry-run',
      body: `
        <div class="banner" style="margin-bottom:16px">
          ${I(permanent ? 'alert' : 'info')}
          <div class="banner-txt">
            <b>${permanent ? '本次包含永久删除项（废纸篓）' : '默认移入废纸篓，可在 Finder 中 Put Back 恢复'}</b><br>
            <span>以下清单与实际执行共用同一条安全决策路径，路径已通过符号链接与系统保护校验</span>
          </div>
        </div>
        <div class="preview-list">
          ${items.map((i) => `
            <div class="preview-item">
              <span class="tag tag-hue" style="width:64px;--tag-hue:${(groups.find((g) => g.name === i.group) || {}).hue || '#4AE3C4'}">${i.group}</span>
              <span class="pi-path">${i.path}</span>
              <span class="pi-size">${OrbitShell.fmt(i.size)}</span>
            </div>`).join('')}
        </div>
        <div class="flex items-center justify-between mt-16">
          <span class="text-sub text-13">共 ${items.length} 项</span>
          <span class="num-big" style="font-size:24px;color:var(--primary)">${OrbitShell.fmt(items.reduce((s, i) => s + i.size, 0))}</span>
        </div>`,
      footer: `
        <button class="btn btn-ghost" data-cancel>取消</button>
        <button class="btn btn-primary" data-confirm>${I('trash')}确认执行</button>`
    });
    m.root.querySelector('[data-cancel]').addEventListener('click', m.close);
    m.root.querySelector('[data-confirm]').addEventListener('click', async () => {
      m.close();
      const res = await OrbitAPI.executeClean(items);
      lastResult = res;
      lastFreed = res.freed;
      phase = 'done';
      renderDone();
    });
  }

  function renderDone() {
    const slot = document.getElementById('viewSlot');
    const skipped = lastResult && lastResult.performed === false;
    const skipMsg = (lastResult && lastResult.message) || '';
    slot.innerHTML = `
      <div class="scan-stage stagger">
        <div class="mascot-stage" style="--stage-hue:${skipped ? '#E8B36A' : '#4AE3C4'}">${OrbitMascots.pixiu(skipped ? 'idle' : 'happy')}</div>
        <div class="text-sub text-13">${skipped ? '写操作已跳过（开发模式）' : '本次已释放'}</div>
        <div class="num-big" style="color:var(--primary);font-size:52px">${skipped ? '未执行' : OrbitShell.fmt(lastFreed)}</div>
        ${skipped ? `<div class="banner" style="margin:16px auto 0;max-width:520px;text-align:left">
          ${I('alert')}<div class="banner-txt"><b>清理未实际写入磁盘</b><br><span>${skipMsg || '当前构建已禁用写操作，未对磁盘做任何修改'}</span></div>
        </div>` : `<p class="text-muted text-12 mt-8">操作记录已写入本地审计日志，可在「设置 → 历史记录」回溯</p>`}
        <div class="mt-24 flex gap-12" style="justify-content:center">
          <button class="btn btn-ghost btn-lg" data-action="home">返回</button>
          <button class="btn btn-primary btn-lg" data-action="again">${I('refresh')}再扫一次</button>
        </div>
      </div>`;
    OrbitShell.toast(skipped ? '已跳过，待审阅后手动执行' : `清理完成，释放 ${OrbitShell.fmt(lastFreed)}`);
    slot.querySelector('[data-action="home"]').addEventListener('click', () => { phase = 'idle'; render(slot); });
    slot.querySelector('[data-action="again"]').addEventListener('click', startScan);
  }

  async function render(slot) {
    if (phase === 'result' && groups.length) return renderResult();
    if (phase === 'done') return renderDone();
    hero(slot, '扫描系统缓存、浏览器数据、开发者工具与废纸篓；运行中的应用、活动数据库与下载中的文件会被自动保护。', '开始扫描');
  }

  return { render };
})();
