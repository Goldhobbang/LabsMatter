import { MATERIALS, CATEGORIES, searchMaterials, findMaterial } from "./materials.js";
import { getState, setState, subscribe, resetState } from "./state/store.js";
import { WasmDirectAdapter } from "./adapters/wasm-direct.js";
import { createRenderer } from "./renderers/renderer-factory.js";
import { loadConfig, saveConfig, applyPreset, matchBinding, formatBinding, ACTION_LABELS } from "./shortcuts.js";

const $ = (id) => document.getElementById(id);
let sim = null;
let renderer = null;
let canvas = null;
let overlay = null;
let octx = null;
let stroke = null;
let hover = null;
let pointer = null;
let panning = null;
let paste = null;
let selection = null;
let clipboard = null;
let keys = loadConfig();
let undoStack = [];
let redoStack = [];
let strokeLog = null;
let simAccum = 0;
let lastFrame = 0;
let tickSamples = 0;
let tickWindow = 0;
let measuredTps = 0;
const SHAPES = ["circle", "square", "triangle"];
const VIEWS = ["Normal", "Fancy", "Heat", "Pressure", "Velocity", "Air velocity", "Air pressure", "Air heat", "Persistent", "Gravity"];
let statusNote = "";
let statusUntil = 0;
let switchingRenderer = false;

async function boot() {
  setAppState("loading", "Starting simulation…");
  canvas = $("viewport");
  overlay = $("overlay");
  octx = overlay.getContext("2d");
  try {
    renderer = await createRenderer(canvas);
  } catch (e) {
    setAppState("error", "Renderer failed: " + (e?.message || e) + ". The simulation was not started.");
    return;
  }
  try {
    const wd = new WasmDirectAdapter();
    await wd.init();
    sim = wd;
    sim.setViewMode(getState().view || 0);
    if (sim.moduleId() !== "realmatter_wasm") {
      setAppState("error", "Unexpected WASM module " + sim.moduleId() + ". Refusing to run a substitute engine.");
      return;
    }
  } catch (e) {
    setAppState("error", "WASM module realmatter_wasm failed: " + (e?.message || e) + ". The legacy JavaScript simulation was not started.");
    return;
  }
  const resizeViewport = () => {
    const box = canvas.parentElement;
    if (!box || box.clientWidth < 2 || box.clientHeight < 2) return;
    try { renderer.resize(box.clientWidth, box.clientHeight); } catch (e) { console.warn(e); }
    overlay.width = Math.max(1, Math.floor(box.clientWidth));
    overlay.height = Math.max(1, Math.floor(box.clientHeight));
  };
  new ResizeObserver(resizeViewport).observe(canvas.parentElement);
  requestAnimationFrame(resizeViewport);
  buildRail();
  buildPalette();
  wireChrome();
  wirePointer();
  wireKeys();
  subscribe(onState);
  onState(getState());
  setAppState("ready", "");
  lastFrame = performance.now();
  tickWindow = lastFrame;
  requestAnimationFrame(loop);
}

function setAppState(state, msg) {
  const app = $("app");
  app.dataset.state = state === "ready" ? "ready" : state;
  const ov = $("viewport-overlay");
  if (state === "ready") ov.hidden = true;
  else {
    ov.hidden = false;
    $("overlay-title").textContent = state === "error" ? "Error" : "Loading…";
    $("overlay-message").textContent = msg || "";
    $("overlay-retry").hidden = state !== "error";
    $("overlay-retry").onclick = () => location.reload();
  }
}

async function switchToWebGL(reason) {
  if (switchingRenderer) return;
  switchingRenderer = true;
  console.error(reason + " Switching the renderer to WebGL2. The WASM simulation was not replaced.");
  try {
    const { WebGL2Renderer } = await import("./renderers/webgl2-renderer.js");
    const next = new WebGL2Renderer();
    next.attach(canvas);
    next.backend = "webgl2";
    next.note = "recovered from WebGPU";
    const box = canvas.parentElement;
    if (box) next.resize(box.clientWidth, box.clientHeight);
    try { renderer.dispose(); } catch { /* already lost */ }
    renderer = next;
    setStatus("WebGPU device lost. Rendering with WebGL2. Simulation unchanged.");
  } catch (e) {
    setAppState("error", "Renderer recovery failed: " + (e?.message || e) + ". The simulation was not replaced.");
  } finally {
    switchingRenderer = false;
  }
}

