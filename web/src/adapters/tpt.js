// Experimental adapter for the new TPT-parity runtime (labsmatter-wasm).
//
// The Rust `realmatter-tpt::Sim` is authoritative. This file only marshals
// calls across the wasm boundary and reads the render buffer. It never
// implements simulation logic in JavaScript.
//
// Coordinates are engine pixels (1 unit == 1 particle == 1 texel). The
// renderer treats each unit as a 4*zoom CSS-pixel quad, exactly like the
// production Cell engine, so the existing camera / cursor mapping is reused.
//
// Wall palette entries use ids in [WALL_ID_BASE, WALL_ID_BASE+64). `setCell`
// routes those to `paint_wall`; `getCellMaterial` reports a wall as its
// palette id when no particle occupies the pixel.

import { WALL_ID_BASE, wallIdOf, isWallMaterial } from "../tpt-materials.js";

export class TptAdapter {
  constructor() {
    this.mod = null;
    this.exports = null;
    this.memory = null;
    this.ready = false;
    this.engine = "tpt";
    this.width = 256;
    this.height = 256;
    this.lastError = null;
    this._tickMs = 0;
  }

  async init(cfg = {}) {
    this.width = cfg.width || 256;
    this.height = cfg.height || 256;
    if (typeof WebAssembly === "undefined") throw new Error("WebAssembly not supported");
    // Resolve generated bindings relative to this module so subpath deploys
    // (GitHub Pages project sites) keep working.
    const paths = [
      "../../pkg-tpt/labsmatter_wasm.js?tpt=2",
      "../pkg-tpt/labsmatter_wasm.js?tpt=2",
      "./pkg-tpt/labsmatter_wasm.js?tpt=2",
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
      } catch (e) {
        lastErr = e;
      }
    }
    if (!this.mod) {
      this.lastError = lastErr?.message || String(lastErr);
      throw new Error(
        "labsmatter_wasm module not found or missing exports. Build it first:\n" +
          "  cargo build -p labsmatter-wasm --target wasm32-unknown-unknown --release\n" +
          "  wasm-bindgen target/wasm32-unknown-unknown/release/labsmatter_wasm.wasm --out-dir web/pkg-tpt --target web --no-typescript\n" +
          "Cause: " + this.lastError
      );
    }
    // Dimensions must be divisible by 4 (engine CELL). Clamp rather than fail.
    const w = Math.max(4, Math.floor(this.width / 4) * 4);
    const h = Math.max(4, Math.floor(this.height / 4) * 4);
    try {
      this.mod.init(w, h);
    } catch (e) {
      this.lastError = e?.message || String(e);
      throw new Error("labsmatter_wasm init failed: " + this.lastError);
    }
    this.ready = true;
    return true;
  }

  isReady() { return this.ready; }

  /** Stable identity for diagnostics: which Rust/WASM module is live. */
  wasmIdentity() {
    if (!this.mod) return "none";
    // TPT-only exports prove this is labsmatter-wasm, not realmatter-wasm.
    if (typeof this.mod.spawn_stickman === "function" && typeof this.mod.get_cell_info === "function") {
      return "labsmatter-wasm";
    }
    return "unknown";
  }

  step(dt = 1 / 60) {
    if (!this.mod || typeof this.mod.step !== "function") return;
    const t0 = performance.now();
    try { this.mod.step(dt); } catch (e) { console.warn(e); }
    this._tickMs = performance.now() - t0;
  }
  tick(dt = 1 / 60) { this.step(dt); }
  pause() {}
  resume() {}
  setSpeed() {}

  reset() {
    if (!this.mod || typeof this.mod.reset !== "function") return false;
    try { return !!this.mod.reset(); } catch { return false; }
  }

  // ---- Painting ---------------------------------------------------------
  setCell(x, y, material) {
    if (!this.mod) return false;
    const id = material | 0;
    try {
      if (isWallMaterial(id)) {
        this.mod.paint_wall(x, y, wallIdOf(id));
        return true;
      }
      return !!this.mod.set_cell(x, y, id);
    } catch (e) {
      console.warn(e);
      return false;
    }
  }

  paint(x, y, material, size = 1) {
    const s = Math.max(1, size | 0);
    if (s <= 1) { this.setCell(x, y, material); return; }
    for (let dy = -s; dy <= s; dy++) {
      for (let dx = -s; dx <= s; dx++) {
        if (dx * dx + dy * dy > s * s) continue;
        this.setCell(x + dx, y + dy, material);
      }
    }
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
        this.setCell(x, y, material);
      }
    }
  }

  fill(x, y, material) {
    if (!this.mod || typeof this.mod.get_cell_material !== "function") return;
    const { w, h } = this.getWorldSize();
    if (x < 0 || y < 0 || x >= w || y >= h) return;
    const target = this.getCellMaterial(x, y);
    if (target === material) return;
    const stack = [[x, y]];
    const seen = new Set();
    let guard = w * h;
    while (stack.length && guard > 0) {
      const [cx, cy] = stack.pop();
      if (cx < 0 || cy < 0 || cx >= w || cy >= h) continue;
      const k = cy * w + cx;
      if (seen.has(k)) continue;
      seen.add(k);
      guard--;
      if (this.getCellMaterial(cx, cy) !== target) continue;
      this.setCell(cx, cy, material);
      stack.push([cx + 1, cy], [cx - 1, cy], [cx, cy + 1], [cx, cy - 1]);
    }
  }

  applyCommand(cmd) {
    if (!cmd || !this.mod) return;
    if (cmd.type === "paint") this.paint(cmd.x, cmd.y, cmd.material ?? 0, cmd.size || 1);
    else if (cmd.type === "line") this.line(cmd.x0, cmd.y0, cmd.x1, cmd.y1, cmd.material ?? 0, cmd.size || 1);
    else if (cmd.type === "rect") this.rect(cmd.x0, cmd.y0, cmd.x1, cmd.y1, cmd.material ?? 0, !!cmd.fill);
    else if (cmd.type === "fill") this.fill(cmd.x, cmd.y, cmd.material ?? 0);
    else if (cmd.type === "paint-wall") this.mod.paint_wall(cmd.x, cmd.y, cmd.wall | 0);
  }

  // ---- Reading ----------------------------------------------------------
  getWorldSize() {
    let w = this.width, h = this.height;
    if (this.mod && typeof this.mod.world_width === "function") { try { w = this.mod.world_width(); } catch {} }
    if (this.mod && typeof this.mod.world_height === "function") { try { h = this.mod.world_height(); } catch {} }
    return { w, h };
  }

  getCellMaterial(x, y) {
    if (!this.mod) return 0;
    try {
      const m = this.mod.get_cell_material(x, y);
      if (m) return m;
      if (typeof this.mod.get_wall === "function") {
        const w = this.mod.get_wall(x, y);
        if (w) return WALL_ID_BASE + w;
      }
      return 0;
    } catch { return 0; }
  }

  getCellTemperature(x, y) {
    if (this.mod && typeof this.mod.get_cell_temperature === "function") {
      try { return this.mod.get_cell_temperature(x, y); } catch {}
    }
    return 0;
  }
  getCellPressure(x, y) {
    if (this.mod && typeof this.mod.get_cell_pressure === "function") {
      try { return this.mod.get_cell_pressure(x, y); } catch {}
    }
    return 0;
  }
  getCellVx(x, y) {
    if (this.mod && typeof this.mod.get_cell_vx === "function") {
      try { return this.mod.get_cell_vx(x, y); } catch {}
    }
    return 0;
  }
  getCellVy(x, y) {
    if (this.mod && typeof this.mod.get_cell_vy === "function") {
      try { return this.mod.get_cell_vy(x, y); } catch {}
    }
    return 0;
  }

  // ---- Extended inspection (QA / inspector) ----------------------------
  _num(fn, x, y, dflt = 0) {
    if (this.mod && typeof this.mod[fn] === "function") {
      try { return this.mod[fn](x, y); } catch {}
    }
    return dflt;
  }
  getCellLife(x, y) { return this._num("get_cell_life", x, y); }
  getCellCtype(x, y) { return this._num("get_cell_ctype", x, y); }
  getCellTmp(x, y) { return this._num("get_cell_tmp", x, y); }
  getCellTmp2(x, y) { return this._num("get_cell_tmp2", x, y); }
  getCellTmp3(x, y) { return this._num("get_cell_tmp3", x, y); }
  getCellTmp4(x, y) { return this._num("get_cell_tmp4", x, y); }
  getCellFlags(x, y) { return this._num("get_cell_flags", x, y); }
  getCellDcolour(x, y) { return this._num("get_cell_dcolour", x, y); }
  getCellPx(x, y) { return this._num("get_cell_px", x, y, -1); }
  getCellPy(x, y) { return this._num("get_cell_py", x, y, -1); }
  getCellEmap(x, y) { return this._num("get_cell_emap", x, y); }
  getCellAmbient(x, y) { return this._num("get_cell_ambient", x, y); }
  getGravityX(x, y) { return this._num("get_gravity_x", x, y); }
  getGravityY(x, y) { return this._num("get_gravity_y", x, y); }
  getEntityHead(which = 0) {
    if (this.mod && typeof this.mod.get_entity_head === "function") {
      try { return this.mod.get_entity_head(which); } catch {}
    }
    return -1;
  }
  getCellInfo(x, y) {
    if (this.mod && typeof this.mod.get_cell_info === "function") {
      try { return this.mod.get_cell_info(x, y); } catch {}
    }
    return null;
  }
  getWorldSettings() {
    if (this.mod && typeof this.mod.get_world_settings === "function") {
      try { return this.mod.get_world_settings(); } catch {}
    }
    return null;
  }

  // ---- Decoration ------------------------------------------------------
  setDeco(x, y, argb) {
    if (this.mod && typeof this.mod.set_deco === "function") {
      try { return !!this.mod.set_deco(x, y, argb >>> 0); } catch {}
    }
    return false;
  }
  getDeco(x, y) { return this._num("get_deco", x, y); }
  clearDeco() {
    if (this.mod && typeof this.mod.clear_all_deco === "function") {
      try { this.mod.clear_all_deco(); return true; } catch {}
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
      // Re-fetch the memory view every frame: wasm memory growth detaches the
      // previous ArrayBuffer. Reusing a stale view renders a black frame.
      const mem = this.exports.memory || this.memory;
      if (!mem) return null;
      this.memory = mem;
      return new Uint8Array(mem.buffer, ptr, len);
    } catch { return null; }
  }

  // ---- Diagnostics ------------------------------------------------------
  getDiagnostics() {
    const { w, h } = this.getWorldSize();
    let tick = 0, particles = 0;
    if (this.mod) {
      if (typeof this.mod.tick_count === "function") { try { tick = Number(this.mod.tick_count()); } catch {} }
      if (typeof this.mod.particle_count === "function") { try { particles = Number(this.mod.particle_count()); } catch {} }
    }
    return {
      fps: 0,
      tickMs: this._tickMs || 0,
      renderMs: 0,
      particles,
      chunks: 0,
      memoryMB: this.memory ? this.memory.buffer.byteLength / (1024 * 1024) : 0,
      state: this.ready ? "READY(TPT)" : "UNAVAILABLE",
      engine: "tpt",
      tick,
      worldW: w,
      worldH: h,
    };
  }

  // ---- Settings / modes -------------------------------------------------
  setViewMode(mode) {
    if (this.mod && typeof this.mod.set_view_mode === "function") {
      try { this.mod.set_view_mode(mode | 0); } catch (e) { console.warn(e); }
    }
  }
  cycleGravity() { try { this.mod?.cycle_gravity?.(); } catch (e) { console.warn(e); } }
  cycleAir() { try { this.mod?.cycle_air?.(); } catch (e) { console.warn(e); } }
  cycleEdge() { try { this.mod?.cycle_edge?.(); } catch (e) { console.warn(e); } }
  toggleNewtonian() { try { this.mod?.toggle_newtonian?.(); } catch (e) { console.warn(e); } }
  toggleAmbient() { try { this.mod?.toggle_ambient?.(); } catch (e) { console.warn(e); } }
  simStatus() { try { return this.mod?.sim_status?.() || ""; } catch { return ""; } }

  // ---- Persistence ------------------------------------------------------
  saveBlob() {
    if (this.mod && typeof this.mod.save_blob === "function") {
      try { return this.mod.save_blob(); } catch (e) { console.warn(e); }
    }
    return null;
  }
  loadBlob(bytes) {
    if (this.mod && typeof this.mod.load_blob === "function") {
      try {
        let arr = bytes;
        if (bytes instanceof ArrayBuffer) arr = new Uint8Array(bytes);
        if (Array.isArray(bytes)) arr = new Uint8Array(bytes);
        const ok = !!this.mod.load_blob(arr);
        // Surface the migration/error report to the caller (and console) so a
        // legacy import or rejection is never silent.
        this.lastLoadReport =
          typeof this.mod.last_load_report === "function" ? this.mod.last_load_report() : "";
        if (this.lastLoadReport) console.info("[labsmatter] " + this.lastLoadReport);
        return ok;
      } catch (e) { console.warn(e); }
    }
    return false;
  }

  // ---- Capabilities -----------------------------------------------------
  canPaint() { return !!(this.mod && typeof this.mod.set_cell === "function"); }
  canReset() { return !!(this.mod && typeof this.mod.reset === "function"); }
  canRead() { return !!(this.mod && typeof this.mod.get_cell_material === "function"); }
  canSave() { return !!(this.mod && typeof this.mod.save_blob === "function" && typeof this.mod.load_blob === "function"); }
  dispose() { this.mod = null; this.exports = null; this.memory = null; this.ready = false; }
}
