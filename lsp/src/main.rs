//! A language server for GEORGE.
//!
//! It speaks the Language Server Protocol over standard input and output, so
//! any editor with an LSP client can use it. What is a mistake comes from the
//! interpreter's own reader, `george::text::parse`, so the editor and
//! `george run` cannot disagree; what each symbol means comes from the coding
//! table in `george::symbols` and the words of the manual in `help.rs`.
//!
//! The server reads one message at a time and keeps nothing but the open
//! documents, which lets it also be compiled for WASI and run inside an
//! editor or a web page with nothing to install (see `editors/vscode` and
//! `web/lsp-worker.js`).

mod help;

use george::machine::{self, ACCUMULATOR_CELLS, SYMBOL_CAPACITY};
use george::symbols::{Sym, GREEK_ASCII, LETTERS};
use george::text::{self, Program, TextError};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};

const SEMANTIC_TYPES: &[&str] = &[
    "comment", "number", "variable", "operator", "keyword", "function", "label", "macro",
];
const SEMANTIC_MODIFIERS: &[&str] = &["declaration", "defaultLibrary"];
const DECLARATION: u32 = 1;
const LIBRARY: u32 = 2;

/// Where a symbol is written: a line, and a start and end counted in
/// characters, all from zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Span {
    line: usize,
    start: usize,
    end: usize,
}

struct Doc {
    lines: Vec<Vec<char>>,
    program: Result<Program, TextError>,
    /// One for each symbol of the program.
    spans: Vec<Span>,
    /// Comments, as spans.
    comments: Vec<Span>,
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_lowercase() || "θαβγλμν".contains(c)
}

/// How far the piece of text starting at `start` runs, as the reader would
/// take it before deciding which symbols it holds.
fn scan(line: &[char], start: usize) -> usize {
    let at = |k: usize| line.get(k).copied();
    let mut k = start;
    match at(k) {
        None => return start,
        Some(c) if is_word_char(c) => {
            while at(k).is_some_and(is_word_char) {
                k += 1;
            }
        }
        Some(c) if c.is_ascii_digit() || c == '.' => {
            while at(k).is_some_and(|c| c.is_ascii_digit() || c == '.') {
                k += 1;
            }
        }
        Some('(') => {
            while at(k).is_some_and(|c| c != ')') {
                k += 1;
            }
            k = (k + 1).min(line.len());
        }
        Some('R' | 'P') => {
            k += 1;
            for _ in 0..2 {
                if at(k) == Some('|') {
                    k += 1;
                }
            }
        }
        Some('|') => {
            k += 1;
            if at(k) == Some('|') {
                k += 1;
            }
        }
        Some(_) => k += 1,
    }
    k
}

impl Doc {
    fn new(text: &str) -> Doc {
        let lines: Vec<Vec<char>> = text.split('\n').map(|l| l.chars().collect()).collect();
        let program = text::parse(text);
        let mut spans = Vec::new();
        if let Ok(p) = &program {
            let starts: Vec<(usize, usize)> = p.positions.iter().map(|pos| (pos.line - 1, pos.col - 1)).collect();
            let mut k = 0;
            while k < starts.len() {
                let (line, start) = starts[k];
                // The digits of a numeral are given the position of the
                // first: each is one character.
                let same = starts[k..].iter().take_while(|&&s| s == (line, start)).count();
                if same > 1 {
                    spans.extend((0..same).map(|n| Span {
                        line,
                        start: start + n,
                        end: start + n + 1,
                    }));
                    k += same;
                    continue;
                }
                let mut end = scan(&lines[line], start).max(start + 1);
                if let Some(&(next_line, next_start)) = starts.get(k + 1) {
                    if next_line == line && next_start > start {
                        end = end.min(next_start);
                    }
                }
                spans.push(Span { line, start, end });
                k += 1;
            }
        }
        let comments = lines
            .iter()
            .enumerate()
            .filter_map(|(line, chars)| {
                let start = chars.iter().position(|&c| c == '#')?;
                let end = chars.len() - usize::from(chars.last() == Some(&'\r'));
                Some(Span { line, start, end })
            })
            .collect();
        Doc {
            lines,
            program,
            spans,
            comments,
        }
    }

    fn symbols(&self) -> &[Sym] {
        self.program.as_ref().map(|p| p.symbols.as_slice()).unwrap_or(&[])
    }

    /// A character column as the protocol counts it, in UTF-16 units.
    fn utf16(&self, line: usize, column: usize) -> usize {
        self.lines
            .get(line)
            .map_or(0, |l| l.iter().take(column).map(|c| c.len_utf16()).sum())
    }

    fn range(&self, span: Span) -> Value {
        json!({
            "start": { "line": span.line, "character": self.utf16(span.line, span.start) },
            "end": { "line": span.line, "character": self.utf16(span.line, span.end) },
        })
    }

