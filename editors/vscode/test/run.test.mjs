// Runs every example through the runner the "Run Program" command uses, with
// the arguments the command gives it, and compares what it punches with
// the .out file and the stop with the .stop file.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

const runner = new URL("../server/run.js", import.meta.url).pathname;
const examples = new URL("../../../examples/", import.meta.url).pathname;
const george = (...args) => spawnSync(process.execPath, [runner, ...args]);

test("the built-in george runs every example", () => {
  const names = readdirSync(examples).filter((f) => f.endsWith(".george"));
  assert.ok(names.length >= 15);
  for (const file of names) {
    const base = examples + file.slice(0, -7);
    const data = existsSync(`${base}.in`) ? ["--data", `${base}.in`] : [];
    const result = george("run", examples + file, ...data);
    assert.equal(result.stdout.toString(), readFileSync(`${base}.out`, "utf8"), file);
    if (existsSync(`${base}.stop`)) {
      assert.equal(result.status, 1, file);
      assert.ok(result.stderr.toString().startsWith(`stop ${readFileSync(`${base}.stop`, "utf8").trim()}:`), file);
    } else {
      assert.equal(result.status, 0, `${file}: ${result.stderr}`);
    }
  }
});

test("a relative path, the trace and the cards", () => {
  const result = spawnSync(process.execPath, [runner, "run", "subroutine.george", "--input", "3,4", "--trace", "--cards"], { cwd: examples });
  assert.equal(result.status, 0, result.stderr.toString());
  assert.equal(result.stdout.toString(), " 500000000 000000000\n");
  assert.match(result.stderr.toString(), /^ {3}0 {2}R {8}3\n {3}1 {2}R {8}3 {2}4\n/);
});

test("a stop is reported with where it happened", () => {
  const file = join(mkdtempSync(join(tmpdir(), "george-")), "pairs.george");
  writeFileSync(file, "* 0 R R + (P) ; 0 ↑\n");
  const result = george("run", file, "--input", "1,2,3");
  assert.equal(result.status, 1);
  assert.equal(result.stdout.toString(), "3\n");
  assert.match(result.stderr.toString(), /^stop 1 11-6x, 1 7-16x: data card wanted\n {2}at symbol 3 "R", line 1, column 7/);
});

test("deck, format, table and stops", () => {
  const deck = george("deck", examples + "squares.george");
  assert.equal(deck.status, 0);
  assert.match(deck.stdout.toString(), /^card {2}1: 1\/2 1\/0 2\/2 0\/2 8\/1 9\/6/);
  assert.match(george("format", examples + "squares.george").stdout.toString(), /^1, 20 rep \(j\) j j × \(P\); \]/);
  assert.match(george("table").stdout.toString(), /rep/);
  assert.match(george("stops").stdout.toString(), /data card wanted/);
});

test("a mistake in the program, and a file that is not there", () => {
  const file = join(mkdtempSync(join(tmpdir(), "george-")), "broken.george");
  writeFileSync(file, "a b +\n  (o) ;\n");
  const broken = george("run", file);
  assert.equal(broken.status, 2);
  assert.match(broken.stderr.toString(), /line 2, column 3: \(o\) is not a name/);
  const missing = george("run", "/no/such/program.george");
  assert.equal(missing.status, 2);
  assert.match(missing.stderr.toString(), /\/no\/such\/program\.george/);
});
