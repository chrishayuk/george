// Runs inside VS Code. Exercises the extension the way a person would: open
// a program, hover, jump to a mark, see problems, run it, and look at its deck.
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const vscode = require("vscode");

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function until(what, probe, seconds = 20) {
  for (let i = 0; i < seconds * 10; i++) {
    const value = await probe();
    if (value) return value;
    await sleep(100);
  }
  throw new Error(`timed out waiting for ${what}`);
}

const hoverText = (hovers) =>
  hovers.flatMap((h) => h.contents).map((c) => (typeof c === "string" ? c : c.value)).join("\n");

exports.run = async function run() {
  const folder = vscode.workspace.workspaceFolders[0].uri.fsPath;
  const document = await vscode.workspace.openTextDocument(path.join(folder, "subroutine.george"));
  await vscode.window.showTextDocument(document);
  assert.equal(document.languageId, "george", "the .george extension selects the language");
  assert.equal(document.lineAt(2).text.slice(0, 13), "R R 6 ↓ (P) ;");

  // The server is ready when it answers a hover.
  const skip = new vscode.Position(2, 6);
  const hovers = await until("a hover from the language server", async () => {
    const found = await vscode.commands.executeCommand("vscode.executeHoverProvider", document.uri, skip);
    return found.length ? found : undefined;
  });
  assert.match(hoverText(hovers), /skip to subroutine/);
  assert.match(hoverText(hovers), /code 6\/1/);
  console.log("hover ok");

  const definitions = await vscode.commands.executeCommand("vscode.executeDefinitionProvider", document.uri, new vscode.Position(2, 4));
  assert.equal(definitions.length, 1);
  assert.equal((definitions[0].range || definitions[0].targetRange).start.line, 4);
  const lenses = await vscode.commands.executeCommand("vscode.executeCodeLensProvider", document.uri);
  assert.ok(lenses.some((lens) => lens.command?.command === "george.run"));
  const tokens = await vscode.commands.executeCommand("vscode.provideDocumentSemanticTokens", document.uri);
  assert.ok(tokens.data.length >= 5 * 20);
  console.log("definition, code lenses and semantic tokens ok");

  assert.deepEqual(vscode.languages.getDiagnostics(document.uri), [], "the program has no problems");
  const broken = await vscode.workspace.openTextDocument({ language: "george", content: "a b + (o) ;\n" });
  await vscode.window.showTextDocument(broken);
  const problems = await until("diagnostics", () => {
    const found = vscode.languages.getDiagnostics(broken.uri);
    return found.length ? found : undefined;
  });
  assert.equal(problems[0].severity, vscode.DiagnosticSeverity.Error);
  assert.equal(problems[0].source, "george");
  assert.equal(problems[0].range.start.character, 6);
  console.log("diagnostics ok");

  // Run Program: a task that ends with exit code 0, with the .in file as data.
  await vscode.window.showTextDocument(document);
  const finished = new Promise((resolve) => {
    const listener = vscode.tasks.onDidEndTaskProcess((event) => {
      if (event.execution.task.definition.type === "george") {
        listener.dispose();
        resolve(event.exitCode);
      }
    });
  });
  await vscode.commands.executeCommand("george.run");
  assert.equal(await Promise.race([finished, sleep(20000).then(() => "timed out")]), 0);
  console.log("run ok");

  await vscode.commands.executeCommand("george.deck", document.uri.toString());
  const deck = await until("the deck", () => vscode.workspace.textDocuments.find((d) => d.uri.scheme === "george-output"));
  assert.match(deck.getText(), /card {2}1: 0\/15 0\/15 6\/2 6\/1 1\/15 2\/0/);
  console.log("deck ok");
};