function loop(now) {
  if (renderer?.fallbackRequested) {
    const reason = renderer.lossReason || "WebGPU device lost";
    renderer.fallbackRequested = false;
    switchToWebGL(reason);
  }
  const frameDt = Math.min(0.25, (now - lastFrame) / 1000);
  lastFrame = now;
  const s = getState();
  const fixed = 1 / 60;
  if (s.playing) {
    simAccum += frameDt * (s.speed || 1);
    let steps = 0;
    while (simAccum >= fixed && steps < 8) {
      sim.step(fixed);
      simAccum -= fixed;
      steps++;
    }
    if (steps >= 8) simAccum = 0;
    tickSamples += steps;
    const elapsed = (now - tickWindow) / 1000;
    if (elapsed >= 1) {
      measuredTps = tickSamples / elapsed;
      tickSamples = 0;
      tickWindow = now;
    }
  } else {
    simAccum = 0;
    measuredTps = 0;
    tickWindow = now;
    tickSamples = 0;
  }
  if (typeof renderer.setOverlay === "function") {
    renderer.setOverlay("grid", s.grid && s.gridSize > 0);
    renderer.setOverlay("gridSize", s.gridSize || 0);
  }
  try {
    const buf = sim.readRenderBuffer();
    const size = sim.getWorldSize();
    if (buf && size.w > 0) renderer.renderRGBA(buf, size.w, size.h, s.camera, {});
  } catch (e) { console.warn(e); }
  drawOverlay();
  paintHud();
  requestAnimationFrame(loop);
}

let paletteKey = "";
function onState(s) {
  $("btn-play").textContent = s.playing ? "Pause" : "Play";
  $("btn-play").setAttribute("aria-pressed", String(!!s.playing));
  $("speed").value = s.speed;
  $("speed-value").textContent = s.speed.toFixed(2) + "×";
  $("hud").hidden = !s.showHud;
  $("btn-grid").classList.toggle("is-on", !!s.grid);
  $("btn-hud").classList.toggle("is-on", !!s.showHud);
  $("btn-preset").textContent = keys.preset === "legacy" ? "Legacy keys" : "Powder keys";
  $("brush-readout").textContent = `${s.brushRx}×${s.brushRy} ${s.shape[0]}`;
  const pri = findMaterial(s.materialId);
  const sec = findMaterial(s.secondaryId);
  $("primary-chip").style.background = pri.color;
  $("primary-chip").title = "Primary " + pri.abbr;
  $("secondary-chip").style.background = sec.color;
  $("secondary-chip").title = "Secondary " + sec.abbr;
  document.querySelectorAll("[data-tool]").forEach((b) => b.classList.toggle("is-on", b.dataset.tool === s.tool));
  document.querySelectorAll("[data-shape]").forEach((b) => b.classList.toggle("is-on", b.dataset.shape === s.shape));
  document.querySelectorAll("[data-view]").forEach((b) => b.classList.toggle("is-on", Number(b.dataset.view) === s.view));
  document.querySelectorAll("[data-cat]").forEach((b) => b.classList.toggle("is-on", b.dataset.cat === s.category));
  const nextKey = `${s.category}\0${s.search}\0${s.materialId}\0${s.secondaryId}`;
  if (nextKey !== paletteKey) {
    paletteKey = nextKey;
    buildPalette();
  }
}

function buildRail() {
  const row = $("cat-rail");
  row.innerHTML = "";
  for (const c of CATEGORIES) {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "rail-btn";
    b.dataset.cat = c.id;
    b.textContent = c.short;
    b.title = c.label;
    b.addEventListener("click", () => setState({ category: c.id }));
    row.appendChild(b);
  }
}

function buildPalette() {
  const s = getState();
  const list = $("material-list");
  const items = searchMaterials(s.search, s.category);
  list.innerHTML = "";
  for (const m of items) {
    const el = document.createElement("button");
    el.type = "button";
    el.className = "el" + (m.id === s.materialId ? " is-pri" : "") + (m.id === s.secondaryId ? " is-sec" : "");
    el.innerHTML = `<i style="background:${m.color}"></i><b>${m.abbr}</b>`;
    el.title = `${m.name} — ${m.desc}`;
    el.addEventListener("click", (e) => setState({ materialId: m.id }));
    el.addEventListener("contextmenu", (e) => { e.preventDefault(); setState({ secondaryId: m.id }); });
    list.appendChild(el);
  }
}

