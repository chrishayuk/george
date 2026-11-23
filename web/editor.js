// Turns the program box into an editor that understands GEORGE: colours,
// problems as you type, hover help and completion, all from the language
// server in lsp-worker.js. It can also show which symbol a trace step obeyed.
//
// The editor itself is CodeMirror, loaded from a CDN because this page has
// no build step. If that cannot be loaded (no network, say), upgrade()
// resolves to null and the plain text box carries on working.

const CDN = "https://esm.sh/";
const PIN = "?deps=@codemirror/state@6.7.6,@codemirror/view@6.43.15";
// The order of the server's semantic token legend.
const TOKEN_TYPES = ["comment", "number", "variable", "operator", "keyword", "function", "label", "macro"];

let worker = null;
let nextId = 0;
const waiting = new Map();

/** Opens `text` in the language server and asks it the given requests. */
export function ask(text, requests) {
  if (!worker) {
    worker = new Worker(new URL("lsp-worker.js", import.meta.url), { type: "module" });
    worker.onmessage = ({ data }) => {
      waiting.get(data.id)?.(data.error ? null : data);
      waiting.delete(data.id);
    };
    worker.onerror = () => {
      for (const resolve of waiting.values()) resolve(null);
      waiting.clear();
    };
  }
  const id = ++nextId;
  return new Promise((resolve) => {
    waiting.set(id, resolve);
    worker.postMessage({ id, wasmUrl: new URL("george-lsp.wasm", import.meta.url).href, text, requests });
  });
}

const offsetOf = (doc, position) => {
  const line = doc.line(Math.min(position.line + 1, doc.lines));
  return Math.min(line.from + position.character, line.to);
};
const positionOf = (doc, offset) => {
  const line = doc.lineAt(offset);
  return { line: line.number - 1, character: offset - line.from };
};

/** The server's small Markdown: paragraphs, bold, code and code blocks. */
export function render(markdown) {
  const element = (tag, text) => {
    const made = document.createElement(tag);
    if (text !== undefined) made.textContent = text;
    return made;
  };
  const box = element("div");
  box.className = "george-hover";
  const fill = (parent, text) => {
    // Code may hold stars (`**`, `*n`), so it is looked for inside bold too.
    for (const piece of text.split(/(\*\*(?:`[^`]+`|[^*`])+\*\*|`[^`]+`)/)) {
      if (piece.startsWith("**") && piece.length > 4) fill(parent.appendChild(element("strong")), piece.slice(2, -2));
      else if (piece.startsWith("`") && piece.length > 2) parent.appendChild(element("code", piece.slice(1, -1)));
      else if (piece) parent.appendChild(document.createTextNode(piece));
    }
  };
  for (const block of markdown.split(/\n{2,}/)) {
    const fence = /^```\w*\n([\s\S]*?)\n?```$/.exec(block.trim());
    if (fence) box.appendChild(element("pre", fence[1]));
    else if (block.startsWith(">")) {
      // The manual, quoted: "> words\n>\n> — manual, section n".
      const quote = box.appendChild(element("blockquote"));
      for (const para of block.split("\n").map((l) => l.replace(/^>\s?/, "")).join("\n").split(/\n\s*\n/)) {
        fill(quote.appendChild(element("p")), para.trim());
      }
    }
    else fill(box.appendChild(element("p")), block);
  }
  return box;
}

/**
 * Replaces a <textarea> with the editor. The textarea stays in the page,
 * hidden, and always holds the current program.
 *
 * @returns {Promise<{set(text: string): void} | null>}
 */
