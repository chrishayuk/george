// Builds everything the extension ships:  node build.mjs
//   server/george-lsp.wasm  the language server, compiled for WASI
//   server/george.wasm      the george program, compiled for WASI
//   dist/extension.js       the extension and its one dependency, bundled
import { execSync } from "node:child_process";
import { copyFileSync } from "node:fs";
import { build } from "esbuild";

const here = new URL(".", import.meta.url).pathname;
const root = new URL("../..", import.meta.url).pathname;
const release = `${root}target/wasm32-wasip1/release`;

execSync("cargo build --release --target wasm32-wasip1 -p george -p george-lsp", { cwd: root, stdio: "inherit" });
copyFileSync(`${release}/george-lsp.wasm`, `${here}server/george-lsp.wasm`);
copyFileSync(`${release}/george.wasm`, `${here}server/george.wasm`);

await build({
  entryPoints: [`${here}src/extension.js`],
  outfile: `${here}dist/extension.js`,
  bundle: true,
  platform: "node",
  format: "cjs",
  target: "node18",
  external: ["vscode"],
  minify: true,
  logLevel: "warning",
});
console.log("server/george-lsp.wasm, server/george.wasm and dist/extension.js written");
