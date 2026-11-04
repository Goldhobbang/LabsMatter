// WebGL2 fallback renderer with instanced drawing where possible.

import { findMaterial } from "../materials.js";

const VS = `#version 300 es
layout(location=0) in vec2 aQuad;
layout(location=1) in vec2 aPos;
layout(location=2) in vec4 aColor;
uniform vec2 uRes;
uniform vec2 uCam;
uniform float uZoom;
uniform float uCell;
out vec4 vColor;
void main() {
  vec2 sz = vec2(uCell*uZoom);
  vec2 p = (aPos + uCam) * sz + aQuad * sz;
  vec2 ndc = (p / uRes) * 2.0 - 1.0;
  gl_Position = vec4(ndc.x, -ndc.y, 0.0, 1.0);
  vColor = aColor;
}
`;

const FS = `#version 300 es
precision mediump float;
in vec4 vColor;
out vec4 frag;
void main() { frag = vColor; }
`;

export class WebGL2Renderer {
  constructor() {
    this.canvas = null; this.gl = null;
    this.program = null;
    this.quadBuf = null;
    this.instBuf = null;
    this.instData = new Float32Array(1024*256*8);
    this.maxInst = 1024*256;
    this.overlay = { grid: true, heat: false };
    this.lastRenderMs = 0; this.fps = 0;
    this._fpsAcc = 0; this._fpsT = 0;
  }

  attach(canvas) {
    this.canvas = canvas;
    this.gl = canvas.getContext("webgl2", { alpha: false, antialias: false, preserveDrawingBuffer: false });
    if (!this.gl) throw new Error("no webgl2");
    // context loss handling
    canvas.addEventListener('webglcontextlost', (e) => { e.preventDefault(); this._lost = true; }, { passive: false });
    canvas.addEventListener('webglcontextrestored', () => { this._lost = false; try { this._initGL(); this.resize(this.canvas.clientWidth, this.canvas.clientHeight); } catch(_){} }, { passive: true });
    this._initGL();
    this.resize(canvas.clientWidth, canvas.clientHeight);
  }

