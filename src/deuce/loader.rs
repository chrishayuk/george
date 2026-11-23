//! A text form of DEUCE store images, which also reads the transcribed UTECOM
//! coding sheets (`sources/coding-sheets-1958/transcription/sheet-NN.txt`)
//! directly.
//!
//! ```text
//! # comment
//! # block 1: DL 5 track 12/5 cards 16-18     <- sheet block header: starts a block
//! dl 5 track 12/5          # the same, written as a directive ("dl 5", "track 12/5")
//! # card rows (above the mc grid):            <- following labels are card rows
//! Y  = 5P1 + 12P5          # rows Y, X, 0, 1: the triad's first four rows
//! # mc rows:                                  <- following labels are minor cycles
//! 0  1 0 30 - 0 1 -        # mc NIS S D C W T go   ("-" = empty = 0)
//! 6  4 22 21 1 9(1) 8 -    # W(J): J is P22-P25, ignored by Control
//! 29 3 3 22 3 30 9 x       # go: "-", "go" or 1 obeys; "x", "X" or 0 is a stopper
//! 7  0 12 24 - 0 0 go+P34  # "+P34": a hole in column 54 (P34) on this row
//! 5  0 0 0 - 0 1 +P1       # "+P1": the word has P1 as well
//! 4  = 15P5                # constant (see `word::parse_constant`)
//! 3  7 21 27 - 0 6 - ?     # a trailing "?" marks a doubtful reading (kept)
//! 24 -                     # a struck row: nothing in this m.c.
//! ts 13 = P17              # temporary stores 13-16
//! ds 21 3 = -1             # double store 19-21, m.c. 2/3 (or 0/1)
//! qs 17 1 = P1             # quadruple store 17-18, words 0-3
//! id = P5                  # the input dynamiciser keys
//! start 1 0                # take the first instruction from DL1, m.c. 0
//! ```
//!
//! On a sheet line, a note of the form `written "W (J) T"` or
//! `W written "W (J)"` (the transcribers' record of a bracketed number between
//! W and T) supplies J when the fields do not, provided W and T agree.

use super::cards::{parse_instruction, parse_row, Card, ROW_NAMES};
use super::word::{p, parse_constant};

/// Where a block of 32 words goes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Block {
    /// The DL the words are coded for (and placed in if there is no track).
    pub dl: Option<u8>,
    /// Drum track number 0-255 (position * 16 + head), written "a/b".
    pub track: Option<u8>,
    /// The card numbers from the sheet header, as written.
    pub cards: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct BlockImage {
    pub block: Block,
    /// Minor cycles 0-31; `None` where nothing is written.
    pub words: [Option<u32>; 32],
    /// Card rows Y, X, 0, 1 above the grid.
    pub intro: [Option<u32>; 4],
    /// Column 54 holes: bit k for m.c. k.
    pub col54: u32,
    /// Column 54 holes on the card rows Y, X, 0, 1.
    pub intro_col54: u8,
    /// Bit k set: m.c. k was written as an instruction (fields), not as a
    /// constant.
    pub instr_form: u32,
    /// Lines marked doubtful ("?"): (line number, text).
    pub doubts: Vec<(usize, String)>,
}

impl BlockImage {
    /// The 32 words, empty minor cycles as zero.
    pub fn values(&self) -> [u32; 32] {
        let mut w = [0u32; 32];
        for (i, v) in self.words.iter().enumerate() {
            w[i] = v.unwrap_or(0);
        }
        w
    }

    /// The card numbers from the header: "16-18" gives (16, 3), "0" gives
    /// (0, 1); anything else (or none) is taken as a triad of 3 cards.
    pub fn card_numbers(&self) -> (Option<u32>, usize) {
        let c = self
            .block
            .cards
            .as_deref()
            .unwrap_or("")
            .split_whitespace()
            .next()
            .unwrap_or("");
        if let Some((a, b)) = c.split_once('-') {
            if let (Ok(a), Ok(b)) = (a.parse::<u32>(), b.parse::<u32>()) {
                if b >= a {
                    return (Some(a), (b - a + 1) as usize);
                }
            }
        } else if let Ok(a) = c.parse::<u32>() {
            return (Some(a), 1);
        }
        (None, 3)
    }

