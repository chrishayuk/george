// Talks to the language server the way an editor does: over standard input
// and output. It tests the WebAssembly build the extension ships and, when
// it has been built, the native program other editors would use.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import test from "node:test";

const here = new URL("..", import.meta.url).pathname;
const native = new URL("../../../target/release/george-lsp", import.meta.url).pathname;
const servers = [["WebAssembly", process.execPath, [`${here}server/lsp.js`, "--stdio"]]];
if (existsSync(native)) servers.push(["native", native, []]);

function connect(command, args) {
  const child = spawn(command, args, { stdio: ["pipe", "pipe", "inherit"] });
  let buffer = Buffer.alloc(0);
  const waiting = [];
  const arrived = [];
  child.stdout.on("data", (chunk) => {
    buffer = Buffer.concat([buffer, chunk]);
    for (;;) {
      const header = buffer.indexOf("\r\n\r\n");
      if (header === -1) return;
      const length = Number(/Content-Length: (\d+)/i.exec(buffer.subarray(0, header).toString())[1]);
      if (buffer.length < header + 4 + length) return;
      const message = JSON.parse(buffer.subarray(header + 4, header + 4 + length).toString());
      buffer = buffer.subarray(header + 4 + length);
      const next = waiting.shift();
      if (next) next(message);
      else arrived.push(message);
    }
  });
  const send = (message) => {
    const body = JSON.stringify({ jsonrpc: "2.0", ...message });
    child.stdin.write(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
  };
  const receive = () => new Promise((resolve) => (arrived.length ? resolve(arrived.shift()) : waiting.push(resolve)));
  let id = 0;
  const request = async (method, params) => {
    send({ id: ++id, method, params });
    const reply = await receive();
    assert.equal(reply.id, id);
    return reply.result;
  };
  const exited = new Promise((resolve) => child.on("exit", resolve));
  return { send, receive, request, exited };
}

const uri = "file:///subroutine.george";
const program = `# Section 1.7, page 9: the square root of the sum of two squares.
R R 6 ↓ (P) ;                       # main program
]                                   # end of main program
* 6 dup mul rev dup × + .5 pow ]    # subroutine
`;

for (const [name, command, args] of servers) {
  test(`${name} server: a session from initialize to exit`, async () => {
    const server = connect(command, args);
    const initialised = await server.request("initialize", { processId: null, rootUri: null, capabilities: {} });
    assert.equal(initialised.serverInfo.name, "george-lsp");
    assert.equal(initialised.capabilities.hoverProvider, true);
    server.send({ method: "initialized", params: {} });

    server.send({ method: "textDocument/didOpen", params: { textDocument: { uri, languageId: "george", version: 1, text: program } } });
    const clean = await server.receive();
    assert.equal(clean.method, "textDocument/publishDiagnostics");
    assert.deepEqual(clean.params.diagnostics, []);

    const down = await server.request("textDocument/hover", { textDocument: { uri }, position: { line: 1, character: 6 } });
    assert.match(down.contents.value, /\*\*`↓`\*\* skip to subroutine/);
    assert.match(down.contents.value, /code 6\/1/);
    const mul = await server.request("textDocument/hover", { textDocument: { uri }, position: { line: 3, character: 9 } });
    assert.match(mul.contents.value, /\*\*`×`\*\* multiplication/);
    assert.match(mul.contents.value, /diadic operator · code 7\/0/);
    assert.match(mul.contents.value, /Written here as `mul`/);

    const definition = await server.request("textDocument/definition", { textDocument: { uri }, position: { line: 1, character: 4 } });
    assert.deepEqual(definition.range.start, { line: 3, character: 0 });

    const lenses = await server.request("textDocument/codeLens", { textDocument: { uri } });
    assert.deepEqual(lenses.map((l) => l.command.command), ["george.run", "george.runTrace", "george.deck"]);

    const symbols = await server.request("textDocument/documentSymbol", { textDocument: { uri } });
    assert.deepEqual(symbols.map((s) => s.name), ["subroutine 6"]);

    const tokens = await server.request("textDocument/semanticTokens/full", { textDocument: { uri } });
    assert.equal(tokens.data.length % 5, 0);
    assert.ok(tokens.data.length >= 5 * 20);

    // Break the program: a letter that is not in the alphabet, then numerals that run together.
    server.send({ method: "textDocument/didChange", params: { textDocument: { uri, version: 2 }, contentChanges: [{ text: "a b + (o) ;\n" }] } });
    const broken = await server.receive();
    assert.equal(broken.params.diagnostics.length, 1);
    assert.equal(broken.params.diagnostics[0].severity, 1);
    assert.deepEqual(broken.params.diagnostics[0].range, { start: { line: 0, character: 6 }, end: { line: 0, character: 9 } });
    server.send({ method: "textDocument/didChange", params: { textDocument: { uri, version: 3 }, contentChanges: [{ text: "3, 4 14 ↓ ]\n* 30 ]\n" }] } });
    const joined = await server.receive();
    assert.deepEqual(joined.params.diagnostics.map((d) => d.code), ["numerals-run-together", "skip-number"]);

    assert.equal(await server.request("shutdown", null), null);
    server.send({ method: "exit" });
    assert.equal(await server.exited, 0);
  });
}