function wireChrome() {
  $("btn-play").onclick = () => setState({ playing: !getState().playing });
  $("btn-step").onclick = () => { sim.step(1 / 60); };
  $("speed").oninput = (e) => setState({ speed: parseFloat(e.target.value) });
  $("btn-undo").onclick = () => undo();
  $("btn-redo").onclick = () => redo();
  $("btn-reset").onclick = () => resetWorld();
  $("btn-save").onclick = () => saveWorld();
  $("btn-load").onclick = () => $("file-load").click();
  $("file-load").onchange = async (e) => {
    const f = e.target.files && e.target.files[0];
    e.target.value = "";
    if (!f) return;
    const ok = sim.loadBlob(new Uint8Array(await f.arrayBuffer()));
    if (ok) {
      undoStack = [];
      redoStack = [];
      setStatus("World loaded.");
    } else {
      setStatus(sim.lastError() || "Save could not be loaded. The world was not replaced.");
    }
  };
  $("btn-preset").onclick = () => {
    keys = applyPreset(keys.preset === "powder" ? "legacy" : "powder");
    saveConfig(keys);
    onState(getState());
  };
  $("btn-help").onclick = () => openHelp();
  $("btn-keys").onclick = () => openKeys();
  $("material-search").oninput = (e) => setState({ search: e.target.value });
  document.querySelectorAll("[data-tool]").forEach((b) => b.addEventListener("click", () => setState({ tool: b.dataset.tool })));
  document.querySelectorAll("[data-shape]").forEach((b) => b.addEventListener("click", () => setState({ shape: b.dataset.shape })));
  document.querySelectorAll("[data-view]").forEach((b) => b.addEventListener("click", () => setView(Number(b.dataset.view))));
  $("btn-grid").onclick = () => setState({ grid: !getState().grid });
  $("btn-hud").onclick = () => setState({ showHud: !getState().showHud });
  for (const id of ["help-dialog", "keys-dialog", "stamp-dialog"]) {
    const d = $(id);
    d.addEventListener("click", (e) => { if (e.target === d) d.close(); });
  }
}

function setView(mode) {
  setState({ view: mode });
  sim.setViewMode(mode);
}

function wirePointer() {
  canvas.addEventListener("contextmenu", (e) => e.preventDefault());
  canvas.addEventListener("pointerdown", onDown);
  window.addEventListener("pointermove", onMove);
  window.addEventListener("pointerup", onUp);
  canvas.addEventListener("wheel", onWheel, { passive: false });
  canvas.addEventListener("pointerleave", () => { hover = null; });
}

function onDown(e) {
  if (!sim) return;
  canvas.setPointerCapture?.(e.pointerId);
  pointer = { id: e.pointerId, x: e.clientX, y: e.clientY };
  const c = cellFromEvent(e);
  hover = c;
  if (e.button === 1) { sample(c); return; }
  if (e.altKey && e.button === 0) {
    panning = { x: e.clientX, y: e.clientY, cam: { ...getState().camera } };
    return;
  }
  if (paste && e.button === 0) { placeStamp(c.x, c.y, paste); paste = null; return; }
  const s = getState();
  if (s.tool === "select" && e.button === 0) {
    stroke = { kind: "select", a: c, b: c };
    return;
  }
  let material = s.materialId;
  if (e.button === 2) material = s.secondaryId;
  if (s.tool === "eraser" && e.button === 0) material = 0;
  if (e.ctrlKey && e.shiftKey) { flood(c.x, c.y, material); return; }
  if (e.shiftKey) { stroke = { kind: "line", a: c, b: c, material }; return; }
  if (e.ctrlKey || e.metaKey) { stroke = { kind: "rect", a: c, b: c, material }; return; }
  stroke = { kind: "paint", material, last: c };
  beginLog();
  dab(c.x, c.y, material, true);
}

function onMove(e) {
  if (canvas) hover = cellFromEvent(e);
  if (panning) {
    const dx = e.clientX - panning.x;
    const dy = e.clientY - panning.y;
    setState({ camera: { ...panning.cam, x: panning.cam.x + dx, y: panning.cam.y + dy } });
    return;
  }
  if (!stroke) return;
  const c = cellFromEvent(e);
  if (stroke.kind === "paint") {
    const line = bresenham(stroke.last.x, stroke.last.y, c.x, c.y);
    for (const p of line) dab(p[0], p[1], stroke.material, true);
    stroke.last = c;
  } else {
    stroke.b = c;
  }
}

function onUp() {
  if (panning) { panning = null; return; }
  if (!stroke) return;
  const s = stroke;
  stroke = null;
  if (s.kind === "line") commitShape(lineCells(s.a, s.b), s.material);
  else if (s.kind === "rect") commitShape(rectCells(s.a, s.b), s.material);
  else if (s.kind === "select") selection = normRect(s.a, s.b);
  else endLog(s.material);
}

