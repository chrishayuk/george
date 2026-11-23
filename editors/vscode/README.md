# GEORGE for Visual Studio Code

Editor support for [GEORGE](https://github.com/chrishayuk/george), the General Order Generator that Charles Hamblin wrote for the English Electric DEUCE in 1957-58. It was a "reverse Polish" scheme with no addresses: each symbol of a program works on a running accumulator of twelve cells. An editor that can say what each symbol does to those cells helps a great deal.

Nothing else needs installing. The language server and the george program are built into the extension as WebAssembly.

## What you get

- **Syntax highlighting** for `.george` files. It covers variables (`a` to `z` without `o`, the Greek letters, and `alpha`, `beta` and so on), names such as `(a)`, numerals, marks (`* 6`), the bar signs of suffixes, the operators and functions, the control symbols (`rep`, `]`, `↓`, `↑`, `wait`), input and output (`R`, `(P)`, `R|`, `P||`, `I`) and `#` comments. It uses the reader's rule that a run of letters which is not a word is one variable per letter, so `ab+` is `a b +`. The language server adds semantic colours on top.
- **Hover** on any symbol to see what it is, its code in the coding table (row/column, as punched), whether it is monadic or diadic, what it does to the accumulator (`… a b → … a+b`), the stop it can give, and a few words of the 1958 manual. Hover works the same for Unicode and ASCII spellings: `×` or `mul`, `÷` or `/`, `↓` or `down`, `↑` or `up`, `∨` or `or`, `~` or `not`, `&` or `and`, `√` or `sqrt`, and `α` or `alpha`. On a numeral it shows the whole number the machine will read. On a mark it shows how often the program skips to it.
- **Problems as you type.** Mistakes come from the interpreter's own reader, placed on the symbol that is wrong. The server also warns about things that are easy to get wrong in a program the reader accepts. Numerals written next to each other run together (`3, 4 14 ↓` calls mark 414 mod 32 = 30). A skip may go to a mark the program does not have. A mark may be set twice. A program may be longer than the 512 symbols the instruction store holds.
- **Go to definition** from `6 ↓` (or the `6`) to `* 6`, and an **outline** of the marks and subroutines.
- **Run Program.** Use the play button at the top of the editor, Cmd+Enter (Ctrl+Enter on Windows and Linux), or the `▶ Run` code lens. The program runs in a terminal, which shows each number punched and then the stop it ended on, if any, with the symbol, line and column. If a file with the same name ending `.in` is next to the program, it is given as the data cards. **Run Program with Trace** also prints each symbol obeyed and the accumulator after it.
- **Show the Deck** (`george deck`) shows the program as punched: symbol codes sixteen to a card. **Show the Printed Form** (`george format`), **Show the Coding Table** (`george table`) and **Show the Stops** (`george stops`) open beside the program.

## Settings

| Setting | What it does |
| --- | --- |
| `george.run.data` | The data cards for Run Program. The default is `${fileDirname}/${fileBasenameNoExtension}.in`, used when that file exists. |
| `george.run.keys` | Numbers set on the input keys, one taken for each `I` (`--keys`). |
| `george.run.singleShot` | Carry on through resumable stops (`--single-shot`). |
| `george.run.cards` | Print each punched card as its twenty columns (`--cards`). |
| `george.server.path` | Use a `george-lsp` program you built yourself as the language server. |
| `george.interpreter.path` | Run programs with a `george` you built yourself. |
| `george.trace.server` | Log the messages exchanged with the language server. |

## Building it

From this folder, with Rust (and its `wasm32-wasip1` target) and Node installed:

```sh
npm install
npm run build        # compiles the server and george to WebAssembly, bundles the extension
npm test             # grammar, language server and runner tests
npm run test:vscode  # opens a real VS Code and drives the extension
npm run package      # writes george-language-0.1.0.vsix
code --install-extension george-language-0.1.0.vsix
```

To try it without installing, open this folder in VS Code and press F5.

## How it fits together

| File | What it is |
| --- | --- |
| `syntaxes/george.tmLanguage.json` | The highlighting grammar. |
| `language-configuration.json` | The comment character, brackets and word pattern. |
| `src/extension.js` | Starts the language server and provides the commands. |
| `server/lsp.js`, `server/george-lsp.wasm` | The language server from `../../lsp`, compiled for WASI. |
| `server/run.js`, `server/george.wasm` | The george program, compiled for WASI. |
| `server/wasi.js` | A small WASI host for Node, with no dependencies. It lets george read the files it is given, and nothing else. |
