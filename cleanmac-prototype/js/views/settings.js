/* 视图：设置（settings）—— FDA 引导、开关、白名单、历史、关于 */
OrbitShell.views.settings = (() => {
  const I = OrbitShell.icon;
  let whitelist = [];
  let history = [];

  const switchRows = [
    { key: 'trashFirst', title: '废纸篓优先', desc: '用户数据与卸载默认移入废纸篓，仅可再生缓存执行永久删除' },
    { key: 'protectRunning', title: '运行中应用保护', desc: '缓存归属进程仍在运行、活动数据库与下载中文件一律跳过' },
    { key: 'zeroTelemetry', title: '零遥测模式', desc: '扫描内容与删除记录从不上传，诊断仅使用本地日志' },
    { key: 'updateCheck', title: '更新检查', desc: '仅访问 GitHub Release 元数据，可随时关闭' },
    { key: 'aiEntry', title: '显示 AI 数据入口', desc: '在侧栏展示 Claude Code / Codex / Cursor 等工具清理分组' },
    { key: 'externalVolumes', title: '扫描外接磁盘', desc: '默认关闭；开启后外接卷使用独立 .Trashes，不与启动卷混用' }
  ];

  async function render(slot) {
    if (!whitelist.length) whitelist = await OrbitAPI.getWhitelist();
    if (!history.length) history = await OrbitAPI.getHistory();
    const s = OrbitState.settings;

    slot.innerHTML = `
      ${s.fdaGranted ? '' : `
      <div class="banner">
        ${I('lock')}
        <div class="banner-txt">
          <b>尚未授予「完全磁盘访问权限」</b><br>
          <span>当前仅能扫描用户可访问目录，系统缓存与部分应用数据会被跳过。macOS 不会主动弹窗，需要你在系统设置中手动开启。</span>
        </div>
        <button class="btn btn-ghost btn-sm" data-fda>${I('external', 'ico-sm')}打开系统设置</button>
      </div>`}

      <div class="grid grid-2 stagger">
        <div class="card">
          <div class="card-title mb-16" style="margin-bottom:14px">${I('settings', 'ico-sm')}通用与安全</div>
          ${switchRows.map((r) => `
            <div class="setting-row">
              <div class="sr-main">
                <div class="sr-title">${r.title}</div>
                <div class="sr-desc">${r.desc}</div>
              </div>
              <button class="switch ${s[r.key] ? 'is-on' : ''}" data-switch="${r.key}" aria-label="${r.title}"><i></i></button>
            </div>`).join('')}
        </div>

        <div>
          <div class="card mb-16">
            <div class="card-title" style="margin-bottom:14px">${I('shield', 'ico-sm')}保护白名单</div>
            <div class="text-12 text-muted" style="margin-bottom:10px;line-height:1.6">名单内路径永不参与删除；名单文件读取失败时应用将 fail-closed 停止清理</div>
            <div data-wl-list>
              ${whitelist.map((w) => `
                <div class="list-row" style="padding:9px 6px;cursor:default">
                  <div class="row-main">
                    <div class="row-title text-13">${w.name}</div>
                    <div class="row-sub text-mono">${w.path}</div>
                  </div>
                  <button class="icon-btn" data-wl-del="${w.id}" aria-label="移除">${I('x')}</button>
                </div>`).join('')}
            </div>
            <div class="flex gap-8 mt-16">
              <input class="card" data-wl-name placeholder="名称，如：银行 App 数据" style="flex:1;height:38px;background:var(--surface-2);border:1px solid var(--border);border-radius:99px;padding:0 14px;color:var(--text);outline:none" />
              <input class="card" data-wl-path placeholder="路径，如：~/Library/..." style="flex:1.4;height:38px;background:var(--surface-2);border:1px solid var(--border);border-radius:99px;padding:0 14px;color:var(--text);outline:none;font-family:'SF Mono',Menlo,monospace;font-size:12px" />
              <button class="btn btn-primary btn-sm" data-wl-add>${I('plus', 'ico-sm')}添加</button>
            </div>
          </div>

          <div class="card">
            <div class="card-title" style="margin-bottom:12px">${I('clock', 'ico-sm')}历史记录</div>
            ${history.map((h) => `
              <div class="setting-row" style="padding:11px 2px">
                <div class="sr-main">
                  <div class="sr-title">${h.type} <span class="tag tag-muted" style="margin-left:6px">${h.time}</span></div>
                  <div class="sr-desc">${h.detail}</div>
                </div>
                <span class="row-size" style="color:var(--primary)">${OrbitShell.fmt(h.freed)}</span>
              </div>`).join('')}
          </div>
        </div>
      </div>

      <div class="card mt-16 flex items-center justify-between">
        <div class="flex items-center gap-16">
          <div class="brand-mark" style="width:42px;height:42px;font-size:18px">曜</div>
          <div>
            <div class="row-title">清曜 <span class="tag tag-muted">v0.1.0 原型</span></div>
            <div class="row-sub">免费开源 · arm64（Apple 芯片）· macOS 26+ · GitHub 与官网分发，不上 App Store</div>
          </div>
        </div>
        <div class="flex gap-8">
          <a class="btn btn-ghost btn-sm" href="https://github.com/" target="_blank" rel="noopener">${I('github', 'ico-sm')}GitHub</a>
          <a class="btn btn-ghost btn-sm" href="#/clean">${I('rotate', 'ico-sm')}开始清理</a>
        </div>
      </div>`;

    // FDA 引导
    const fdaBtn = slot.querySelector('[data-fda]');
    if (fdaBtn) fdaBtn.addEventListener('click', async () => {
      await OrbitAPI.openFdaSettings();
      OrbitState.setSetting('fdaGranted', true);
      OrbitShell.toast('已跳转系统设置，授权后返回清曜即可', 'shieldCheck');
      render(slot);
    });

    // 开关
    slot.querySelectorAll('[data-switch]').forEach((btn) => {
      btn.addEventListener('click', () => {
        const k = btn.dataset.switch;
        OrbitState.setSetting(k, !OrbitState.settings[k]);
        btn.classList.toggle('is-on', OrbitState.settings[k]);
      });
    });

    // 白名单删除
    slot.querySelectorAll('[data-wl-del]').forEach((btn) => {
      btn.addEventListener('click', () => {
        whitelist = whitelist.filter((w) => w.id !== btn.dataset.wlDel);
        render(slot);
      });
    });

    // 白名单新增
    slot.querySelector('[data-wl-add]').addEventListener('click', () => {
      const name = slot.querySelector('[data-wl-name]').value.trim();
      const path = slot.querySelector('[data-wl-path]').value.trim();
      if (!name || !path) { OrbitShell.toast('请填写名称与路径', 'alert'); return; }
      whitelist.push({ id: `w${Date.now()}`, name, path });
      OrbitShell.toast(`已添加保护项：${name}`);
      render(slot);
    });
  }

  return { render };
})();
