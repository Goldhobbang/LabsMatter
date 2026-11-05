// Production WebGPU renderer: blits the live WASM RGBA8 simulation buffer
// to the canvas through a GPU texture. Zoom/pan are applied as a UV
// transform in the fragment shader; no CPU-side pixel work per frame.

const WGSL = /* wgsl */ `
struct Params {
  // offset of the world origin in device pixels, world size in device px
  offPx: vec2<f32>,
  worldPx: vec2<f32>,
  // canvas size in device pixels, world size in cells
  canvasPx: vec2<f32>,
  worldCells: vec2<f32>,
  // background color
  bg: vec4<f32>,
  grid: f32,
};

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var simTex: texture_2d<f32>;
@group(0) @binding(2) var simSamp: sampler;

struct VSOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs(@builtin(vertex_index) vi: u32) -> VSOut {
  // Fullscreen triangle.
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(3.0, -1.0),
    vec2<f32>(-1.0, 3.0),
  );
  var out: VSOut;
  out.pos = vec4<f32>(pos[vi], 0.0, 1.0);
  return out;
}

@fragment
fn fs(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
  // Device-pixel coordinate -> world cell coordinate.
  let cell = (frag.xy - p.offPx) / (p.worldPx / p.worldCells);
  if (cell.x < 0.0 || cell.y < 0.0 || cell.x >= p.worldCells.x || cell.y >= p.worldCells.y) {
    return p.bg;
  }
  let uv = (floor(cell) + 0.5) / p.worldCells;
  // texture_2d y=0 is the first row; our buffer is row-major y-outer.
  let c = textureSample(simTex, simSamp, vec2<f32>(uv.x, uv.y));
  var color: vec4<f32>;
  if (c.a <= 0.003) {
    color = p.bg;
  } else {
    color = vec4<f32>(c.rgb, 1.0);
  }
  if (p.grid > 0.5) {
    let f = fract(cell);
    if (f.x < 0.08 || f.y < 0.08) {
      color = mix(color, vec4<f32>(0.82, 0.86, 0.9, 1.0), 0.35);
    }
  }
  return color;
}
`;

export class WebGPURenderer {
  constructor() {
    this.canvas = null;
    this.device = null;
    this.context = null;
    this.format = null;
    this.pipeline = null;
    this.uniformBuf = null;
    this.sampler = null;
    this.simTex = null;
    this.texW = 0;
    this.texH = 0;
    this.bindGroup = null;
    this.dpr = 1;
    this.overlay = { grid: true, heat: false };
    this.lastRenderMs = 0;
    this.fps = 0;
    this._fpsAcc = 0; this._fpsT = 0;
    this._lost = false;
    this.backend = "webgpu";
  }

  async init(canvas) {
    if (typeof navigator === "undefined" || !navigator.gpu) throw new Error("WebGPU not available");
    const adapter = await navigator.gpu.requestAdapter({ powerPreference: "high-performance" });
    if (!adapter) throw new Error("No WebGPU adapter");
    this.device = await adapter.requestDevice();
    this.device.lost.then((info) => { this._lost = true; console.warn("WebGPU device lost:", info?.reason); });
    this.canvas = canvas;
    this.context = canvas.getContext("webgpu");
    if (!this.context) throw new Error("WebGPU canvas context unavailable");
    this.format = navigator.gpu.getPreferredCanvasFormat();
    this.context.configure({ device: this.device, format: this.format, alphaMode: "opaque" });
    const mod = this.device.createShaderModule({ code: WGSL });
    this.pipeline = this.device.createRenderPipeline({
      layout: "auto",
      vertex: { module: mod, entryPoint: "vs" },
      fragment: { module: mod, entryPoint: "fs", targets: [{ format: this.format }] },
      primitive: { topology: "triangle-list" },
    });
    this.uniformBuf = this.device.createBuffer({
      size: 64,
      usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });
    this.sampler = this.device.createSampler({ magFilter: "nearest", minFilter: "nearest" });
    this.resize(canvas.clientWidth, canvas.clientHeight);
    return true;
  }