  _initGL() {
    const gl = this.gl;
    const vs = gl.createShader(gl.VERTEX_SHADER); gl.shaderSource(vs, VS); gl.compileShader(vs);
    const fs = gl.createShader(gl.FRAGMENT_SHADER); gl.shaderSource(fs, FS); gl.compileShader(fs);
    this.program = gl.createProgram(); gl.attachShader(this.program, vs); gl.attachShader(this.program, fs); gl.linkProgram(this.program);
    this.quadBuf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, this.quadBuf);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([0,0,1,0,0,1,1,1]), gl.STATIC_DRAW);
    this.instBuf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, this.instBuf);
    gl.bufferData(gl.ARRAY_BUFFER, this.instData.byteLength, gl.DYNAMIC_DRAW);
    gl.enable(gl.BLEND); gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
  }

  resize(w, h) {
    if (!this.canvas || !this.gl || this._lost) return;
    const dpr = Math.min(window.devicePixelRatio||1,2);
    this.canvas.width = Math.max(1, Math.floor(w*dpr));
    this.canvas.height = Math.max(1, Math.floor(h*dpr));
    this.gl.viewport(0,0,this.canvas.width,this.canvas.height);
  }

  setOverlay(n,o){this.overlay[n]=o;}

  renderRGBA(buf, gw, gh, camera, opts) {
    const t0 = performance.now();
    const gl = this.gl;
    if (this._lost || !gl || !buf || gw <= 0 || gh <= 0) { this.lastRenderMs = performance.now()-t0; return; }
    const w = this.canvas.width, h = this.canvas.height;
    if (w < 1 || h < 1) { this.lastRenderMs = performance.now()-t0; return; }
    gl.clearColor(0.02,0.03,0.06,1); gl.clear(gl.COLOR_BUFFER_BIT);
    if (!this._rgbaTex) this._rgbaTex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, this._rgbaTex);
    gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, 0);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    const rows = Math.min(gh, Math.floor(buf.length / (gw * 4)));
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gw, rows, 0, gl.RGBA, gl.UNSIGNED_BYTE, buf.subarray(0, gw * rows * 4));
    // Fullscreen textured quad via the instanced shader path is overkill;
    // use a minimal blit program created lazily.
    if (!this._blit) {
      const vs = gl.createShader(gl.VERTEX_SHADER);
      gl.shaderSource(vs, `#version 300 es\nlayout(location=0) in vec2 aP; out vec2 vUV; void main(){ vUV=vec2(aP.x*0.5+0.5, 0.5-aP.y*0.5); gl_Position=vec4(aP,0.,1.);}`);
      gl.compileShader(vs);
      const fs = gl.createShader(gl.FRAGMENT_SHADER);
      gl.shaderSource(fs, `#version 300 es\nprecision mediump float; uniform sampler2D uT; uniform vec2 uOff; uniform vec2 uScale; in vec2 vUV; out vec4 o; void main(){ vec2 uv=(vUV-uOff)/uScale; if(uv.x<0.||uv.y<0.||uv.x>1.||uv.y>1.) discard; o=texture(uT, vec2(uv.x, 1.0-uv.y)); }`);
      gl.compileShader(fs);
      const pr = gl.createProgram();
      gl.attachShader(pr, vs); gl.attachShader(pr, fs); gl.linkProgram(pr);
      const vb = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, vb);
      gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1,-1, 1,-1, -1,1, 1,1]), gl.STATIC_DRAW);
      this._blit = { pr, vb };
    }
    const zoom = camera.zoom || 1, cellPx = 4 * zoom;
    const dprVis = Math.min(window.devicePixelRatio || 1, 2);
    const cssW = w / dprVis, cssH = h / dprVis;
    gl.useProgram(this._blit.pr);
    gl.bindBuffer(gl.ARRAY_BUFFER, this._blit.vb);
    gl.enableVertexAttribArray(0); gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
    gl.uniform2f(gl.getUniformLocation(this._blit.pr, "uOff"), -camera.x / cssW, -camera.y / cssH);
    gl.uniform2f(gl.getUniformLocation(this._blit.pr, "uScale"), (gw * cellPx) / cssW, (gh * cellPx) / cssH);
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this._rgbaTex);
    gl.uniform1i(gl.getUniformLocation(this._blit.pr, "uT"), 0);
    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
    this.lastRenderMs = performance.now()-t0;
    const now=performance.now(); this._fpsAcc++; if (now-this._fpsT>=500){this.fps=this._fpsAcc*1000/(now-this._fpsT);this._fpsAcc=0;this._fpsT=now;}
  }

  render(snapshot, camera, opts) {
    const t0 = performance.now();
    const gl = this.gl;
    if (this._lost || !gl || !snapshot) { this.lastRenderMs = performance.now()-t0; return; }
    const w = this.canvas.width, h = this.canvas.height; if (w < 1 || h < 1) { this.lastRenderMs = performance.now()-t0; return; }
    gl.clearColor(0.02,0.03,0.06,1); gl.clear(gl.COLOR_BUFFER_BIT);

    const gw=snapshot.width,gh=snapshot.height,grid=snapshot.grid,temp=snapshot.temp;
    const zoom=camera.zoom||1, cellPx=4*zoom, offX=camera.x, offY=camera.y;
    const dprVis = Math.min(window.devicePixelRatio||1,2);
    const cssW = w / dprVis;
    const cssH = h / dprVis;
    const startX=Math.max(0,Math.floor(-offX/cellPx));
    const startY=Math.max(0,Math.floor(-offY/cellPx));
    const endX=Math.min(gw, Math.ceil((cssW - offX)/cellPx));
    const endY=Math.min(gh, Math.ceil((cssH - offY)/cellPx));

    gl.useProgram(this.program);
    gl.uniform2f(gl.getUniformLocation(this.program,"uRes"), w, h);
    gl.uniform2f(gl.getUniformLocation(this.program,"uCam"), offX, offY);
    gl.uniform1f(gl.getUniformLocation(this.program,"uZoom"), zoom);
    gl.uniform1f(gl.getUniformLocation(this.program,"uCell"), 4);

    let count=0;
    const data=this.instData;
    for (let y=startY;y<endY&&count<this.maxInst;y++) {
      for (let x=startX;x<endX&&count<this.maxInst;x++) {
        const m=grid[y*gw+x]; if (m===0) continue;
        let r,g,b,a=1;
        if (this.overlay.heat && temp) {
          const t=temp[y*gw+x];
          const k=Math.min(1,Math.max(0,(t-200)/1300));
          r=Math.min(1,k*2); g=k<0.5?k*2:2-k*2; b=Math.max(0,1-k*2);
        } else {
          const mat=findMaterial(m);
          const hex=mat.color.replace("#","");
          r=parseInt(hex.substr(0,2),16)/255; g=parseInt(hex.substr(2,2),16)/255; b=parseInt(hex.substr(4,2),16)/255;
        }
        const base=count*8;
        data[base+0]=x; data[base+1]=y; data[base+2]=r; data[base+3]=g; data[base+4]=b; data[base+5]=a;
        count++;
      }
    }

    if (count>0) {
      gl.bindBuffer(gl.ARRAY_BUFFER, this.instBuf);
      gl.bufferSubData(gl.ARRAY_BUFFER, 0, data.subarray(0, count*8));
      // setup attributes
      gl.bindBuffer(gl.ARRAY_BUFFER, this.quadBuf);
      gl.enableVertexAttribArray(0); gl.vertexAttribPointer(0,2,gl.FLOAT,false,0,0);
      gl.vertexAttribDivisor(0,0);
      gl.bindBuffer(gl.ARRAY_BUFFER, this.instBuf);
      gl.enableVertexAttribArray(1); gl.vertexAttribPointer(1,2,gl.FLOAT,false,32,0); gl.vertexAttribDivisor(1,1);
      gl.enableVertexAttribArray(2); gl.vertexAttribPointer(2,4,gl.FLOAT,false,32,8); gl.vertexAttribDivisor(2,1);
      gl.drawArraysInstanced(gl.TRIANGLE_STRIP, 0, 4, count);
    }

    this.lastRenderMs = performance.now()-t0;
    const now=performance.now(); this._fpsAcc++; if (now-this._fpsT>=500){this.fps=this._fpsAcc*1000/(now-this._fpsT);this._fpsAcc=0;this._fpsT=now;}
  }

  getFps(){return this.fps;} getRenderMs(){return this.lastRenderMs;}
  dispose(){ if(this.gl) this.gl.getExtension('WEBGL_lose_context')?.loseContext(); this.gl=null; }
}
