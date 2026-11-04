// Renderer factory: prefers WebGPU, falls back to WebGL2, then Canvas2D.

import { Canvas2DRenderer } from "../adapters/renderer.js";

export async function createRenderer(canvas) {
  // Try WebGPU
  if (typeof navigator !== "undefined" && "gpu" in navigator) {
    try {
      const { WebGPURenderer } = await import("./webgpu-renderer.js");
      const r = new WebGPURenderer();
      await r.init(canvas);
      r.backend = "webgpu";
      return r;
    } catch (e) {
      console.warn("WebGPU init failed, falling back:", e?.message);
    }
  }
  // Try WebGL2
  try {
    const gl = canvas.getContext("webgl2", { failIfMajorPerformanceCaveat: false });
    if (gl) {
      const { WebGL2Renderer } = await import("./webgl2-renderer.js");
      const r = new WebGL2Renderer();
      r.attach(canvas);
      r.backend = "webgl2";
      return r;
    }
  } catch (e) {
    console.warn("WebGL2 init failed:", e?.message);
  }
  // Fallback Canvas2D
  try {
    const r = new Canvas2DRenderer();
    r.attach(canvas);
    r.backend = "canvas2d";
    return r;
  } catch (e) {
    console.error("All renderers failed:", e);
    throw e;
  }
}

export function getPreferredBackend() {
  if (typeof navigator !== "undefined" && "gpu" in navigator) return "webgpu";
  if (typeof document !== "undefined") {
    const c = document.createElement("canvas");
    if (c.getContext("webgl2")) return "webgl2";
  }
  return "canvas2d";
}
