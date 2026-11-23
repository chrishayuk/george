//! Punched cards as DEUCE saw them.
//!
//! An 80-column Hollerith card has twelve rows, which pass the brushes and
//! knives in the order Y, X, 0, 1, ..., 9 (manual §2.6, p. 7). DEUCE reads and
//! punches only columns 21 to 52 (manual §2.6, p. 7; NS y 88 §11, sheet 20),
//! and a row is a word with P1 in column 21 and P32 in column 52 (NS y 88
//! §11, sheet 20: "P1 is in column 21, P2 in column 22 and so on, P32 being in
//! column 52"; manual §2.6: "binary numbers are always punched with their
//! least significant digit on the left"). A hole in column 54 stops the
//! reader after that row (manual §6.3, p. 39).
//!
//! The text form of a deck used here has one card per block:
//!
//! ```text
//! card                     # optional; starts a new card
//! Y:  .                    # a blank row (the label "Y:" is optional)
//! X:  2 0 2 1 26 25 X      # an instruction: NIS S D C W [(J)] T [go]
//! 0:  = 5P1 + 12P5         # a constant
//! 1:  |10100000000000000000000000000000|   # 32 digits, column 21 first
//! 9:  . /54                # "/54" marks a hole in column 54
//! ```
//!
//! A card has at most 12 rows; missing rows at the end are blank. A line
//! `card` or a blank line ends the current card.

use super::word::{parse_constant, Instr};

pub const ROW_NAMES: [&str; 12] = ["Y", "X", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Card {
    /// Rows Y, X, 0-9: columns 21-52 as words (column 21 is P1).
    pub rows: [u32; 12],
    /// Bit r set: row r has a hole in column 54.
    pub col54: u16,
}

impl Card {
    pub fn blank() -> Card {
        Card::default()
    }

    /// The 80 columns of row `r` as characters: '1' for a hole, '0' for none.
    pub fn row_columns(&self, r: usize) -> String {
        let mut s = String::with_capacity(80);
        for col in 1..=80 {
            let hole = match col {
                21..=52 => self.rows[r] >> (col - 21) & 1 == 1,
                54 => self.col54 >> r & 1 == 1,
                _ => false,
            };
            s.push(if hole { '1' } else { '0' });
        }
        s
    }
}

/// Parse one row description (without its optional label).
pub fn parse_row(text: &str) -> Result<(u32, bool), String> {
    let mut t = text.trim();
    let mut col54 = false;
    if let Some(r) = t.strip_suffix("/54") {
        col54 = true;
        t = r.trim();
    }
    if t.is_empty() || t == "." || t == "-" || t.eq_ignore_ascii_case("blank") {
        return Ok((0, col54));
    }
    if let Some(c) = t.strip_prefix('=') {
        return Ok((parse_constant(c)?, col54));
    }
    if let Some(b) = t.strip_prefix('|') {
        let b = b.strip_suffix('|').ok_or("binary row must be written |...|")?;
        let digits: Vec<char> = b.chars().filter(|c| !c.is_whitespace()).collect();
        if digits.len() != 32 {
            return Err(format!("binary row needs 32 digits, got {}", digits.len()));
        }
        let mut w = 0u32;
        for (i, c) in digits.iter().enumerate() {
            match c {
                '1' | '*' | '#' => w |= 1 << i,
                '0' | '.' => {}
                _ => return Err(format!("bad digit {c:?} in binary row")),
            }
        }
        return Ok((w, col54));
    }
    Ok((parse_instruction(t)?.encode(), col54))
}

/// Parse instruction fields `NIS S D C W [(J)] T [go]`. A field written `-`
/// or `.` is zero. W may be written `W(J)`. The go field is `1` (obey), `0`,
/// `x` or `X` (stopper); missing means obey.
pub fn parse_instruction(text: &str) -> Result<Instr, String> {
    // separate "9(1)8" style into tokens
    let spaced = text.replace('(', " ( ").replace(')', " ) ");
    let mut toks: Vec<&str> = spaced.split_whitespace().collect();
    let mut j = 0u8;
    if let Some(i) = toks.iter().position(|&t| t == "(") {
        if toks.get(i + 2) != Some(&")") {
            return Err(format!("bad bracketed field in {text:?}"));
        }
        j = field(toks[i + 1], 15)?;
        toks.drain(i..i + 3);
    }
    let mut go = true;
    if toks.len() == 7 {
        go = match toks[6] {
            "1" | "-" | "." => true,
            "0" | "x" | "X" => false,
            other => return Err(format!("go digit {other:?} in {text:?}")),
        };
        toks.pop();
    } else if toks.len() == 6 {
    } else {
        return Err(format!("expected NIS S D C W T [go], got {text:?}"));
    }
    let f = |i: usize, max: u8| field(toks[i], max);
    // NIS is written 8 for DL8 on some sheets; the word holds 0 (PM App. 2).
    let mut ins = Instr::new(f(0, 8)? % 8, f(1, 31)?, f(2, 31)?, f(3, 3)?, f(4, 31)?, f(5, 31)?, go);
    ins.j = j;
    Ok(ins)
}

fn field(tok: &str, max: u8) -> Result<u8, String> {
    if tok == "-" || tok == "." {
        return Ok(0);
    }
    let v: u8 = tok.parse().map_err(|_| format!("cannot read field {tok:?}"))?;
    if v > max {
        return Err(format!("field {v} out of range 0-{max}"));
    }
    Ok(v)
}

/// Parse a deck in the text form described in the module documentation.
pub fn parse_deck(src: &str) -> Result<Vec<Card>, String> {
    let mut cards = Vec::new();
    let mut cur: Option<(Card, usize)> = None;
    for (n, raw) in src.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        let lineno = n + 1;
        if line.is_empty() || line.eq_ignore_ascii_case("card") || line.starts_with("card ") {
            if let Some((c, _)) = cur.take() {
                cards.push(c);
            }
            if !line.is_empty() {
                cur = Some((Card::blank(), 0));
            }
            continue;
        }
        let (card, idx) = cur.get_or_insert((Card::blank(), 0));
        let mut body = line;
        if let Some((label, rest)) = line.split_once(':') {
            let label = label.trim();
            let r = ROW_NAMES
                .iter()
                .position(|&name| name.eq_ignore_ascii_case(label))
                .ok_or_else(|| format!("line {lineno}: unknown row {label:?}"))?;
            if r < *idx {
                return Err(format!("line {lineno}: row {label} out of order"));
            }
            *idx = r;
            body = rest;
        }
        if *idx >= 12 {
            return Err(format!("line {lineno}: more than 12 rows on a card"));
        }
        let (w, h) = parse_row(body).map_err(|e| format!("line {lineno}: {e}"))?;
        card.rows[*idx] = w;
        if h {
            card.col54 |= 1 << *idx;
        }
        *idx += 1;
    }
    if let Some((c, _)) = cur {
        cards.push(c);
    }
    Ok(cards)
}