function onWheel(e) {
  e.preventDefault();
  const dir = e.deltaY < 0 ? 1 : -1;
  const s = getState();
  if (keys.wheel === "zoom") {
    const zoom = Math.min(6, Math.max(0.25, s.camera.zoom * (dir > 0 ? 1.15 : 1 / 1.15)));
    setState({ camera: { ...s.camera, zoom } });
    return;
  }
  let rx = s.brushRx;
  let ry = s.brushRy;
  if (e.shiftKey) rx = Math.max(0, Math.min(24, rx + dir));
  else if (e.ctrlKey || e.metaKey) ry = Math.max(0, Math.min(24, ry + dir));
  else { rx = Math.max(0, Math.min(24, rx + dir)); ry = Math.max(0, Math.min(24, ry + dir)); }
  setState({ brushRx: rx, brushRy: ry });
}

function dab(x, y, material, record) {
  const s = getState();
  for (const [cx, cy] of brushCells(x, y, s.brushRx, s.brushRy, s.shape)) {
    if (record) note(cx, cy);
    sim.mod.set_cell(cx, cy, material);
  }
}

function commitShape(cells, material) {
  beginLog();
  for (const [x, y] of cells) { note(x, y); sim.mod.set_cell(x, y, material); }
  endLog(material);
}

function brushCells(cx, cy, rx, ry, shape) {
  if (rx === 0 && ry === 0) return [[cx, cy]];
  const out = [];
  for (let y = cy - ry; y <= cy + ry; y++) {
    for (let x = cx - rx; x <= cx + rx; x++) {
      const nx = rx === 0 ? 0 : (x - cx) / rx;
      const ny = ry === 0 ? 0 : (y - cy) / ry;
      let hit = nx * nx + ny * ny <= 1;
      if (shape === "square") hit = true;
      if (shape === "triangle") hit = ny >= -1 && Math.abs(nx) <= (ny + 1) * 0.5;
      if (hit) out.push([x, y]);
    }
  }
  return out.length ? out : [[cx, cy]];
}

function bresenham(x0, y0, x1, y1) {
  const pts = [];
  let x = x0, y = y0;
  const dx = Math.abs(x1 - x0), dy = Math.abs(y1 - y0);
  const sx = x0 < x1 ? 1 : -1, sy = y0 < y1 ? 1 : -1;
  let err = dx - dy;
  for (let n = 0; n < 2000; n++) {
    pts.push([x, y]);
    if (x === x1 && y === y1) break;
    const e2 = 2 * err;
    if (e2 > -dy) { err -= dy; x += sx; }
    if (e2 < dx) { err += dx; y += sy; }
  }
  return pts;
}

function lineCells(a, b) {
  const s = getState();
  const set = new Set();
  const out = [];
  for (const [x, y] of bresenham(a.x, a.y, b.x, b.y)) {
    for (const c of brushCells(x, y, s.brushRx, s.brushRy, s.shape)) {
      const k = c[0] + "," + c[1];
      if (set.has(k)) continue;
      set.add(k);
      out.push(c);
    }
  }
  return out;
}

function rectCells(a, b) {
  const r = normRect(a, b);
  const out = [];
  for (let y = r.y0; y <= r.y1; y++) for (let x = r.x0; x <= r.x1; x++) out.push([x, y]);
  return out;
}

function normRect(a, b) {
  return { x0: Math.min(a.x, b.x), y0: Math.min(a.y, b.y), x1: Math.max(a.x, b.x), y1: Math.max(a.y, b.y) };
}

function flood(x, y, material) {
  const target = sim.getCellMaterial(x, y);
  if (target === material) return;
  const { w, h } = sim.getWorldSize();
  const stack = [[x, y]];
  const seen = new Set();
  beginLog();
  let guard = w * h;
  while (stack.length && guard > 0) {
    const [cx, cy] = stack.pop();
    if (cx < 0 || cy < 0 || cx >= w || cy >= h) continue;
    const k = cx + cy * w;
    if (seen.has(k)) continue;
    seen.add(k);
    guard--;
    if (sim.getCellMaterial(cx, cy) !== target) continue;
    note(cx, cy);
    sim.mod.set_cell(cx, cy, material);
    stack.push([cx + 1, cy], [cx - 1, cy], [cx, cy + 1], [cx, cy - 1]);
  }
  endLog(material);
}

function sample(c) {
  const id = sim.getCellMaterial(c.x, c.y);
  setState({ materialId: id, category: findMaterial(id).category });
}

