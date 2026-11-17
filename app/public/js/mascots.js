/* 清曜 漫画形象库：Q 版内联 SVG，统一描边语言（圆角线帽/3.5px 主描边），零图片依赖、双主题自适应
   形象规范见 design-contract.md「漫画形象规范」；只允许通过 mood 切换表情，不得在视图中临时改形 */
const OrbitMascots = (() => {
  const OPEN = 'stroke-linecap="round" stroke-linejoin="round"';

  /* ---------- 貔貅（清理）：mood = idle | working | happy ---------- */
  function pixiu(mood = 'idle') {
    const eyeNormal = `
      <g class="m-eye">
        <circle cx="92" cy="92" r="11" fill="#FFFFFF" stroke="var(--m-stroke)" stroke-width="3" ${OPEN}/>
        <circle cx="94" cy="93" r="5.2" fill="var(--m-stroke)"/>
        <circle cx="96" cy="90.5" r="1.8" fill="#FFFFFF"/>
      </g>
      <g class="m-eye">
        <circle cx="128" cy="92" r="11" fill="#FFFFFF" stroke="var(--m-stroke)" stroke-width="3" ${OPEN}/>
        <circle cx="130" cy="93" r="5.2" fill="var(--m-stroke)"/>
        <circle cx="132" cy="90.5" r="1.8" fill="#FFFFFF"/>
      </g>`;
    const eyeHappy = `
      <path class="m-eye-happy" d="M82 94 q10 -12 20 0" fill="none" stroke="var(--m-stroke)" stroke-width="3.6" ${OPEN}/>
      <path class="m-eye-happy" d="M118 94 q10 -12 20 0" fill="none" stroke="var(--m-stroke)" stroke-width="3.6" ${OPEN}/>`;
    const browIdle = `
      <path d="M82 76 q10 -5 20 0" fill="none" stroke="var(--m-stroke)" stroke-width="3" ${OPEN}/>
      <path d="M118 76 q10 -5 20 0" fill="none" stroke="var(--m-stroke)" stroke-width="3" ${OPEN}/>`;
    const browWork = `
      <path d="M82 78 l18 5" fill="none" stroke="var(--m-stroke)" stroke-width="3" ${OPEN}/>
      <path d="M138 78 l-18 5" fill="none" stroke="var(--m-stroke)" stroke-width="3" ${OPEN}/>`;
    const mouthIdle = `
      <path d="M100 116 q10 8 20 0" fill="none" stroke="var(--m-stroke)" stroke-width="3" ${OPEN}/>
      <path d="M101 115 l3 7 l3 -5 Z" fill="#FFFFFF" stroke="var(--m-stroke)" stroke-width="1.6"/>
      <path d="M119 115 l-3 7 l-3 -5 Z" fill="#FFFFFF" stroke="var(--m-stroke)" stroke-width="1.6"/>`;
    const mouthWork = `<ellipse cx="110" cy="118" rx="9.5" ry="8" fill="var(--m-stroke)"/>
      <ellipse cx="107" cy="121" rx="3.4" ry="2.2" fill="#E8636B"/>`;
    const mouthHappy = `<path d="M98 113 q12 16 24 0 q-12 6 -24 0 Z" fill="var(--m-stroke)"/>
      <path d="M102 117 q8 7 16 0" fill="none" stroke="#FFFFFF" stroke-width="2.2" ${OPEN}/>`;

    return `
    <svg viewBox="0 0 220 220" class="mascot mascot-pixiu" data-mood="${mood}" role="img" aria-label="貔貅漫画形象">
      <ellipse class="m-shadow" cx="110" cy="196" rx="58" ry="9"/>
      <!-- 祥云尾 -->
      <path class="m-tail" d="M158 158 q26 -4 26 -22 q0 -12 -12 -12 q-9 0 -10 9 q8 -2 10 5 q1 8 -10 9 Z"
            fill="var(--m-deep)" stroke="var(--m-stroke)" stroke-width="3" ${OPEN}/>
      <!-- 鬃毛 -->
      <g fill="var(--m-deep)" stroke="var(--m-stroke)" stroke-width="3" ${OPEN}>
        <circle cx="72" cy="62" r="12"/><circle cx="64" cy="86" r="11"/><circle cx="80" cy="44" r="12"/>
        <circle cx="110" cy="36" r="12"/><circle cx="140" cy="44" r="12"/><circle cx="156" cy="62" r="11"/>
        <circle cx="160" cy="88" r="10"/>
      </g>
      <!-- 独角 -->
      <path d="M110 40 C105 24 112 14 119 9 C116 22 121 30 128 37 Z"
            fill="var(--m-cream)" stroke="var(--m-stroke)" stroke-width="3" ${OPEN}/>
      <!-- 耳朵 -->
      <path d="M78 52 q-10 -8 -4 -20 q12 4 16 14 Z" fill="var(--m-main)" stroke="var(--m-stroke)" stroke-width="3" ${OPEN}/>
      <path d="M142 52 q10 -8 4 -20 q-12 4 -16 14 Z" fill="var(--m-main)" stroke="var(--m-stroke)" stroke-width="3" ${OPEN}/>
      <!-- 身体 -->
      <path d="M62 172 C56 132 80 112 110 112 C140 112 164 132 158 172 C154 190 134 196 110 196 C86 196 66 190 62 172 Z"
            fill="var(--m-main)" stroke="var(--m-stroke)" stroke-width="3.6" ${OPEN}/>
      <ellipse cx="110" cy="168" rx="27" ry="21" fill="var(--m-cream)" stroke="var(--m-stroke)" stroke-width="2.6" ${OPEN}/>
      <ellipse cx="90" cy="192" rx="12" ry="7" fill="var(--m-main)" stroke="var(--m-stroke)" stroke-width="2.6" ${OPEN}/>
      <ellipse cx="130" cy="192" rx="12" ry="7" fill="var(--m-main)" stroke="var(--m-stroke)" stroke-width="2.6" ${OPEN}/>
      <!-- 头 -->
      <circle cx="110" cy="90" r="50" fill="var(--m-main)" stroke="var(--m-stroke)" stroke-width="3.6" ${OPEN}/>
      ${mood === 'happy' ? eyeHappy : eyeNormal}
      ${mood === 'working' ? browWork : browIdle}
      <!-- 鼻 -->
      <path d="M105 105 q5 -4 10 0 q-1 5 -5 5 q-4 0 -5 -5 Z" fill="var(--m-stroke)"/>
      ${mood === 'working' ? mouthWork : mood === 'happy' ? mouthHappy : mouthIdle}
      <!-- 工作态：吸入的缓存碎屑 -->
      ${mood === 'working' ? `
        <g class="m-suck">
          <rect x="26" y="104" width="9" height="9" rx="2.5" fill="var(--m-cream)" stroke="var(--m-stroke)" stroke-width="2"/>
          <rect x="14" y="126" width="7" height="7" rx="2" fill="var(--m-deep)" stroke="var(--m-stroke)" stroke-width="2"/>
          <circle cx="34" cy="140" r="4.6" fill="var(--m-cream)" stroke="var(--m-stroke)" stroke-width="2"/>
          <path d="M40 96 h14 M44 120 h12 M42 146 h10" stroke="var(--m-stroke)" stroke-width="2.4" ${OPEN} opacity="0.7"/>
        </g>` : ''}
      <!-- 开心态：灵光星花 -->
      ${mood === 'happy' ? `
        <g class="m-spark" fill="var(--m-deep)" stroke="var(--m-stroke)" stroke-width="2" ${OPEN}>
          <path d="M58 66 l3 7 l7 3 l-7 3 l-3 7 l-3 -7 l-7 -3 l7 -3 Z"/>
          <path d="M164 70 l2.4 5.6 l5.6 2.4 l-5.6 2.4 l-2.4 5.6 l-2.4 -5.6 l-5.6 -2.4 l5.6 -2.4 Z"/>
        </g>` : ''}
    </svg>`;
  }

  /* ---------- 仓颉（AI 数据）：mood = idle | working ---------- */
  function cangjie(mood = 'idle') {
    return `
    <svg viewBox="0 0 220 220" class="mascot mascot-cangjie" data-mood="${mood}" role="img" aria-label="仓颉漫画形象">
      <ellipse class="m-shadow" cx="110" cy="196" rx="58" ry="9"/>
      ${mood === 'working' ? '<circle class="m-halo" cx="110" cy="74" r="56" fill="var(--m-main)" opacity="0.16"/>' : ''}
      <!-- 长袍与宽袖 -->
      <path d="M70 196 L78 128 Q80 110 110 108 Q140 110 142 128 L150 196 Z"
            fill="var(--m-main)" stroke="var(--m-stroke)" stroke-width="3.6" ${OPEN}/>
      <ellipse cx="74" cy="152" rx="16" ry="26" fill="var(--m-deep)" stroke="var(--m-stroke)" stroke-width="3" ${OPEN}/>
      <ellipse cx="146" cy="152" rx="16" ry="26" fill="var(--m-deep)" stroke="var(--m-stroke)" stroke-width="3" ${OPEN}/>
      <!-- 交领与腰带 -->
      <path d="M96 110 L110 136 L124 110" fill="none" stroke="var(--m-cream)" stroke-width="6" ${OPEN}/>
      <rect x="84" y="150" width="52" height="9" rx="4" fill="var(--m-stroke)"/>
      <!-- 手中书卷 -->
      <rect x="82" y="160" width="56" height="27" rx="8" fill="var(--m-cream)" stroke="var(--m-stroke)" stroke-width="3" ${OPEN}/>
      <path d="M96 164 v19 M110 164 v19 M124 164 v19" stroke="var(--m-stroke)" stroke-width="1.8" ${OPEN} opacity="0.55"/>
      <circle cx="84" cy="173" r="5" fill="var(--m-skin)" stroke="var(--m-stroke)" stroke-width="2.4"/>
      <circle cx="136" cy="173" r="5" fill="var(--m-skin)" stroke="var(--m-stroke)" stroke-width="2.4"/>
      <!-- 头 -->
      <circle cx="110" cy="76" r="40" fill="var(--m-skin)" stroke="var(--m-stroke)" stroke-width="3.6" ${OPEN}/>
      <circle cx="72" cy="80" r="6.5" fill="var(--m-skin)" stroke="var(--m-stroke)" stroke-width="3" ${OPEN}/>
      <circle cx="148" cy="80" r="6.5" fill="var(--m-skin)" stroke="var(--m-stroke)" stroke-width="3" ${OPEN}/>
      <!-- 冠帽 -->
      <path d="M84 52 L88 32 L132 32 L136 52 Z" fill="var(--m-deep)" stroke="var(--m-stroke)" stroke-width="3" ${OPEN}/>
      <rect x="74" y="50" width="72" height="11" rx="4.5" fill="var(--m-stroke)"/>
      <circle cx="110" cy="28" r="4.5" fill="var(--m-cream)" stroke="var(--m-stroke)" stroke-width="2"/>
      <!-- 四目：额上两目 + 主双目 -->
      <g class="m-eye">
        <circle cx="100" cy="61" r="4.4" fill="#FFFFFF" stroke="var(--m-stroke)" stroke-width="2.2" ${OPEN}/>
        <circle cx="101" cy="61" r="2" fill="var(--m-stroke)"/>
        <circle cx="120" cy="61" r="4.4" fill="#FFFFFF" stroke="var(--m-stroke)" stroke-width="2.2" ${OPEN}/>
        <circle cx="121" cy="61" r="2" fill="var(--m-stroke)"/>
      </g>
      <g class="m-eye">
        <circle cx="96" cy="82" r="9.2" fill="#FFFFFF" stroke="var(--m-stroke)" stroke-width="3" ${OPEN}/>
        <circle cx="${mood === 'working' ? 98 : 97}" cy="82" r="4.4" fill="var(--m-stroke)"/>
        <circle cx="124" cy="82" r="9.2" fill="#FFFFFF" stroke="var(--m-stroke)" stroke-width="3" ${OPEN}/>
        <circle cx="${mood === 'working' ? 126 : 125}" cy="82" r="4.4" fill="var(--m-stroke)"/>
      </g>
      <path d="M88 72 q8 -4 15 0 M117 72 q8 -4 15 0" fill="none" stroke="var(--m-stroke)" stroke-width="2.6" ${OPEN}/>
      <!-- 鼻、口、须 -->
      <path d="M108 92 q4 3 0 6" fill="none" stroke="var(--m-stroke)" stroke-width="2.4" ${OPEN}/>
      <path d="M103 102 q7 5 14 0" fill="none" stroke="var(--m-stroke)" stroke-width="2.6" ${OPEN}/>
      <path d="M106 106 q4 8 4 14 q0 -8 4 -14 Z" fill="var(--m-stroke)"/>
      <!-- 工作态：上浮字符灵光 -->
      ${mood === 'working' ? `
        <g class="m-glyphs" fill="none" stroke="var(--m-deep)" stroke-width="2.6" ${OPEN}>
          <path d="M46 58 h10 M51 53 v10 M47 66 h8"/>
          <path d="M164 50 h9 M168 46 v9"/>
          <rect x="38" y="84" width="8" height="8" rx="2"/>
          <circle cx="176" cy="86" r="4.4"/>
        </g>` : ''}
    </svg>`;
  }

  return { pixiu, cangjie };
})();
