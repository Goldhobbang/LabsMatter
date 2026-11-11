// Renderer factory: prefers WebGPU, falls back to WebGL2, then Canvas2D.

import { Canvas2DRenderer } from "../adapters/renderer.js?viewport=2";

export async function createRenderer(canvas) {
  // Try WebGPU
  if (typeof navigator !== "undefined" && "gpu" in navigator) {
    try {
      const { WebGPURenderer } = await import("./webgpu-renderer.js?viewport=2");
      const r = new WebGPURenderer();
      await r.init(canvas);
      r.backend = "webgpu";
      return r;
    } catch (e) {
      console.warn("WebGPU init failed, falling back:", e?.message);
    }
  }
  // Try WebGL2. Do not call getContext here first: a second getContext with
  // different attributes returns null in Firefox and the real renderer never
  // attaches. WebGL2Renderer.attach is the only caller.
  try {
    const { WebGL2Renderer } = await import("./webgl2-renderer.js?viewport=2");
    const r = new WebGL2Renderer();
    r.attach(canvas);
    if (!r.gl) throw new Error("no webgl2");
    r.backend = "webgl2";
    return r;
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
