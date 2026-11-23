// Runs the george program:  node run.js run program.george [--data cards.in] ...
// The arguments are george's own (see `george help`). Files named in them
// are made absolute, since george, compiled for WASI, sees the file system
// from "/" and may only read it.
"use strict";

const fs = require("fs");
const path = require("path");
const { run } = require("./wasi.js");

const args = process.argv.slice(2).map((arg) => {
  if (arg.startsWith("-") || !fs.existsSync(arg)) return arg;
  return path.resolve(arg).split(path.sep).join("/");
});
process.exitCode = run(path.join(__dirname, "george.wasm"), { args });
