import { MATERIALS, CATEGORIES, searchMaterials, findMaterial } from "./materials.js";
import { getState, setState, subscribe, resetState } from "./state/store.js";
import { MockSimulation, GRID_W, GRID_H } from "./adapters/simulation.js";
import { WasmDirectAdapter } from "./adapters/wasm-direct.js";
import { createRenderer, getPreferredBackend } from "./renderers/renderer-factory.js";

const $ = (id) => document.getElementById(id);

let sim = null;
let renderer = null;
let canvas = null;
let rafId = null;
let lastFrame = performance.now();
let simAccum = 0;
let running = false;
let stroke = null; // current drag stroke
let hoveredCell = null;
let spaceDown = false;
// Measured simulation throughput (ticks actually executed per second).
let tickSamples = 0;
let tickWindowStart = performance.now();
let measuredTps = 0;
let spacePanUsed = false;
let materialListKey = "";

// ---- Boot ----
async function boot() {
  setAppState("loading", "Initializing simulation…");
  canvas = $("viewport");
  try {
    renderer = await createRenderer(canvas);
    log("info", `Renderer: ${renderer.backend || getPreferredBackend()}`);
  } catch (e) {
    const { Canvas2DRenderer } = await import("./adapters/renderer.js");
    renderer = new Canvas2DRenderer();
    renderer.attach(canvas);
    log("warn", "Using Canvas2D fallback");
  }

  // Use real WASM
  try {
    const wd = new WasmDirectAdapter();
    await wd.init();
    sim = wd;
    log("info", "Using WASM simulation (real module)");
  } catch (e) {
    log("warn", "WASM init failed, using mock fallback: " + (e?.message||e));
    sim = new MockSimulation();
    try {
      await sim.init();
    } catch (e2) {
      setAppState("error", "Failed to initialize simulation: " + (e2?.message || e2));
      log("error", "init failed: " + (e2?.message || e2));
      return;
    }
  }

  // wire resize + visibility
  const resizeViewport = () => {
    const box = canvas.parentElement;
    if (!box) return;
    // Size from the viewport box, not the canvas. A canvas with height:100%
    // reports 0 until its parent has a resolved height, and a 0 resize used
    // to freeze the drawing buffer at one pixel (a black viewport).
    const w = box.clientWidth;
    const h = box.clientHeight;
    if (w < 2 || h < 2) return;
    try {
      if (typeof renderer.resize === "function") renderer.resize(w, h);
      else if (typeof renderer.attach === "function") renderer.attach(canvas);
    } catch (e) { log("warn", "resize error: " + e.message); }
  };
  const ro = new ResizeObserver(resizeViewport);
  ro.observe(canvas.parentElement);
  requestAnimationFrame(resizeViewport);
  document.addEventListener('visibilitychange', () => {});

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
  log("info", "LabsMatter UI ready.");
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
  const FIXED = 1/60;
  const MAX_STEPS = 8;
  const loop = (now) => {
    const frameDt = Math.min(0.25, (now - lastFrame) / 1000);
    lastFrame = now;
    const s = getState();
    if (s.playing) {
      sim.resume?.();
      simAccum += frameDt * (s.speed || 1);
      let steps = 0;
      while (simAccum >= FIXED && steps < MAX_STEPS) {
        if (typeof sim.tick === "function") sim.tick(FIXED);
        else if (typeof sim.step === "function") sim.step(FIXED);
        simAccum -= FIXED;
        steps++;
      }
      if (steps >= MAX_STEPS && simAccum >= FIXED) simAccum = 0;
      tickSamples += steps;
      const elapsed = (now - tickWindowStart) / 1000;
      if (elapsed >= 1.0) {
        measuredTps = tickSamples / elapsed;
        tickSamples = 0;
        tickWindowStart = now;
      }
    } else {
      sim.pause?.();
      simAccum = 0;
      measuredTps = 0;
      tickWindowStart = now;
      tickSamples = 0;
    }
    if (sim.setSpeed) sim.setSpeed(s.speed);
    if (typeof renderer.setOverlay === "function") {
      renderer.setOverlay("grid", s.grid);
      renderer.setOverlay("heat", s.heat);
    }
    try {
      const snap = typeof sim.getSnapshot === "function" ? sim.getSnapshot() : null;
      if (snap) {
        renderer.render(snap, s.camera, {});
      } else if (typeof sim.readRenderBuffer === "function") {
        const buf = sim.readRenderBuffer();
        const size = typeof sim.getWorldSize === "function" ? sim.getWorldSize() : null;
        if (buf && size && size.w > 0 && size.h > 0) {
          renderer.renderRGBA(buf, size.w, size.h, s.camera, { heat: s.heat });
        } else {
          renderer.render(null, s.camera, {});
        }
      } else {
        renderer.render(null, s.camera, {});
      }
    } catch(e) {
      log("warn", "render error: " + (e?.message||e));
    }
    updateHud();
    rafId = requestAnimationFrame(loop);
  };
  rafId = requestAnimationFrame(loop);
}