function setStatus(msg) {
  statusNote = msg;
  statusUntil = performance.now() + 6000;
  console.info(msg);
}
function readCell(x, y) {
  if (sim.mod && typeof sim.mod.cell_state === "function") {
    const s = sim.mod.cell_state(x, y);
    return { m: s[0] | 0, t: s[1], vx: s[2], vy: s[3], f: s[4] | 0, life: s[5], p: s[6] };
  }
  return { m: sim.getCellMaterial(x, y) };
}
function writeCell(x, y, c) {
  if (!c || c.t == null || typeof sim.mod.set_cell_full !== "function") {
    sim.mod.set_cell(x, y, c.m | 0);
    return;
  }
  sim.mod.set_cell_full(x, y, c.m | 0, c.t, c.vx, c.vy, c.f | 0, c.life, c.p);
}
function beginLog() { strokeLog = new Map(); }
function note(x, y) {
  if (!strokeLog) return;
  const k = x + "," + y;
  if (!strokeLog.has(k)) strokeLog.set(k, readCell(x, y));
}
function endLog() {
  if (!strokeLog || strokeLog.size === 0) { strokeLog = null; return; }
  const cells = [];
  for (const [k, before] of strokeLog) {
    const [x, y] = k.split(",").map(Number);
    cells.push({ x, y, before, after: readCell(x, y) });
  }
  undoStack.push({ cells });
  if (undoStack.length > 30) undoStack.shift();
  redoStack.length = 0;
  strokeLog = null;
}
function undo() {
  const op = undoStack.pop();
  if (!op) return;
  const list = op.cells || op.before || [];
  for (const c of list) writeCell(c.x, c.y, c.before || { m: c.m });
  redoStack.push(op);
}
function redo() {
  const op = redoStack.pop();
  if (!op) return;
  const list = op.cells || op.before || [];
  for (const c of list) {
    if (c.after) writeCell(c.x, c.y, c.after);
    else sim.mod.set_cell(c.x, c.y, op.material | 0);
  }
  undoStack.push(op);
}

function activeRect() {
  if (selection) return selection;
  if (!hover) return null;
  const s = getState();
  return { x0: hover.x - s.brushRx, y0: hover.y - s.brushRy, x1: hover.x + s.brushRx, y1: hover.y + s.brushRy };
}
function grab(rect) {
  if (!rect) return null;
  const cells = [];
  for (let y = rect.y0; y <= rect.y1; y++) {
    for (let x = rect.x0; x <= rect.x1; x++) {
      const st = readCell(x, y);
      if (st.m) cells.push({ dx: x - rect.x0, dy: y - rect.y0, ...st });
    }
  }
  return { w: rect.x1 - rect.x0 + 1, h: rect.y1 - rect.y0 + 1, cells, name: "stamp" };
}
function copySel(clear) {
  const data = grab(selection || activeRect());
  if (!data) return;
  clipboard = data;
  localStorage.setItem("labsmatter-last-stamp", JSON.stringify(data));
  if (clear && selection) {
    beginLog();
    for (let y = selection.y0; y <= selection.y1; y++) {
      for (let x = selection.x0; x <= selection.x1; x++) { note(x, y); sim.mod.set_cell(x, y, 0); }
    }
    endLog(0);
  }
}
function normalizeStamp(cells) {
  if (!cells.length) return cells;
  const minX = Math.min(...cells.map((c) => c.dx));
  const minY = Math.min(...cells.map((c) => c.dy));
  return cells.map((c) => ({ ...c, dx: c.dx - minX, dy: c.dy - minY }));
}
// Screen-space 90° turn used by the existing stamp: (dx, dy) -> (dy, -dx).
// Velocity follows the same turn. Spark stores a conductor id in vx, so that
// value stays put. There is no player, portal, pipe, or SOAP index to retarget.
function turnVelocity(c) {
  if ((c.m | 0) === 16 || c.vx == null) return c;
  const vx = c.vy;
  const vy = -c.vx;
  return { ...c, vx, vy };
}
function rotatePaste() {
  if (!paste) return;
  paste = { ...paste, cells: normalizeStamp(paste.cells.map((c) => turnVelocity({ ...c, dx: c.dy, dy: -c.dx }))) };
}
function flipPaste(axis) {
  if (!paste) return;
  paste = {
    ...paste,
    cells: normalizeStamp(paste.cells.map((c) => {
      const next = {
        ...c,
        dx: axis === "x" ? -c.dx : c.dx,
        dy: axis === "y" ? -c.dy : c.dy,
      };
      if ((c.m | 0) === 16 || c.vx == null) return next;
      if (axis === "x") next.vx = -c.vx;
      if (axis === "y") next.vy = -c.vy;
      return next;
    })),
  };
}
function placeStamp(x, y, data) {
  beginLog();
  for (const c of data.cells) {
    note(x + c.dx, y + c.dy);
    writeCell(x + c.dx, y + c.dy, c);
  }
  endLog();
}
function saveStamp() {
  const data = grab(selection || activeRect());
  if (!data || !data.cells.length) return;
  data.name = "stamp-" + new Date().toISOString().slice(11, 19);
  const all = loadStamps();
  all.unshift(data);
  localStorage.setItem("labsmatter-stamps", JSON.stringify(all.slice(0, 24)));
  localStorage.setItem("labsmatter-last-stamp", JSON.stringify(data));
  clipboard = data;
}
function loadStamps() {
  try { return JSON.parse(localStorage.getItem("labsmatter-stamps") || "[]"); } catch { return []; }
}
function loadLastStamp() {
  try {
    const raw = localStorage.getItem("labsmatter-last-stamp");
    if (raw) paste = JSON.parse(raw);
  } catch { /* ignore */ }
}
function openStamps() {
  const body = $("stamp-body");
  body.innerHTML = "";
  const all = loadStamps();
  if (!all.length) body.textContent = "No stamps yet. Select an area and press S.";
  for (const st of all) {
    const row = document.createElement("div");
    row.className = "stamp-row";
    row.innerHTML = `<span>${st.name} · ${st.w}×${st.h}</span>`;
    const b = document.createElement("button");
    b.className = "tb";
    b.type = "button";
    b.textContent = "Place";
    b.onclick = () => { paste = st; $("stamp-dialog").close(); };
    row.appendChild(b);
    body.appendChild(row);
  }
  $("stamp-dialog").showModal();
}

