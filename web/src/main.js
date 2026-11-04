import { MATERIALS, CATEGORIES, searchMaterials, findMaterial } from "./materials.js";
import { getState, setState, subscribe, resetState } from "./state/store.js";
import { MockSimulation, GRID_W, GRID_H } from "./adapters/simulation.js";
import { Canvas2DRenderer } from "./adapters/renderer.js";

const $ = (id) => document.getElementById(id);

let sim = null;
let renderer = null;
let canvas = null;
let rafId = null;
let lastFrame = performance.now();
let running = false;
let stroke = null; // current drag stroke
let hoveredCell = null;
let spaceDown = false;

// ---- Boot ----
async function boot() {
  setAppState("loading", "Initializing simulation…");
  canvas = $("viewport");
  renderer = new Canvas2DRenderer();
  renderer.attach(canvas);

  sim = new MockSimulation();
  try {
    await sim.init();
  } catch (e) {
    setAppState("error", "Failed to initialize simulation: " + (e?.message || e));
    log("error", "init failed: " + (e?.message || e));
    return;
  }

  // wire resize
  const ro = new ResizeObserver(() => {
    renderer.resize(canvas.clientWidth, canvas.clientHeight);
  });
  ro.observe(canvas.parentElement);

  // build material UI
  buildCategories();
  renderMaterialList();
  wireToolbar();
  wireTools();
  wirePanels();
  wireViewport();
  wireShortcuts();
  wireDialogs();

  subscribe(onStateChange);
  onStateChange(getState());

  setAppState("ready", "");
  log("info", "REALMATTER UI ready (mock simulation).");
  startLoop();
}

function setAppState(state, msg) {
  const app = $("app");
  if (state === "ready") {
    app.dataset.state = "ready";
    $("viewport-overlay").hidden = true;
  } else {
    app.dataset.state = state;
    $("viewport-overlay").hidden = false;
    $("overlay-title").textContent = state === "loading" ? "Loading…" : "Error";
    $("overlay-message").textContent = msg || "";
    $("overlay-retry").hidden = state !== "error";
    $("overlay-dismiss").hidden = state !== "error";
  }
  if (state === "error") {
    $("overlay-retry").onclick = () => location.reload();
    $("overlay-dismiss").onclick = () => setAppState("ready", "");
  }
}

// ---- Render loop ----
function startLoop() {
  lastFrame = performance.now();
  const loop = (now) => {
    const dt = Math.min(0.1, (now - lastFrame) / 1000);
    lastFrame = now;
    const s = getState();
    if (s.playing) sim.resume(); else sim.pause();
    sim.setSpeed(s.speed);
    sim.tick(dt);
    renderer.setOverlay("grid", s.grid);
    renderer.setOverlay("heat", s.heat);
    renderer.render(sim.getSnapshot(), s.camera, {});
    updateHud();
    rafId = requestAnimationFrame(loop);
  };
  rafId = requestAnimationFrame(loop);
}

function updateHud() {
  const s = getState();
  const d = sim.getDiagnostics();
  const fps = renderer.getFps();
  $("hud-fps").textContent = `FPS ${fps.toFixed(0)}`;
  $("hud-particles").textContent = `Particles ${d.particles.toLocaleString()}`;
  $("hud-coords").textContent = hoveredCell ? `(${hoveredCell.x}, ${hoveredCell.y})` : "—";
  $("hud-state").textContent = s.playing ? "RUNNING" : "PAUSED";
  $("d-fps").textContent = fps.toFixed(1);
  $("d-tick").textContent = d.tickMs.toFixed(2) + " ms";
  $("d-render").textContent = renderer.getRenderMs().toFixed(2) + " ms";
  $("d-particles").textContent = d.particles.toLocaleString();
  $("d-chunks").textContent = d.chunks.toLocaleString();
  $("d-memory").textContent = d.memoryMB.toFixed(2) + " MB";
  $("d-state").textContent = d.state;
  $("status-coords").textContent = hoveredCell ? `cell ${hoveredCell.x}, ${hoveredCell.y}` : "";
}