/// Write a deck in the text form, one row per line, instructions decoded.
pub fn format_deck(cards: &[Card]) -> String {
    let mut out = String::new();
    for (i, c) in cards.iter().enumerate() {
        out.push_str(&format!("card {}\n", i + 1));
        for (r, &w) in c.rows.iter().enumerate() {
            let bits: String = (0..32).map(|b| if w >> b & 1 == 1 { '1' } else { '0' }).collect();
            out.push_str(&format!("{:>2}: |{}|", ROW_NAMES[r], bits));
            if c.col54 >> r & 1 == 1 {
                out.push_str(" /54");
            }
            out.push('\n');
        }
    }
    out
}

/// The standard triad of three cards that fills DL `n` (1-8) with `words`
/// by the initial-input method (manual §8.1, pp. 56-58): the first four rows
/// are blank, "N, 0-N, 1, 26, 25 X", "N, 0-N, 30, 31 X" and
/// "1, 0-N, 30, 31 X" (or, for the last triad, "M, 0-N, 30, m-1 X" to go to
/// the first instruction in DL M, minor cycle m); then minor cycles 0-31 on
/// the remaining 32 rows (NS y 88 §11, sheet 20).
pub fn triad(n: u8, words: &[u32; 32], enter: Option<(u8, u8)>) -> [Card; 3] {
    assert!((1..=8).contains(&n), "the simple triad fills DL1-8 only");
    let nis = n % 8; // DL8 is NIS 0
    let mut rows = [0u32; 36];
    rows[1] = Instr::new(nis, 0, n, 1, 26, 25, false).encode();
    rows[2] = Instr::new(nis, 0, n, 0, 30, 31, false).encode();
    rows[3] = match enter {
        None => Instr::new(1, 0, n, 0, 30, 31, false).encode(),
        Some((m_dl, m_mc)) => Instr::new(m_dl % 8, 0, n, 0, 30, (m_mc + 31) % 32, false).encode(),
    };
    rows[4..].copy_from_slice(words);
    let mut cards = [Card::blank(); 3];
    for (i, w) in rows.iter().enumerate() {
        cards[i / 12].rows[i % 12] = *w;
    }
    cards
}

/// The initial card of the manual §8.3, p. 61: a leading blank row, clear
/// TCB, and bring the store into step so that the first word of the next
/// triad is stored in an even minor cycle.
pub fn initial_card() -> Card {
    let mut c = Card::blank();
    c.rows[2] = Instr::new(0, 4, 24, 0, 0, 0, false).encode(); // 0 row: 4-24, 0, 0 X
    c.rows[4] = Instr::new(0, 31, 21, 1, 0, 1, false).encode(); // 2 row: 31-21, 1, 0, 1 X
    c.rows[6] = Instr::new(0, 27, 22, 0, 0, 0, false).encode(); // 4 row: 27-22, 0, 0 X
    c.rows[8] = Instr::new(0, 21, 28, 1, 0, 1, false).encode(); // 6 row: 21-28, 1, 0, 1 X
    c.rows[10] = Instr::new(0, 30, 21, 1, 0, 1, false).encode(); // 8 row: 30-21, 1, 0, 1 X
    c
}
