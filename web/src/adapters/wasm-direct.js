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
    const paths = [
      "../../pkg/realmatter_wasm.js",
      "./pkg/realmatter_wasm.js",
      "/pkg/realmatter_wasm.js",
      "../pkg/realmatter_wasm.js",
      "./web/pkg/realmatter_wasm.js",
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
      try { this.mod.step(dt); } catch (e) { console.warn(e); }
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
    try {
      for (let dy = -s; dy <= s; dy++) {
        for (let dx = -s; dx <= s; dx++) {
          if (s > 1 && dx * dx + dy * dy > s * s) continue;
          this.mod.set_cell(x + dx, y + dy, material);
        }
      }
    } catch (e) { console.warn(e); }
  }
  line(x0, y0, x1, y1, material, size = 1) {
    const steps = Math.max(Math.abs(x1 - x0), Math.abs(y1 - y0));
    for (let i = 0; i <= steps; i++) {
      const t = steps === 0 ? 0 : i / steps;
      this.paint(Math.round(x0 + (x1 - x0) * t), Math.round(y0 + (y1 - y0) * t), material, Math.max(1, size - 1));
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
    let guard = w * h;
    while (stack.length && guard-- > 0) {
      const [cx, cy] = stack.pop();
      if (cx < 0 || cy < 0 || cx >= w || cy >= h) continue;
      const k = cy * w + cx;
      if (seen.has(k)) continue;
      seen.add(k);
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
  getDiagnostics() {
    let w = this.width, h = this.height;
    if (this.mod && typeof this.mod.world_width === "function") { try { w = this.mod.world_width(); } catch {} }
    if (this.mod && typeof this.mod.world_height === "function") { try { h = this.mod.world_height(); } catch {} }
    return { fps: 0, tickMs: 0, renderMs: 0, particles: 0, chunks: 0, memoryMB: this.memory ? this.memory.buffer.byteLength / (1024 * 1024) : 0, state: this.ready ? "READY(WASM)" : "UNAVAILABLE", tick: 0, worldW: w, worldH: h };
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
  canPaint() { return !!(this.mod && typeof this.mod.set_cell === "function"); }
  canReset() { return !!(this.mod && (typeof this.mod.reset === "function" || typeof this.mod.init === "function")); }
  canRead() { return !!(this.mod && typeof this.mod.get_cell_material === "function"); }
  canSave() { return !!(this.mod && typeof this.mod.save_blob === "function" && typeof this.mod.load_blob === "function"); }
  dispose() { this.mod = null; this.exports = null; this.memory = null; this.ready = false; }
}