function updateHud() {
  const s = getState();
  const d = sim.getDiagnostics ? sim.getDiagnostics() : {};
  const fps = typeof renderer.getFps === "function" ? renderer.getFps() : 0;
  const ws = typeof sim.getWorldSize === "function" ? sim.getWorldSize() : null;
  $("hud-fps").textContent = `FPS ${fps.toFixed(0)}${renderer.backend ? " ["+renderer.backend+"]" : ""} | TPS ${s.playing ? measuredTps.toFixed(0) : "0"}`;
  $("hud-particles").textContent = `Particles ${(d.particles||0).toLocaleString()}${ws ? ` | ${ws.w}x${ws.h}` : ""}`;
  $("hud-coords").textContent = hoveredCell ? `(${hoveredCell.x}, ${hoveredCell.y})` : "—";
  $("hud-state").textContent = s.playing ? "RUNNING" : "PAUSED";
  $("d-fps").textContent = fps.toFixed(1);
  $("d-tick").textContent = (d.tickMs||0).toFixed(2) + " ms";
  $("d-render").textContent = (typeof renderer.getRenderMs==="function"?renderer.getRenderMs():0).toFixed(2) + " ms";
  $("d-particles").textContent = (d.particles||0).toLocaleString();
  $("d-chunks").textContent = (d.chunks||0).toLocaleString();
  $("d-memory").textContent = (d.memoryMB||0).toFixed(2) + " MB";
  $("d-state").textContent = d.state || (s.playing?"RUNNING":"PAUSED");
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
  const showFps = s.showFps !== false;
  $("hud-fps").hidden = !showFps;
  const fpsRow = $("d-fps") && $("d-fps").parentElement;
  if (fpsRow) fpsRow.hidden = !showFps;
  const listKey = `${s.search}\0${s.category}\0${s.materialId}`;
  if (listKey !== materialListKey) {
    materialListKey = listKey;
    renderMaterialList();
  }
  updateInspector(s.selectedMaterial);
}

// ---- Toolbar ----
function wireToolbar() {
  // Mobile: toggle the materials/tools panel as an overlay (see .panel--left.is-open).
  const menuBtn = $("btn-menu");
  if (menuBtn) {
    menuBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      const panel = $("materials-panel");
      if (!panel) return;
      const open = !panel.classList.contains("is-open");
      panel.classList.toggle("is-open", open);
      menuBtn.setAttribute("aria-expanded", String(open));
    });
    // Close when tapping outside or pressing Escape.
    const closeMenu = () => {
      const panel = $("materials-panel");
      if (panel && panel.classList.contains("is-open")) {
        panel.classList.remove("is-open");
        menuBtn.setAttribute("aria-expanded", "false");
      }
    };
    document.addEventListener("click", (e) => {
      const panel = $("materials-panel");
      if (!panel || !panel.classList.contains("is-open")) return;
      if (panel.contains(e.target) || menuBtn.contains(e.target)) return;
      closeMenu();
    });
    canvas.addEventListener("pointerdown", closeMenu, { passive: true });
    window.addEventListener("keydown", (e) => { if (e.key === "Escape") closeMenu(); });
  }

  $("btn-play").addEventListener("click", () => {
    const s = getState();
    setState({ playing: !s.playing });
    log("info", s.playing ? "Paused." : "Resumed.");
  });
  $("btn-step").addEventListener("click", () => {
    if (typeof sim.step === "function") sim.step();
    else if (typeof sim.tick === "function") sim.tick();
    log("info", "Stepped one tick.");
  });
  $("btn-reset").addEventListener("click", () => {
    if (sim && sim.canReset && !sim.canReset()) log("warn", "Reset unavailable");
    resetWorld();
  });
  $("speed").addEventListener("input", (e) => {
    setState({ speed: parseFloat(e.target.value) });
  });
  $("btn-save").addEventListener("click", () => {
    try {
      if (!sim.saveBlob || !sim.canSave || !sim.canSave()) { log("warn", "Save unavailable: WASM save_blob missing"); return; }
      const blob = sim.saveBlob();
      if (!blob || blob.length === 0) { log("warn", "Save produced empty blob"); return; }
      const url = URL.createObjectURL(new Blob([blob], { type: "application/octet-stream" }));
      const a = document.createElement("a");
      a.href = url;
      a.download = "labsmatter-world.rmt";
      document.body.appendChild(a);
      a.click();
      a.remove();
      setTimeout(() => URL.revokeObjectURL(url), 5000);
      log("info", `World saved (${blob.length} bytes)`);
    } catch (e) { log("error", "save failed: " + (e?.message || e)); }
  });
  $("btn-load").addEventListener("click", () => { $("file-load").click(); });
  $("file-load").addEventListener("change", async (e) => {
    const f = e.target.files && e.target.files[0];
    if (!f) return;
    try {
      if (!sim.loadBlob || !sim.canSave || !sim.canSave()) { log("warn", "Load unavailable: WASM load_blob missing"); return; }
      const bytes = new Uint8Array(await f.arrayBuffer());
      const ok = sim.loadBlob(bytes);
      log(ok ? "info" : "error", ok ? `World loaded (${bytes.length} bytes)` : "Load failed: invalid save blob");
    } catch (err) { log("error", "load failed: " + (err?.message || err)); }
    e.target.value = "";
  });
  $("btn-grid").addEventListener("click", () => setState({ grid: !getState().grid }));  $("btn-heat").addEventListener("click", () => {
    const on = !getState().heat;
    setState({ heat: on });
    // The live WASM path streams a color buffer without per-cell
    // temperatures, so the heat overlay only applies in snapshot mode.
    if (on && sim && typeof sim.getSnapshot === "function" && !sim.getSnapshot()) {
      log("info", "Heat overlay: color view only (temperature tint unavailable in live WASM mode).");
    }
  });
  $("btn-settings").addEventListener("click", () => showDialog("settings-dialog"));
  $("btn-help").addEventListener("click", () => showDialog("help-dialog"));
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
  const cell = 4 * (s.camera.zoom || 1);
  // Camera and the renderers are in CSS pixels. Dividing by devicePixelRatio
  // again mapped the cursor to the wrong cell on HiDPI displays.
  const x = Math.floor((e.clientX - rect.left - s.camera.x) / cell);
  const y = Math.floor((e.clientY - rect.top - s.camera.y) / cell);
  return { x, y };
}

