import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { extname, join, normalize } from "node:path";
import { chromium } from "file:///C:/Users/diamo/AppData/Local/npm-cache/_npx/e41f203b7505f1fb/node_modules/playwright/index.mjs";
const WEB = join(process.cwd(), "web");
const MIME = { ".html":"text/html",".js":"text/javascript",".mjs":"text/javascript",".wasm":"application/wasm",".css":"text/css",".svg":"image/svg+xml",".json":"application/json" };
const server = createServer(async (req,res)=>{ try { let p=normalize(decodeURIComponent(new URL(req.url,"http://x").pathname)); if(p==="/"||p==="\\")p="/index.html"; const f=join(WEB,p); const d=await readFile(f); res.writeHead(200,{"content-type":MIME[extname(f)]||"application/octet-stream"}); res.end(d);} catch { res.writeHead(404); res.end(); } });
await new Promise(r=>server.listen(5173,"127.0.0.1",r));
const browser = await chromium.launch({ headless:true, executablePath:"C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe", args:["--no-sandbox","--use-gl=angle","--use-angle=swiftshader"] });
const out = {};
async function run(q,label){
  const page = await browser.newPage({ viewport:{width:1280,height:800} });
  const errs=[]; page.on("console",m=>{if(m.type()==="error")errs.push(m.text())}); page.on("pageerror",e=>errs.push(e.message));
  const r={label,errs};
  try{
    await page.goto(`http://127.0.0.1:5173/${q}`,{waitUntil:"load",timeout:30000});
    await page.waitForFunction(()=>document.getElementById("app")?.dataset.state==="ready",{timeout:30000});
    r.engine=await page.evaluate(()=>window.labsmatter.engine);
    r.backend=await page.evaluate(()=>window.labsmatter.renderer?.backend);
    r.toolPanel=await page.evaluate(()=>{const p=document.getElementById("tool-panel");return !!p&&!p.hidden;});
    if(r.engine==="tpt"){
      const box=await page.locator("#viewport").boundingBox();
      const sx=box.x+box.width*0.4, sy=box.y+box.height*0.4;
      const wx=Math.floor((sx-box.x)/4), wy=Math.floor((sy-box.y)/4);
      r.world={wx,wy};
      // Walls
      await page.locator('[data-cat="walls"]').click();
      r.wallButtons=await page.evaluate(()=>document.querySelectorAll("#material-list button").length);
      await page.evaluate(()=>{const b=[...document.querySelectorAll("#material-list button")].find(x=>x.textContent.includes("WALL"));b&&b.click();});
      await page.mouse.move(sx,sy); await page.mouse.down(); await page.mouse.move(sx+20,sy); await page.mouse.up();
      await page.waitForTimeout(100);
      r.wall=await page.evaluate(({x,y})=>{const s=window.labsmatter.sim;const info=JSON.parse(s.getCellInfo(x,y));return{wall:s.mod.get_wall(x,y),emap:s.getCellEmap(x,y),infoWall:info.wall};},{x:wx,y:wy});
      // Deco
      await page.locator('[data-cat="powders"]').click();
      await page.evaluate(()=>{const b=[...document.querySelectorAll("#material-list button")].find(x=>x.textContent.includes("SAND"));b&&b.click();});
      await page.mouse.move(sx,sy); await page.mouse.down(); await page.mouse.move(sx+20,sy); await page.mouse.up();
      await page.locator("#btn-deco").click();
      r.decoActive=await page.locator("#btn-deco").evaluate(el=>el.classList.contains("is-active"));
      await page.mouse.move(sx,sy); await page.mouse.down(); await page.mouse.move(sx+20,sy); await page.mouse.up();
      r.decoCount=await page.evaluate(()=>{const s=window.labsmatter.sim;const {w,h}=s.getWorldSize();let n=0;for(let y=0;y<h;y++)for(let x=0;x<w;x++)if(s.getDeco(x,y))n++;return n;});
      // Amb+ UI
      await page.locator("#btn-deco").click();
      await page.locator('[data-simtool="9"]').click();
      const a0=await page.evaluate(({x,y})=>window.labsmatter.sim.getCellAmbient(x,y),{x:wx,y:wy});
      await page.mouse.move(sx,sy); await page.mouse.down(); await page.mouse.move(sx+10,sy); await page.mouse.up();
      const a1=await page.evaluate(({x,y})=>window.labsmatter.sim.getCellAmbient(x,y),{x:wx,y:wy});
      r.ambient=[a0,a1,a1>a0];
      r.info=await page.evaluate(({x,y})=>{const s=window.labsmatter.sim;return {info:s.getCellInfo(x,y),settings:s.getWorldSettings()};},{x:wx,y:wy});
      r.roundtrip=await page.evaluate(()=>{const s=window.labsmatter.sim;const b=s.saveBlob();const c=s.getDiagnostics().particles;const ok=s.loadBlob(b);return{ok,before:c,after:s.getDiagnostics().particles,report:s.lastLoadReport||""};});
      r.hud=await page.locator("#hud-fps").textContent();
    }
    r.ok=true;
  }catch(e){r.fatal=e?.message||String(e);}
  out[label]=r; await page.close();
}
await run("?engine=tpt","tpt");
await run("","prod");
console.log(JSON.stringify(out,null,2));
await browser.close(); server.close();
