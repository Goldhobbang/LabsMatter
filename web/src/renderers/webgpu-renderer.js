// Production WebGPU renderer: blits the live WASM RGBA8 simulation buffer
// to the canvas through a GPU texture. Zoom/pan are applied as a UV
// transform in the fragment shader; no CPU-side pixel work per frame.
// NOTE (env): WebGPU glow/bloom is implemented but hardware-unverified in this
// environment (no adapter). Runtime behavior is unchanged.

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
  glowStrength: f32,
  grid: f32,
  glowTexel: vec2<f32>,
};

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var simTex: texture_2d<f32>;
@group(0) @binding(2) var simSamp: sampler;
@group(0) @binding(3) var glowTex: texture_2d<f32>;

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
  let inside = cell.x >= 0.0 && cell.y >= 0.0 && cell.x < p.worldCells.x && cell.y < p.worldCells.y;
  // textureSample must run in uniform control flow. Branching on frag.xy
  // before the sample makes Dawn reject the module on Windows, which leaves
  // the canvas cleared (black) while the CPU particle count still rises.
  let span = max(p.worldCells, vec2<f32>(1.0));
  let clamped = clamp(floor(cell), vec2<f32>(0.0), span - vec2<f32>(1.0));
  let uv = (clamped + vec2<f32>(0.5)) / span;
  let c = textureSample(simTex, simSamp, uv);
  let show = select(false, c.a > 0.003, inside);
  var base = select(p.bg, vec4<f32>(c.rgb, 1.0), show);
  // Additive glow with a larger 5x5 box-blur kernel (bloom). Only sampled when
  // a glow buffer is active (strength > 0) so a stale/empty texture cannot
  // tint the frame and no work is spent when bloom is disabled.
  var gl = vec3<f32>(0.0);
  if (p.glowStrength > 0.0) {
    for (var ky: i32 = -2; ky <= 2; ky = ky + 1) {
      for (var kx: i32 = -2; kx <= 2; kx = kx + 1) {
        let off = vec2<f32>(f32(kx) * p.glowTexel.x, f32(ky) * p.glowTexel.y);
        gl += textureSample(glowTex, simSamp, uv + off).rgb;
      }
    }
    gl /= 25.0;
    base = vec4<f32>(base.rgb + gl * p.glowStrength, base.a);
  }
  let gridSpacing = max(p.grid, 1.0);
  let f = fract(cell / gridSpacing);
  let edge = (f.x < 0.08) || (f.y < 0.08);
  let onGrid = select(false, edge, inside) && (p.grid > 0.5);
  let gridded = mix(base, vec4<f32>(0.82, 0.86, 0.9, 1.0), 0.35);
  return select(base, gridded, onGrid);
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
    this.glowTex = null;
    this._blackGlow = null;
    this._glowActive = false;
    this.glowW = 0;
    this.glowH = 0;
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
    this.device.lost.then((info) => {
      this._lost = true;
      this.fallbackRequested = true;
      this.lossReason = "WebGPU device lost: " + (info?.reason || "unknown");
      console.error(this.lossReason + ". The WASM simulation was not replaced.");
    });
    this.format = navigator.gpu.getPreferredCanvasFormat();
    const mod = this.device.createShaderModule({ code: WGSL });
    if (typeof mod.getCompilationInfo === "function") {
      const compiled = await mod.getCompilationInfo();
      const errors = compiled.messages.filter((m) => m.type === "error");
      if (errors.length) {
        try { this.device.destroy(); } catch {}
        this.device = null;
        throw new Error(errors.map((m) => m.message).join("; "));
      }
    }
    // Claim the canvas only after the shader is valid. A failed WebGPU
    // context would block the WebGL2 fallback on the same canvas.
    this.pipeline = this.device.createRenderPipeline({
      layout: "auto",
      vertex: { module: mod, entryPoint: "vs" },
      fragment: { module: mod, entryPoint: "fs", targets: [{ format: this.format }] },
      primitive: { topology: "triangle-list" },
    });
    this.canvas = canvas;
    this.context = canvas.getContext("webgpu");
    if (!this.context) {
      try { this.device.destroy(); } catch {}
      this.device = null;
      throw new Error("WebGPU canvas context unavailable");
    }
    this.context.configure({ device: this.device, format: this.format, alphaMode: "opaque" });
    this.uniformBuf = this.device.createBuffer({
      size: 64,
      usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });
    this.sampler = this.device.createSampler({ magFilter: "nearest", minFilter: "nearest" });
    this.resize(canvas.clientWidth, canvas.clientHeight);
    return true;
  }

  _ensureBlackGlow() {
    if (this._blackGlow) return;
    this._blackGlow = this.device.createTexture({
      size: { width: 1, height: 1 },
      format: "rgba8unorm",
      usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST,
    });
    this.device.queue.writeTexture(
      { texture: this._blackGlow },
      new Uint8Array([0, 0, 0, 255]),
      { bytesPerRow: 4 },
      { width: 1, height: 1 }
    );
  }

  _rebuildBindGroup() {
    if (!this.simTex) return;
    this._ensureBlackGlow();
    const glowView = (this.glowTex || this._blackGlow).createView();
    this.bindGroup = this.device.createBindGroup({
      layout: this.pipeline.getBindGroupLayout(0),
      entries: [
        { binding: 0, resource: { buffer: this.uniformBuf } },
        { binding: 1, resource: this.simTex.createView() },
        { binding: 2, resource: this.sampler },
        { binding: 3, resource: glowView },
      ],
    });
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
    this._rebuildBindGroup();
  }

  _ensureGlow(gw, gh) {
    if (this.glowTex && this.glowW === gw && this.glowH === gh) return;
    if (this.glowTex) { try { this.glowTex.destroy(); } catch {} }
    this.glowTex = this.device.createTexture({
      size: { width: Math.max(1, gw), height: Math.max(1, gh) },
      format: "rgba8unorm",
      usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST,
    });
    this.glowW = gw; this.glowH = gh;
    this._rebuildBindGroup();
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
      const tight = gw * 4;
      const bytesPerRow = Math.ceil(tight / 256) * 256;
      const n = tight * gh;
      if (!this._upload || this._upload.length !== n) this._upload = new Uint8Array(n);
      this._upload.set(buf.subarray(0, n));
      let src = this._upload;
      if (bytesPerRow !== tight) {
        const padded = bytesPerRow * gh;
        if (!this._padded || this._padded.length !== padded) this._padded = new Uint8Array(padded);
        for (let y = 0; y < gh; y++) {
          this._padded.set(this._upload.subarray(y * tight, y * tight + tight), y * bytesPerRow);
        }
        src = this._padded;
      }
      this.device.queue.writeTexture(
        { texture: this.simTex },
        src,
        { bytesPerRow, rowsPerImage: gh },
        { width: gw, height: gh }
      );
      // Optional additive glow buffer.
      const glow = opts && opts.glow;
      const wantGlow = !!(glow && glow.length >= n);
      let glowStrength = 0.0;
      if (wantGlow) {
        this._ensureGlow(gw, gh);
        if (!this._glowUpload || this._glowUpload.length !== n) this._glowUpload = new Uint8Array(n);
        this._glowUpload.set(glow.subarray(0, n));
        let glowSrc = this._glowUpload;
        if (bytesPerRow !== tight) {
          const padded = bytesPerRow * gh;
          if (!this._glowPadded || this._glowPadded.length !== padded) this._glowPadded = new Uint8Array(padded);
          for (let y = 0; y < gh; y++) {
            this._glowPadded.set(this._glowUpload.subarray(y * tight, y * tight + tight), y * bytesPerRow);
          }
          glowSrc = this._glowPadded;
        }
        this.device.queue.writeTexture(
          { texture: this.glowTex },
          glowSrc,
          { bytesPerRow, rowsPerImage: gh },
          { width: gw, height: gh }
        );
        glowStrength = opts && opts.glowStrength != null ? opts.glowStrength : 1.0;
      }
      // Keep the bind group's glow texture in sync with glow availability so a
      // disabled/absent glow buffer can never reuse stale emissive data.
      if (wantGlow !== this._glowActive) {
        if (!wantGlow) {
          if (this.glowTex) { try { this.glowTex.destroy(); } catch {} }
          this.glowTex = null;
          this.glowW = 0;
          this.glowH = 0;
        }
        this._glowActive = wantGlow;
        this._rebuildBindGroup();
      }
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
      const gridSize = this.overlay.gridSize || 8;
      padded[12] = glowStrength;
      padded[13] = this.overlay.grid && cellPx * gridSize >= 4 ? gridSize : 0;
      padded[14] = 1 / Math.max(1, gw);
      padded[15] = 1 / Math.max(1, gh);
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
