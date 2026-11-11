// Renderer adapter. UI talks only through this interface.
// Canvas2DRenderer is the offline, dependency-free implementation.
// A WebGPU renderer (realmatter-renderer) can replace it via the same interface.

import { findMaterial } from "../materials.js";

export class RendererAdapter {
  attach(canvas) {}
  resize(w, h) {}
  render(snapshot, camera, opts) {}
  setOverlay(name, on) {}
  dispose() {}
}

export class Canvas2DRenderer extends RendererAdapter {
  constructor() {
    super();
    this.canvas = null;
    this.ctx = null;
    this.dpr = 1;
    this.gridW = 0;
    this.gridH = 0;
    this.cellPx = 4;
    this.overlay = { grid: true, heat: false };
    this.lastRenderMs = 0;
    this.fps = 0;
    this._fpsAcc = 0;
    this._fpsFrames = 0;
    this._fpsT = 0;
  }
  attach(canvas) {
    this.canvas = canvas;
    this.ctx = canvas.getContext("2d", { alpha: false });
    this.resize(canvas.clientWidth, canvas.clientHeight);
  }
  resize(w, h) {
    if (!this.canvas) return;
    if (!(w > 1 && h > 1)) return;
    this.dpr = Math.min(window.devicePixelRatio || 1, 2);
    this.canvas.width = Math.max(1, Math.floor(w * this.dpr));
    this.canvas.height = Math.max(1, Math.floor(h * this.dpr));
    this.ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
  }
  setOverlay(name, on) { this.overlay[name] = on; }
  renderRGBA(buf, gw, gh, camera, opts) {
    const t0 = performance.now();
    if (!this.ctx || !buf || gw <= 0 || gh <= 0) { this.lastRenderMs = performance.now() - t0; return; }
    const ctx = this.ctx;
    const w = this.canvas.width / this.dpr;
    const h = this.canvas.height / this.dpr;
    ctx.imageSmoothingEnabled = false;
    ctx.fillStyle = "#05060a";
    ctx.fillRect(0, 0, w, h);
    const zoom = camera.zoom || 1;
    const cellX = this.cellPx * zoom * (camera.scaleX || 1);
    const cellY = this.cellPx * zoom * (camera.scaleY || 1);
    const offX = camera.x;
    const offY = camera.y;
    // Draw via offscreen ImageData at cell resolution, then scale up.
    let img = this._rgbaImg;
    if (!img || img.width !== gw || img.height !== gh) {
      img = new ImageData(gw, gh);
      this._rgbaImg = img;
      this._rgbaCanvas = document.createElement("canvas");
      this._rgbaCanvas.width = gw;
      this._rgbaCanvas.height = gh;
    }
    const dst = img.data;
    const n = Math.min(buf.length, gw * gh * 4);
    dst.set(buf.subarray(0, n));
    const octx = this._rgbaCanvas.getContext("2d");
    octx.putImageData(img, 0, 0);
    ctx.drawImage(this._rgbaCanvas, offX, offY, gw * cellX, gh * cellY);
    drawGrid(ctx, this.overlay.grid, this.overlay.gridSize, cellX, cellY, offX, offY, gw, gh, w, h);
    this.lastRenderMs = performance.now() - t0;
    const now = performance.now();
    this._fpsAcc += 1;
    if (now - this._fpsT >= 500) {
      this.fps = (this._fpsAcc * 1000) / (now - this._fpsT);
      this._fpsAcc = 0;
      this._fpsT = now;
    }
  }
  render(snapshot, camera, opts) {
    const t0 = performance.now();
    if (!this.ctx || !snapshot) { this.lastRenderMs = performance.now() - t0; return; }
    const ctx = this.ctx;
    const w = this.canvas.width / this.dpr;
    const h = this.canvas.height / this.dpr;
    ctx.fillStyle = "#05060a";
    ctx.fillRect(0, 0, w, h);

    const { width: gw, height: gh, grid, temp } = snapshot;
    this.gridW = gw; this.gridH = gh;
    const zoom = camera.zoom;
    const cellX = this.cellPx * zoom * (camera.scaleX || 1);
    const cellY = this.cellPx * zoom * (camera.scaleY || 1);
    const offX = camera.x;
    const offY = camera.y;

    // visible range
    const startX = Math.max(0, Math.floor(-offX / cellX));
    const startY = Math.max(0, Math.floor(-offY / cellY));
    const endX = Math.min(gw, Math.ceil((w - offX) / cellX));
    const endY = Math.min(gh, Math.ceil((h - offY) / cellY));

    for (let y = startY; y < endY; y++) {
      for (let x = startX; x < endX; x++) {
        const m = grid[y * gw + x];
        if (m === 0) continue;
        const mat = findMaterial(m);
        let color = mat.color;
        if (this.overlay.heat) {
          const t = temp[y * gw + x];
          // map 200..1500 K to blue..red
          const k = Math.min(1, Math.max(0, (t - 200) / 1300));
          color = heatColor(k);
        }
        ctx.fillStyle = color;
        ctx.fillRect(offX + x * cellX, offY + y * cellY, cellX + 0.5, cellY + 0.5);
      }
    }

    drawGrid(ctx, this.overlay.grid, this.overlay.gridSize, cellX, cellY, offX, offY, gw, gh, w, h);

    // HUD crosshair center
    ctx.strokeStyle = "rgba(79,195,247,0.25)";
    ctx.beginPath();
    ctx.moveTo(w / 2, 0); ctx.lineTo(w / 2, h);
    ctx.moveTo(0, h / 2); ctx.lineTo(w, h / 2);
    ctx.stroke();

    this.lastRenderMs = performance.now() - t0;
    // fps
    const now = performance.now();
    this._fpsAcc += 1;
    if (now - this._fpsT >= 500) {
      this.fps = (this._fpsAcc * 1000) / (now - this._fpsT);
      this._fpsAcc = 0;
      this._fpsT = now;
    }
  }
  getFps() { return this.fps; }
  getRenderMs() { return this.lastRenderMs; }
  dispose() { this.canvas = null; this.ctx = null; }
}

