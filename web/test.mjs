// Runs every example through the WebAssembly build and the page's WASI
// host, exactly as the page does, and compares what it punches with the
// .out file (and the stop with the .stop file). Then asks the language
// server, through the code the page's editor uses, about the same examples.
//   node web/test.mjs
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { columns, invocation, punchCount, punched, report, trace } from "./george.js";
import { answer } from "./lsp-worker.js";
import { run } from "./wasi.js";

const root = new URL("..", import.meta.url).pathname;
const module = new WebAssembly.Module(readFileSync(root + "web/george.wasm"));
let failed = 0;
let checks = 0;

function george(what) {
  const { args, files } = invocation(what);
  const text = { 1: "", 2: "" };
  const decoders = { 1: new TextDecoder(), 2: new TextDecoder() };
  const code = run(module, args, new Uint8Array(0), (fd, bytes) => {
    text[fd] += decoders[fd].decode(bytes, { stream: true });
  }, files);
  return { code, out: text[1], err: text[2] };
}

function check(what, ok, detail = "") {
  checks++;
  if (ok) return;
  failed++;
  console.log(`FAIL ${what} ${detail}`);
}

// The value of a card from its twenty columns: sign, d.ddddd, sign, exponent.
function cardValue(card) {
  const c = columns(card);
  const mantissa = Number(c.mantissa.slice(0, 1) + "." + c.mantissa.slice(1));
  const exponent = Number(c.exponent) * (c.exponentSign === "-" ? -1 : 1);
  return (c.sign === "-" ? -1 : 1) * mantissa * 10 ** exponent;
}

const names = readdirSync(root + "examples").filter((f) => f.endsWith(".george")).map((f) => f.slice(0, -7)).sort();
for (const name of names) {
  const base = `${root}examples/${name}`;
  const program = readFileSync(`${base}.george`, "utf8");
  const data = existsSync(`${base}.in`) ? readFileSync(`${base}.in`, "utf8") : "";
  const expected = readFileSync(`${base}.out`, "utf8");
  const stop = existsSync(`${base}.stop`) ? readFileSync(`${base}.stop`, "utf8").trim() : null;

  const result = george({ program, data });
  const said = report(result.err, result.code);
  check(`${name}: punched`, result.out === expected, JSON.stringify(result).slice(0, 300));
  if (stop) check(`${name}: stop`, result.code === 1 && said.stop?.indication === stop, JSON.stringify(said));
  else check(`${name}: no stop`, result.code === 0 && said.stop === null && result.err === "", result.err);

  // The twenty columns of each card say the same number as the decimal.
  const cards = punched(george({ program, data, mode: "cards" }).out);
  const numbers = punched(result.out);
  check(`${name}: cards`, cards.length === numbers.length && cards.every((card, k) => card.length === 20
    && Math.abs(cardValue(card) - Number(numbers[k])) <= 1e-5 * Math.abs(Number(numbers[k]))), cards.slice(0, 3).join("|"));

  // The trace: one step per symbol obeyed, each naming a symbol of the
  // program, never more than twelve cells, and the punching symbols in it
  // account for every number punched.
  const traced = george({ program, data, mode: "trace" });
  const steps = trace(traced.err);
  const count = steps.reduce((n, step, k) => n + punchCount(step.symbol, k ? steps[k - 1].cells : []), 0);
  check(`${name}: trace`, steps.length > 0 && steps.every((s) => s.cells.length <= 12) && count === punched(traced.out).length,
    `${steps.length} steps, ${count} punches counted, ${punched(traced.out).length} punched`);
}

// What the page shows besides.
const examples = JSON.parse(readFileSync(root + "web/examples.json", "utf8"));
check("examples.json", examples.length === names.length && examples[0].name === "squares" && examples.every((e) => e.program && "data" in e),
  `${examples.length} of ${names.length}`);
check("examples.json data", examples.find((e) => e.name === "mean").data.startsWith("1\n2\n"));

// The README's trace.
let r = george({ program: readFileSync(root + "examples/subroutine.george", "utf8"), data: "3\n4\n", mode: "trace" });
let steps = trace(r.err);
check("trace of subroutine", steps.length === 15 && steps[2].symbol === "6" && steps[2].cells.join() === "3,4,6"
  && steps[3].symbol === "↓" && steps[4].at === 9 && steps[13].symbol === "(P)" && steps[14].cells.length === 0, JSON.stringify(steps.slice(0, 5)));

// A stop, with where it happened.
r = george({ program: readFileSync(root + "examples/pairs.george", "utf8"), data: "1\n2\n3\n4\n" });
let said = report(r.err, r.code);
check("stop", r.out === "3\n7\n" && said.stop.cause === "data card wanted" && said.stop.line === 4 && said.stop.column === 5 && said.stop.symbol === "R",
  JSON.stringify(said));

// A resumable stop, got past with a single shot; wait and the keys.
r = george({ program: "1, 0 ÷ (P) ; I wait (P)", keys: "7", singleShot: true });
said = report(r.err, r.code);
check("single shot", r.code === 0 && r.out === "0\n7\n" && said.resumed.length === 1 && said.resumed[0].indication === "1 4-24x"
  && said.resumed[0].resumed === "resumes giving result zero" && said.lights.join() === "7", JSON.stringify({ r, said }));
r = george({ program: "1, 0 ÷ (P)" });
said = report(r.err, r.code);
check("divisor zero", r.code === 1 && said.stop.indication === "1 4-24x" && said.stop.column === 6, JSON.stringify(said));

