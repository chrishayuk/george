// Builds the files the playground needs:  node web/build.mjs
//   web/george.wasm      the george program, compiled for WASI
//   web/george-lsp.wasm  the language server, compiled for WASI
//   web/examples.json    the example programs, with their data cards
import { execSync } from "node:child_process";
import { copyFileSync, existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";

const root = new URL("..", import.meta.url).pathname;
execSync("cargo build --release --target wasm32-wasip1 -p george -p george-lsp", { cwd: root, stdio: "inherit" });
copyFileSync(`${root}target/wasm32-wasip1/release/george.wasm`, `${root}web/george.wasm`);
copyFileSync(`${root}target/wasm32-wasip1/release/george-lsp.wasm`, `${root}web/george-lsp.wasm`);

// Shown first, in this order; the rest follow alphabetically.
const first = ["squares", "sum-of-squares", "subroutine", "polynomial", "factorials", "newton", "converge",
  "mean", "mean-one-pass", "nested", "matrix", "normalise", "any-zero", "pairs", "simpson", "simpson-corrected"];

const names = readdirSync(`${root}examples`).filter((f) => f.endsWith(".george")).map((f) => f.slice(0, -7));
names.sort((a, b) => {
  const [i, j] = [first.indexOf(a), first.indexOf(b)];
  if (i !== -1 || j !== -1) return (i === -1 ? 99 : i) - (j === -1 ? 99 : j);
  return a.localeCompare(b);
});
const read = (path) => (existsSync(path) ? readFileSync(path, "utf8") : "");
const examples = names.map((name) => {
  const program = readFileSync(`${root}examples/${name}.george`, "utf8");
  // The comment at the top says what the program is.
  const head = [];
  for (const line of program.split("\n")) {
    if (!line.startsWith("#") || (head.length && line.trim() === "#")) break;
    head.push(line.replace(/^#\s?/, "").trim());
  }
  const about = head.join(" ");
  return { name, about, program, data: read(`${root}examples/${name}.in`) };
});
writeFileSync(`${root}web/examples.json`, JSON.stringify(examples));
console.log(`web/george.wasm, web/george-lsp.wasm and web/examples.json (${examples.length} programs) written`);
