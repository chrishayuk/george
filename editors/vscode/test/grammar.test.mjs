// Tokenises GEORGE with the same TextMate engine VS Code uses and checks the
// scopes the grammar assigns, and that its words are the reader's words.
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { createRequire } from "node:module";
import test from "node:test";
import oniguruma from "vscode-oniguruma";
import textmate from "vscode-textmate";

const require = createRequire(import.meta.url);
const grammarPath = new URL("../syntaxes/george.tmLanguage.json", import.meta.url).pathname;
const root = new URL("../../../", import.meta.url).pathname;

const wasm = readFileSync(require.resolve("vscode-oniguruma/release/onig.wasm"));
await oniguruma.loadWASM(wasm.buffer.slice(wasm.byteOffset, wasm.byteOffset + wasm.byteLength));
const registry = new textmate.Registry({
  onigLib: Promise.resolve({
    createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
    createOnigString: (text) => new oniguruma.OnigString(text),
  }),
  loadGrammar: async () => textmate.parseRawGrammar(readFileSync(grammarPath, "utf8"), grammarPath),
});
const grammar = await registry.loadGrammar("source.george");

/** Each non-blank piece of text with its most specific scope. */
function scopes(source) {
  const out = [];
  let state = textmate.INITIAL;
  for (const line of source.split("\n")) {
    const result = grammar.tokenizeLine(line, state);
    for (const token of result.tokens) {
      const text = line.slice(token.startIndex, token.endIndex);
      if (text.trim()) out.push([text.trim(), token.scopes.at(-1)]);
    }
    state = result.ruleStack;
  }
  return out;
}

test("a subroutine call, a mark and a comment", () => {
  assert.deepEqual(scopes("R R 6 ↓ (P) ; ] # main\n* 6 dup × rev"), [
    ["R", "support.function.io.read.george"],
    ["R", "support.function.io.read.george"],
    ["6", "constant.numeric.george"],
    ["↓", "keyword.control.george"],
    ["(", "punctuation.section.parens.begin.george"],
    ["P", "support.function.io.punch.george"],
    [")", "punctuation.section.parens.end.george"],
    [";", "keyword.operator.cancel.george"],
    ["]", "keyword.control.george"],
    ["#", "punctuation.definition.comment.george"],
    ["main", "comment.line.number-sign.george"],
    ["*", "keyword.control.mark.george"],
    ["6", "entity.name.label.george"],
    ["dup", "support.function.accumulator.george"],
    ["×", "keyword.operator.arithmetic.george"],
    ["rev", "support.function.accumulator.george"],
  ]);
});

test("a run of letters that is not a word is one variable per letter", () => {
  assert.deepEqual(scopes("ab+ repx rep (x)"), [
    ["a", "variable.other.george"],
    ["b", "variable.other.george"],
    ["+", "keyword.operator.arithmetic.george"],
    ["r", "variable.other.george"],
    ["e", "variable.other.george"],
    ["p", "variable.other.george"],
    ["x", "variable.other.george"],
    ["rep", "keyword.control.george"],
    ["(", "punctuation.section.parens.begin.george"],
    ["x", "variable.other.name.george"],
    [")", "punctuation.section.parens.end.george"],
  ]);
});

test("ASCII spellings, Greek letters, suffixes and blocks", () => {
  assert.deepEqual(scopes("mul / down up or not and sqrt √ alpha β j|a i j||(b) 1, 20 R|(a) P||(b) I o"), [
    ["mul", "keyword.operator.word.george"],
    ["/", "keyword.operator.arithmetic.george"],
    ["down", "keyword.control.george"],
    ["up", "keyword.control.george"],
    ["or", "keyword.operator.word.george"],
    ["not", "keyword.operator.word.george"],
    ["and", "keyword.operator.word.george"],
    ["sqrt", "support.function.george"],
    ["√", "support.function.george"],
    ["alpha", "variable.other.greek.george"],
    ["β", "variable.other.george"],
    ["j", "variable.other.george"],
    ["|", "keyword.operator.suffix.george"],
    ["a", "variable.other.george"],
    ["i", "variable.other.george"],
    ["j", "variable.other.george"],
    ["||", "keyword.operator.suffix.george"],
    ["(", "punctuation.section.parens.begin.george"],
    ["b", "variable.other.name.george"],
    [")", "punctuation.section.parens.end.george"],
    ["1", "constant.numeric.george"],
    [",", "punctuation.separator.george"],
    ["20", "constant.numeric.george"],
    ["R|", "support.function.io.block.george"],
    ["(", "punctuation.section.parens.begin.george"],
    ["a", "variable.other.name.george"],
    [")", "punctuation.section.parens.end.george"],
    ["P||", "support.function.io.block.george"],
    ["(", "punctuation.section.parens.begin.george"],
    ["b", "variable.other.name.george"],
    [")", "punctuation.section.parens.end.george"],
    ["I", "support.function.io.keys.george"],
    ["o", "invalid.illegal.letter.george"],
  ]);
});

test("the grammar's words are the words of src/text.rs", () => {
  const text = readFileSync(root + "src/text.rs", "utf8");
  const words = [...text.matchAll(/\("([a-z]+)", Sym::/g)].map((m) => m[1]);
  assert.ok(words.length >= 21);
  const grammarText = readFileSync(grammarPath, "utf8");
  for (const word of words) assert.ok(new RegExp(`[(|:]${word}[|)]`).test(grammarText), word);
});

test("nothing in the examples is marked illegal", () => {
  const dir = root + "examples/";
  for (const file of readdirSync(dir).filter((f) => f.endsWith(".george"))) {
    const bad = scopes(readFileSync(dir + file, "utf8")).filter(([, scope]) => scope.startsWith("invalid"));
    assert.deepEqual(bad, [], file);
  }
});