function shot() {
  const buf = sim.readRenderBuffer();
  const { w, h } = sim.getWorldSize();
  if (!buf) return;
  const c = document.createElement("canvas");
  c.width = w; c.height = h;
  const ctx = c.getContext("2d");
  const img = ctx.createImageData(w, h);
  const n = Math.min(buf.length, w * h * 4);
  img.data.set(buf.subarray(0, n));
  for (let i = 0; i < n; i += 4) {
    if (img.data[i + 3] === 0) {
      img.data[i] = 5;
      img.data[i + 1] = 8;
      img.data[i + 2] = 15;
      img.data[i + 3] = 255;
    }
  }
  ctx.putImageData(img, 0, 0);
  c.toBlob((blob) => {
    if (!blob) return;
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = "labsmatter.png";
    a.click();
    setTimeout(() => URL.revokeObjectURL(a.href), 2000);
  });
}

function saveWorld() {
  const blob = sim.saveBlob();
  if (!blob || !blob.length) {
    setStatus(sim.lastError() || "Save failed. Nothing was written.");
    return;
  }
  const a = document.createElement("a");
  a.href = URL.createObjectURL(new Blob([blob], { type: "application/octet-stream" }));
  a.download = "labsmatter-world.rmt";
  a.click();
  setTimeout(() => URL.revokeObjectURL(a.href), 2000);
}
function resetWorld() {
  if (!confirm("Clear the world?")) return;
  sim.reset();
  undoStack = [];
  redoStack = [];
  selection = null;
  paste = null;
}

function wireKeys() {
  window.addEventListener("keydown", (e) => {
    const tag = e.target && e.target.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA") {
      if (e.key === "Escape") e.target.blur();
      return;
    }
    if (document.querySelector("dialog[open]")) return;
    const hit = matchBinding(keys.bindings, e);
    if (!hit) {
      if (e.code.startsWith("Arrow")) panKey(e);
      return;
    }
    if (e.repeat && hit.action !== "brush-dec" && hit.action !== "brush-inc") return;
    e.preventDefault();
    runAction(hit);
  });
}

function panKey(e) {
  const s = getState();
  const cam = { ...s.camera };
  if (e.code === "ArrowLeft") cam.x += 24;
  if (e.code === "ArrowRight") cam.x -= 24;
  if (e.code === "ArrowUp") cam.y += 24;
  if (e.code === "ArrowDown") cam.y -= 24;
  setState({ camera: cam });
  e.preventDefault();
}

