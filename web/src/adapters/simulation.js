// Adapter interface between UI and simulation core.
// The UI never imports simulation internals; it talks only through this adapter.
// MockSimulation provides an offline, in-browser stand-in until the WASM core lands.

export class SimulationAdapter {
  async init(config) { throw new Error("not implemented"); }
  isReady() { return false; }
  tick(dt) {}
  pause() {}
  resume() {}
  step() {}
  setSpeed(mult) {}
  reset() {}
  applyCommand(cmd) {}
  getSnapshot() { return null; }
  subscribe(fn) { return () => {}; }
  getDiagnostics() { return {}; }
  dispose() {}
}

// ---- Mock implementation ----
const W = 128, H = 128; // grid cells
export class MockSimulation extends SimulationAdapter {
  constructor() {
    super();
    this.grid = new Uint16Array(W * H); // material id per cell
    this.temp = new Float32Array(W * H);
    this.temp.fill(293);
    this.playing = false;
    this.speed = 1;
    this.acc = 0;
    this.listeners = new Set();
    this.ready = false;
    this.tickCount = 0;
    this.lastTickMs = 0;
    this.commands = [];
    this.seed = 1;
  }
  async init() {
    this.ready = true;
    // scatter some sand & water for demo
    for (let i = 0; i < 400; i++) {
      const x = 20 + (Math.random() * 40 | 0);
      const y = 20 + (Math.random() * 40 | 0);
      this.grid[y * W + x] = 2;
      this.temp[y * W + x] = 293;
    }
    for (let i = 0; i < 200; i++) {
      const x = 70 + (Math.random() * 30 | 0);
      const y = 60 + (Math.random() * 30 | 0);
      this.grid[y * W + x] = 3;
    }
    this._emit();
    return true;
  }
  isReady() { return this.ready; }
  pause() { this.playing = false; this._emit(); }
  resume() { this.playing = true; }
  step() { this._tick(1 / 60); }
  setSpeed(m) { this.speed = m; }
  reset() {
    this.grid.fill(0); this.temp.fill(293); this.tickCount = 0; this.acc = 0;
    this._emit();
  }
  applyCommand(cmd) {
    if (!cmd) return;
    switch (cmd.type) {
      case "paint": {
        const { x, y, material, size } = cmd;
        for (let dy = -size; dy <= size; dy++)
          for (let dx = -size; dx <= size; dx++) {
            if (dx * dx + dy * dy > size * size) continue;
            const cx = x + dx, cy = y + dy;
            if (cx < 0 || cy < 0 || cx >= W || cy >= H) continue;
            this.grid[cy * W + cx] = material;
            this.temp[cy * W + cx] = material === 0 ? 293 : (this.temp[cy * W + cx] || 293);
          }
        break;
      }
      case "line": {
        const { x0, y0, x1, y1, material, size } = cmd;
        const steps = Math.max(Math.abs(x1 - x0), Math.abs(y1 - y0));
        for (let i = 0; i <= steps; i++) {
          const t = steps === 0 ? 0 : i / steps;
          const x = Math.round(x0 + (x1 - x0) * t);
          const y = Math.round(y0 + (y1 - y0) * t);
          this.applyCommand({ type: "paint", x, y, material, size: size - 1 < 1 ? 1 : size - 1 });
        }
        break;
      }
      case "rect": {
        const { x0, y0, x1, y1, material, fill } = cmd;
        const minX = Math.min(x0, x1), maxX = Math.max(x0, x1);
        const minY = Math.min(y0, y1), maxY = Math.max(y0, y1);
        for (let y = minY; y <= maxY; y++)
          for (let x = minX; x <= maxX; x++) {
            if (!fill && x !== minX && x !== maxX && y !== minY && y !== maxY) continue;
            if (x < 0 || y < 0 || x >= W || y >= H) continue;
            this.grid[y * W + x] = material;
          }
        break;
      }
      case "fill": {
        const { x, y, material } = cmd;
        if (x < 0 || y < 0 || x >= W || y >= H) break;
        const target = this.grid[y * W + x];
        if (target === material) break;
        const stack = [[x, y]];
        while (stack.length) {
          const [cx, cy] = stack.pop();
          if (cx < 0 || cy < 0 || cx >= W || cy >= H) continue;
          if (this.grid[cy * W + cx] !== target) continue;
          this.grid[cy * W + cx] = material;
          stack.push([cx + 1, cy], [cx - 1, cy], [cx, cy + 1], [cx, cy - 1]);
        }
        break;
      }
    }
    this._emit();
  }
  tick(dt) {
    if (!this.playing) return;
    this.acc += dt * this.speed;
    const step = 1 / 60;
    let steps = 0;
    while (this.acc >= step && steps < 8) {
      this._tick(step);
      this.acc -= step;
      steps++;
    }
    if (steps === 8 && this.acc >= step) this.acc = 0; // drop excess
  }
  _tick(dt) {
    const t0 = performance.now();
    // Simple cellular-ish update: powders fall, liquids flow down/sideways, gases rise.
    const g = this.grid, T = this.temp;
    // work on a copy for determinism
    const next = new Uint16Array(g);
    const nT = new Float32Array(T);
    for (let y = H - 1; y >= 0; y--) {
      for (let x = 0; x < W; x++) {
        const i = y * W + x;
        const m = g[i];
        if (m === 0) continue;
        // gravity for powders/liquids
        if (m === 2 || m === 11 || m === 15) { // powders
          if (y + 1 < H && g[(y + 1) * W + x] === 0) {
            next[i] = 0; next[(y + 1) * W + x] = m; nT[(y + 1) * W + x] = T[i]; nT[i] = 293;
          }
        } else if (m === 3 || m === 4 || m === 9 || m === 10) { // liquids
          if (y + 1 < H && g[(y + 1) * W + x] === 0) {
            next[i] = 0; next[(y + 1) * W + x] = m; nT[(y + 1) * W + x] = T[i];
          } else {
            // sideways
            const dir = (this.seed + x + y) & 1 ? 1 : -1;
            const nx = x + dir;
            if (nx >= 0 && nx < W && g[y * W + nx] === 0) {
              next[i] = 0; next[y * W + nx] = m; nT[y * W + nx] = T[i];
            }
          }
        } else if (m === 5 || m === 12) { // gases rise
          if (y - 1 >= 0 && g[(y - 1) * W + x] === 0) {
            next[i] = 0; next[(y - 1) * W + x] = m; nT[(y - 1) * W + x] = T[i];
          }
        }
      }
    }
    // fire spreads to adjacent wood/oil/gunpowder
    for (let y = 0; y < H; y++) {
      for (let x = 0; x < W; x++) {
        const i = y * W + x;
        if (g[i] === 6) {
          for (const [dx, dy] of [[1,0],[-1,0],[0,1],[0,-1]]) {
            const nx = x + dx, ny = y + dy;
            if (nx < 0 || ny < 0 || nx >= W || ny >= H) continue;
            const j = ny * W + nx;
            if (g[j] === 7 || g[j] === 4 || g[j] === 15) {
              if (Math.random() < 0.05) next[j] = 6;
            }
          }
          // fire dies out
          if (Math.random() < 0.02) next[i] = 12;
        }
      }
    }
    this.grid = next;
    this.temp = nT;
    this.tickCount++;
    this.lastTickMs = performance.now() - t0;
    this._emit();
  }
  getSnapshot() {
    return { width: W, height: H, grid: this.grid, temp: this.temp, tick: this.tickCount };
  }
  subscribe(fn) { this.listeners.add(fn); return () => this.listeners.delete(fn); }
  _emit() { for (const fn of this.listeners) fn(this.getSnapshot()); }
  getDiagnostics() {
    let particles = 0;
    for (let i = 0; i < this.grid.length; i++) if (this.grid[i] !== 0) particles++;
    const chunks = Math.ceil((W * H) / (16 * 16));
    return {
      fps: 0, // filled by renderer loop
      tickMs: this.lastTickMs,
      renderMs: 0,
      particles,
      chunks,
      memoryMB: (this.grid.byteLength + this.temp.byteLength) / (1024 * 1024),
      state: this.playing ? "RUNNING" : "PAUSED",
      tick: this.tickCount,
    };
  }
  dispose() { this.listeners.clear(); }
}

export const GRID_W = W;
export const GRID_H = H;
