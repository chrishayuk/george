# GEORGE in your editor

Two pieces give an editor an understanding of GEORGE:

- **`george-lsp`**, a language server in [`../lsp`](../lsp). It works with any editor that speaks the Language Server Protocol.
- **A Visual Studio Code extension** in [`vscode`](vscode). It bundles the server and the george program, compiled to WebAssembly, and adds syntax highlighting and commands to run a program and show its deck. Its [README](vscode/README.md) lists the features.

The server finds mistakes with `george::text::parse`, the reader `george run` uses, so the editor and the interpreter cannot disagree about what is a mistake. What it says about each symbol comes from the coding table in `src/symbols.rs` and the words of the 1958 manual.

## The language server

```sh
cargo install --git https://github.com/chrishayuk/george george-lsp
```

or, in a clone of this repository, `cargo build --release -p george-lsp`, which writes `target/release/george-lsp`. The language server is its own crate in the workspace, so `george` itself keeps no dependencies.

It talks on standard input and output and takes no arguments. It provides:

- **diagnostics**: the reader's own error, and warnings for numerals that run together, a skip to a mark that is not in the program, a mark set twice, and more than 512 symbols
- **hover**: what the symbol is, its row/column code, monadic or diadic, what it does to the accumulator, the stop it can give, and the manual's words
- **go to definition** from a skip to its mark, and **document symbols** for the marks and subroutines
- **completion** of the words of the text form (`dup`, `rev`, `rep`, `sqrt`, `alpha` ...)
- **semantic tokens**, so an editor with no GEORGE grammar still gets highlighting
- **code lenses** above the first line: `▶ Run`, `Trace` and `Deck`. They name the commands `george.run`, `george.runTrace` and `george.deck`, with the document's URI as the argument. The VS Code extension and the web playground provide those commands. In another editor, bind them yourself or ignore the lenses.
- a request of its own, `george/symbols`, which lists every symbol with its range and code. The web playground uses it to show which symbol each trace step obeyed.

Unicode and ASCII spellings are the same symbols to the server: `×`/`mul`, `÷`/`/`/`div`, `↓`/`down`, `↑`/`up`, `∨`/`or`, `~`/`not`, `&`/`and`, `√`/`sqrt`, and the Greek letters and their names.

### Neovim (0.11 or later)

```lua
vim.filetype.add({ extension = { george = "george" } })
vim.lsp.config("george", {
  cmd = { "/path/to/george-lsp" },
  filetypes = { "george" },
})
vim.lsp.enable("george")

-- Optional: run the file in a terminal split.
vim.api.nvim_create_user_command("GeorgeRun", function()
  local file = vim.fn.expand("%:p")
  local data = vim.fn.expand("%:p:r") .. ".in"
  local cmd = "george run " .. vim.fn.shellescape(file)
  if vim.fn.filereadable(data) == 1 then cmd = cmd .. " --data " .. vim.fn.shellescape(data) end
  vim.cmd("botright split | terminal " .. cmd)
end, {})
```

Set `vim.bo.commentstring = "# %s"` for the filetype if you want comment toggling.

### Helix

In `languages.toml`:

```toml
[language-server.george-lsp]
command = "/path/to/george-lsp"

[[language]]
name = "george"
scope = "source.george"
file-types = ["george"]
comment-token = "#"
language-servers = ["george-lsp"]
```

### Emacs (eglot)

```elisp
(define-derived-mode george-mode prog-mode "GEORGE"
  (setq-local comment-start "# "))
(add-to-list 'auto-mode-alist '("\\.george\\'" . george-mode))
(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs '(george-mode "/path/to/george-lsp")))
```

These configurations follow each editor's documented format but have not been run here. The server itself is tested over standard input and output by `vscode/test/lsp.test.mjs`, against both the WebAssembly build and the native one.

## Running a program from the command line

```sh
george run program.george --data program.in
george run program.george --input 3,4 --trace
george deck program.george
```

See the main [README](../README.md) for every option.
