/* 视图：软件（software）—— 应用列表 + 残留抽屉 + 废纸篓优先卸载 */
OrbitShell.views.software = (() => {
  const I = OrbitShell.icon;
  let apps = [];
  let keepData = true;

  const typeMeta = {
    app: { label: '应用容器', cls: 'tag' },
    cache: { label: '缓存', cls: 'tag' },
    pref: { label: '偏好设置', cls: 'tag tag-muted' },
    daemon: { label: '后台项', cls: 'tag tag-warning' },
    data: { label: '个人数据', cls: 'tag tag-hue' }
  };

  async function render(slot) {
    if (!apps.length) {
      slot.innerHTML = `<div class="scan-stage"><div class="skeleton" style="height:56px;margin-bottom:12px"></div>
        <div class="skeleton" style="height:56px;margin-bottom:12px"></div>
        <div class="skeleton" style="height:56px"></div></div>`;
      apps = await OrbitAPI.listApps();
    }
    slot.innerHTML = `
      <div class="card stagger" style="padding:8px">
        ${apps.map((a) => `
          <div class="list-row" data-app="${a.id}">
            <span class="app-ic" style="background:${a.hue}">${a.initial}</span>
            <div class="row-main">
              <div class="row-title">${a.name} <span class="tag tag-muted">${a.developer}</span></div>
              <div class="row-sub">${I('clock', 'ico-sm')} 最后使用：${a.lastUsed}</div>
            </div>
            <span class="row-size">${OrbitShell.fmt(a.size)}</span>
            <button class="btn btn-danger btn-sm" data-uninstall="${a.id}">${I('trash', 'ico-sm')}卸载</button>
          </div>`).join('')}
      </div>
      <p class="text-muted text-12 mt-16" style="text-align:center">
        ${I('shieldCheck', 'ico-sm')} 卸载默认保留个人数据，应用与缓存移入废纸篓，可 Put Back 恢复
      </p>`;

    slot.querySelectorAll('[data-uninstall]').forEach((btn) => {
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        openDrawer(btn.dataset.uninstall);
      });
    });
    slot.querySelectorAll('[data-app]').forEach((row) => {
      row.addEventListener('click', () => openDrawer(row.dataset.app));
    });
  }

  function openDrawer(appId) {
    const app = apps.find((a) => a.id === appId);
    if (!app) return;
    keepData = true;
    const d = OrbitShell.drawer({
      title: `卸载 ${app.name}`,
      body: `
        <div class="flex items-center gap-12 mb-16">
          <span class="app-ic" style="background:${app.hue};width:52px;height:52px;font-size:20px">${app.initial}</span>
          <div>
            <div class="text-16" style="font-weight:700">${app.name}</div>
            <div class="text-sub text-12">${app.developer} · 占用 ${OrbitShell.fmt(app.residuals.reduce((s, r) => s + r.size, 0))}</div>
          </div>
        </div>
        <div class="setting-row" style="padding:10px 2px">
          <div class="sr-main">
            <div class="sr-title">保留个人数据</div>
            <div class="sr-desc">设置、登录态、模型与文档默认保留；关闭后将一并清理</div>
          </div>
          <button class="switch is-on" data-keep-switch aria-label="保留个人数据开关"><i></i></button>
        </div>
        <div class="divider"></div>
        <div class="text-12 text-muted mb-16">发现 ${app.residuals.length} 处关联内容</div>
        <div data-residual-list>
          ${app.residuals.map((r) => {
            const meta = typeMeta[r.type] || typeMeta.cache;
            const protectedByKeep = r.keep === true;
            return `
            <div class="list-row" data-residual="${r.type}" style="cursor:default;${protectedByKeep ? 'opacity:0.55' : ''}">
              <div class="row-main">
                <div class="row-title">${r.name} <span class="${meta.cls}" ${r.type === 'data' ? 'style="--tag-hue:#9DB2D8"' : ''}>${meta.label}</span></div>
                <div class="row-sub text-mono">${r.path}${r.note ? ` · ${r.note}` : ''}</div>
              </div>
              <span class="row-size">${OrbitShell.fmt(r.size)}</span>
              ${protectedByKeep ? `<span class="tag tag-muted" data-keep-tag>保留</span>` : ''}
            </div>`;
          }).join('')}
        </div>`,
      footer: `
        <button class="btn btn-ghost" data-cancel>取消</button>
        <button class="btn btn-danger" data-confirm>${I('trash')}移入废纸篓</button>`
    });

    const sw = d.root.querySelector('[data-keep-switch]');
    sw.addEventListener('click', () => {
      keepData = !keepData;
      sw.classList.toggle('is-on', keepData);
      d.root.querySelectorAll('[data-residual="data"]').forEach((row) => {
        row.style.opacity = keepData ? '0.55' : '1';
        const tag = row.querySelector('[data-keep-tag]');
        if (keepData && !tag) row.insertAdjacentHTML('beforeend', '<span class="tag tag-muted" data-keep-tag>保留</span>');
        if (!keepData && tag) tag.remove();
      });
    });
    d.root.querySelector('[data-cancel]').addEventListener('click', d.close);
    d.root.querySelector('[data-confirm]').addEventListener('click', () => confirmUninstall(app, d));
  }

  function confirmUninstall(app, drawerCtx) {
    const m = OrbitShell.modal({
      title: `确认卸载 ${app.name}`,
      body: `
        <div class="banner" style="margin-bottom:16px">
          ${I('alert')}
          <div class="banner-txt">
            <b>应用与${keepData ? '缓存、偏好设置' : '全部关联数据'}将移入废纸篓</b><br>
            <span>${keepData ? '个人数据已保留在原目录，重新安装可恢复使用' : '个人数据将一并清理，此操作难以恢复'}</span>
          </div>
        </div>
        <div class="preview-list">
          ${app.residuals.filter((r) => !(keepData && r.keep)).map((r) => `
            <div class="preview-item"><span class="pi-path">${r.path}</span><span class="pi-size">${OrbitShell.fmt(r.size)}</span></div>
          `).join('')}
        </div>`,
      footer: `<button class="btn btn-ghost" data-back>返回</button>
        <button class="btn btn-danger" data-do>${I('trash')}确认卸载</button>`
    });
    m.root.querySelector('[data-back]').addEventListener('click', m.close);
    m.root.querySelector('[data-do]').addEventListener('click', async () => {
      m.close();
      await OrbitAPI.uninstallApp(app.id, { keepData });
      drawerCtx.close();
      apps = apps.filter((a) => a.id !== app.id);
      OrbitShell.toast(`${app.name} 已移入废纸篓`, 'trash');
      render(document.getElementById('viewSlot'));
    });
  }

  return { render };
})();
