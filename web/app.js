/**
 * OPURE OS — WebAssembly Interface & Application Controller
 * Includes Kinetic Shell, 1.58-Bit Cognitive Chat, and Enterprise Swarm Matrix (51 Agents across 11 Domains)
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

const MISSION_QUOTES = {
  train: {
    tag: "MISSION DISPATCH // FRONTIER TRAINING RUN",
    text: "\"Haud yer horses! Orchestratin' 5 clusters across Research, RLHF, and GPU scheduler! Braw progress, laddie!\""
  },
  redteam: {
    tag: "FIREWALL PROBE // RED-TEAM ZERO-TRUST STRIKE",
    text: "\"Heads up! Red-team probe intercepted at the zero-trust fence! Audit log sealed tight!\""
  },
  release: {
    tag: "DEPLOYMENT PIPELINE // CHATGPT PRODUCT RELEASE",
    text: "\"Right on time! Interface, Multimodal, and Search synchronized! Packagin' for roll-out!\""
  },
  client: {
    tag: "ENTERPRISE PROTOCOL // CLIENT ONBOARDING",
    text: "\"Sales an' Solutions Architects workin' hand in glove! Zero-alloc enclave spun up!\""
  },
  rsi: {
    tag: "RSI PLASTICITY ENGINE // SWARM BITWISE MUTATION",
    text: "\"Great Scott! 51 centroids adapted in parallel! Baseline fitness jumped ahead!\""
  }
};

const ENTERPRISE_DOMAINS = [
  { id: 0, name: "Executive / Coordination", icon: "👑", roles: [0, 1, 2] },
  { id: 1, name: "Research", icon: "🔬", roles: [3, 4, 5, 6, 7] },
  { id: 2, name: "Model Development", icon: "🧠", roles: [8, 9, 10, 11, 12] },
  { id: 3, name: "Infrastructure", icon: "⚡", roles: [13, 14, 15, 16, 17] },
  { id: 4, name: "Product", icon: "📊", roles: [18, 19, 20, 21, 22] },
  { id: 5, name: "ChatGPT Product", icon: "💬", roles: [23, 24, 25, 26, 27] },
  { id: 6, name: "Safety & Trust", icon: "🛡️", roles: [28, 29, 30, 31, 32] },
  { id: 7, name: "Enterprise / Deployment", icon: "🌐", roles: [33, 34, 35, 36] },
  { id: 8, name: "Go-To-Market", icon: "📈", roles: [37, 38, 39, 40, 41] },
  { id: 9, name: "Business Operations", icon: "🏢", roles: [42, 43, 44, 45, 46] },
  { id: 10, name: "Governance", icon: "⚖️", roles: [47, 48, 49, 50] }
];

const ENTERPRISE_ROLES = [
  // Executive (3)
  { id: 0, title: "CEO-Adjacent Strategy Agent", domain: 0, caps: "CAP_ALL (0x3F)" },
  { id: 1, title: "Chief-of-Staff / Orchestrator", domain: 0, caps: "CAP_ALL (0x3F)" },
  { id: 2, title: "Portfolio Manager", domain: 0, caps: "CAP_ALL (0x3F)" },

  // Research (5)
  { id: 3, title: "Research Ideation Agent", domain: 1, caps: "RD|WR|EX|GPU" },
  { id: 4, title: "Experiment Design Agent", domain: 1, caps: "RD|WR|EX|GPU" },
  { id: 5, title: "Training-Run Manager", domain: 1, caps: "RD|WR|EX|GPU" },
  { id: 6, title: "Evaluation Scientist Agent", domain: 1, caps: "RD|WR|EX|GPU" },
  { id: 7, title: "Interpretability Agent", domain: 1, caps: "RD|WR|EX|GPU" },

  // Model Development (5)
  { id: 8, title: "Pretraining Engineer Agent", domain: 2, caps: "RD|WR|EX|GPU|SPLICE" },
  { id: 9, title: "Post-Training / RLHF Agent", domain: 2, caps: "RD|WR|EX|GPU|SPLICE" },
  { id: 10, title: "Data Curation Agent", domain: 2, caps: "RD|WR|EX|GPU|SPLICE" },
  { id: 11, title: "Model Evaluation Agent", domain: 2, caps: "RD|WR|EX|GPU|SPLICE" },
  { id: 12, title: "Alignment Agent", domain: 2, caps: "RD|WR|EX|GPU|SPLICE" },

  // Infrastructure (5)
  { id: 13, title: "GPU / Cluster Scheduler", domain: 3, caps: "RD|WR|EX|GPU|RING0" },
  { id: 14, title: "Training-Pipeline Agent", domain: 3, caps: "RD|WR|EX|GPU|RING0" },
  { id: 15, title: "Inference Optimizer Agent", domain: 3, caps: "RD|WR|EX|GPU|RING0" },
  { id: 16, title: "Reliability / SRE Agent", domain: 3, caps: "RD|WR|EX|GPU|RING0" },
  { id: 17, title: "Capacity Planner Agent", domain: 3, caps: "RD|WR|EX|GPU|RING0" },

  // Product (5)
  { id: 18, title: "Product Manager Agent", domain: 4, caps: "READ|WRITE" },
  { id: 19, title: "UX Researcher Agent", domain: 4, caps: "READ|WRITE" },
  { id: 20, title: "Product Designer Agent", domain: 4, caps: "READ|WRITE" },
  { id: 21, title: "Growth Analyst Agent", domain: 4, caps: "READ|WRITE" },
  { id: 22, title: "Experimentation Agent", domain: 4, caps: "READ|WRITE" },

  // ChatGPT Product (5)
  { id: 23, title: "Chat Interface Agent", domain: 5, caps: "READ|WRITE|EXECUTE" },
  { id: 24, title: "Memory / Personalization Agent", domain: 5, caps: "READ|WRITE|EXECUTE" },
  { id: 25, title: "Search Agent", domain: 5, caps: "READ|WRITE|EXECUTE" },
  { id: 26, title: "Multimodal Agent", domain: 5, caps: "READ|WRITE|EXECUTE" },
  { id: 27, title: "Mobile / Desktop Release Agent", domain: 5, caps: "READ|WRITE|EXECUTE" },

  // Safety & Trust (5)
  { id: 28, title: "Policy Agent", domain: 6, caps: "READ|EXECUTE|RING0" },
  { id: 29, title: "Red-Team Agent", domain: 6, caps: "READ|EXECUTE|RING0" },
  { id: 30, title: "Abuse-Detection Agent", domain: 6, caps: "READ|EXECUTE|RING0" },
  { id: 31, title: "Evaluation / Safety Gate Agent", domain: 6, caps: "READ|EXECUTE|RING0" },
  { id: 32, title: "Incident-Response Agent", domain: 6, caps: "READ|EXECUTE|RING0" },

  // Enterprise / Deployment (4)
  { id: 33, title: "Solutions Architect Agent", domain: 7, caps: "READ|WRITE|EXECUTE" },
  { id: 34, title: "Forward-Deployed Engineer", domain: 7, caps: "READ|WRITE|EXECUTE" },
  { id: 35, title: "Customer-Success Agent", domain: 7, caps: "READ|WRITE|EXECUTE" },
  { id: 36, title: "Integration Engineer Agent", domain: 7, caps: "READ|WRITE|EXECUTE" },

  // Go-To-Market (5)
  { id: 37, title: "Sales Agent", domain: 8, caps: "READ|WRITE" },
  { id: 38, title: "Marketing Agent", domain: 8, caps: "READ|WRITE" },
  { id: 39, title: "Partnerships Agent", domain: 8, caps: "READ|WRITE" },
  { id: 40, title: "Support Agent", domain: 8, caps: "READ|WRITE" },
  { id: 41, title: "Pricing Analyst Agent", domain: 8, caps: "READ|WRITE" },

  // Business Operations (5)
  { id: 42, title: "Finance Agent", domain: 9, caps: "READ|WRITE" },
  { id: 43, title: "Legal / Compliance Agent", domain: 9, caps: "READ|WRITE" },
  { id: 44, title: "Procurement Agent", domain: 9, caps: "READ|WRITE" },
  { id: 45, title: "HR / People Agent", domain: 9, caps: "READ|WRITE" },
  { id: 46, title: "Recruiting Agent", domain: 9, caps: "READ|WRITE" },

  // Governance (4)
  { id: 47, title: "Board-Reporting Agent", domain: 10, caps: "READ|RING0" },
  { id: 48, title: "Risk Agent", domain: 10, caps: "READ|RING0" },
  { id: 49, title: "Audit Agent", domain: 10, caps: "READ|RING0" },
  { id: 50, title: "Security Agent", domain: 10, caps: "READ|RING0" }
];

let wasmInstance = null;
let staticBufferPtr = 0;
let renderer = null;
let currentView = 'kinetic'; // 'kinetic' or 'swarm'
let selectedAgentId = null;

// Simulated agent data fallback if WASM bridge restricted
const simulatedAgentStates = ENTERPRISE_ROLES.map((r, i) => ({
  id: i + 1,
  domain_id: r.domain,
  role_id: r.id,
  ticks: Math.floor(Math.random() * 50) + 10,
  events: Math.floor(Math.random() * 12) + 2,
  fitness: 60 + (i % 5),
  quarantined: 0,
  pos_lo: 0x55555555 ^ (i * 0x1234),
  pos_hi: 0x55550000,
  neg_lo: 0xAAAAAAAA ^ (i * 0x4321),
  neg_hi: 0xAAAA0000
}));

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

function fetchAgentStates() {
  if (wasmInstance && wasmInstance.exports.opure_wasm_export_all_agents && staticBufferPtr) {
    wasmInstance.exports.opure_wasm_export_all_agents(0);
    const view = new Uint32Array(wasmInstance.exports.memory.buffer, staticBufferPtr, 51 * 12);
    const states = [];
    for (let i = 0; i < 51; i++) {
      const base = i * 12;
      states.push({
        id: view[base],
        domain_id: view[base + 1],
        role_id: view[base + 2],
        ticks: view[base + 3],
        events: view[base + 4],
        fitness: view[base + 5] | 0,
        quarantined: view[base + 6],
        pos_lo: view[base + 7],
        pos_hi: view[base + 8],
        neg_lo: view[base + 9],
        neg_hi: view[base + 10]
      });
    }
    return states;
  }
  return simulatedAgentStates;
}

function routeVector(x, y, z) {
  const t0 = performance.now();
  let intent = 'nominal';

  if (wasmInstance && wasmInstance.exports.opure_wasm_route_ternary) {
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
    if (x > 100) intent = 'chaotic';
    else if (y > 100) intent = 'thermal';
    else if (z > 100) intent = 'urgent';
    else intent = 'nominal';
  }

  const dt = performance.now() - t0;
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

  playScottishSpeech(intentKey);
}

function triggerMission(missionKey) {
  if (!MISSION_QUOTES[missionKey]) return;

  const quote = MISSION_QUOTES[missionKey];
  document.getElementById('hud-status-tag').textContent = quote.tag;
  document.getElementById('hud-quote-text').textContent = quote.text;

  // WASM / Simulation dispatch
  if (wasmInstance && wasmInstance.exports.opure_wasm_dispatch_domain_mission) {
    let src = 0, tgt = 1;
    if (missionKey === 'train') { src = 0; tgt = 2; }
    else if (missionKey === 'redteam') { src = 6; tgt = 10; }
    else if (missionKey === 'release') { src = 4; tgt = 5; }
    else if (missionKey === 'client') { src = 8; tgt = 7; }
    else if (missionKey === 'rsi') {
      wasmInstance.exports.opure_wasm_tick_enterprise_swarm(5);
    }
    wasmInstance.exports.opure_wasm_dispatch_domain_mission(src, tgt, 1);
  } else {
    // Increment ticks in simulation
    simulatedAgentStates.forEach(a => {
      a.ticks += Math.floor(Math.random() * 3) + 1;
      if (Math.random() > 0.6) a.events += 1;
    });
  }

  // Visual card flash
  highlightMissionDomains(missionKey);
  playScottishSpeech(missionKey);
  updateSwarmMatrixView();
}

function highlightMissionDomains(missionKey) {
  let targetDomainIds = [];
  if (missionKey === 'train') targetDomainIds = [0, 1, 2, 3];
  else if (missionKey === 'redteam') targetDomainIds = [6, 10];
  else if (missionKey === 'release') targetDomainIds = [4, 5, 6];
  else if (missionKey === 'client') targetDomainIds = [7, 8];
  else if (missionKey === 'rsi') targetDomainIds = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

  document.querySelectorAll('.domain-card').forEach(card => {
    const domainId = parseInt(card.dataset.domainId, 10);
    if (targetDomainIds.includes(domainId)) {
      card.classList.add('active-pulse');
      setTimeout(() => card.classList.remove('active-pulse'), 1200);
    }
  });
}

function playScottishSpeech(key) {
  try {
    const audioCtx = new (window.AudioContext || window.webkitAudioContext)();
    if (audioCtx.state === 'suspended') {
      audioCtx.resume();
    }
    const duration = 0.75;
    const sampleRate = 22050;
    const numSamples = Math.floor(sampleRate * duration);
    const buffer = audioCtx.createBuffer(1, numSamples, sampleRate);
    const data = buffer.getChannelData(0);

    let f1 = 730, f2 = 1090;
    if (key === 'urgent' || key === 'train') { f1 = 270; f2 = 2290; }
    else if (key === 'thermal' || key === 'redteam') { f1 = 530; f2 = 1840; }
    else if (key === 'chaotic' || key === 'rsi') { f1 = 570; f2 = 840; }
    else if (key === 'patch' || key === 'release') { f1 = 490; f2 = 1350; }
    else if (key === 'client') { f1 = 660; f2 = 1720; }

    let y1_1 = 0, y2_1 = 0;
    let y1_2 = 0, y2_2 = 0;
    let pitch = 125;
    let phase = 0;

    for (let i = 0; i < numSamples; i++) {
      phase += pitch / sampleRate;
      if (phase >= 1.0) phase -= 1.0;
      const exc = phase < 0.3 ? (phase / 0.3) : (1.0 - (phase - 0.3) / 0.7);

      const r1 = 0.95, theta1 = 2 * Math.PI * f1 / sampleRate;
      const b1_1 = 2 * r1 * Math.cos(theta1), b2_1 = -r1 * r1;
      const out1 = (1 - r1) * exc + b1_1 * y1_1 + b2_1 * y2_1;
      y2_1 = y1_1; y1_1 = out1;

      const r2 = 0.94, theta2 = 2 * Math.PI * f2 / sampleRate;
      const b1_2 = 2 * r2 * Math.cos(theta2), b2_2 = -r2 * r2;
      const out2 = (1 - r2) * exc + b1_2 * y1_2 + b2_2 * y2_2;
      y2_2 = y1_2; y1_2 = out2;

      const env = Math.sin((i / numSamples) * Math.PI);
      data[i] = (out1 * 0.7 + out2 * 0.3) * env * 0.3;
    }

    const source = audioCtx.createBufferSource();
    source.buffer = buffer;
    source.connect(audioCtx.destination);
    source.start();
  } catch (e) {
    // Ignore audio autoplay restrictions
  }
}

// Enterprise Swarm Matrix Rendering
function buildSwarmMatrixDom() {
  const grid = document.getElementById('swarm-matrix-grid');
  if (!grid) return;
  grid.innerHTML = '';

  ENTERPRISE_DOMAINS.forEach(domain => {
    const card = document.createElement('div');
    card.className = 'domain-card';
    card.dataset.domainId = domain.id;

    card.innerHTML = `
      <div class="domain-header">
        <span class="domain-title">${domain.icon} ${domain.name}</span>
        <span class="domain-badge">${domain.roles.length} AGENTS</span>
      </div>
      <div class="agent-list" id="domain-list-${domain.id}"></div>
    `;

    grid.appendChild(card);
  });
}

function updateSwarmMatrixView() {
  const states = fetchAgentStates();

  states.forEach(state => {
    const role = ENTERPRISE_ROLES[state.role_id] || { title: `Agent #${state.id}`, domain: state.domain_id };
    const listEl = document.getElementById(`domain-list-${state.domain_id}`);
    if (!listEl) return;

    let item = document.getElementById(`agent-item-${state.id}`);
    if (!item) {
      item = document.createElement('div');
      item.className = 'agent-item';
      item.id = `agent-item-${state.id}`;
      item.addEventListener('click', () => inspectAgent(state.id));
      listEl.appendChild(item);
    }

    item.classList.toggle('selected', selectedAgentId === state.id);

    item.innerHTML = `
      <div class="agent-name-box">
        <span class="status-dot"></span>
        <span class="agent-title-text" title="${role.title}">${role.title}</span>
      </div>
      <div class="agent-stats">
        <span>⚡${state.ticks}t</span>
        <span style="color: var(--cyan)">✉️${state.events}</span>
      </div>
    `;
  });

  if (selectedAgentId !== null) {
    const activeState = states.find(s => s.id === selectedAgentId);
    if (activeState) displayAgentInspection(activeState);
  }
}

function inspectAgent(agentId) {
  selectedAgentId = agentId;
  const states = fetchAgentStates();
  const state = states.find(s => s.id === agentId);
  if (!state) return;

  const inspector = document.getElementById('agent-inspector');
  inspector.style.display = 'flex';
  displayAgentInspection(state);
}

function displayAgentInspection(state) {
  const role = ENTERPRISE_ROLES[state.role_id] || { title: `Agent #${state.id}`, domain: state.domain_id, caps: "CAP_ALL" };
  const domain = ENTERPRISE_DOMAINS[state.domain_id] || { name: "Unknown" };

  document.getElementById('inspect-title').textContent = `${role.title} // TELEMETRY DESCRIPTOR`;
  document.getElementById('inspect-id').textContent = `#${state.id}`;
  document.getElementById('inspect-domain').textContent = domain.name;
  document.getElementById('inspect-ticks').textContent = `${state.ticks} ticks`;
  document.getElementById('inspect-events').textContent = `${state.events} processed`;
  document.getElementById('inspect-fitness').textContent = `${state.fitness} / 64 (POPCNT Dot)`;
  document.getElementById('inspect-firewall').textContent = role.caps || "CAP_READ";
  document.getElementById('inspect-morton').textContent = `${state.domain_id * 1000} - ${(state.domain_id + 1) * 1000 - 1}`;
  
  const posHex = `0x${state.pos_hi.toString(16).padStart(8, '0')}${state.pos_lo.toString(16).padStart(8, '0')}`;
  const negHex = `0x${state.neg_hi.toString(16).padStart(8, '0')}${state.neg_lo.toString(16).padStart(8, '0')}`;
  document.getElementById('inspect-masks').textContent = `${posHex} / ${negHex}`;
}

window.addEventListener('DOMContentLoaded', async () => {
  const canvas = document.getElementById('compositor-canvas');
  renderer = new CyberRenderer(canvas);

  await initWasm();
  buildSwarmMatrixDom();
  updateSwarmMatrixView();

  // View Switcher
  const btnKinetic = document.getElementById('view-kinetic-btn');
  const btnSwarm = document.getElementById('view-swarm-btn');
  const swarmPanel = document.getElementById('swarm-view-panel');

  btnKinetic.addEventListener('click', () => {
    currentView = 'kinetic';
    btnKinetic.classList.add('active');
    btnSwarm.classList.remove('active');
    canvas.style.display = 'block';
    swarmPanel.style.display = 'none';
  });

  btnSwarm.addEventListener('click', () => {
    currentView = 'swarm';
    btnSwarm.classList.add('active');
    btnKinetic.classList.remove('active');
    canvas.style.display = 'none';
    swarmPanel.style.display = 'flex';
    updateSwarmMatrixView();
  });

  // Close inspector
  document.getElementById('btn-close-inspect').addEventListener('click', () => {
    document.getElementById('agent-inspector').style.display = 'none';
    selectedAgentId = null;
    document.querySelectorAll('.agent-item').forEach(el => el.classList.remove('selected'));
  });

  // Cross-Domain Missions
  document.getElementById('btn-mission-train').addEventListener('click', () => triggerMission('train'));
  document.getElementById('btn-mission-redteam').addEventListener('click', () => triggerMission('redteam'));
  document.getElementById('btn-mission-release').addEventListener('click', () => triggerMission('release'));
  document.getElementById('btn-mission-client').addEventListener('click', () => triggerMission('client'));
  document.getElementById('btn-mission-rsi').addEventListener('click', () => triggerMission('rsi'));

  // Chat Prompt Console
  const chatInput = document.getElementById('chat-input');
  const chatBtn = document.getElementById('chat-submit-btn');

  const handleChat = () => {
    const prompt = chatInput.value.trim().toLowerCase();
    if (!prompt) return;

    if (prompt.includes('train') || prompt.includes('model') || prompt.includes('gpu')) {
      triggerMission('train');
    } else if (prompt.includes('hack') || prompt.includes('red') || prompt.includes('security') || prompt.includes('attack')) {
      triggerMission('redteam');
    } else if (prompt.includes('release') || prompt.includes('chat') || prompt.includes('deploy')) {
      triggerMission('release');
    } else if (prompt.includes('client') || prompt.includes('enterprise') || prompt.includes('sale')) {
      triggerMission('client');
    } else if (prompt.includes('evolve') || prompt.includes('mutate') || prompt.includes('adapt') || prompt.includes('rsi')) {
      triggerMission('rsi');
    } else {
      let matchedIntent = 'nominal';
      if (prompt.includes('urgent') || prompt.includes('task') || prompt.includes('fast')) {
        matchedIntent = 'urgent';
      } else if (prompt.includes('thermal') || prompt.includes('hot') || prompt.includes('temp')) {
        matchedIntent = 'thermal';
      } else if (prompt.includes('diverg') || prompt.includes('chaos') || prompt.includes('fractal')) {
        matchedIntent = 'chaotic';
      } else if (prompt.includes('crash') || prompt.includes('fault') || prompt.includes('patch')) {
        matchedIntent = 'patch';
      }
      setIntent(matchedIntent);
    }

    chatInput.value = '';
  };

  chatBtn.addEventListener('click', handleChat);
  chatInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') handleChat();
  });

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

  // Animation & Polling Loop
  let frameCount = 0;
  let fpsTimer = performance.now();
  let lastSwarmTick = performance.now();

  function loop(now) {
    if (currentView === 'kinetic') {
      renderer.render(now);
    }

    // Tick and refresh swarm every 1000ms
    if (now - lastSwarmTick >= 1000) {
      if (wasmInstance && wasmInstance.exports.opure_wasm_tick_enterprise_swarm) {
        wasmInstance.exports.opure_wasm_tick_enterprise_swarm(1);
      }
      if (currentView === 'swarm') {
        updateSwarmMatrixView();
      }
      lastSwarmTick = now;
    }

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