// ---- State change ----
function onStateChange(s) {
  $("btn-play").setAttribute("aria-pressed", String(s.playing));
  $("btn-play").querySelector(".label").textContent = s.playing ? "Pause" : "Play";
  $("btn-play").querySelector(".icon").textContent = s.playing ? "⏸" : "▶";
  $("speed").value = s.speed;
  $("speed-value").textContent = s.speed.toFixed(2) + "×";
  $("speed").setAttribute("aria-valuenow", s.speed);
  $("brush-size").value = s.brushSize;
  $("brush-size-value").textContent = s.brushSize;
  document.querySelectorAll(".tool-btn").forEach((b) => {
    const active = b.dataset.tool === s.tool;
    b.classList.toggle("is-active", active);
    b.setAttribute("aria-checked", String(active));
  });
  $("btn-grid").setAttribute("aria-pressed", String(s.grid));
  $("btn-heat").setAttribute("aria-pressed", String(s.heat));
  $("app").classList.toggle("high-contrast", s.highContrast);
  $("app").classList.toggle("reduced-motion", s.reducedMotion);
  renderMaterialList();
  updateInspector(s.selectedMaterial);
}

// ---- Toolbar ----
function wireToolbar() {
  $("btn-play").addEventListener("click", () => {
    const s = getState();
    setState({ playing: !s.playing });
    log("info", s.playing ? "Paused." : "Resumed.");
  });
  $("btn-step").addEventListener("click", () => {
    sim.step();
    log("info", "Stepped one tick.");
  });
  $("btn-reset").addEventListener("click", () => {
    if (!confirm("Reset the world?")) return;
    sim.reset();
    resetState();
    log("warn", "World reset.");
  });
  $("speed").addEventListener("input", (e) => {
    setState({ speed: parseFloat(e.target.value) });
  });
  $("btn-grid").addEventListener("click", () => setState({ grid: !getState().grid }));
  $("btn-heat").addEventListener("click", () => setState({ heat: !getState().heat }));
  $("btn-settings").addEventListener("click", () => $("settings-dialog").showModal());
  $("btn-help").addEventListener("click", () => $("help-dialog").showModal());
}

// ---- Tools ----
function wireTools() {
  document.querySelectorAll(".tool-btn").forEach((b) => {
    b.addEventListener("click", () => setState({ tool: b.dataset.tool }));
  });
  $("brush-size").addEventListener("input", (e) => setState({ brushSize: parseInt(e.target.value, 10) }));
}

// ---- Panels ----
function buildCategories() {
  const row = $("material-categories");
  row.innerHTML = "";
  for (const c of CATEGORIES) {
    const btn = document.createElement("button");
    btn.className = "chip";
    btn.textContent = c.label;
    btn.dataset.cat = c.id;
    btn.addEventListener("click", () => {
      setState({ category: c.id });
      document.querySelectorAll("#material-categories .chip").forEach((x) => x.classList.toggle("is-active", x.dataset.cat === c.id));
    });
    if (c.id === getState().category) btn.classList.add("is-active");
    row.appendChild(btn);
  }
}

function renderMaterialList() {
  const s = getState();
  const list = $("material-list");
  const items = searchMaterials(s.search, s.category);
  list.innerHTML = "";
  for (const m of items) {
    const el = document.createElement("div");
    el.className = "material-item" + (m.id === s.materialId ? " is-selected" : "");
    el.setAttribute("role", "option");
    el.setAttribute("aria-selected", String(m.id === s.materialId));
    el.innerHTML = `<span class="material-swatch" style="background:${m.color}"></span><span class="material-name">${m.name}</span>`;
    el.addEventListener("click", () => {
      setState({ materialId: m.id, selectedMaterial: m.id });
      log("info", `Selected material: ${m.name}`);
    });
    el.addEventListener("mouseenter", () => updateInspector(m.id));
    list.appendChild(el);
  }
}

function wirePanels() {
  $("material-search").addEventListener("input", (e) => {
    setState({ search: e.target.value });
  });
  // settings dialog bindings
  const syncSettings = () => {
    const s = getState();
    $("set-grid").checked = s.grid;
    $("set-heat").checked = s.heat;
    $("set-fps").checked = s.showFps;
    $("set-reduced").checked = s.reducedMotion;
    $("set-contrast").checked = s.highContrast;
  };
  $("settings-dialog").addEventListener("open", syncSettings);
  $("btn-settings").addEventListener("click", syncSettings);
  $("set-grid").addEventListener("change", (e) => setState({ grid: e.target.checked }));
  $("set-heat").addEventListener("change", (e) => setState({ heat: e.target.checked }));
  $("set-fps").addEventListener("change", (e) => setState({ showFps: e.target.checked }));
  $("set-reduced").addEventListener("change", (e) => setState({ reducedMotion: e.target.checked }));
  $("set-contrast").addEventListener("change", (e) => setState({ highContrast: e.target.checked }));
}