    /// From one span to another, which may be on a later line.
    fn range_between(&self, from: Span, to: Span) -> Value {
        json!({
            "start": { "line": from.line, "character": self.utf16(from.line, from.start) },
            "end": { "line": to.line, "character": self.utf16(to.line, to.end) },
        })
    }

    /// A protocol position as a line and a character column.
    fn place(&self, position: &Value) -> (usize, usize) {
        let line = position["line"].as_u64().unwrap_or(0) as usize;
        let wanted = position["character"].as_u64().unwrap_or(0) as usize;
        let mut units = 0;
        let mut column = 0;
        for c in self.lines.get(line).into_iter().flatten() {
            if units >= wanted {
                break;
            }
            units += c.len_utf16();
            column += 1;
        }
        (line, column)
    }

    /// The symbol under a position, or the one just before it.
    fn symbol_at(&self, position: &Value) -> Option<usize> {
        let (line, column) = self.place(position);
        let inside = self
            .spans
            .iter()
            .position(|s| s.line == line && s.start <= column && column < s.end);
        inside.or_else(|| self.spans.iter().position(|s| s.line == line && s.end == column))
    }

    fn written(&self, k: usize) -> String {
        let s = self.spans[k];
        self.lines[s.line][s.start..s.end].iter().collect()
    }

    /// The numeral sequences: the runs of numeral symbols that the machine
    /// reads as one number. The numeral after an asterisk is a mark and
    /// belongs to none.
    fn numerals(&self) -> Vec<(usize, usize)> {
        let s = self.symbols();
        let mut runs = Vec::new();
        let mut k = 0;
        while k < s.len() {
            let is_mark = k > 0 && s[k - 1] == Sym::Star;
            if s[k].is_numeral() && !is_mark {
                let first = k;
                while k + 1 < s.len() && s[k + 1].is_numeral() {
                    k += 1;
                }
                runs.push((first, k));
            }
            k += 1;
        }
        runs
    }

    fn numeral_run(&self, k: usize) -> Option<(usize, usize)> {
        self.numerals().into_iter().find(|&(a, b)| a <= k && k <= b)
    }

    /// The number a run of numerals stands for, as the machine reads it.
    fn numeral_text(&self, (first, last): (usize, usize)) -> String {
        self.symbols()[first..=last]
            .iter()
            .map(|s| match s {
                Sym::Digit(d) if *d < 10 => char::from(b'0' + d),
                Sym::Digit(_) => '?',
                _ => '.',
            })
            .collect()
    }

    /// The marks set in the program: mark number and the index of its asterisk.
    fn marks(&self) -> Vec<(u8, usize)> {
        let s = self.symbols();
        (0..s.len())
            .filter(|&k| s[k] == Sym::Star && (k == 0 || s[k - 1] != Sym::Star))
            .filter_map(|k| match s.get(k + 1) {
                Some(Sym::Digit(n) | Sym::High(n)) => Some((*n, k)),
                _ => None,
            })
            .collect()
    }

    /// For a skip (↓ or ↑) written straight after a number, the number, the
    /// mark it means and where its numerals are.
    fn literal_skip(&self, k: usize) -> Option<(String, u8, (usize, usize))> {
        let s = self.symbols();
        if !matches!(s.get(k), Some(Sym::Down | Sym::Up)) || k == 0 {
            return None;
        }
        let run = self.numeral_run(k - 1).filter(|&(_, last)| last == k - 1)?;
        let text = self.numeral_text(run);
        let value: f64 = if text == "." { 0.0 } else { text.parse().ok()? };
        Some((text, (value.floor() as i64).rem_euclid(32) as u8, run))
    }

    fn mark_span(&self, star: usize) -> Span {
        let (a, b) = (self.spans[star], self.spans[star + 1]);
        if a.line == b.line {
            Span {
                line: a.line,
                start: a.start,
                end: b.end,
            }
        } else {
            a
        }
    }
}

fn diagnostic(range: Value, severity: u8, code: &str, message: String) -> Value {
    json!({ "range": range, "severity": severity, "code": code, "source": "george", "message": message })
}

