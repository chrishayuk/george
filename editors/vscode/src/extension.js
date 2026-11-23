// The GEORGE extension: starts the language server, and adds commands that
// run the program in the editor (showing what it punches and the stop it
// ends on) and show it as a deck of cards, in printed form, and the coding
// table and the stops.
"use strict";

const vscode = require("vscode");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { execFile } = require("child_process");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");

/** @type {LanguageClient | undefined} */
let client;
const SCHEME = "george-output";
const listings = new Map(); // uri -> text, for the listings shown
const changed = new vscode.EventEmitter();

function setting(name) {
  const value = vscode.workspace.getConfiguration("george").get(name);
  return typeof value === "string" ? value.trim() : value;
}

async function startServer(context) {
  const custom = setting("server.path");
  // Built in: the server compiled to WebAssembly, run by the editor's own
  // Node, so nothing has to be installed.
  const builtIn = { module: context.asAbsolutePath(path.join("server", "lsp.js")), transport: TransportKind.stdio };
  const serverOptions = custom ? { command: custom, args: [] } : { run: builtIn, debug: builtIn };
  client = new LanguageClient("george", "GEORGE", serverOptions, {
    documentSelector: [{ language: "george" }],
  });
  await client.start();
}

async function restartServer(context) {
  if (client) await client.stop().catch(() => {});
  client = undefined;
  await startServer(context);
}

/** The program a command is for: the one a code lens names, or the one in the editor. */
async function programFor(target) {
  let document;
  if (target) {
    const uri = typeof target === "string" ? vscode.Uri.parse(target) : target;
    document = await vscode.workspace.openTextDocument(uri);
  } else {
    document = vscode.window.activeTextEditor?.document;
  }
  if (!document || document.languageId !== "george") {
    vscode.window.showInformationMessage("Open a GEORGE program (a .george file) first.");
    return undefined;
  }
  let file = document.uri.fsPath;
  if (document.isUntitled || document.uri.scheme !== "file") {
    // Not on disk yet: use a copy.
    file = path.join(os.tmpdir(), "untitled.george");
    fs.writeFileSync(file, document.getText());
  } else if (document.isDirty) {
    await document.save();
  }
  return { document, file };
}

/** The george command line: the built-in WebAssembly one, or one set in the settings. */
function command(context, args) {
  const custom = setting("interpreter.path");
  if (custom) return { program: custom, args, env: {} };
  return {
    program: process.execPath,
    args: [context.asAbsolutePath(path.join("server", "run.js")), ...args],
    env: { ELECTRON_RUN_AS_NODE: "1" },
  };
}

function runArguments(file, trace) {
  const args = ["run", file];
  const folder = vscode.workspace.getWorkspaceFolder(vscode.Uri.file(file))?.uri.fsPath ?? path.dirname(file);
  const data = String(setting("run.data") || "")
    .replaceAll("${fileDirname}", path.dirname(file))
    .replaceAll("${fileBasenameNoExtension}", path.basename(file, path.extname(file)))
    .replaceAll("${workspaceFolder}", folder);
  if (data && fs.existsSync(data)) args.push("--data", data);
  const keys = setting("run.keys");
  if (keys) args.push("--keys", keys.replace(/\s+/g, ""));
  if (setting("run.singleShot")) args.push("--single-shot");
  if (setting("run.cards")) args.push("--cards");
  if (trace) args.push("--trace");
  return args;
}

/** Runs the program in a terminal: the numbers punched, then the stop, if any. */
async function runProgram(context, target, trace) {
  const found = await programFor(target);
  if (!found) return;
  const { program, args, env } = command(context, runArguments(found.file, trace));
  const execution = new vscode.ProcessExecution(program, args, { env, cwd: path.dirname(found.file) });
  const task = new vscode.Task(
    { type: "george" },
    vscode.TaskScope.Workspace,
    `${trace ? "Trace" : "Run"} ${path.basename(found.file)}`,
    "george",
    execution,
    []
  );
  task.presentationOptions = {
    reveal: vscode.TaskRevealKind.Always,
    panel: vscode.TaskPanelKind.Dedicated,
    focus: true,
    clear: true,
    // Without this the terminal first prints the whole command line.
    echo: false,
    showReuseMessage: false,
  };
  await vscode.tasks.executeTask(task);
}

/** Runs george and resolves to what it printed. */
function capture(context, args) {
  const { program, args: argv, env } = command(context, args);
  return new Promise((resolve) => {
    execFile(program, argv, { env: { ...process.env, ...env }, maxBuffer: 16 << 20 }, (error, stdout, stderr) => {
      resolve({ code: error ? error.code ?? 1 : 0, stdout: String(stdout), stderr: String(stderr) });
    });
  });
}

/** Shows what a george command prints, beside the program, as a read-only document. */
async function showListing(context, name, args, title) {
  const result = await capture(context, args);
  const text = result.stdout + (result.stderr ? `\n${result.stderr}` : "");
  const uri = vscode.Uri.from({ scheme: SCHEME, path: `/${name}` });
  listings.set(uri.toString(), `${title}\n\n${text}`);
  changed.fire(uri);
  const document = await vscode.workspace.openTextDocument(uri);
  await vscode.window.showTextDocument(document, { viewColumn: vscode.ViewColumn.Beside, preview: true, preserveFocus: true });
}

async function listProgram(context, target, mode) {
  const found = await programFor(target);
  if (!found) return;
  const base = path.basename(found.file, ".george");
  const title = mode === "deck"
    ? `${path.basename(found.file)} as punched: symbol codes row/column, sixteen to a card (section 2.2)`
    : `${path.basename(found.file)} in the manual's printed form`;
  await showListing(context, `${base}.${mode}`, [mode, found.file], title);
}

async function activate(context) {
  context.subscriptions.push(
    vscode.workspace.registerTextDocumentContentProvider(SCHEME, {
      onDidChange: changed.event,
      provideTextDocumentContent: (uri) => listings.get(uri.toString()) ?? "",
    }),
    vscode.commands.registerCommand("george.run", (target) => runProgram(context, target, false)),
    vscode.commands.registerCommand("george.runTrace", (target) => runProgram(context, target, true)),
    vscode.commands.registerCommand("george.deck", (target) => listProgram(context, target, "deck")),
    vscode.commands.registerCommand("george.format", (target) => listProgram(context, target, "format")),
    vscode.commands.registerCommand("george.table", () =>
      showListing(context, "coding-table", ["table"], "The GEORGE coding table (Appendix I)")),
    vscode.commands.registerCommand("george.stops", () =>
      showListing(context, "stops", ["stops"], "The stops (section 2.4): indication, cause, and what a single shot does")),
    vscode.commands.registerCommand("george.restartServer", () => restartServer(context)),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration("george.server.path")) restartServer(context);
    })
  );
  await startServer(context);
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate, runArguments };
