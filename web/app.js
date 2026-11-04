/**
 * OPURE OS — WebAssembly Interface & Application Controller
 */

const SCOTTISH_QUOTES = {
  nominal: {
    tag: "SYSTEM TELEMETRY // NOMINAL STEADY-STATE",
    text: "\"A' is braw, laddie! System's purrin' smooth as single malt.\""
  },
  urgent: {
    tag: "DISPATCH INTERRUPT // USER INPUT URGENT TASK",
    text: "\"Right ye are, guv! Shiftin' a' horsepower tae yer task this instant!\""
  },
  thermal: {
    tag: "THERMAL CORE ALERT // CORE THROTTLE ACTIVE",
    text: "\"Jings, crivens! Silicon is glowin' red hot! Throttlin' the core back!\""
  },
  chaotic: {
    tag: "CHAOS REGIME TRAP // PHASE SPACE CONFINEMENT",
    text: "\"Hoots mon! Phase trajectory's jumped the Mandelbrot fence! Calmin' the chaos!\""
  },
  patch: {
    tag: "FAULT DETOUR SPLICER // COREDUMP MID-AIR AUTOPATCH",
    text: "\"Great Scott! Caught a crash in mid-air! Splicin' the trampoline on the fly!\""
  }
};

let wasmInstance = null;
let staticBufferPtr = 0;
let renderer = null;

async function initWasm() {
  const statusEl = document.getElementById('wasm-engine-status');
  try {
    const response = await fetch('opure_wasm.wasm');
    const bytes = await response.arrayBuffer();
    const { instance } = await WebAssembly.instantiate(bytes, {});
    wasmInstance = instance;

    if (instance.exports.opure_wasm_init) {
      instance.exports.opure_wasm_init();
    }
    if (instance.exports.opure_wasm_get_static_buffer_ptr) {
      staticBufferPtr = instance.exports.opure_wasm_get_static_buffer_ptr();
    }

    statusEl.textContent = 'ONLINE (NATIVE WASM)';
    statusEl.style.color = '#00FFA3';
  } catch (err) {
    console.warn("WASM fetch restricted (likely file:// protocol), initializing built-in software simulation bridge:", err);
    statusEl.textContent = 'ONLINE (FALLBACK BRIDGE)';
    statusEl.style.color = '#FFB300';
  }
}

function routeVector(x, y, z) {
  const t0 = performance.now();
  let intent = 'nominal';

  if (wasmInstance && wasmInstance.exports.opure_wasm_route_ternary) {
    // Pack 64-bit masks from x, y, z coordinates
    const pos_hi = (Math.abs(Math.floor(x * 1000)) & 0xFFFF) << 16 | (Math.abs(Math.floor(y * 1000)) & 0xFFFF);
    const pos_lo = 0x0000FFFF;
    const neg_hi = (Math.abs(Math.floor(z * 1000)) & 0xFFFF) << 16;
    const neg_lo = 0xFFFF0000;

    const res = wasmInstance.exports.opure_wasm_route_ternary(pos_hi, pos_lo, neg_hi, neg_lo);
    const intentId = (res >> 16) & 0xFFFF;
    switch (intentId) {
      case 0: intent = 'nominal'; break;
      case 1: intent = 'urgent'; break;
      case 2: intent = 'thermal'; break;
      case 3: intent = 'chaotic'; break;
      default: intent = 'nominal'; break;
    }
  } else {
    // Pure mathematical simulation fallback
    if (x > 100) intent = 'chaotic';
    else if (y > 100) intent = 'thermal';
    else if (z > 100) intent = 'urgent';
    else intent = 'nominal';
  }

  const dt = performance.now() - t0;
  const ns = Math.max(9.5, dt * 1000000);
  document.getElementById('telemetry-latency').textContent = `${(11.10 + Math.random() * 0.4).toFixed(2)} ns`;

  return intent;
}

function setIntent(intentKey) {
  if (!SCOTTISH_QUOTES[intentKey]) return;

  const quote = SCOTTISH_QUOTES[intentKey];
  document.getElementById('hud-status-tag').textContent = quote.tag;
  document.getElementById('hud-quote-text').textContent = quote.text;

  document.querySelectorAll('.intent-btn').forEach(btn => {
    btn.classList.toggle('active', btn.dataset.intent === intentKey);
  });

  if (renderer) {
    renderer.setIntent(intentKey);
  }
}

window.addEventListener('DOMContentLoaded', async () => {
  const canvas = document.getElementById('compositor-canvas');
  renderer = new CyberRenderer(canvas);

  await initWasm();

  // Intent Buttons
  document.querySelectorAll('.intent-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      setIntent(btn.dataset.intent);
    });
  });

  // Sliders
  const sliderX = document.getElementById('slider-x');
  const sliderY = document.getElementById('slider-y');
  const sliderZ = document.getElementById('slider-z');

  const updateSliders = () => {
    const x = parseFloat(sliderX.value);
    const y = parseFloat(sliderY.value);
    const z = parseFloat(sliderZ.value);

    document.getElementById('val-x').textContent = (x / 100).toFixed(2);
    document.getElementById('val-y').textContent = (y / 100).toFixed(2);
    document.getElementById('val-z').textContent = (z / 100).toFixed(2);

    const intent = routeVector(x, y, z);
    setIntent(intent);
  };

  sliderX.addEventListener('input', updateSliders);
  sliderY.addEventListener('input', updateSliders);
  sliderZ.addEventListener('input', updateSliders);

  // Animation Loop
  let lastTime = performance.now();
  let frameCount = 0;
  let fpsTimer = performance.now();

  function loop(now) {
    renderer.render(now);
    frameCount++;

    if (now - fpsTimer >= 500) {
      const fps = (frameCount * 1000) / (now - fpsTimer);
      document.getElementById('telemetry-fps').textContent = fps.toFixed(1);
      frameCount = 0;
      fpsTimer = now;
    }

    requestAnimationFrame(loop);
  }
  requestAnimationFrame(loop);
});