    /// The block as punched: the card rows Y, X, 0, 1 then minor cycles
    /// 0-31 on successive rows (BP §11, sheet 20), on as many cards as the
    /// header gives (normally a triad of three; one for "cards 0"). Words
    /// that fall beyond the last card are left out.
    pub fn cards(&self) -> Vec<Card> {
        let (_, n) = self.card_numbers();
        let mut cards = vec![Card::blank(); n];
        for r in 0..(12 * n).min(36) {
            let (w, hole) = if r < 4 {
                (self.intro[r].unwrap_or(0), self.intro_col54 >> r & 1 == 1)
            } else {
                (self.words[r - 4].unwrap_or(0), self.col54 >> (r - 4) & 1 == 1)
            };
            cards[r / 12].rows[r % 12] = w;
            if hole {
                cards[r / 12].col54 |= 1 << (r % 12);
            }
        }
        cards
    }
}

#[derive(Clone, Debug, Default)]
pub struct Image {
    pub blocks: Vec<BlockImage>,
    pub ts: Vec<(u8, u32)>,
    pub ds: Vec<((u8, u8), u32)>,
    pub qs: Vec<((u8, u8), u32)>,
    pub id: Option<u32>,
    pub start: Option<(u8, u8)>,
}

/// Parse a track written "a/b" (position/head) or as a number 0-255.
pub fn parse_track(s: &str) -> Result<u8, String> {
    if let Some((a, b)) = s.split_once('/') {
        let a: u8 = a.trim().parse().map_err(|_| format!("bad track {s:?}"))?;
        let b: u8 = b.trim().parse().map_err(|_| format!("bad track {s:?}"))?;
        if a > 15 || b > 15 {
            return Err(format!("track {s:?} out of range"));
        }
        Ok(a * 16 + b)
    } else {
        s.trim().parse().map_err(|_| format!("bad track {s:?}"))
    }
}

#[derive(PartialEq)]
enum Section {
    Mc,
    CardRows,
}

/// Parse a sheet header "block k: DL n track t cards c" (after the '#').
fn block_header(comment: &str) -> Option<Result<Block, String>> {
    let c = comment.trim();
    let rest = c.strip_prefix("block")?;
    let (_, fields) = rest.split_once(':')?;
    let toks: Vec<&str> = fields.split_whitespace().collect();
    let mut block = Block::default();
    let mut i = 0;
    while i + 1 < toks.len() {
        let (key, val) = (toks[i].to_ascii_lowercase(), toks[i + 1]);
        match key.as_str() {
            "dl" if val != "-" => match val.parse::<u8>() {
                Ok(d) if (1..=12).contains(&d) => block.dl = Some(d),
                _ => return Some(Err(format!("bad DL {val:?} in block header"))),
            },
            "track" if val != "-" => match parse_track(val) {
                Ok(t) => block.track = Some(t),
                Err(e) => return Some(Err(e)),
            },
            "cards" => block.cards = Some(toks[i + 1..].join(" ")),
            _ => {}
        }
        if key == "cards" {
            break;
        }
        i += 2;
    }
    Some(Ok(block))
}

/// From a note like `W/T written "9 (1) 8"` or `W written "2 (15)", T 23`,
/// the bracketed number, if W (and T, when given) agree with the fields.
fn bracket_from_note(note: &str, w: u8, t: u8) -> Option<u8> {
    let start = note.find("written \"")? + 9;
    let quoted = &note[start..];
    let quoted = &quoted[..quoted.find('"')?];
    let (before, after) = quoted.split_once('(')?;
    let (j, after) = after.split_once(')')?;
    let j: u8 = j.trim().parse().ok()?;
    let wv: u8 = before.trim().parse().ok()?;
    if wv != w || j > 15 {
        return None;
    }
    let after = after.trim();
    if !after.is_empty() {
        let tv: u8 = after.split_whitespace().next()?.parse().ok()?;
        if tv != t {
            return None;
        }
    }
    Some(j)
}

