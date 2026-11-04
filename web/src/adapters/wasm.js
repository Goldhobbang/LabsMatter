// WASM integration adapter for realmatter-wasm.
// Consumes the approved wasm-api contract shape. Falls back gracefully if unavailable.

export class WasmSimulationAdapter {
  constructor() {
    this.module = null;
    this.ready = false;
    this.width = 256;
    this.height = 256;
    this.listeners = new Set();
  }

  async init(config = {}) {
    this.width = config.width || 256;
    this.height = config.height || 256;
    try {
      if (typeof WebAssembly === "undefined") throw new Error("WebAssembly not supported");
      const paths = ["./pkg/realmatter_wasm.js", "/pkg/realmatter_wasm.js", "../pkg/realmatter_wasm.js", "./web/pkg/realmatter_wasm.js"];
      for (const p of paths) {
        try {
          const mod = await import(/* @vite-ignore */ p);
          if (mod?.default) {
            try { await mod.default(); } catch {}
          }
          // Verify exports match contract
          if (mod && (typeof mod.init === "function" || typeof mod.step === "function")) {
            this.module = mod;
            break;
          }
        } catch {}
      }
      if (!this.module) throw new Error("wasm module not found or missing exports");
      if (typeof this.module.init === "function") {
        const res = this.module.init(this.width, this.height);
        if (res instanceof Promise) await res;
      }
      this.ready = true;
      this._emit();
      return true;
    } catch (e) {
      this.ready = false;
      throw e;
    }
  }

  isReady() { return this.ready && !!this.module; }

  pause() {}
  resume() {}
  step(dt = 1/60) {
    if (this.module?.step) this.module.step(dt);
  }
  setSpeed() {}
  reset() {
    // Re-init as fallback
    if (this.module?.init) this.module.init(this.width, this.height);
  }
  applyCommand(cmd) {
    if (!this.module) return;
    if (cmd.type === "paint") {
      const { x, y, material } = cmd;
      if (typeof this.module.set_cell === "function") {
        try { this.module.set_cell(x, y, material); } catch(e) { console.warn(e); }
      }
    }
    if (cmd.type === "line" || cmd.type === "rect" || cmd.type === "fill") {
      // Best effort mapping if extended API exists
      if (typeof this.module.apply_command === "function") {
        try { this.module.apply_command(JSON.stringify(cmd)); } catch {}
      }
    }
  }
  getSnapshot() {
    // If module exposes render buffers, we could read them; for now return null
    // to indicate "use module's own rendering" or adapter handles separately
    return null;
  }
  subscribe(fn) { this.listeners.add(fn); return () => this.listeners.delete(fn); }
  _emit() { for (const fn of this.listeners) fn(null); }
  getDiagnostics() {
    return {
      fps: 0, tickMs: 0, renderMs: 0, particles: 0, chunks: 0,
      memoryMB: 0, state: this.ready ? "READY" : "UNAVAILABLE", tick: 0,
    };
  }
  getRenderBuffer() {
    if (!this.module) return null;
    try {
      if (typeof this.module.render_buffer_ptr !== "function") return null;
      const ptr = this.module.render_buffer_ptr();
      const len = this.module.render_buffer_len?.() || 0;
      if (!ptr || len === 0) return null;
      const mem = this.module.memory || this.module.wasm_memory;
      if (!mem) return null;
      const buf = new Uint8Array(mem.buffer, ptr, len);
      return { data: buf, ptr, len };
    } catch { return null; }
  }
  dispose() { this.listeners.clear(); this.module = null; this.ready = false; }
}