function runAction(hit) {
  const s = getState();
  switch (hit.action) {
    case "pause": setState({ playing: !s.playing }); break;
    case "step":
      setState({ playing: false });
      sim.step(1 / 60);
      break;
    case "search": $("material-search").focus(); $("material-search").select(); break;
    case "brush-shape": {
      const i = (SHAPES.indexOf(s.shape) + 1) % SHAPES.length;
      setState({ shape: SHAPES[i] });
      break;
    }
    case "brush-dec": setState({ brushRx: Math.max(0, s.brushRx - 4), brushRy: Math.max(0, s.brushRy - 4) }); break;
    case "brush-inc": setState({ brushRx: Math.min(24, s.brushRx + 4), brushRy: Math.min(24, s.brushRy + 4) }); break;
    case "brush-dec-1": setState({ brushRx: Math.max(0, s.brushRx - 1), brushRy: Math.max(0, s.brushRy - 1) }); break;
    case "brush-inc-1": setState({ brushRx: Math.min(24, s.brushRx + 1), brushRy: Math.min(24, s.brushRy + 1) }); break;
    case "hud": setState({ showHud: !s.showHud }); break;
    case "grid": {
      const next = s.gridSize >= 32 ? 32 : Math.max(4, (s.gridSize || 4) * 2);
      setState({ grid: true, gridSize: s.gridSize === 0 ? 4 : next });
      break;
    }
    case "grid-dec": {
      const next = s.gridSize <= 4 ? 0 : Math.floor(s.gridSize / 2);
      setState({ grid: next > 0, gridSize: next });
      break;
    }
    case "gravity": sim.cycleGravity(); break;
    case "air": sim.cycleAir(); break;
    case "edge": sim.cycleEdge(); break;
    case "newton": sim.toggleNewtonian(); break;
    case "ambient": sim.toggleAmbient(); break;
    case "stamp-rot": rotatePaste(); break;
    case "stamp-flip-x": flipPaste("x"); break;
    case "stamp-flip-y": flipPaste("y"); break;
    case "zoom": setState({ camera: { ...s.camera, zoom: s.camera.zoom > 1.2 ? 1 : 3 } }); break;
    case "zoom-in": setState({ camera: { ...s.camera, zoom: Math.min(6, s.camera.zoom * 1.15) } }); break;
    case "zoom-out": setState({ camera: { ...s.camera, zoom: Math.max(0.25, s.camera.zoom / 1.15) } }); break;
    case "zoom-reset": setState({ camera: { x: 0, y: 0, zoom: 1 } }); break;
    case "view": setView(hit.arg | 0); break;
    case "heat-toggle": setView(s.view === 2 ? 0 : 2); break;
    case "shot": shot(); break;
    case "stamp-save": saveStamp(); break;
    case "stamp-load": loadLastStamp(); break;
    case "stamp-lib": openStamps(); break;
    case "select": setState({ tool: "select" }); break;
    case "tool":
      if (hit.arg === "eraser") setState({ tool: "eraser" });
      else if (hit.arg === "fill") setState({ tool: "brush" });
      else setState({ tool: "brush" });
      break;
    case "undo": undo(); break;
    case "redo": redo(); break;
    case "copy": copySel(false); break;
    case "cut": copySel(true); break;
    case "paste": loadLastStamp(); break;
    case "fullscreen":
      if (!document.fullscreenElement) document.documentElement.requestFullscreen?.();
      else document.exitFullscreen?.();
      break;
    case "reset": resetWorld(); break;
    case "help": openHelp(); break;
    default: break;
  }
}

function openHelp() {
  const rows = keys.bindings.map((b) => `<tr><td>${formatBinding(b)}</td><td>${b.action}${b.arg != null ? " " + b.arg : ""}</td></tr>`).join("");
  $("help-body").innerHTML = `<table class="keys"><tr><td>Left drag</td><td>Draw primary</td></tr><tr><td>Right drag</td><td>Draw secondary</td></tr><tr><td>Middle click</td><td>Sample material</td></tr><tr><td>Wheel</td><td>${keys.wheel === "brush" ? "Brush size (Shift horizontal, Ctrl vertical)" : "Zoom"}</td></tr><tr><td>Shift drag</td><td>Line</td></tr><tr><td>Ctrl drag</td><td>Filled rectangle</td></tr><tr><td>Ctrl+Shift click</td><td>Flood fill</td></tr><tr><td>Alt drag</td><td>Pan</td></tr><tr><td>Arrows</td><td>Pan</td></tr>${rows}</table>`;
  if (!$("help-dialog").open) $("help-dialog").showModal();
}

let rebindAction = null;
function openKeys() {
  const body = $("keys-body");
  body.innerHTML = "";
  for (const [action, label] of ACTION_LABELS) {
    const found = keys.bindings.find((b) => b.action === action);
    const row = document.createElement("div");
    row.className = "stamp-row";
    const name = document.createElement("span");
    name.textContent = label;
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "tb";
    btn.textContent = found ? formatBinding(found) : "unbound";
    btn.onclick = () => { rebindAction = action; btn.textContent = "press a key…"; };
    row.append(name, btn);
    body.appendChild(row);
  }
  if (!$("keys-dialog").open) $("keys-dialog").showModal();
}