/// Parse the contents of one row (after its label): a constant, an
/// instruction with go suffixes, or nothing. Returns (word, col54, doubt).
fn parse_sheet_row(body: &str, note: &str) -> Result<Option<(u32, bool)>, String> {
    let mut body = body.trim();
    if let Some(b) = body.strip_suffix('?') {
        body = b.trim();
    }
    if body.is_empty() || body == "-" {
        return Ok(None);
    }
    if body.starts_with('=') || body.starts_with('|') || body == "." {
        return parse_row(body).map(Some);
    }
    // instruction: separate the go field and its "+P1" / "+P34" marks
    let mut toks: Vec<String> = body
        .replace('(', " (")
        .replace(')', ") ")
        .split_whitespace()
        .map(String::from)
        .collect();
    let mut extra = 0u32;
    let mut col54 = false;
    if let Some(last) = toks.last().cloned() {
        let lower = last.to_ascii_lowercase();
        if lower.contains("+p") {
            let (go, marks) = lower.split_once('+').unwrap();
            for m in marks.split('+') {
                match m {
                    "p34" => col54 = true,
                    m => {
                        let k: u32 = m
                            .trim_start_matches('p')
                            .parse()
                            .map_err(|_| format!("bad mark {last:?}"))?;
                        if !(1..=32).contains(&k) {
                            return Err(format!("bad mark {last:?}"));
                        }
                        extra |= p(k);
                    }
                }
            }
            let go = if go.is_empty() || go == "go" { "-" } else { go };
            *toks.last_mut().unwrap() = go.to_string();
        } else if lower == "go" {
            *toks.last_mut().unwrap() = "-".into();
        }
    }
    let joined = toks.join(" ");
    let mut ins = parse_instruction(&joined)?;
    if ins.j == 0 {
        if let Some(j) = bracket_from_note(note, ins.w, ins.t) {
            ins.j = j;
        }
    }
    Ok(Some((ins.encode() | extra, col54)))
}