/// The problems in a document: the reader's own error, or the things that
/// are easy to get wrong in a program it accepts.
fn diagnostics(doc: &Doc) -> Vec<Value> {
    let program = match &doc.program {
        Err(e) => {
            let (line, start) = (e.pos.line - 1, e.pos.col - 1);
            let end = doc.lines.get(line).map_or(start + 1, |l| scan(l, start).max(start + 1));
            return vec![diagnostic(
                doc.range(Span { line, start, end }),
                1,
                "text",
                e.message.clone(),
            )];
        }
        Ok(p) => p,
    };
    let s = &program.symbols;
    let mut out = Vec::new();
    if s.len() > SYMBOL_CAPACITY {
        let k = SYMBOL_CAPACITY;
        out.push(diagnostic(
            doc.range_between(doc.spans[k], doc.spans[s.len() - 1]),
            1,
            "too-many-symbols",
            format!(
                "{} symbols: there is room for {SYMBOL_CAPACITY}, so the machine stops with {} ({}). Sectionalise the program.",
                s.len(),
                machine::TOO_MANY_SYMBOLS.indication,
                machine::TOO_MANY_SYMBOLS.cause
            ),
        ));
    }
    // "When two numbers must be written consecutively they should be
    // separated by a comma."
    for run in doc.numerals() {
        let (first, last) = run;
        let joined = (first..last).any(|k| {
            let (a, b) = (doc.spans[k], doc.spans[k + 1]);
            a.line != b.line || a.end != b.start
        });
        if joined {
            let pieces: Vec<String> = (first..=last).map(|k| doc.written(k)).collect();
            let mut written = String::new();
            for k in first..=last {
                if k > first {
                    let (a, b) = (doc.spans[k - 1], doc.spans[k]);
                    if a.line != b.line || a.end != b.start {
                        written.push(' ');
                    }
                }
                written.push_str(&pieces[k - first]);
            }
            out.push(diagnostic(
                doc.range_between(doc.spans[first], doc.spans[last]),
                2,
                "numerals-run-together",
                format!(
                    "\"{written}\" is read as one number, {}: numbers written one after another should be separated by a comma (section 1.2)",
                    doc.numeral_text(run)
                ),
            ));
        }
    }
    // A mark set twice.
    let marks = doc.marks();
    for (n, (mark, star)) in marks.iter().enumerate() {
        if marks[..n].iter().any(|(m, _)| m == mark) {
            out.push(diagnostic(
                doc.range(doc.mark_span(*star)),
                2,
                "mark-twice",
                format!(
                    "mark {mark} is set twice: the machine stops with {} (single shot: {})",
                    machine::MARK_TWICE.indication,
                    machine::MARK_TWICE.resume.unwrap_or("")
                ),
            ));
        }
    }
    // A skip to a mark that is nowhere.
    for k in 0..s.len() {
        let Some((text, mark, (first, _))) = doc.literal_skip(k) else {
            continue;
        };
        let range = doc.range_between(doc.spans[first], doc.spans[k]);
        if !marks.iter().any(|(m, _)| *m == mark) {
            let taken = if text.parse::<f64>().is_ok_and(|v| v >= 32.0) {
                format!("{text} is mark {mark}, since skip numbers are taken modulo 32, and there is no mark * {mark}")
            } else {
                format!("there is no mark * {mark} in the program")
            };
            out.push(diagnostic(
                range,
                2,
                "no-mark",
                format!(
                    "{taken}: if this skip is taken the machine stops with {}",
                    machine::NO_MARK.indication
                ),
            ));
        } else if text.parse::<f64>().is_ok_and(|v| v >= 32.0 || v.fract() != 0.0) {
            out.push(diagnostic(
                range,
                3,
                "skip-number",
                format!("{text} as a skip number is mark {mark}: skip numbers are whole numbers taken modulo 32"),
            ));
        }
    }
    out
}

fn letter_number(sym: Sym) -> Option<u8> {
    match sym {
        Sym::Var(v) | Sym::Name(v) => Some(v),
        _ => None,
    }
}

fn store_of(v: u8) -> String {
    let greek = GREEK_ASCII
        .iter()
        .find(|(_, n)| *n == v)
        .map(|(name, _)| format!(" (written `{name}` in ASCII)"));
    format!(
        "letter {} of the alphabet, number {v}{}",
        LETTERS[v as usize],
        greek.unwrap_or_default()
    )
}

