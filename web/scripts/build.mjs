// LabsMatter static build: assembles a deployable directory (dist/) with
// exactly the runtime files. No dependencies, no bundler — the front end is
// plain ES modules served as-is.
//
// Usage: node scripts/build.mjs   (from the web/ directory)
import { cp, mkdir, rm, writeFile, stat } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const webRoot = resolve(here, "..");
const dist = join(webRoot, "dist");

const ENTRIES = [
  "index.html",
  "styles",
  "src",
  "pkg",
  // Experimental TPT-parity runtime. Optional: the production deploy does not
  // require it, but when present it ships so `?engine=tpt` works from dist/.
  "pkg-tpt",
  "og-image.svg",
];

async function main() {
  await rm(dist, { recursive: true, force: true });
  await mkdir(dist, { recursive: true });

  const copied = [];
  for (const entry of ENTRIES) {
    const from = join(webRoot, entry);
    const to = join(dist, entry);
    try {
      const info = await stat(from);
      await cp(from, to, { recursive: info.isDirectory() });
      copied.push(entry);
    } catch (err) {
      if (entry === "pkg") {
        throw new Error(
          "web/pkg is missing. Build the WASM module first:\n" +
          "  cargo build -p realmatter-wasm --target wasm32-unknown-unknown --release\n" +
          "  wasm-bindgen target/wasm32-unknown-unknown/release/realmatter_wasm.wasm \\\n" +
          "    --out-dir web/pkg --target web --no-typescript"
        );
      }
      console.warn(`skip missing optional entry: ${entry}`);
    }
  }

  // Disable Jekyll processing on GitHub Pages (harmless elsewhere).
  await writeFile(join(dist, ".nojekyll"), "");

  // Fail loudly if the WASM payload is absent — a deploy without it is useless.
  const wasm = join(dist, "pkg", "realmatter_wasm_bg.wasm");
  const wasmInfo = await stat(wasm);
  if (wasmInfo.size < 10_000) {
    throw new Error(`WASM artifact looks wrong (${wasmInfo.size} bytes): ${wasm}`);
  }
  const glue = await stat(join(dist, "pkg", "realmatter_wasm.js"));

  console.log(`LabsMatter build complete -> ${dist}`);
  for (const e of copied) console.log(`  + ${e}`);
  console.log(`  + .nojekyll`);
  console.log(`  wasm: ${(wasmInfo.size / 1024).toFixed(1)} KB, glue: ${glue.size} B`);
}

main().catch((err) => {
  console.error("build failed:", err.message);
  process.exit(1);
});