pub fn parse_image(src: &str) -> Result<Image, String> {
    let mut img = Image::default();
    let mut section = Section::Mc;
    for (n, raw) in src.lines().enumerate() {
        let lineno = n + 1;
        let err = |e: String| format!("line {lineno}: {e}");
        let (code, note) = match raw.find('#') {
            Some(i) => (raw[..i].trim(), &raw[i + 1..]),
            None => (raw.trim(), ""),
        };
        if code.is_empty() {
            if let Some(h) = block_header(note) {
                img.blocks.push(BlockImage {
                    block: h.map_err(err)?,
                    ..Default::default()
                });
                section = Section::Mc;
            } else if note.trim_start().starts_with("card rows") {
                section = Section::CardRows;
            } else if note.trim_start().starts_with("mc rows") {
                section = Section::Mc;
            }
            continue;
        }
        let doubt = code.ends_with('?');
        let (head, body) = code.split_once(char::is_whitespace).unwrap_or((code, ""));
        let lower = head.to_ascii_lowercase();
        match lower.as_str() {
            "dl" | "track" => {
                let mut block = Block::default();
                let toks: Vec<&str> = code.split_whitespace().collect();
                for pair in toks.chunks(2) {
                    let val = pair.get(1).ok_or_else(|| err(format!("{} needs a value", pair[0])))?;
                    match pair[0].to_ascii_lowercase().as_str() {
                        "dl" => {
                            let d: u8 = val.parse().map_err(|_| err(format!("bad DL {val:?}")))?;
                            if !(1..=12).contains(&d) {
                                return Err(err(format!("DL {d} out of range")));
                            }
                            block.dl = Some(d);
                        }
                        "track" => block.track = Some(parse_track(val).map_err(err)?),
                        k => return Err(err(format!("unexpected {k:?}"))),
                    }
                }
                img.blocks.push(BlockImage {
                    block,
                    ..Default::default()
                });
                section = Section::Mc;
            }
            "row" => {
                let (label, rest) = body.trim().split_once(char::is_whitespace).unwrap_or((body.trim(), ""));
                set_intro(&mut img, label, rest, note).map_err(err)?;
            }
            "ts" | "ds" | "qs" => {
                let (lhs, rhs) = code.split_once('=').ok_or_else(|| err("expected =".into()))?;
                let nums: Vec<u8> = lhs
                    .split_whitespace()
                    .skip(1)
                    .map(|t| t.parse().map_err(|_| err(format!("bad number {t:?}"))))
                    .collect::<Result<_, _>>()?;
                let v = parse_constant(rhs).map_err(err)?;
                match (lower.as_str(), nums.as_slice()) {
                    ("ts", [s]) if (13..=16).contains(s) => img.ts.push((*s, v)),
                    ("ds", [s, k]) if (19..=21).contains(s) => img.ds.push(((*s, k % 2), v)),
                    ("qs", [s, k]) if (17..=18).contains(s) && *k < 4 => img.qs.push(((*s, *k), v)),
                    _ => return Err(err(format!("bad store reference {lhs:?}"))),
                }
            }
            "id" => {
                let (_, rhs) = code.split_once('=').ok_or_else(|| err("expected id = value".into()))?;
                img.id = Some(parse_constant(rhs).map_err(err)?);
            }
            "start" => {
                let a: Vec<u8> = body
                    .split_whitespace()
                    .map(|t| t.parse().map_err(|_| err(format!("bad number {t:?}"))))
                    .collect::<Result<_, _>>()?;
                match a.as_slice() {
                    [d, m] if (1..=8).contains(d) && *m < 32 => img.start = Some((*d, *m)),
                    _ => return Err(err("start needs DL (1-8) and minor cycle".into())),
                }
            }
            _ if section == Section::CardRows || lower == "y" || lower == "x" => {
                set_intro(&mut img, head, body, note).map_err(err)?;
            }
            _ => {
                let mc: usize = head
                    .parse()
                    .map_err(|_| err(format!("expected a minor cycle number, got {head:?}")))?;
                if mc > 31 {
                    return Err(err(format!("minor cycle {mc} out of range")));
                }
                let row = parse_sheet_row(body, note).map_err(err)?;
                let blk = img
                    .blocks
                    .last_mut()
                    .ok_or_else(|| err("minor cycle before any block".into()))?;
                if doubt {
                    blk.doubts.push((lineno, raw.trim().to_string()));
                }
                if let Some((w, hole)) = row {
                    if !body.trim_start().starts_with('=') && !body.trim_start().starts_with('|') {
                        blk.instr_form |= 1 << mc;
                    }
                    if blk.words[mc].is_some() {
                        return Err(err(format!("minor cycle {mc} given twice")));
                    }
                    blk.words[mc] = Some(w);
                    if hole {
                        blk.col54 |= 1 << mc;
                    }
                }
            }
        }
    }
    Ok(img)
}

fn set_intro(img: &mut Image, label: &str, body: &str, note: &str) -> Result<(), String> {
    let r = ROW_NAMES[..4]
        .iter()
        .position(|&x| x.eq_ignore_ascii_case(label))
        .ok_or_else(|| format!("card row must be Y, X, 0 or 1, not {label:?}"))?;
    let row = parse_sheet_row(body, note)?;
    let blk = img.blocks.last_mut().ok_or("card row before any block")?;
    if let Some((w, hole)) = row {
        blk.intro[r] = Some(w);
        if hole {
            blk.intro_col54 |= 1 << r;
        }
    }
    Ok(())
}