export async function upgrade(textarea, { onRun }) {
  let cm;
  try {
    const [state, view, commands, lint, autocomplete] = await Promise.all([
      import(`${CDN}@codemirror/state@6.7.6`),
      import(`${CDN}@codemirror/view@6.43.15${PIN}`),
      import(`${CDN}@codemirror/commands@6.11.1${PIN}`),
      import(`${CDN}@codemirror/lint@6.9.8${PIN}`),
      import(`${CDN}@codemirror/autocomplete@6.20.4${PIN}`),
    ]);
    cm = { ...state, ...view, ...commands, ...lint, ...autocomplete };
  } catch {
    return null;
  }
  const { EditorState, StateEffect, StateField, Decoration, EditorView, hoverTooltip, keymap } = cm;

  const setTokens = StateEffect.define();
  const tokens = StateField.define({
    create: () => Decoration.none,
    update(value, transaction) {
      for (const effect of transaction.effects) if (effect.is(setTokens)) return effect.value;
      return value.map(transaction.changes);
    },
    provide: (field) => EditorView.decorations.from(field),
  });
  // Modifier 1 of the legend marks names, (a), which get a class of their own.
  const marks = TOKEN_TYPES.map((type) => Decoration.mark({ class: `george-token-${type}` }));
  const basic = TOKEN_TYPES.map((type) => Decoration.mark({ class: `george-token-${type} george-token-name` }));

  // The server's five-numbers-per-token list, as editor decorations.
  const decorate = (doc, data) => {
    const ranges = [];
    let line = 0;
    let character = 0;
    for (let i = 0; i + 4 < data.length; i += 5) {
      line += data[i];
      character = data[i] ? data[i + 1] : character + data[i + 1];
      if (line >= doc.lines) break;
      const from = offsetOf(doc, { line, character });
      const to = Math.min(from + data[i + 2], doc.line(line + 1).to);
      const mark = (data[i + 4] & 1 ? basic : marks)[data[i + 3]];
      if (to > from && mark) ranges.push(mark.range(from, to));
    }
    return Decoration.set(ranges, true);
  };

  const severities = ["error", "error", "warning", "info", "hint"];
  const problems = cm.linter(async (editor) => {
    const doc = editor.state.doc;
    const answer = await ask(doc.toString(), [{ method: "textDocument/semanticTokens/full" }]);
    if (!answer || editor.state.doc !== doc) return [];
    editor.dispatch({ effects: setTokens.of(decorate(doc, answer.results[0]?.data || [])) });
    return answer.diagnostics.map((problem) => {
      const from = offsetOf(doc, problem.range.start);
      return { from, to: Math.max(offsetOf(doc, problem.range.end), from), severity: severities[problem.severity] || "info", message: problem.message, source: "george" };
    });
  }, { delay: 120 });

  const hover = hoverTooltip(async (editor, offset) => {
    const doc = editor.state.doc;
    const answer = await ask(doc.toString(), [{ method: "textDocument/hover", params: { position: positionOf(doc, offset) } }]);
    const found = answer?.results[0];
    if (!found || editor.state.doc !== doc) return null;
    return { pos: offsetOf(doc, found.range.start), end: offsetOf(doc, found.range.end), above: true, create: () => ({ dom: render(found.contents.value) }) };
  }, { hoverTime: 200 });

  // The words of the text form and the Greek letters' ASCII names: offered
  // when asked for (Ctrl+Space), or once three letters are typed, since a
  // letter or two is usually a variable.
  const kinds = { 6: "variable", 14: "keyword" };
  const complete = async (context) => {
    const word = context.matchBefore(/[a-z]+/);
    if (!context.explicit && !(word && word.text.length >= 3)) return null;
    const doc = context.state.doc;
    const answer = await ask(doc.toString(), [{ method: "textDocument/completion", params: { position: positionOf(doc, context.pos) } }]);
    const items = answer?.results[0] || [];
    if (!items.length) return null;
    return {
      from: word ? word.from : context.pos,
      validFor: /^[a-z]*$/,
      options: items.map((item) => ({ label: item.label, detail: item.detail, type: kinds[item.kind] || "text" })),
    };
  };

  // The symbol a trace step obeyed.
  const setCurrent = StateEffect.define();
  const current = StateField.define({
    create: () => Decoration.none,
    update(value, transaction) {
      for (const effect of transaction.effects) if (effect.is(setCurrent)) return effect.value;
      return transaction.docChanged ? Decoration.none : value;
    },
    provide: (field) => EditorView.decorations.from(field),
  });
  const currentMark = Decoration.mark({ class: "george-current" });

  const host = document.createElement("div");
  host.className = "george-editor";
  host.id = "editor";
  textarea.before(host);
  const editor = new EditorView({
    parent: host,
    state: EditorState.create({
      doc: textarea.value,
      extensions: [
        cm.history(),
        keymap.of([{ key: "Mod-Enter", run: () => { onRun(); return true; } }, ...cm.defaultKeymap, ...cm.historyKeymap]),
        tokens, current, problems, hover,
        cm.autocompletion({ override: [complete] }),
        EditorState.tabSize.of(4),
        EditorView.contentAttributes.of({ "aria-label": "G-program", spellcheck: "false", autocapitalize: "off", autocorrect: "off" }),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) textarea.value = update.state.doc.toString();
        }),
      ],
    }),
  });
  textarea.hidden = true;
  return {
    set(text) {
      if (editor.state.doc.toString() !== text) editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: text } });
    },
    /** Shows a range ({start, end} in protocol positions) as the current symbol, or none. */
    highlight(range) {
      const doc = editor.state.doc;
      if (!range) {
        editor.dispatch({ effects: setCurrent.of(Decoration.none) });
        return;
      }
      const from = offsetOf(doc, range.start);
      const to = Math.max(offsetOf(doc, range.end), from + 1);
      editor.dispatch({ effects: [setCurrent.of(Decoration.set([currentMark.range(from, Math.min(to, doc.length))])), EditorView.scrollIntoView(from, { y: "nearest" })] });
    },
    /** Puts the cursor at a line and column, both from 1, as george reports them. */
    select(line, column) {
      const doc = editor.state.doc;
      const from = offsetOf(doc, { line: line - 1, character: column - 1 });
      editor.dispatch({ selection: { anchor: from, head: Math.min(from + 1, doc.line(Math.min(line, doc.lines)).to) }, scrollIntoView: true });
      editor.focus();
    },
  };
}