fn codes(doc: &Doc, first: usize, last: usize) -> String {
    doc.symbols()[first..=last]
        .iter()
        .map(|s| {
            let (r, c) = s.code();
            format!("{r}/{c}")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Hover help for the symbol under the cursor.
fn hover(doc: &Doc, position: &Value) -> Value {
    let Some(k) = doc.symbol_at(position) else {
        return Value::Null;
    };
    let s = doc.symbols();
    let sym = s[k];
    let mut span = doc.spans[k];
    let mut range = doc.range(span);
    let help = help::of(sym);
    let written = doc.written(k);
    let printed = sym.text();
    let mut parts: Vec<String> = Vec::new();
    let is_mark = k > 0 && s[k - 1] == Sym::Star && sym.is_numeral();

    if is_mark || sym == Sym::Star {
        let star = if sym == Sym::Star { k } else { k - 1 };
        span = doc.mark_span(star);
        range = doc.range(span);
        let n = match s.get(star + 1) {
            Some(Sym::Digit(n) | Sym::High(n)) => *n,
            _ => 0,
        };
        let calls = (0..s.len())
            .filter(|&j| doc.literal_skip(j).is_some_and(|(_, m, _)| m == n))
            .count();
        parts.push(format!(
            "**`* {n}`** mark {n}: the head of a subroutine, or a place marker"
        ));
        parts.push(format!(
            "mark · codes {} · `{n} ↓` calls it as a subroutine, `{n} ↑` skips to it{}",
            codes(doc, star, star + 1),
            match calls {
                0 => String::new(),
                1 => " · skipped to once here".into(),
                c => format!(" · skipped to {c} times here"),
            }
        ));
        parts.push("The number of a subroutine, as written at the beginning of the subroutine itself, must be represented by a single numeral symbol: 0 to 9, or the symbols 10 to 31 of the coding table.".into());
        parts.push(format!(
            "> {}\n>\n> — manual, section {}",
            help::of(Sym::Star).manual,
            help::of(Sym::Star).section
        ));
        return json!({ "contents": { "kind": "markdown", "value": parts.join("\n\n") }, "range": range });
    }

    if sym.is_numeral() {
        let run = doc.numeral_run(k).unwrap_or((k, k));
        let text = doc.numeral_text(run);
        range = doc.range_between(doc.spans[run.0], doc.spans[run.1]);
        parts.push(format!("**`{text}`** numeral: puts {text} in the next cell"));
        parts.push(format!(
            "number · {} symbol{} · codes {} (a decimal point is any symbol of column 3)",
            run.1 - run.0 + 1,
            if run.1 > run.0 { "s" } else { "" },
            codes(doc, run.0, run.1)
        ));
        if let Some((_, mark, _)) = (run.1 + 1 < s.len()).then(|| doc.literal_skip(run.1 + 1)).flatten() {
            parts.push(format!("Used here as a skip number: mark {mark}."));
        }
    } else {
        let (r, c) = sym.code();
        let title = match letter_number(sym) {
            Some(_) => {
                let bar = k > 0 && matches!(s[k - 1], Sym::Bar | Sym::DoubleBar);
                let suffixed = match (k > 0).then(|| s[k - 1]) {
                    Some(Sym::Bar) => " in the vector store, suffixed by the bar sign before it",
                    Some(Sym::DoubleBar) => " in the matrix store, suffixed by the double bar before it",
                    _ => "",
                };
                match sym {
                    Sym::Name(_) if bar => format!("name{suffixed}: store the last cell, leaving it there"),
                    Sym::Name(_) if k > 0 && s[k - 1] == Sym::Rep => {
                        "name: the controlled variable of the repetition".into()
                    }
                    Sym::Name(_)
                        if k > 0
                            && matches!(
                                s[k - 1],
                                Sym::ReadVector | Sym::ReadMatrix | Sym::PunchVector | Sym::PunchMatrix
                            ) =>
                    {
                        "name: the block read or punched".into()
                    }
                    Sym::Name(_) => help.title.into(),
                    _ if bar => format!("variable{suffixed}"),
                    _ => "variable: puts its value in the next cell".into(),
                }
            }
            None => help.title.into(),
        };
        parts.push(format!("**`{printed}`** {title}"));
        let mut line = format!("{} · code {r}/{c}", help.kind);
        if let Some(v) = letter_number(sym) {
            line += &format!(" · {}", store_of(v));
        }
        parts.push(line);
        let effect = match sym {
            Sym::Var(_) if k > 0 && s[k - 1] == Sym::Bar => "… j → … aⱼ",
            Sym::Var(_) if k > 0 && s[k - 1] == Sym::DoubleBar => "… i j → … aᵢⱼ",
            Sym::Name(_) if k > 0 && s[k - 1] == Sym::Bar => "… v j → … v   (and aⱼ = v)",
            Sym::Name(_) if k > 0 && s[k - 1] == Sym::DoubleBar => "… v i j → … v   (and aᵢⱼ = v)",
            Sym::Name(_)
                if k > 0
                    && matches!(
                        s[k - 1],
                        Sym::Rep | Sym::ReadVector | Sym::ReadMatrix | Sym::PunchVector | Sym::PunchMatrix
                    ) =>
            {
                help::of(s[k - 1]).effect
            }
            _ => help.effect,
        };
        parts.push(format!("Accumulator: `{effect}`"));
    }
    if !sym.is_numeral() && written != printed && !matches!(sym, Sym::Var(_) | Sym::Name(_)) {
        parts.push(format!("Written here as `{written}`; the manual prints `{printed}`."));
    } else if !help.spellings.is_empty() {
        parts.push(format!("Also written {}.", help.spellings));
    }
    if matches!(sym, Sym::Down | Sym::Up) {
        if let Some((text, mark, _)) = doc.literal_skip(k) {
            let found = doc.marks().iter().any(|(m, _)| *m == mark);
            parts.push(format!(
                "Here: to mark {mark}{}{}.",
                if text.parse::<f64>().is_ok_and(|v| v >= 32.0) {
                    format!(" ({text} modulo 32)")
                } else {
                    String::new()
                },
                if found { "" } else { ", which is not in the program" }
            ));
        }
    }
    parts.push(format!("> {}\n>\n> — manual, section {}", help.manual, help.section));
    if let Some(stop) = help.stop {
        parts.push(format!(
            "Can stop with **{}**: {}{}.",
            stop.indication,
            stop.cause,
            stop.resume.map(|r| format!(" (single shot: {r})")).unwrap_or_default()
        ));
    }
    json!({ "contents": { "kind": "markdown", "value": parts.join("\n\n") }, "range": range })
}

/// From a skip, or the number before it, to the mark it goes to.
fn definition(doc: &Doc, uri: &str, position: &Value) -> Value {
    let Some(k) = doc.symbol_at(position) else {
        return Value::Null;
    };
    let s = doc.symbols();
    let skip = if matches!(s[k], Sym::Down | Sym::Up) {
        Some(k)
    } else {
        doc.numeral_run(k).map(|(_, last)| last + 1).filter(|&j| j < s.len())
    };
    let Some((_, mark, _)) = skip.and_then(|j| doc.literal_skip(j)) else {
        return Value::Null;
    };
    match doc.marks().iter().find(|(m, _)| *m == mark) {
        Some((_, star)) => json!({ "uri": uri, "range": doc.range(doc.mark_span(*star)) }),
        None => Value::Null,
    }
}

/// The marks, as an outline of the program.
fn outline(doc: &Doc) -> Value {
    let s = doc.symbols();
    let mut out = Vec::new();
    for (mark, star) in doc.marks() {
        let span = doc.mark_span(star);
        // A subroutine is a mark straight after a bracket, or the first mark
        // after the end of the main program.
        let subroutine = star > 0 && s[star - 1] == Sym::Bracket;
        let line: String = doc.lines[span.line].iter().collect();
        let detail = line.split('#').next().unwrap_or("").trim().to_string();
        out.push(json!({
            "name": if subroutine { format!("subroutine {mark}") } else { format!("mark {mark}") },
            "detail": detail,
            "kind": if subroutine { 12 } else { 20 },
            "range": doc.range(span),
            "selectionRange": doc.range(span),
        }));
    }
    json!(out)
}

fn completion() -> Value {
    let mut items: Vec<Value> = help::WORDS
        .iter()
        .map(|(word, sym)| {
            let help = help::of(*sym);
            json!({
                "label": word,
                "kind": 14,
                "detail": format!("{} · {} · `{}`", sym.text(), help.title, help.effect),
                "documentation": { "kind": "markdown", "value": format!("{}\n\n> {}", help.kind, help.manual) },
            })
        })
        .collect();
    for (name, v) in GREEK_ASCII {
        items.push(json!({
            "label": name,
            "kind": 6,
            "detail": format!("{}: the variable {}", LETTERS[v as usize], LETTERS[v as usize]),
        }));
    }
    json!(items)
}

/// Run, trace and deck, above the first line. The commands belong to the
/// client: the VS Code extension and the web page provide them.
fn code_lenses(doc: &Doc, uri: &str) -> Value {
    if doc.program.is_err() {
        return json!([]);
    }
    let range = json!({ "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } });
    let n = doc.symbols().len();
    json!([
        { "range": range, "command": { "title": "▶ Run", "command": "george.run", "arguments": [uri] } },
        { "range": range, "command": { "title": "Trace", "command": "george.runTrace", "arguments": [uri] } },
        { "range": range, "command": { "title": format!("Deck: {n} symbols, {} cards", n.div_ceil(16).max(1)), "command": "george.deck", "arguments": [uri] } },
    ])
}

fn semantic(sym: Sym, after_star: bool) -> (&'static str, u32) {
    use Sym::*;
    match sym {
        _ if after_star => ("label", 0),
        Star => ("label", 0),
        Digit(_) | High(_) => ("number", 0),
        Var(_) => ("variable", 0),
        Name(_) => ("variable", DECLARATION),
        Neg | Mod | Log | Exp | Sqrt | Sin | Cos | Pow | Rem | Max | Dup | Rev => ("function", LIBRARY),
        Rep | Bracket | Down | Up | Wait => ("keyword", 0),
        Read | Punch | ReadVector | PunchVector | ReadMatrix | PunchMatrix | Keys => ("macro", 0),
        _ => ("operator", 0),
    }
}

/// Semantic tokens, in the protocol's five-numbers-per-token encoding.
fn semantic_tokens(doc: &Doc) -> Vec<u32> {
    let type_index = |name: &str| SEMANTIC_TYPES.iter().position(|&t| t == name).unwrap() as u32;
    let s = doc.symbols();
    let mut tokens: Vec<(Span, &str, u32)> = doc.comments.iter().map(|&c| (c, "comment", 0)).collect();
    for (k, &span) in doc.spans.iter().enumerate() {
        let (kind, modifiers) = semantic(s[k], k > 0 && s[k - 1] == Sym::Star);
        tokens.push((span, kind, modifiers));
    }
    tokens.sort_by_key(|(s, _, _)| (s.line, s.start));
    let mut data = Vec::new();
    let (mut last_line, mut last_start) = (0usize, 0usize);
    for (span, kind, modifiers) in tokens {
        let start = doc.utf16(span.line, span.start);
        let length = doc.utf16(span.line, span.end) - start;
        if length == 0 {
            continue;
        }
        let delta_line = span.line - last_line;
        let delta_start = if delta_line == 0 { start - last_start } else { start };
        data.extend([
            delta_line as u32,
            delta_start as u32,
            length as u32,
            type_index(kind),
            modifiers,
        ]);
        (last_line, last_start) = (span.line, start);
    }
    data
}

/// Every symbol of the program, in order, with where it is written and its
/// code: what the web page uses to show which symbol a trace line obeyed.
fn symbol_list(doc: &Doc) -> Value {
    if doc.program.is_err() {
        return Value::Null;
    }
    let list: Vec<Value> = doc
        .symbols()
        .iter()
        .enumerate()
        .map(|(k, sym)| {
            let (r, c) = sym.code();
            json!({ "range": doc.range(doc.spans[k]), "text": doc.written(k), "printed": sym.text(), "code": format!("{r}/{c}") })
        })
        .collect();
    json!({ "symbols": list, "cells": ACCUMULATOR_CELLS })
}

struct Server {
    docs: BTreeMap<String, Doc>,
    shutdown: bool,
    exit: bool,
}

impl Server {
    fn new() -> Self {
        Server {
            docs: BTreeMap::new(),
            shutdown: false,
            exit: false,
        }
    }

    /// Handles one incoming message and returns the messages to send back.
    fn handle(&mut self, message: &Value) -> Vec<Value> {
        let method = message["method"].as_str().unwrap_or("");
        let params = &message["params"];
        let uri = params["textDocument"]["uri"].as_str().unwrap_or("").to_string();

        let Some(id) = message.get("id").filter(|_| !method.is_empty()) else {
            // A notification, or a reply to something we never ask.
            return match method {
                "textDocument/didOpen" => self.open(uri, params["textDocument"]["text"].as_str().unwrap_or("")),
                "textDocument/didChange" => {
                    // Full sync: the last change holds the whole text.
                    let changes = params["contentChanges"].as_array();
                    match changes.and_then(|c| c.last()).and_then(|c| c["text"].as_str()) {
                        Some(text) => self.open(uri, text),
                        None => vec![],
                    }
                }
                "textDocument/didClose" => {
                    self.docs.remove(&uri);
                    vec![json!({ "jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
                        "params": { "uri": uri, "diagnostics": [] } })]
                }
                "exit" => {
                    self.exit = true;
                    vec![]
                }
                _ => vec![],
            };
        };

        let reply = |result: Value| vec![json!({ "jsonrpc": "2.0", "id": id, "result": result })];
        let fail = |code: i32, text: &str| {
            vec![json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": text } })]
        };

        match method {
            "initialize" => return reply(self.capabilities()),
            "shutdown" => {
                self.shutdown = true;
                return reply(Value::Null);
            }
            _ => {}
        }
        if !method.starts_with("textDocument/") && !method.starts_with("george/") {
            return fail(-32601, &format!("method not supported: {method}"));
        }
        let Some(doc) = self.docs.get(&uri) else {
            return reply(Value::Null);
        };
        let position = &params["position"];
        match method {
            "textDocument/hover" => reply(hover(doc, position)),
            "textDocument/definition" => reply(definition(doc, &uri, position)),
            "textDocument/documentSymbol" => reply(outline(doc)),
            "textDocument/completion" => reply(completion()),
            "textDocument/codeLens" => reply(code_lenses(doc, &uri)),
            "textDocument/semanticTokens/full" => reply(json!({ "data": semantic_tokens(doc) })),
            "george/symbols" => reply(symbol_list(doc)),
            _ => fail(-32601, &format!("method not supported: {method}")),
        }
    }

    fn capabilities(&self) -> Value {
        json!({
            "serverInfo": { "name": "george-lsp", "version": env!("CARGO_PKG_VERSION") },
            "capabilities": {
                "positionEncoding": "utf-16",
                "textDocumentSync": { "openClose": true, "change": 1 },
                "hoverProvider": true,
                "definitionProvider": true,
                "documentSymbolProvider": true,
                "completionProvider": {},
                "codeLensProvider": { "resolveProvider": false },
                "semanticTokensProvider": {
                    "legend": { "tokenTypes": SEMANTIC_TYPES, "tokenModifiers": SEMANTIC_MODIFIERS },
                    "full": true,
                },
            },
        })
    }

    fn open(&mut self, uri: String, text: &str) -> Vec<Value> {
        let doc = Doc::new(text);
        let found = diagnostics(&doc);
        self.docs.insert(uri.clone(), doc);
        vec![json!({ "jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
            "params": { "uri": uri, "diagnostics": found } })]
    }
}

fn read_message(input: &mut impl BufRead) -> io::Result<Option<Value>> {
    let mut length = None;
    loop {
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        let line = line.trim_end();
        if line.is_empty() {
            if length.is_some() {
                break;
            }
            continue;
        }
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            length = value.trim().parse::<usize>().ok();
        }
    }
    let mut body = vec![0; length.unwrap()];
    input.read_exact(&mut body)?;
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn main() -> io::Result<()> {
    if std::env::args().any(|arg| arg == "--version") {
        println!("george-lsp {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut output = io::stdout().lock();
    let mut server = Server::new();
    while let Some(message) = read_message(&mut input)? {
        for reply in server.handle(&message) {
            let body = reply.to_string();
            write!(output, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
        }
        output.flush()?;
        if server.exit {
            break;
        }
    }
    std::process::exit(if server.shutdown { 0 } else { 1 });
}

#[cfg(test)]
mod tests {
    use super::*;

    const URI: &str = "file:///test.george";
    const PROGRAM: &str = "\
# Section 1.7: the square root of the sum of two squares.
R R 6 ↓ (P) ;                       # main program
]                                   # end of main program
* 6 dup mul rev dup × + .5 pow ]    # subroutine
";

    fn server_with(text: &str) -> (Server, Vec<Value>) {
        let mut server = Server::new();
        let out = server.handle(&json!({ "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": { "textDocument": { "uri": URI, "languageId": "george", "version": 1, "text": text } } }));
        (server, out)
    }

    fn ask(server: &mut Server, method: &str, line: u32, character: u32) -> Value {
        let params = json!({ "textDocument": { "uri": URI }, "position": { "line": line, "character": character } });
        let mut out = server.handle(&json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }));
        assert_eq!(out.len(), 1);
        out.remove(0)["result"].take()
    }

    fn hover_text(server: &mut Server, line: u32, character: u32) -> String {
        ask(server, "textDocument/hover", line, character)["contents"]["value"]
            .as_str()
            .unwrap_or("")
            .to_string()
    }

    fn problems(text: &str) -> Vec<Value> {
        let (_, out) = server_with(text);
        out[0]["params"]["diagnostics"].as_array().unwrap().clone()
    }

    #[test]
    fn a_good_program_has_no_problems() {
        assert!(problems(PROGRAM).is_empty());
    }

    #[test]
    fn every_example_is_read_as_the_interpreter_reads_it() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples");
        let mut n = 0;
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "george") {
                let text = std::fs::read_to_string(&path).unwrap();
                let errors: Vec<_> = problems(&text).into_iter().filter(|d| d["severity"] == 1).collect();
                assert!(errors.is_empty(), "{}: {errors:?}", path.display());
                // Each symbol's span starts where the reader says it is.
                let doc = Doc::new(&text);
                let program = text::parse(&text).unwrap();
                for (span, pos) in doc.spans.iter().zip(&program.positions) {
                    assert_eq!(span.line + 1, pos.line);
                    assert!(span.start + 1 >= pos.col && span.start < pos.col + 9);
                    assert!(span.end > span.start);
                }
                n += 1;
            }
        }
        assert!(n >= 15);
    }

    #[test]
    fn a_mistake_is_placed_where_the_reader_found_it() {
        let found = problems("a b + (o) ;\n");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0]["severity"], 1);
        assert_eq!(found[0]["range"]["start"], json!({ "line": 0, "character": 6 }));
        assert_eq!(found[0]["range"]["end"], json!({ "line": 0, "character": 9 }));
        assert!(found[0]["message"].as_str().unwrap().contains("is not a name"));
        let found = problems("1, 2 rep j ]");
        assert!(found[0]["message"]
            .as_str()
            .unwrap()
            .contains("rep must be followed by a name"));
    }

    #[test]
    fn numerals_that_run_together_are_pointed_out() {
        let found = problems("3, 4 14 ↓ ]\n* 30 (P) ]");
        assert_eq!(found.len(), 2, "{found:?}");
        assert_eq!(found[0]["code"], "numerals-run-together");
        assert!(found[0]["message"].as_str().unwrap().contains("414"));
        assert_eq!(found[0]["range"]["start"]["character"], 3);
        assert_eq!(found[0]["range"]["end"]["character"], 7);
        assert_eq!(found[1]["code"], "skip-number");
        assert!(problems("4, 3, 16 × + (P)").is_empty());
    }

    #[test]
    fn a_skip_to_a_missing_mark_is_a_warning() {
        let found = problems("* 0 R R + (P) ; 1 ↑");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0]["code"], "no-mark");
        assert_eq!(found[0]["severity"], 2);
        assert!(found[0]["message"].as_str().unwrap().contains("7 25-29"));
        assert!(problems("* 0 R R + (P) ; 0 ↑").is_empty());
        let twice = problems("* 3 * 3 ]");
        assert_eq!(twice[0]["code"], "mark-twice");
    }

    #[test]
    fn hover_tells_what_a_symbol_does() {
        let (mut server, _) = server_with(PROGRAM);
        let down = hover_text(&mut server, 1, 6);
        assert!(down.starts_with("**`↓`** skip to subroutine"), "{down}");
        assert!(down.contains("code 6/1"));
        assert!(down.contains("Here: to mark 6."));
        let mul = hover_text(&mut server, 3, 9);
        assert!(mul.contains("**`×`** multiplication"), "{mul}");
        assert!(mul.contains("diadic operator · code 7/0"));
        assert!(mul.contains("`… a b → … a×b`"));
        assert!(mul.contains("Written here as `mul`"));
        let punch = hover_text(&mut server, 1, 9);
        assert!(punch.contains("**`(P)`** punch the last cell"), "{punch}");
        assert!(punch.contains("does not \"cancel\" the cell"));
        let half = hover_text(&mut server, 3, 26);
        assert!(half.starts_with("**`.5`** numeral"), "{half}");
        assert!(half.contains("codes 0/3 5/2"));
        let mark = hover_text(&mut server, 3, 2);
        assert!(mark.starts_with("**`* 6`** mark 6"), "{mark}");
        assert!(mark.contains("skipped to once"));
        let read = hover_text(&mut server, 1, 0);
        assert!(
            read.contains("Can stop with **1 11-6x, 1 7-16x**: data card wanted"),
            "{read}"
        );
        assert_eq!(ask(&mut server, "textDocument/hover", 0, 5), Value::Null);
    }

    #[test]
    fn hover_on_letters_names_and_suffixes() {
        let (mut server, _) = server_with("1, 20 rep (j) j|a alpha + j|(β) ; ]");
        let name = hover_text(&mut server, 0, 11);
        assert!(name.contains("controlled variable"), "{name}");
        let var = hover_text(&mut server, 0, 16);
        assert!(var.contains("vector store"), "{var}");
        assert!(var.contains("code 0/4"));
        let alpha = hover_text(&mut server, 0, 19);
        assert!(alpha.contains("**`α`** variable"), "{alpha}");
        assert!(alpha.contains("code 10/5"));
        assert!(alpha.contains("written `alpha` in ASCII"));
        let beta = hover_text(&mut server, 0, 29);
        assert!(beta.contains("**`(β)`** name in the vector store"), "{beta}");
        assert!(beta.contains("code 11/7"));
    }

    #[test]
    fn definition_goes_from_a_skip_to_its_mark() {
        let (mut server, _) = server_with(PROGRAM);
        let found = ask(&mut server, "textDocument/definition", 1, 4);
        assert_eq!(found["range"]["start"], json!({ "line": 3, "character": 0 }));
        assert_eq!(found["range"]["end"], json!({ "line": 3, "character": 3 }));
        let outline = ask(&mut server, "textDocument/documentSymbol", 0, 0);
        assert_eq!(outline[0]["name"], "subroutine 6");
    }

    #[test]
    fn semantic_tokens_and_the_symbol_list() {
        let (mut server, _) = server_with(PROGRAM);
        let data = ask(&mut server, "textDocument/semanticTokens/full", 0, 0)["data"]
            .as_array()
            .unwrap()
            .clone();
        assert_eq!(data.len() % 5, 0);
        // the comment, then R (macro, 7) on the next line
        assert_eq!(data[..5], [json!(0), json!(0), json!(57), json!(0), json!(0)]);
        assert_eq!(data[5..10], [json!(1), json!(0), json!(1), json!(7), json!(0)]);
        let list = ask(&mut server, "george/symbols", 0, 0);
        let symbols = list["symbols"].as_array().unwrap();
        assert_eq!(symbols.len(), text::parse(PROGRAM).unwrap().symbols.len());
        assert_eq!(symbols[2]["text"], "6");
        assert_eq!(symbols[3]["printed"], "↓");
        assert_eq!(symbols[3]["code"], "6/1");
        assert_eq!(list["cells"], 12);
        let lenses = ask(&mut server, "textDocument/codeLens", 0, 0);
        assert_eq!(lenses[0]["command"]["command"], "george.run");
        assert_eq!(lenses[0]["command"]["arguments"][0], URI);
    }

    #[test]
    fn a_message_read_from_the_wire() {
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"shutdown"}"#;
        let wire = format!("Content-Length: {}\r\n\r\n{body}", body.len());
        let message = read_message(&mut wire.as_bytes()).unwrap().unwrap();
        assert_eq!(message["method"], "shutdown");
    }
}