function updateInspector(materialId) {
  const m = findMaterial(materialId ?? getState().materialId);
  const el = $("inspector");
  const rows = [
    ["Name", m.name],
    ["Category", m.category],
    ["Phase", m.phase],
    ["Density", m.density + " kg/m³"],
    ["Temperature", m.temp + " K"],
    ["Conductivity", m.conductivity + " W/mK"],
    ["Heat cap.", m.heatCap + " J/kgK"],
    ["Melting pt.", m.melt ? m.melt + " K" : "—"],
    ["Boiling pt.", m.boil ? m.boil + " K" : "—"],
  ];
  el.innerHTML = rows.map(([k, v]) => `<div class="row"><span class="key">${k}</span><span class="val">${v}</span></div>`).join("") +
    `<p class="muted" style="margin-top:8px">${m.desc}</p>`;
}

// ---- Viewport interaction ----
function cellFromEvent(e) {
  const rect = canvas.getBoundingClientRect();
  const s = getState();
  const cell = 4 * s.camera.zoom;
  const x = Math.floor((e.clientX - rect.left - s.camera.x) / cell);
  const y = Math.floor((e.clientY - rect.top - s.camera.y) / cell);
  return { x, y };
}

function wireViewport() {
  canvas.addEventListener("mousemove", (e) => {
    hoveredCell = cellFromEvent(e);
  });
  canvas.addEventListener("mouseleave", () => { hoveredCell = null; });

  canvas.addEventListener("mousedown", (e) => {
    if (e.button !== 0) return;
    if (spaceDown) return;
    const c = cellFromEvent(e);
    const s = getState();
    if (s.tool === "brush" || s.tool === "eraser") {
      stroke = { type: "paint", points: [c], material: s.tool === "eraser" ? 0 : s.materialId, size: s.brushSize };
      sim.applyCommand({ type: "paint", x: c.x, y: c.y, material: stroke.material, size: stroke.size });
    } else if (s.tool === "line" || s.tool === "rect") {
      stroke = { type: s.tool, start: c, current: c, material: s.materialId, size: s.brushSize };
    } else if (s.tool === "fill") {
      sim.applyCommand({ type: "fill", x: c.x, y: c.y, material: s.materialId });
    }
  });

  canvas.addEventListener("mousemove", (e) => {
    if (!stroke) return;
    const c = cellFromEvent(e);
    const s = getState();
    if (stroke.type === "paint") {
      stroke.points.push(c);
      sim.applyCommand({ type: "paint", x: c.x, y: c.y, material: stroke.material, size: stroke.size });
    } else {
      stroke.current = c;
    }
  });

  window.addEventListener("mouseup", () => {
    if (!stroke) return;
    if (stroke.type === "line") {
      sim.applyCommand({ type: "line", x0: stroke.start.x, y0: stroke.start.y, x1: stroke.current.x, y1: stroke.current.y, material: stroke.material, size: stroke.size });
    } else if (stroke.type === "rect") {
      sim.applyCommand({ type: "rect", x0: stroke.start.x, y0: stroke.start.y, x1: stroke.current.x, y1: stroke.current.y, material: stroke.material, fill: false });
    }
    stroke = null;
  });

  // pan with middle mouse / space+drag
  let panning = false, panStart = null;
  canvas.addEventListener("mousedown", (e) => {
    if (e.button === 1 || (e.button === 0 && spaceDown)) {
      panning = true;
      panStart = { x: e.clientX, y: e.clientY, cam: { ...getState().camera } };
      e.preventDefault();
    }
  });
  window.addEventListener("mousemove", (e) => {
    if (!panning || !panStart) return;
    const dx = e.clientX - panStart.x;
    const dy = e.clientY - panStart.y;
    setState({ camera: { ...panStart.cam, x: panStart.cam.x + dx, y: panStart.cam.y + dy } });
  });
  window.addEventListener("mouseup", () => { panning = false; panStart = null; });

  // zoom
  canvas.addEventListener("wheel", (e) => {
    e.preventDefault();
    const s = getState();
    const factor = e.deltaY < 0 ? 1.15 : 1 / 1.15;
    const newZoom = Math.min(6, Math.max(0.25, s.camera.zoom * factor));
    setState({ camera: { ...s.camera, zoom: newZoom } });
  }, { passive: false });

  // touch: single finger paint, two finger pan/zoom
  let touches = [];
  canvas.addEventListener("touchstart", (e) => {
    touches = [...e.touches];
    if (e.touches.length === 1) {
      const t = e.touches[0];
      const fake = { clientX: t.clientX, clientY: t.clientY, button: 0 };
      canvas.dispatchEvent(new MouseEvent("mousedown", fake));
    }
  }, { passive: true });
  canvas.addEventListener("touchmove", (e) => {
    if (e.touches.length === 1) {
      const t = e.touches[0];
      const fake = { clientX: t.clientX, clientY: t.clientY };
      canvas.dispatchEvent(new MouseEvent("mousemove", fake));
    }
    e.preventDefault();
  }, { passive: false });
  canvas.addEventListener("touchend", () => {
    window.dispatchEvent(new MouseEvent("mouseup"));
  });
}