  _ensureTexture(gw, gh) {
    if (this.simTex && this.texW === gw && this.texH === gh) return;
    if (this.simTex) { try { this.simTex.destroy(); } catch {} }
    this.simTex = this.device.createTexture({
      size: { width: Math.max(1, gw), height: Math.max(1, gh) },
      format: "rgba8unorm",
      usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST,
    });
    this.texW = gw; this.texH = gh;
    this.bindGroup = this.device.createBindGroup({
      layout: this.pipeline.getBindGroupLayout(0),
      entries: [
        { binding: 0, resource: { buffer: this.uniformBuf } },
        { binding: 1, resource: this.simTex.createView() },
        { binding: 2, resource: this.sampler },
      ],
    });
  }

  resize(w, h) {
    if (!this.canvas || this._lost) return;
    if (!(w > 1 && h > 1)) return;
    this.dpr = Math.min(window.devicePixelRatio || 1, 2);
    this.canvas.width = Math.max(1, Math.floor(w * this.dpr));
    this.canvas.height = Math.max(1, Math.floor(h * this.dpr));
    try { if (this.context && this.device) this.context.configure({ device: this.device, format: this.format, alphaMode: "opaque" }); } catch {}
  }

  setOverlay(name, on) { this.overlay[name] = on; }

  renderRGBA(buf, gw, gh, camera, opts) {
    const t0 = performance.now();
    if (this._lost || !this.device || !this.context || !this.pipeline || !buf || gw <= 0 || gh <= 0) {
      this.lastRenderMs = performance.now() - t0;
      return;
    }
    try {
      this._ensureTexture(gw, gh);
      const bytesPerRow = gw * 4;
      const n = bytesPerRow * gh;
      if (!this._upload || this._upload.length !== n) this._upload = new Uint8Array(n);
      this._upload.set(buf.subarray(0, n));
      this.device.queue.writeTexture(
        { texture: this.simTex },
        this._upload,
        { bytesPerRow, rowsPerImage: gh },
        { width: gw, height: gh }
      );
      const zoom = camera.zoom || 1;
      const cellPx = 4 * zoom;
      const offX = (camera.x || 0) * this.dpr;
      const offY = (camera.y || 0) * this.dpr;
      const worldPxW = gw * cellPx * this.dpr;
      const worldPxH = gh * cellPx * this.dpr;
      const params = new Float32Array([
        offX, offY, worldPxW, worldPxH,
        this.canvas.width, this.canvas.height, gw, gh,
        0.02, 0.024, 0.04, 1.0,
      ]);
      // Pad to 64 bytes (16 floats).
      const padded = new Float32Array(16);
      padded.set(params.subarray(0, 12));
      padded[12] = this.overlay.grid && cellPx >= 6 ? 1 : 0;
      this.device.queue.writeBuffer(this.uniformBuf, 0, padded);
      const encoder = this.device.createCommandEncoder();
      const pass = encoder.beginRenderPass({
        colorAttachments: [{
          view: this.context.getCurrentTexture().createView(),
          loadOp: "clear",
          clearValue: { r: 0.02, g: 0.024, b: 0.04, a: 1.0 },
          storeOp: "store",
        }],
      });
      pass.setPipeline(this.pipeline);
      pass.setBindGroup(0, this.bindGroup);
      pass.draw(3, 1, 0, 0);
      pass.end();
      this.device.queue.submit([encoder.finish()]);
    } catch (e) {
      console.warn("WebGPU render failed:", e?.message);
    }
    this.lastRenderMs = performance.now() - t0;
    const now = performance.now();
    this._fpsAcc++;
    if (now - this._fpsT >= 500) { this.fps = (this._fpsAcc * 1000) / (now - this._fpsT); this._fpsAcc = 0; this._fpsT = now; }
  }

  render(snapshot, camera, opts) {
    // Snapshot path is not used by the app (it streams the RGBA buffer).
    // Keep the frame alive without doing CPU pixel work.
    const t0 = performance.now();
    this.lastRenderMs = performance.now() - t0;
  }

  getFps() { return this.fps; }
  getRenderMs() { return this.lastRenderMs; }
  dispose() {
    try { if (this.simTex) this.simTex.destroy(); } catch {}
    try { if (this.uniformBuf) this.uniformBuf.destroy(); } catch {}
    try { if (this.device) this.device.destroy(); } catch {}
    this.device = null; this.canvas = null; this.simTex = null;
  }
}
