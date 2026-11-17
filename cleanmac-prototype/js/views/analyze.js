/* 视图：分析（analyze · 千里眼）—— 旭日环 + 排名清单 + 显式操作，替代原树图
   交互约定：点扇区/清单行进入下一层；点中心或面包屑返回；保护目录只读、可用空间不可删 */
OrbitShell.views.analyze = (() => {
  const I = OrbitShell.icon;
  let root = null;
  const stack = [];
  const PALETTE = ['#E8B36A', '#F2785C', '#9DB2D8', '#4AE3C4', '#F5C451', '#8A93A8', '#7DD3FC', '#C48BD6'];
  const CX = 170, CY = 170, R_IN = 60, R_MID = 106, R_OUT = 150;

  function colorOf(item, idx) {
    if (item.free) return 'rgba(74,227,196,0.45)';
    if (item.protected) return '#3A4358';
    return PALETTE[idx % PALETTE.length];
  }
  const tint = (hex) => (hex.startsWith('#') ? hex + '99' : hex);

  const polar = (r, deg) => {
    const a = ((deg - 90) * Math.PI) / 180;
    return [CX + r * Math.cos(a), CY + r * Math.sin(a)];
  };
  function arcPath(rIn, rOut, a0, a1) {
    const [x0, y0] = polar(rOut, a0), [x1, y1] = polar(rOut, a1);
    const [x2, y2] = polar(rIn, a1), [x3, y3] = polar(rIn, a0);
    const large = a1 - a0 > 180 ? 1 : 0;
    return `M${x0.toFixed(2)} ${y0.toFixed(2)} A${rOut} ${rOut} 0 ${large} 1 ${x1.toFixed(2)} ${y1.toFixed(2)} L${x2.toFixed(2)} ${y2.toFixed(2)} A${rIn} ${rIn} 0 ${large} 0 ${x3.toFixed(2)} ${y3.toFixed(2)} Z`;
  }

  function buildSunburst(node) {
    const children = node.children || [];
    const total = children.reduce((s, c) => s + c.size, 0) || 1;
    let angle = 0;
    const inner = [], outer = [], labels = [];
    children.forEach((child, idx) => {
      const span = (child.size / total) * 360;
      const a0 = angle, a1 = angle + span;
      const fill = colorOf(child, idx);
      const drillable = !!child.children && !child.protected && !child.free;
      inner.push({ item: child, d: arcPath(R_IN, R_MID, a0, a1), fill, drillable, idx });
      if (span >= 13) {
        const [lx, ly] = polar((R_IN + R_MID) / 2, (a0 + a1) / 2);
        labels.push({ x: lx, y: ly, text: child.name, span });
      }
      // 外环：有子项按孙项切分，否则用淡色补齐保持环完整
      if (child.children && child.children.length) {
        let subAngle = a0;
        const subTotal = child.children.reduce((s, g) => s + g.size, 0) || 1;
        child.children.forEach((g, gIdx) => {
          const gSpan = (g.size / subTotal) * span;
          outer.push({ item: g, parent: child, d: arcPath(R_MID, R_OUT, subAngle, subAngle + gSpan), fill: tint(fill), drillable: !!g.children, childIdx: idx, grandIdx: gIdx });
          subAngle += gSpan;
        });
      } else {
        outer.push({ item: child, parent: child, d: arcPath(R_MID, R_OUT, a0, a1), fill: tint(fill), drillable, childIdx: idx, grandIdx: null });
      }
      angle = a1;
    });
    return { inner, outer, labels };
  }

  function typeIcon(item) {
    if (item.protected) return 'lock';
    if (item.free) return 'drive';
    return item.children ? 'folder' : 'file';
  }

  async function render(slot) {
    if (!root) {
      slot.innerHTML = `<div class="scan-stage"><div class="skeleton" style="height:420px"></div></div>`;
      root = await OrbitAPI.analyzeDisk();
      stack.push(root);
    }
    const node = stack[stack.length - 1];
    const children = [...(node.children || [])].sort((a, b) => b.size - a.size);
    const total = children.reduce((s, c) => s + c.size, 0) || 1;
    const maxSize = children[0]?.size || 1;
    const reclaimable = children.filter((c) => !c.protected && !c.free).reduce((s, c) => s + c.size, 0);
    const { inner, outer, labels } = buildSunburst(node);
    const canUp = stack.length > 1;

    slot.innerHTML = `
      <div class="card mb-16 flex items-center justify-between" style="padding:14px 18px">
        <div class="breadcrumb" data-crumbs>
          ${stack.map((n, idx) => `${idx > 0 ? '<span class="sep">/</span>' : ''}<button data-crumb="${idx}">${n.name}</button>`).join('')}
        </div>
        <div class="flex items-center gap-8">
          <button class="btn btn-ghost btn-sm" data-up ${canUp ? '' : 'disabled'}>${I('arrowLeft', 'ico-sm')}返回上层</button>
        </div>
      </div>

      <div class="analyze-layout">
        <div class="card sunburst-card">
          <div class="card-title" style="margin-bottom:6px">${I('pie', 'ico-sm')}占用全景</div>
          <div class="card-sub" style="margin-bottom:10px">扇区角度 = 占用比例；内环为本级项目，外环为其下一层</div>
          <svg viewBox="0 0 340 340" class="sunburst" role="img" aria-label="磁盘占用旭日环">
            ${outer.map((s) => `
              <path class="sb-arc ${s.drillable ? '' : 'is-leaf'}" d="${s.d}" fill="${s.fill}" ${s.drillable ? `data-outer="${s.childIdx},${s.grandIdx}"` : ''}>
                <title>${s.item.name} · ${OrbitShell.fmt(s.item.size)}${s.item.protected ? '（系统保护，只读）' : ''}</title>
              </path>`).join('')}
            ${inner.map((s) => `
              <path class="sb-arc ${s.drillable ? '' : 'is-leaf'}" d="${s.d}" fill="${s.fill}" data-drill="${s.drillable ? s.idx : ''}">
                <title>${s.item.name} · ${OrbitShell.fmt(s.item.size)}（${(s.item.size / total * 100).toFixed(1)}%）${s.item.protected ? ' · 系统保护' : ''}</title>
              </path>`).join('')}
            ${labels.map((l) => `<text class="sb-label" x="${l.x.toFixed(1)}" y="${l.y.toFixed(1)}" text-anchor="middle">${l.text.length > 6 ? l.text.slice(0, 5) + '…' : l.text}</text>`).join('')}
            <g class="sb-center ${canUp ? '' : 'is-root'}" data-center>
              <circle cx="${CX}" cy="${CY}" r="${R_IN - 4}" class="sb-center-disc"/>
              <text x="${CX}" y="${CY - 14}" class="sb-center-name">${node.name.length > 9 ? node.name.slice(0, 8) + '…' : node.name}</text>
              <text x="${CX}" y="${CY + 8}" class="sb-center-size">${OrbitShell.fmt(node.size)}</text>
              <text x="${CX}" y="${CY + 28}" class="sb-center-hint">${canUp ? '点击返回上层' : '根目录'}</text>
            </g>
          </svg>
          <div class="flex items-center gap-16 mt-8 text-12 text-muted" style="flex-wrap:wrap">
            <span class="flex items-center gap-8"><i class="sb-dot" style="background:#3A4358"></i>系统保护（只读）</span>
            <span class="flex items-center gap-8"><i class="sb-dot" style="background:rgba(74,227,196,0.45)"></i>可用空间</span>
            <span class="flex items-center gap-8">${I('info', 'ico-sm')}点击彩色扇区进入下一层</span>
          </div>
        </div>

        <div class="card inspector-card">
          <div class="flex items-center justify-between" style="margin-bottom:14px">
            <div class="card-title">${I('layers', 'ico-sm')}本级项目（按大小排序）</div>
            <span class="tag tag-hue" style="--tag-hue:#E8B36A">可回收 ${OrbitShell.fmt(reclaimable)}</span>
          </div>
          <div class="inspector-list">
            ${children.map((item, idx) => {
              const pct = (item.size / total) * 100;
              const barW = Math.max(3, (item.size / maxSize) * 100);
              const drillable = !!item.children && !item.protected && !item.free;
              const deletable = !item.protected && !item.free;
              return `
              <div class="inspector-row" data-row="${idx}">
                <div class="ir-main" data-row-drill="${drillable ? idx : ''}">
                  <div class="ir-top">
                    <span class="ir-rank">${idx + 1}</span>
                    <span class="ir-ico" style="color:${colorOf(item, idx)}">${I(typeIcon(item), 'ico-sm')}</span>
                    <span class="ir-name">${item.name}</span>
                    ${item.protected ? '<span class="tag tag-muted">系统保护</span>' : ''}
                    ${item.free ? '<span class="tag">可用空间</span>' : ''}
                    <span class="ir-size text-mono">${OrbitShell.fmt(item.size)}</span>
                    <span class="ir-pct text-mono">${pct.toFixed(1)}%</span>
                  </div>
                  <div class="prop-bar"><i style="width:${barW}%;background:${colorOf(item, idx)}"></i></div>
                  <div class="ir-path text-mono">${item.path || '（目录）'}${drillable ? ' · 点击进入' : ''}</div>
                </div>
                <div class="ir-actions">
                  ${deletable ? `<button class="icon-btn" data-trash="${idx}" aria-label="移到废纸篓" title="移到废纸篓">${I('trash')}</button>` : ''}
                  ${drillable ? `<button class="icon-btn" data-go="${idx}" aria-label="进入目录" title="进入目录">${I('chevronRight')}</button>` : ''}
                </div>
              </div>`;
            }).join('')}
          </div>
        </div>
      </div>`;

    // 面包屑与返回
    slot.querySelectorAll('[data-crumb]').forEach((b) => b.addEventListener('click', () => {
      stack.splice(Number(b.dataset.crumb) + 1);
      render(slot);
    }));
    const upBtn = slot.querySelector('[data-up]');
    if (canUp) upBtn.addEventListener('click', () => { stack.pop(); render(slot); });
    const center = slot.querySelector('[data-center]');
    if (canUp) center.addEventListener('click', () => { stack.pop(); render(slot); });

    // 扇区下钻
    slot.querySelectorAll('path.sb-arc[data-drill]').forEach((p) => {
      p.addEventListener('click', () => {
        if (p.dataset.drill === '') return;
        const child = children[Number(p.dataset.drill)];
        if (child?.children) { stack.push(child); render(slot); }
      });
    });
    slot.querySelectorAll('path.sb-arc[data-outer]').forEach((p) => {
      p.addEventListener('click', () => {
        const [ci, gi] = p.dataset.outer.split(',').map(Number);
        const target = gi === null || Number.isNaN(gi) ? children[ci] : children[ci]?.children?.[gi];
        if (target?.children) { stack.push(target); render(slot); }
      });
    });

    // 清单行下钻与删除
    slot.querySelectorAll('[data-row-drill]').forEach((el) => {
      if (el.dataset.rowDrill === '') return;
      el.addEventListener('click', () => {
        const child = children[Number(el.dataset.rowDrill)];
        if (child?.children) { stack.push(child); render(slot); }
      });
    });
    slot.querySelectorAll('[data-go]').forEach((b) => b.addEventListener('click', (e) => {
      e.stopPropagation();
      const child = children[Number(b.dataset.go)];
      if (child?.children) { stack.push(child); render(slot); }
    }));
    slot.querySelectorAll('[data-trash]').forEach((b) => b.addEventListener('click', (e) => {
      e.stopPropagation();
      confirmTrash(children[Number(b.dataset.trash)]);
    }));
  }

  function confirmTrash(item) {
    const m = OrbitShell.modal({
      title: '移到废纸篓',
      body: `
        <div class="banner" style="margin-bottom:16px">
          ${I('info')}
          <div class="banner-txt">
            <b>${item.name}（${OrbitShell.fmt(item.size)}）</b><br>
            <span>将通过 NSFileManager 废纸篓接口移动，可在 Finder 中 Put Back 恢复；路径已通过安全漏斗校验</span>
          </div>
        </div>
        <div class="preview-list"><div class="preview-item"><span class="pi-path">${item.path || '（目录）'}</span><span class="pi-size">${OrbitShell.fmt(item.size)}</span></div></div>`,
      footer: `<button class="btn btn-ghost" data-cancel>取消</button>
        <button class="btn btn-primary" data-confirm>${I('trash')}移到废纸篓</button>`
    });
    m.root.querySelector('[data-cancel]').addEventListener('click', m.close);
    m.root.querySelector('[data-confirm]').addEventListener('click', async () => {
      m.close();
      if (!item.path) { OrbitShell.toast('原型演示数据不含真实路径，已模拟操作', 'info'); return; }
      await OrbitAPI.moveToTrash(item.path);
      OrbitShell.toast(`${item.name} 已移到废纸篓`, 'trash');
    });
  }

  return { render };
})();
