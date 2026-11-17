/* Orbit 外壳与路由：导航只此一处定义，视图注册到 views，禁止自造导航 */
const OrbitShell = (() => {
  const NAV = [
    { key: 'clean', label: '清理', icon: 'droplet', hue: '#4AE3C4', title: '清理 · 貔貅', desc: '貔貅吞纳百秽，缓存与日志一扫而空，磁盘重回清爽' },
    { key: 'ai', label: 'AI 数据', icon: 'sparkles', hue: '#9DB2D8', title: 'AI 数据 · 仓颉', desc: '仓颉造字理卷，Agent 的缓存、旧会话与闲置 CLI 逐项过目' },
    { key: 'software', label: '软件', icon: 'box', hue: '#F2785C', title: '软件 · 钟馗', desc: '钟馗送客出户，卸载之前，先看清每一处残留' },
    { key: 'analyze', label: '分析', icon: 'grid', hue: '#E8B36A', title: '分析 · 千里眼', desc: '千里眼明察秋毫，整盘树图让最大的文件无处遁形' },
    { key: 'status', label: '状态', icon: 'activity', hue: '#F5C451', title: '状态 · 顺风耳', desc: '顺风耳谛听脉动，只看不动，实时掌握 Mac 的脉象' },
    { key: 'settings', label: '设置', icon: 'settings', hue: '#8A93A8', title: '设置 · 女娲', desc: '女娲补天立极，权限、白名单与隐私边界由你执掌' }
  ];
  const views = {};
  let currentKey = null;
  const THEME_KEY = 'orbit.prototype.theme';

  function currentTheme() {
    return document.documentElement.dataset.theme === 'light' ? 'light' : 'dark';
  }

  function renderThemeToggle() {
    const side = document.getElementById('titlebarSide');
    const theme = currentTheme();
    side.innerHTML = `
      <button class="theme-toggle" data-theme-toggle aria-label="切换深浅色主题" title="${theme === 'light' ? '切换到深色' : '切换到浅色'}">
        ${OrbitIcons.svg(theme === 'light' ? 'moon' : 'sun')}
      </button>`;
    side.querySelector('[data-theme-toggle]').addEventListener('click', () => {
      const next = currentTheme() === 'light' ? 'dark' : 'light';
      document.documentElement.dataset.theme = next;
      try { localStorage.setItem(THEME_KEY, next); } catch (e) {}
      renderThemeToggle();
    });
  }

  function renderNav() {
    document.getElementById('appNav').innerHTML = `
      <div class="nav-brand">
        <div class="brand-mark">曜</div>
        <div>
          <div class="brand-name">清曜</div>
          <div class="brand-sub">Mac 空间清理</div>
        </div>
      </div>
      ${NAV.map((n) => `
        <a class="nav-item" data-nav="${n.key}" href="#/${n.key}" style="--dot:${n.hue}">
          ${OrbitIcons.svg(n.icon, 'nav-ico')}
          <span>${n.label}</span>
          <span class="orb-dot"></span>
        </a>`).join('')}
      <div class="nav-footer">
        <div class="disk-line"><span>磁盘可用</span><span class="text-mono">48 GB</span></div>
        <div class="disk-bar"><i style="width:89%"></i></div>
        <div class="disk-line mt-8" style="margin-top:8px"><span>零遥测 · 本地运行</span></div>
      </div>`;
  }

  function head(nav) {
    return `
      <div class="page-head" style="--page-hue:${nav.hue}">
        <div>
          <div class="page-title"><span class="title-orb"></span>${nav.title}</div>
          <div class="page-desc">${nav.desc}</div>
        </div>
        <div class="page-actions" data-page-actions></div>
      </div>`;
  }

  async function route() {
    const key = (location.hash.replace('#/', '') || 'clean').split('?')[0];
    const nav = NAV.find((n) => n.key === key) || NAV[0];
    currentKey = nav.key;
    document.querySelectorAll('.nav-item').forEach((el) =>
      el.classList.toggle('active', el.dataset.nav === nav.key));
    document.getElementById('windowTitle').textContent = `清曜 — ${nav.label}`;
    const root = document.getElementById('appContent');
    root.innerHTML = head(nav) + `<div id="viewSlot"></div>`;
    root.scrollTop = 0;
    const view = views[nav.key];
    if (view?.render) await view.render(document.getElementById('viewSlot'));
  }

  /* ---------- 全局浮层 ---------- */
  function modal({ title, body, footer, onClose }) {
    const root = document.getElementById('modalRoot');
    root.innerHTML = `
      <div class="overlay" data-close>
        <div class="modal" role="dialog" aria-modal="true">
          <div class="modal-head">
            <div class="modal-title">${title}</div>
            <button class="icon-btn" data-close aria-label="关闭">${OrbitIcons.svg('x')}</button>
          </div>
          <div class="modal-body">${body}</div>
          ${footer ? `<div class="modal-foot">${footer}</div>` : ''}
        </div>
      </div>`;
    const close = () => { root.innerHTML = ''; onClose?.(); };
    root.querySelectorAll('[data-close]').forEach((el) =>
      el.addEventListener('click', (e) => { if (e.target === el || el.hasAttribute('data-close') && el.tagName === 'BUTTON') close(); }));
    return { close, root };
  }

  function drawer({ title, body, footer }) {
    const dRoot = document.getElementById('drawerRoot');
    dRoot.innerHTML = `<div class="drawer-overlay" data-dclose></div>
      <aside class="drawer" role="dialog">
        <div class="modal-head">
          <div class="modal-title">${title}</div>
          <button class="icon-btn" data-dclose>${OrbitIcons.svg('x')}</button>
        </div>
        <div class="drawer-body">${body}</div>
        ${footer ? `<div class="modal-foot">${footer}</div>` : ''}
      </aside>`;
    const close = () => { dRoot.innerHTML = ''; };
    dRoot.querySelectorAll('[data-dclose]').forEach((el) => el.addEventListener('click', close));
    return { close, root: dRoot };
  }

  function toast(msg, icon = 'check') {
    const wrap = document.getElementById('toastRoot');
    const el = document.createElement('div');
    el.className = 'toast';
    el.innerHTML = `${OrbitIcons.svg(icon)}<span>${msg}</span>`;
    wrap.appendChild(el);
    setTimeout(() => { el.style.opacity = '0'; el.style.transform = 'translateY(8px)'; el.style.transition = 'all 220ms'; }, 2600);
    setTimeout(() => el.remove(), 2900);
  }

  return {
    NAV, views,
    boot() {
      renderNav();
      renderThemeToggle();
      window.addEventListener('hashchange', route);
      route();
    },
    route, current: () => currentKey,
    head, modal, drawer, toast,
    icon: (n, c) => OrbitIcons.svg(n, c),
    fmt: (n) => OrbitState.fmtBytes(n)
  };
})();
