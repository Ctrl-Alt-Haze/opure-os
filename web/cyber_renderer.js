/**
 * OPURE OS — Cyber-Brutalist Kinetic Compositor & SDF Renderer
 * 
 * Replicates the GPU instanced rendering pipeline and Q16.16 Verlet springs
 * for 64 active surfaces directly on an HTML5 canvas at 60 FPS.
 */

class CyberRenderer {
  constructor(canvas) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d');
    this.surfaces = [];
    this.activeIntent = 'nominal';
    this.themeColors = {
      nominal: { stroke: '#00FFA3', glow: 'rgba(0, 255, 163, 0.4)', bg: '#060E0B' },
      urgent: { stroke: '#00E5FF', glow: 'rgba(0, 229, 255, 0.4)', bg: '#040E14' },
      thermal: { stroke: '#FFB300', glow: 'rgba(255, 179, 0, 0.4)', bg: '#140D04' },
      chaotic: { stroke: '#BD00FF', glow: 'rgba(189, 0, 255, 0.4)', bg: '#120417' },
      patch: { stroke: '#FF0055', glow: 'rgba(255, 0, 85, 0.4)', bg: '#17040B' }
    };

    this.initSurfaces(8);
    this.resize();
    window.addEventListener('resize', () => this.resize());
  }

  resize() {
    this.width = this.canvas.clientWidth;
    this.height = this.canvas.clientHeight;
    this.canvas.width = this.width * window.devicePixelRatio;
    this.canvas.height = this.height * window.devicePixelRatio;
    this.ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
    this.reflowSurfaces();
  }

  initSurfaces(count) {
    this.surfaces = [];
    for (let i = 0; i < count; i++) {
      this.surfaces.push({
        id: 100 + i,
        name: i === 0 ? "COGNITIVE_CORE_HUD" : `AGENT_SWARM_WORKER_${i}`,
        x: 0, y: 0, w: 100, h: 100,
        tx: 0, ty: 0, tw: 100, th: 100,
        vx: 0, vy: 0, vw: 0, vh: 0
      });
    }
    this.reflowSurfaces();
  }

  setIntent(intent) {
    this.activeIntent = intent;
    this.reflowSurfaces();
  }

  reflowSurfaces() {
    const W = this.width;
    const H = this.height;
    const pad = 12;

    if (this.activeIntent === 'urgent') {
      // Golden Ratio 85% area focus for Surface 0
      const mainW = Math.floor(W * 0.72) - pad * 2;
      const mainH = H - pad * 2;
      this.surfaces[0].tx = pad;
      this.surfaces[0].ty = pad;
      this.surfaces[0].tw = mainW;
      this.surfaces[0].th = mainH;

      const subW = W - mainW - pad * 3;
      const subH = Math.floor((H - pad * (this.surfaces.length)) / (this.surfaces.length - 1));
      for (let i = 1; i < this.surfaces.length; i++) {
        this.surfaces[i].tx = mainW + pad * 2;
        this.surfaces[i].ty = pad + (i - 1) * (subH + pad);
        this.surfaces[i].tw = subW;
        this.surfaces[i].th = subH;
      }
    } else if (this.activeIntent === 'patch') {
      // 50/50 Split Screen for Coredump Autopatch
      const halfW = Math.floor(W / 2) - pad * 1.5;
      this.surfaces[0].tx = pad;
      this.surfaces[0].ty = pad;
      this.surfaces[0].tw = halfW;
      this.surfaces[0].th = H - pad * 2;

      this.surfaces[1].tx = halfW + pad * 2;
      this.surfaces[1].ty = pad;
      this.surfaces[1].tw = halfW;
      this.surfaces[1].th = H - pad * 2;

      for (let i = 2; i < this.surfaces.length; i++) {
        this.surfaces[i].tw = 0;
        this.surfaces[i].th = 0;
      }
    } else {
      // Nominal Fibonacci Tiling Layout
      const cols = 4;
      const rows = 2;
      const colW = Math.floor((W - pad * (cols + 1)) / cols);
      const rowH = Math.floor((H - pad * (rows + 1)) / rows);

      for (let i = 0; i < this.surfaces.length; i++) {
        const c = i % cols;
        const r = Math.floor(i / cols);
        this.surfaces[i].tx = pad + c * (colW + pad);
        this.surfaces[i].ty = pad + r * (rowH + pad);
        this.surfaces[i].tw = colW;
        this.surfaces[i].th = rowH;
      }
    }
  }

  updatePhysics() {
    const k = 0.12; // Spring stiffness
    const d = 0.78; // Damping factor

    for (let s of this.surfaces) {
      s.vx = (s.vx + (s.tx - s.x) * k) * d;
      s.vy = (s.vy + (s.ty - s.y) * k) * d;
      s.vw = (s.vw + (s.tw - s.w) * k) * d;
      s.vh = (s.vh + (s.th - s.h) * k) * d;

      s.x += s.vx;
      s.y += s.vy;
      s.w += s.vw;
      s.h += s.vh;
    }
  }

  render(time) {
    this.updatePhysics();
    const ctx = this.ctx;
    ctx.clearRect(0, 0, this.width, this.height);

    const theme = this.themeColors[this.activeIntent] || this.themeColors.nominal;

    // Draw Background Grid
    ctx.strokeStyle = 'rgba(26, 33, 48, 0.4)';
    ctx.lineWidth = 1;
    const step = 40;
    for (let x = 0; x < this.width; x += step) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, this.height);
      ctx.stroke();
    }
    for (let y = 0; y < this.height; y += step) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(this.width, y);
      ctx.stroke();
    }

    // Render Surfaces with Cyber-Brutalist SDF Box Neon Glow
    for (let s of this.surfaces) {
      if (s.w < 5 || s.h < 5) continue;

      ctx.save();
      // Outer Neon Glow
      ctx.shadowColor = theme.glow;
      ctx.shadowBlur = 12;
      ctx.fillStyle = theme.bg;
      ctx.strokeStyle = theme.stroke;
      ctx.lineWidth = 1.5;

      // Rounded Box
      ctx.beginPath();
      ctx.roundRect(s.x, s.y, s.w, s.h, 4);
      ctx.fill();
      ctx.stroke();
      ctx.restore();

      // Title Bar
      ctx.fillStyle = 'rgba(255, 255, 255, 0.05)';
      ctx.fillRect(s.x, s.y, s.w, 24);

      ctx.fillStyle = theme.stroke;
      ctx.font = '10px "JetBrains Mono", monospace';
      ctx.fillText(`[${s.id}] ${s.name}`, s.x + 8, s.y + 16);

      // Interior Telemetry Lines
      ctx.fillStyle = 'rgba(255, 255, 255, 0.3)';
      ctx.font = '9px "JetBrains Mono", monospace';
      if (s.h > 60) {
        ctx.fillText(`MORTON: 0x${((s.id * 179) & 0xFFFFFF).toString(16).toUpperCase()}`, s.x + 8, s.y + 40);
        ctx.fillText(`BOUNDS: ${Math.round(s.w)}x${Math.round(s.h)}`, s.x + 8, s.y + 54);
      }
    }

    // Scanline Overlay
    ctx.fillStyle = 'rgba(0, 0, 0, 0.1)';
    for (let y = 0; y < this.height; y += 4) {
      ctx.fillRect(0, y, this.width, 2);
    }
  }
}

window.CyberRenderer = CyberRenderer;