function showDialog(id) {
  const d = $(id);
  if (d && !d.open) d.showModal();
}

function resetWorld() {
  if (!confirm("Reset the world?")) return;
  try {
    if (sim && sim.reset) sim.reset();
    resetState();
    log("info", "World reset.");
  } catch (e) {
    log("error", "reset failed: " + (e?.message || e));
  }
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
      if (sim.canPaint && !sim.canPaint()) { log("warn", "Painting unavailable: missing WASM export (set_cell)"); return; }
      stroke = { type: "paint", points: [c], material: s.tool === "eraser" ? 0 : s.materialId, size: s.brushSize };
      sim.applyCommand({ type: "paint", x: c.x, y: c.y, material: stroke.material, size: stroke.size });
    } else if (s.tool === "line" || s.tool === "rect") {
      if (sim.canPaint && !sim.canPaint()) { log("warn", "Shape tools unavailable: missing WASM export"); return; }
      stroke = { type: s.tool, start: c, current: c, material: s.materialId, size: s.brushSize };
    } else if (s.tool === "fill") {
      if (sim.canPaint && !sim.canPaint()) { log("warn", "Fill unavailable: missing WASM export"); return; }
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
    if (spaceDown) spacePanUsed = true;
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
  const typing = (e) => {
    const tag = (e.target && e.target.tagName) || "";
    return tag === "INPUT" || tag === "TEXTAREA";
  };
  window.addEventListener("keydown", (e) => {
    if (e.code === "Space") {
      spaceDown = true;
      if (!typing(e)) e.preventDefault();
    }
    if (typing(e) || e.repeat) return;
    const s = getState();
    switch (e.key) {
      case " ":
        // Play/pause is applied on keyup so a Space+drag pan does not also
        // toggle the simulation, and key-repeat cannot flip it every frame.
        break;
      case ".":
        if (typeof sim.step === "function") sim.step(); else if (typeof sim.tick === "function") sim.tick(); log("info", "Stepped one tick."); break;
      case "r": case "R":
        e.preventDefault();
        resetWorld();
        break;
      case "b": case "B": setState({ tool: "brush" }); break;
      case "e": case "E": setState({ tool: "eraser" }); break;
      case "l": case "L": setState({ tool: "line" }); break;
      case "f": case "F": setState({ tool: "fill" }); break;
      case "q": case "Q": setState({ tool: "rect" }); break;
      case "g": case "G": setState({ grid: !s.grid }); break;
      case "h": case "H": setState({ heat: !s.heat }); break;
      case "ArrowLeft": e.preventDefault(); setState({ camera: { ...s.camera, x: s.camera.x + 20 } }); break;
      case "ArrowRight": e.preventDefault(); setState({ camera: { ...s.camera, x: s.camera.x - 20 } }); break;
      case "ArrowUp": e.preventDefault(); setState({ camera: { ...s.camera, y: s.camera.y + 20 } }); break;
      case "ArrowDown": e.preventDefault(); setState({ camera: { ...s.camera, y: s.camera.y - 20 } }); break;
      case "+": case "=": setState({ camera: { ...s.camera, zoom: Math.min(6, s.camera.zoom * 1.15) } }); break;
      case "-": case "_": setState({ camera: { ...s.camera, zoom: Math.max(0.25, s.camera.zoom / 1.15) } }); break;
      case "0": setState({ camera: { x: 0, y: 0, zoom: 1 } }); break;
      case "Escape":
        stroke = null; log("info", "Stroke cancelled."); break;
      case "?":
        showDialog("help-dialog"); break;
      case ",":
        showDialog("settings-dialog"); break;
    }
  });
  window.addEventListener("keyup", (e) => {
    if (e.code !== "Space") return;
    const held = spaceDown;
    spaceDown = false;
    if (!held || spacePanUsed || typing(e)) {
      spacePanUsed = false;
      return;
    }
    spacePanUsed = false;
    setState({ playing: !getState().playing });
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