// ---- Keyboard shortcuts ----
function wireShortcuts() {
  window.addEventListener("keydown", (e) => {
    if (e.code === "Space") spaceDown = true;
    const tag = (e.target && e.target.tagName) || "";
    if (tag === "INPUT" || tag === "TEXTAREA") return;
    const s = getState();
    switch (e.key) {
      case " ":
        e.preventDefault();
        setState({ playing: !s.playing });
        break;
      case ".":
        sim.step(); log("info", "Stepped one tick."); break;
      case "r": case "R":
        if (e.shiftKey) { /* rect tool handled below? no */ }
        if (!e.ctrlKey && !e.metaKey) { /* reset requires confirm; use Shift+R? */ }
        break;
      case "b": case "B": setState({ tool: "brush" }); break;
      case "e": case "E": setState({ tool: "eraser" }); break;
      case "l": case "L": setState({ tool: "line" }); break;
      case "f": case "F": setState({ tool: "fill" }); break;
      case "q": case "Q": setState({ tool: "rect" }); break;
      case "g": case "G": setState({ grid: !s.grid }); break;
      case "h": case "H": setState({ heat: !s.heat }); break;
      case "ArrowLeft": setState({ camera: { ...s.camera, x: s.camera.x + 20 } }); break;
      case "ArrowRight": setState({ camera: { ...s.camera, x: s.camera.x - 20 } }); break;
      case "ArrowUp": setState({ camera: { ...s.camera, y: s.camera.y + 20 } }); break;
      case "ArrowDown": setState({ camera: { ...s.camera, y: s.camera.y - 20 } }); break;
      case "+": case "=": setState({ camera: { ...s.camera, zoom: Math.min(6, s.camera.zoom * 1.15) } }); break;
      case "-": case "_": setState({ camera: { ...s.camera, zoom: Math.max(0.25, s.camera.zoom / 1.15) } }); break;
      case "0": setState({ camera: { x: 0, y: 0, zoom: 1 } }); break;
      case "Escape":
        stroke = null; log("info", "Stroke cancelled."); break;
      case "?":
        $("help-dialog").showModal(); break;
      case ",":
        $("settings-dialog").showModal(); break;
    }
    if ((e.key === "r" || e.key === "R") && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      if (confirm("Reset the world?")) { sim.reset(); resetState(); log("warn", "World reset."); }
    }
    window.addEventListener("keyup", (e) => { if (e.code === "Space") spaceDown = false; });
  });
}

function wireDialogs() {
  // close on backdrop click
  for (const id of ["settings-dialog", "help-dialog"]) {
    const d = $(id);
    d.addEventListener("click", (e) => { if (e.target === d) d.close(); });
  }
}

// ---- Logging ----
function log(level, msg) {
  const el = $("diag-log");
  const entry = document.createElement("div");
  entry.className = "entry " + (level === "error" ? "error" : level === "warn" ? "warn" : "");
  const time = new Date().toLocaleTimeString();
  entry.textContent = `[${time}] ${msg}`;
  el.appendChild(entry);
  el.scrollTop = el.scrollHeight;
  while (el.childElementCount > 200) el.removeChild(el.firstChild);
  $("status-text").textContent = msg;
}

// ---- Start ----
boot().catch((e) => {
  setAppState("error", "Boot failed: " + (e?.message || e));
  console.error(e);
});
