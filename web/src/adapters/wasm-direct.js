// Direct WASM adapter using the generated wasm-bindgen bindings (real module).
// Instantiation goes through the generated JS glue so imports (externref
// table, throw helpers) are always correct. Falls back with a clear error
// if the pkg/ artifacts are missing.
export class WasmDirectAdapter {
  constructor() {
    this.mod = null;
    this.exports = null;
    this.memory = null;
    this.ready = false;
    this.width = 256;
    this.height = 256;
  }
  async init(cfg = {}) {
    this.width = cfg.width || 256;
    this.height = cfg.height || 256;
    if (typeof WebAssembly === "undefined") throw new Error("WebAssembly not supported");
    // Resolve the generated bindings relative to this module so the app works
    // under any base path (e.g. GitHub Pages project sites at /LabsMatter/).
    // `import.meta.url` is .../src/adapters/wasm-direct.js, so ../../pkg is the
    // site root's pkg/ directory. No absolute '/pkg' paths: those break under
    // a subpath deployment.
    const paths = [
      "../../pkg/realmatter_wasm.js?sim=10",
      "../pkg/realmatter_wasm.js?sim=10",
      "./pkg/realmatter_wasm.js?sim=10",
    ];
    let lastErr = null;
    for (const p of paths) {
      try {
        const url = new URL(p, import.meta.url).href;
        const mod = await import(/* @vite-ignore */ url);
        if (typeof mod.default !== "function") continue;
        const ex = await mod.default();
        if (!ex || !ex.memory) continue;
        if (typeof mod.init !== "function" || typeof mod.step !== "function") continue;
        this.mod = mod;
        this.exports = ex;
        this.memory = ex.memory;
        break;
      } catch (e) { lastErr = e; }
    }
    if (!this.mod) throw new Error("wasm module not found or missing exports: " + (lastErr?.message || lastErr));
    this.mod.init(this.width, this.height);
    this.ready = true;
    return true;
  }
  isReady() { return this.ready; }
  step(dt = 1 / 60) {
    if (this.mod && typeof this.mod.step === "function") {
      const t0 = performance.now();
      try { this.mod.step(dt); } catch (e) { console.warn(e); }
      this._tickMs = performance.now() - t0;
    }
  }
  tick(dt = 1 / 60) { this.step(dt); }
  pause() {}
  resume() {}
  setSpeed() {}
  reset() {
    if (this.mod && typeof this.mod.reset === "function") { try { return !!this.mod.reset(); } catch {} }
    if (this.mod && typeof this.mod.init === "function") { try { this.mod.init(this.width, this.height); return true; } catch {} }
    return false;
  }
  paint(x, y, material, size = 1) {
    if (!this.mod || typeof this.mod.set_cell !== "function") return;
    const s = Math.max(1, size | 0);
    // Size 1 is a single cell. The old loop used radius max(size, 1) and
    // skipped the circle test at radius 1, so the smallest brush painted 3×3.
    if (s <= 1) {
      try { this.mod.set_cell(x, y, material); } catch (e) { console.warn(e); }
      return;
    }
    try {
      for (let dy = -s; dy <= s; dy++) {
        for (let dx = -s; dx <= s; dx++) {
          if (dx * dx + dy * dy > s * s) continue;
          this.mod.set_cell(x + dx, y + dy, material);
        }
      }
    } catch (e) { console.warn(e); }
  }
  line(x0, y0, x1, y1, material, size = 1) {
    const steps = Math.max(Math.abs(x1 - x0), Math.abs(y1 - y0));
    for (let i = 0; i <= steps; i++) {
      const t = steps === 0 ? 0 : i / steps;
      this.paint(Math.round(x0 + (x1 - x0) * t), Math.round(y0 + (y1 - y0) * t), material, Math.max(1, size | 0));
    }
  }
  rect(x0, y0, x1, y1, material, fill = false) {
    const minX = Math.min(x0, x1), maxX = Math.max(x0, x1);
    const minY = Math.min(y0, y1), maxY = Math.max(y0, y1);
    for (let y = minY; y <= maxY; y++) {
      for (let x = minX; x <= maxX; x++) {
        if (!fill && x !== minX && x !== maxX && y !== minY && y !== maxY) continue;
        try { this.mod.set_cell(x, y, material); } catch {}
      }
    }
  }
  fill(x, y, material) {
    if (!this.mod || typeof this.mod.get_cell_material !== "function") return;
    const w = this.getWorldSize().w, h = this.getWorldSize().h;
    if (x < 0 || y < 0 || x >= w || y >= h) return;
    let target = 0;
    try { target = this.mod.get_cell_material(x, y); } catch { return; }
    if (target === material) return;
    const stack = [[x, y]];
    const seen = new Set();
    // Bound visits by the grid, not by raw stack pops. The old guard counted
    // duplicate neighbor pushes, so a fill of the empty world stopped halfway.
    let guard = w * h;
    while (stack.length && guard > 0) {
      const [cx, cy] = stack.pop();
      if (cx < 0 || cy < 0 || cx >= w || cy >= h) continue;
      const k = cy * w + cx;
      if (seen.has(k)) continue;
      seen.add(k);
      guard--;
      let m = -1;
      try { m = this.mod.get_cell_material(cx, cy); } catch { continue; }
      if (m !== target) continue;
      try { this.mod.set_cell(cx, cy, material); } catch {}
      stack.push([cx + 1, cy], [cx - 1, cy], [cx, cy + 1], [cx, cy - 1]);
    }
  }
  applyCommand(cmd) {
    if (!cmd || !this.mod) return;
    if (cmd.type === "paint") {
      const mat = cmd.material === 0 ? 0 : (cmd.material ?? 2);
      this.paint(cmd.x, cmd.y, mat, cmd.size || 1);
    } else if (cmd.type === "line") {
      this.line(cmd.x0, cmd.y0, cmd.x1, cmd.y1, cmd.material ?? 2, cmd.size || 1);
    } else if (cmd.type === "rect") {
      this.rect(cmd.x0, cmd.y0, cmd.x1, cmd.y1, cmd.material ?? 2, !!cmd.fill);
    } else if (cmd.type === "fill") {
      this.fill(cmd.x, cmd.y, cmd.material ?? 2);
    }
  }
  getSnapshot() { return null; }
  _refreshCounts() {
    const now = performance.now();
    if (this._countAt && now - this._countAt < 250) return;
    this._countAt = now;
    if (this.mod && typeof this.mod.particle_count === "function") {
      try {
        this._particles = this.mod.particle_count() | 0;
        return;
      } catch { /* count from the buffer below */ }
    }
    const { w, h } = this.getWorldSize();
    const buf = this.readRenderBuffer();
    let particles = 0;
    let chunks = 0;
    if (buf && w > 0 && h > 0) {
      const cs = 32;
      const cw = Math.ceil(w / cs);
      const ch = Math.ceil(h / cs);
      const hit = new Uint8Array(cw * ch);
      const stride = w * 4;
      for (let y = 0; y < h; y++) {
        const row = y * stride;
        const cy = (y / cs) | 0;
        for (let x = 0; x < w; x++) {
          if (buf[row + x * 4 + 3] !== 0) {
            particles++;
            hit[cy * cw + ((x / cs) | 0)] = 1;
          }
        }
      }
      for (let i = 0; i < hit.length; i++) chunks += hit[i];
    }
    this._particles = particles;
    this._chunks = chunks;
  }
  getDiagnostics() {
    let w = this.width, h = this.height;
    if (this.mod && typeof this.mod.world_width === "function") { try { w = this.mod.world_width(); } catch {} }
    if (this.mod && typeof this.mod.world_height === "function") { try { h = this.mod.world_height(); } catch {} }
    this._refreshCounts();
    let tick = 0;
    if (this.mod && typeof this.mod.tick_count === "function") {
      try { tick = Number(this.mod.tick_count()); } catch {}
    }
    return {
      fps: 0,
      tickMs: this._tickMs || 0,
      renderMs: 0,
      particles: this._particles || 0,
      chunks: this._chunks || 0,
      memoryMB: this.memory ? this.memory.buffer.byteLength / (1024 * 1024) : 0,
      state: this.ready ? "READY(WASM)" : "UNAVAILABLE",
      tick,
      worldW: w,
      worldH: h,
    };
  }
  getWorldSize() {
    let w = this.width, h = this.height;
    if (this.mod && typeof this.mod.world_width === "function") { try { w = this.mod.world_width(); } catch {} }
    if (this.mod && typeof this.mod.world_height === "function") { try { h = this.mod.world_height(); } catch {} }
    return { w, h };
  }
  getCellMaterial(x, y) { if (this.mod && typeof this.mod.get_cell_material === "function") { try { return this.mod.get_cell_material(x, y); } catch {} } return 0; }
  getCellTemperature(x, y) { if (this.mod && typeof this.mod.get_cell_temperature === "function") { try { return this.mod.get_cell_temperature(x, y); } catch {} } return 0; }
  saveBlob() { if (this.mod && typeof this.mod.save_blob === "function") { try { return this.mod.save_blob(); } catch (e) { console.warn(e); } } return null; }
  loadBlob(bytes) {
    if (this.mod && typeof this.mod.load_blob === "function") {
      try {
        let arr = bytes;
        if (bytes instanceof ArrayBuffer) arr = new Uint8Array(bytes);
        if (Array.isArray(bytes)) arr = new Uint8Array(bytes);
        return !!this.mod.load_blob(arr);
      } catch (e) { console.warn(e); }
    }
    return false;
  }
  readRenderBuffer() {
    if (!this.mod || !this.exports) return null;
    try {
      if (typeof this.mod.render_buffer_ptr !== "function") return null;
      const ptr = this.mod.render_buffer_ptr();
      const len = typeof this.mod.render_buffer_len === "function" ? this.mod.render_buffer_len() : 0;
      if (!ptr || len === 0) return null;
      const mem = this.exports.memory || this.memory;
      if (!mem) return null;
      this.memory = mem;
      return new Uint8Array(mem.buffer, ptr, len);
    } catch { return null; }
  }
  cycleGravity() { try { this.mod.cycle_gravity?.(); } catch (e) { console.warn(e); } }
  cycleAir() { try { this.mod.cycle_air?.(); } catch (e) { console.warn(e); } }
  cycleEdge() { try { this.mod.cycle_edge?.(); } catch (e) { console.warn(e); } }
  toggleNewtonian() { try { this.mod.toggle_newtonian?.(); } catch (e) { console.warn(e); } }
  toggleAmbient() { try { this.mod.toggle_ambient?.(); } catch (e) { console.warn(e); } }
  simStatus() { try { return this.mod.sim_status?.() || ""; } catch { return ""; } }
  moduleId() { try { return this.mod.module_id?.() || "realmatter_wasm"; } catch { return "realmatter_wasm"; } }
  lastError() { try { return this.mod.last_error?.() || ""; } catch { return ""; } }
  renderMillis() { try { return Number(this.mod.render_millis?.() || 0); } catch { return 0; } }
  setViewMode(mode) {
    if (this.mod && typeof this.mod.set_view_mode === "function") {
      try { this.mod.set_view_mode(mode | 0); } catch (e) { console.warn(e); }
    }
  }
  getCellPressure(x, y) {
    if (this.mod && typeof this.mod.get_cell_pressure === "function") {
      try { return this.mod.get_cell_pressure(x, y); } catch { /* ignore */ }
    }
    return 0;
  }
  getCellVx(x, y) {
    if (this.mod && typeof this.mod.get_cell_vx === "function") {
      try { return this.mod.get_cell_vx(x, y); } catch { /* ignore */ }
    }
    return 0;
  }
  getCellVy(x, y) {
    if (this.mod && typeof this.mod.get_cell_vy === "function") {
      try { return this.mod.get_cell_vy(x, y); } catch { /* ignore */ }
    }
    return 0;
  }
  canPaint() { return !!(this.mod && typeof this.mod.set_cell === "function"); }
  canReset() { return !!(this.mod && (typeof this.mod.reset === "function" || typeof this.mod.init === "function")); }
  canRead() { return !!(this.mod && typeof this.mod.get_cell_material === "function"); }
  canSave() { return !!(this.mod && typeof this.mod.save_blob === "function" && typeof this.mod.load_blob === "function"); }
  dispose() { this.mod = null; this.exports = null; this.memory = null; this.ready = false; }
}