// A mistake in the text, placed.
r = george({ program: "a b +\n  (o) ;" });
said = report(r.err, r.code);
check("text mistake", r.code === 2 && said.problem.line === 2 && said.problem.column === 3 && /is not a name/.test(said.problem.message), JSON.stringify(said));

// A program that never ends is given up.
r = george({ program: "* 0 0 ↑", limit: 1000 });
said = report(r.err, r.code);
check("limit", r.code === 1 && said.limit === 1000, JSON.stringify(said));

// The other commands.
r = george({ program: "1, 20 rep (j) j j × (P) ; ]", mode: "deck" });
check("deck", r.code === 0 && r.out.startsWith("card  1: 1/2 1/0 2/2 0/2 8/1 9/6") && r.out.includes("(Y-row: last card)"), r.out);
r = george({ program: "1, 20 rep (j) j j mul (P); ]", mode: "format" });
check("format", r.code === 0 && r.out.trim() === "1, 20 rep (j) j j × (P); ]", r.out);
r = george({ mode: "table" });
check("table", r.code === 0 && r.out.includes("rep") && r.out.split("\n").length > 16, r.out.slice(0, 80));
r = george({ mode: "stops" });
check("stops", r.code === 0 && r.out.includes("data card wanted"), r.out.slice(0, 80));

// The language server, as the editor in the page asks it (lsp-worker.js).
const server = new WebAssembly.Module(readFileSync(root + "web/george-lsp.wasm"));
const at = (line, character) => ({ position: { line, character } });
const subroutine = readFileSync(root + "examples/subroutine.george", "utf8");
let a = answer(server, subroutine, [
  { method: "textDocument/semanticTokens/full" },
  { method: "textDocument/hover", params: at(2, 6) }, // ↓
  { method: "textDocument/hover", params: at(4, 8) }, // ×
  { method: "george/symbols" },
  { method: "textDocument/codeLens" },
  { method: "textDocument/definition", params: at(2, 4) },
  { method: "textDocument/noSuchThing" },
]);
check("server: a good program has no problems", a.diagnostics.length === 0, JSON.stringify(a.diagnostics));
const data = a.results[0]?.data ?? [];
check("server: semantic tokens", data.length % 5 === 0 && data.length >= 5 * 20 && data.slice(0, 5).join() === `0,0,${subroutine.split("\n")[0].length},0,0`, JSON.stringify(data.slice(0, 10)));
check("server: hover on a skip", /^\*\*`↓`\*\* skip to subroutine/.test(a.results[1]?.contents.value) && /code 6\/1/.test(a.results[1].contents.value),
  JSON.stringify(a.results[1]));
check("server: hover on an operator", /diadic operator · code 7\/0/.test(a.results[2]?.contents.value) && /`… a b → … a×b`/.test(a.results[2].contents.value),
  JSON.stringify(a.results[2]));
const symbols = a.results[3]?.symbols ?? [];
check("server: the symbol list", symbols.length === 19 && symbols[3].printed === "↓" && symbols[9].text === "dup" && symbols[9].range.start.line === 4,
  JSON.stringify(symbols.slice(0, 4)));
check("server: code lenses", a.results[4]?.[0]?.command.command === "george.run", JSON.stringify(a.results[4]));
check("server: definition", a.results[5]?.range.start.line === 4 && a.results[5].range.start.character === 0, JSON.stringify(a.results[5]));
check("server: a request it does not know is answered with nothing", a.results[6] === null, JSON.stringify(a.results[6]));

// The trace names symbols by their index; the server's list puts each in the text.
const traceSteps = trace(george({ program: subroutine, data: "3\n4\n", mode: "trace" }).err);
check("server: trace steps fall on symbols", traceSteps.every((s) => symbols[s.at] && (symbols[s.at].printed === s.symbol || /^[0-9.]/.test(s.symbol))),
  JSON.stringify(traceSteps.map((s) => [s.at, s.symbol, symbols[s.at]?.printed])));

// Mistakes are found where they are, with the reader's own message.
a = answer(server, "a b +\n  (o) ;\n", []);
check("server: a mistake", a.diagnostics.length === 1 && a.diagnostics[0].severity === 1 && /is not a name/.test(a.diagnostics[0].message)
  && a.diagnostics[0].range.start.line === 1 && a.diagnostics[0].range.start.character === 2, JSON.stringify(a.diagnostics));
a = answer(server, "3, 4 14 ↓ ]\n", []);
check("server: numerals that run together", a.diagnostics.some((d) => d.code === "numerals-run-together" && /414/.test(d.message))
  && a.diagnostics.some((d) => d.code === "no-mark"), JSON.stringify(a.diagnostics));
// ASCII spellings are the same symbols.
a = answer(server, "R R mul sqrt alpha down 0 up", [{ method: "textDocument/hover", params: at(0, 5) }, { method: "textDocument/hover", params: at(0, 15) }]);
check("server: ASCII spellings", /\*\*`×`\*\*/.test(a.results[0]?.contents.value) && /\*\*`α`\*\* variable/.test(a.results[1]?.contents.value), JSON.stringify(a.results));

// The server finds no error in any example.
const faulted = examples.filter((e) => answer(server, e.program, []).diagnostics.some((d) => d.severity === 1)).map((e) => e.name);
check("server: every example", faulted.length === 0, faulted.join());

console.log(`${names.length} examples run through WebAssembly, ${checks} checks, ${failed} failed`);
process.exit(failed ? 1 : 0);