window.addEventListener("keydown", (e) => {
  if (!rebindAction || !$("keys-dialog").open) return;
  if (e.key === "Escape") { rebindAction = null; openKeys(); return; }
  e.preventDefault();
  e.stopPropagation();
  const action = rebindAction;
  keys.bindings = keys.bindings.filter((b) => b.action !== action);
  keys.bindings.push({ action, code: e.code, ctrl: !!(e.ctrlKey || e.metaKey), shift: !!e.shiftKey, alt: !!e.altKey, arg: null });
  keys.preset = "custom";
  saveConfig(keys);
  rebindAction = null;
  openKeys();
}, true);

function cellFromEvent(e) {
  const rect = canvas.getBoundingClientRect();
  const s = getState();
  const cell = 4 * (s.camera.zoom || 1);
  return {
    x: Math.floor((e.clientX - rect.left - s.camera.x) / cell),
    y: Math.floor((e.clientY - rect.top - s.camera.y) / cell),
  };
}

function drawOverlay() {
  if (!octx || !canvas) return;
  const w = overlay.width, h = overlay.height;
  octx.clearRect(0, 0, w, h);
  const s = getState();
  const cell = 4 * (s.camera.zoom || 1);
  octx.strokeStyle = "rgba(255,255,255,.85)";
  octx.lineWidth = 1;
  const drawCell = (x, y) => {
    octx.strokeRect(s.camera.x + x * cell + 0.5, s.camera.y + y * cell + 0.5, cell - 1, cell - 1);
  };
  if (selection) {
    octx.strokeStyle = "rgba(120,200,255,.9)";
    for (let y = selection.y0; y <= selection.y1; y++) for (let x = selection.x0; x <= selection.x1; x++) {
      if (x === selection.x0 || x === selection.x1 || y === selection.y0 || y === selection.y1) drawCell(x, y);
    }
  }
  if (stroke && (stroke.kind === "line" || stroke.kind === "rect" || stroke.kind === "select")) {
    const cells = stroke.kind === "line" ? lineCells(stroke.a, stroke.b) : rectCells(stroke.a, stroke.b);
    octx.strokeStyle = stroke.kind === "select" ? "rgba(120,200,255,.9)" : "rgba(255,255,255,.8)";
    for (const [x, y] of cells) drawCell(x, y);
  } else if (hover && !stroke) {
    for (const [x, y] of brushCells(hover.x, hover.y, s.brushRx, s.brushRy, s.shape)) drawCell(x, y);
  }
  if (paste && hover) {
    octx.fillStyle = "rgba(255,255,255,.25)";
    for (const c of paste.cells) {
      octx.fillRect(s.camera.x + (hover.x + c.dx) * cell, s.camera.y + (hover.y + c.dy) * cell, cell, cell);
    }
  }
}

function paintHud() {
  const s = getState();
  if (!s.showHud) return;
  const fps = renderer && renderer.getFps ? renderer.getFps() : 0;
  const d = sim.getDiagnostics ? sim.getDiagnostics() : {};
  const note = statusUntil > performance.now() ? `  ${statusNote}` : "";
  const adapter = `${renderer?.backend || "none"}${renderer?.note ? " " + renderer.note : ""}`;
  const mod = sim.moduleId ? sim.moduleId() : "";
  const compose = sim.renderMillis ? sim.renderMillis() : 0;
  $("hud-fps").textContent = `FPS ${fps.toFixed(0)}  TPS ${s.playing ? measuredTps.toFixed(0) : "0"}  compose ${compose.toFixed(2)} ms  ${adapter}  ${mod}${note}`;
  $("hud-parts").textContent = `${(d.particles || 0).toLocaleString()} pts  ${VIEWS[s.view] || ""}  ${sim.simStatus ? sim.simStatus() : ""}  grid ${s.grid ? s.gridSize : 0}`;
  if (!hover) { $("hud-cell").textContent = "—"; return; }
  const m = findMaterial(sim.getCellMaterial(hover.x, hover.y));
  const t = sim.getCellTemperature(hover.x, hover.y);
  const p = sim.getCellPressure(hover.x, hover.y);
  const vx = sim.getCellVx ? sim.getCellVx(hover.x, hover.y) : 0;
  const vy = sim.getCellVy ? sim.getCellVy(hover.x, hover.y) : 0;
  $("hud-cell").textContent = `${hover.x},${hover.y}  ${t.toFixed(0)}K  ${(p / 1000).toFixed(1)}kPa  vx ${vx.toFixed(2)} vy ${vy.toFixed(2)}`;
  $("hud-mat").textContent = `${m.abbr} ${m.name}`;
}

boot().catch((e) => setAppState("error", String(e?.message || e)));