function drawGrid(ctx, enabled, gridCells, cellX, cellY, offX, offY, gw, gh, viewW, viewH) {
  const step = Math.max(1, Math.floor(gridCells || 8));
  if (!enabled || Math.min(cellX, cellY) * step < 4) return;
  const spacingX = step * cellX;
  const spacingY = step * cellY;
  const startX = Math.max(0, Math.ceil(-offX / spacingX));
  const startY = Math.max(0, Math.ceil(-offY / spacingY));
  const endX = Math.min(Math.floor(gw / step), Math.floor((viewW - offX) / spacingX));
  const endY = Math.min(Math.floor(gh / step), Math.floor((viewH - offY) / spacingY));
  if (startX > endX && startY > endY) return;

  ctx.strokeStyle = "rgba(255,255,255,0.12)";
  ctx.lineWidth = 1;
  ctx.beginPath();
  if (startX <= endX) {
    for (let x = startX; x <= endX; x++) {
      const px = Math.round(offX + x * spacingX) + 0.5;
      ctx.moveTo(px, Math.max(0, offY));
      ctx.lineTo(px, Math.min(viewH, offY + gh * cellY));
    }
  }
  if (startY <= endY) {
    for (let y = startY; y <= endY; y++) {
      const py = Math.round(offY + y * spacingY) + 0.5;
      ctx.moveTo(Math.max(0, offX), py);
      ctx.lineTo(Math.min(viewW, offX + gw * cellX), py);
    }
  }
  ctx.stroke();
}

function heatColor(k) {
  // blue -> cyan -> green -> yellow -> red
  const r = Math.round(255 * Math.min(1, k * 2));
  const g = Math.round(255 * (k < 0.5 ? k * 2 : 2 - k * 2));
  const b = Math.round(255 * Math.max(0, 1 - k * 2));
  return `rgb(${r},${g},${b})`;
}
