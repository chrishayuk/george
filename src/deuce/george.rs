//! Running GEORGE IA itself on the simulator: the basic pack from the
//! transcribed coding sheets, the G-program initial card, program cards and
//! data cards. See `sources/deuce/RUNNING-GEORGE.md`.

use std::path::Path;

use super::cards::Card;
use super::loader::{parse_image, BlockImage};
use super::{Machine, Outcome};

/// The transcribed sheets, read from `dir`.
pub fn read_sheets(dir: &Path) -> Result<Vec<BlockImage>, String> {
    let mut names: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .map(|e| e.unwrap().path())
        .filter(|p| {
            let n = p.file_name().unwrap().to_string_lossy();
            n.starts_with("sheet-") && n.ends_with(".txt")
        })
        .collect();
    names.sort();
    let mut blocks = Vec::new();
    for path in names {
        let src = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let img = parse_image(&src).map_err(|e| format!("{}: {e}", path.display()))?;
        blocks.extend(img.blocks);
    }
    Ok(blocks)
}

/// Words whose unused digit P31 must be 1 although the sheet, which has no
/// column for it, cannot show it: (track, m.c.). See RUNNING-GEORGE.md.
pub const P31_SET: &[(u8, u8)] = &[
    // 10_29 "convert to 16-10" (flow diagrams p.16): T = 30 stands for -2,
    // i.e. the word is negative from P26 up, so P31 = P32 = 1.
    (13 * 16 + 9, 29),
];

/// Words written with go whose P31 must be 0, because counting by spill
/// carries into P31: the next-keyword quasi templates "2, 21-17, 0, 0" (flow
/// diagrams p.16), which NK increments by P22 + P27 so that T overflows into
/// P31 at the end of a keyword track; and the read-in's store template.
pub const P31_CLEAR: &[(u8, u8)] = &[
    (12 * 16 + 12, 30),
    (12 * 16 + 15, 8),
    (13 * 16 + 10, 10),
    (13 * 16 + 10, 30),
    // The read-in's store quasi template "1, 14-9, 1 (15) 31" (flow diagrams
    // p.5, "8_9 [14-9_0, J=15]"): after 32 symbols W spills through J into
    // T = 31, which carries into P31.
    (13 * 16 + 12, 9),
];

/// Corrections to the sheets as written, applied when GEORGE is run (the
/// transcription is left as Hamblin wrote it): (track, m.c., the word as
/// corrected, in the sheet's field notation). See RUNNING-GEORGE.md.
pub const CORRECTIONS: &[(u8, u8, &str)] = &[
    // Sheet 3 block 1 (DL5, 12/5) m.c. 29 is written "4 4 24 - 0 1": 4-24
    // then on to 4_0, which is the multiply's 19_2-13. Flow diagram p.20
    // (divide) goes from 5_29 4-24 to 4_31 23-21_3, which needs T = 0.
    (12 * 16 + 5, 29, "4 4 24 - 0 0 -"),
    // Sheet 12 block 3 (DL8, 14/2) m.c. 18 is written "1 21 8 2 0 1": the
    // end of "R| and P| modify", on to 1_21 (NKD). Flow diagram p.42 ends
    // modify at "8_18 21-8_20,21 (2)" -> 1_31, where assemble planted the
    // link ("7_15 16-1_31", link Q31 in 16). That needs T = 11.
    (14 * 16 + 2, 18, "1 21 8 2 0 11 -"),
    // Sheet 17 block 2 (DL6, 15/0) m.c. 28, one of Hamblin's two additions
    // to S06/M12, is written "1 27 22 - 1 27": add P1 into 21_3, then on to
    // 1_25. Flow diagram p.47 (punch translate) has "6_28 27-22_3" followed
    // by "6_10 21_3-14" (the other addition, which then goes to 1_25), so
    // that TS14 holds the signed exponent for the sign test at 1_16. As
    // written 6_10 is never reached, TS14 still holds P10, and a number with
    // binary exponent -2 or less punches zeros without end. Going on to 6_10
    // needs NIS 6, T 12.
    (15 * 16, 28, "6 27 22 - 1 12 -"),
    // Sheet 8 block 1 (DL5, 13/4, the translator's keyword table) m.c. 6,
    // the keyword for `wait`, is written clearly "3 15 30 - 0 6": 15-30 ->
    // 3_8, which calls track 12/15 (the skips) and enters it at 7_9. Flow
    // diagram p.9 has this keyword amended to "14-30 -> 1_3" (14 written
    // over 15, 1_3 over 3_8), and p.23 (wait) has "k 14-30, 1_3 11-7(32),
    // 7_4 8-24, ... 7_18 16-29 X". The sheets agree with the amendment:
    // 12/14 m.c. 4-18 is p.23's wait, and 12/1 m.c. 3 is "11-7(32)" on to
    // 7_4. As Q_0 that keyword is NIS 1, 14-30, T 1.
    (13 * 16 + 4, 6, "1 14 30 - 0 1 -"),
];

/// Reconstructions of cells that cannot be read securely on the scan,
/// supplied at run time (the transcription keeps the literal reading):
/// (track, m.c., word in field notation). See RUNNING-GEORGE.md.
pub const RECONSTRUCTIONS: &[(u8, u8, &str)] = &[
    // Sheet 21 block 3 (DL8, 15/13) m.c. 21 is struck and overwritten in two
    // layers ("0 29 27 2 26 26" / "0 21 23 - 30 6"). 8_18 (T 1) leads here.
    // Flow diagram p.38 (sin/cos) has "8_21 28-26 (2) (waste time)" between
    // 8_18 21_2-15 and 8_17 15-23_3; "0 28 26 2 26 26" does exactly that
    // (C 2, W = T = 26: transfers in m.c. 17-18 and goes on to 8_17).
    (15 * 16 + 13, 21, "0 28 26 2 26 26 -"),
];

/// Apply `CORRECTIONS` and `RECONSTRUCTIONS` (unless GEORGE_AS_WRITTEN is set).
pub fn correct(blocks: &[BlockImage]) -> Vec<BlockImage> {
    let mut out = blocks.to_vec();
    if std::env::var("GEORGE_AS_WRITTEN").is_ok() {
        return out;
    }
    for &(t, k, text) in CORRECTIONS.iter().chain(RECONSTRUCTIONS) {
        if let Some(b) = out.iter_mut().find(|b| b.block.track == Some(t)) {
            let w = super::cards::parse_instruction(text).expect("correction").encode();
            let p31 = b.words[k as usize].map(|old| old & (1 << 30)).unwrap_or(0);
            b.words[k as usize] = Some(w | p31);
        }
    }
    out
}

/// Set P31 in the words listed in `P31_SET`.
pub fn set_p31(blocks: &[BlockImage]) -> Vec<BlockImage> {
    let mut out = blocks.to_vec();
    let rule = std::env::var("GEORGE_P31").unwrap_or_default();
    for b in &mut out {
        for k in 0..32 {
            if b.instr_form >> k & 1 == 1 {
                if let Some(w) = &mut b.words[k] {
                    let t = (*w >> 25) & 31;
                    let go = *w >> 31 == 1;
                    let tracks: Vec<u8> = rule
                        .split(',')
                        .filter_map(|x| super::loader::parse_track(x).ok())
                        .collect();
                    let clear = P31_CLEAR.contains(&(b.block.track.unwrap_or(255), k as u8));
                    let set = match rule.as_str() {
                        "all" => go,
                        "" | "default" => go && !clear,
                        "t16" => go && t >= 16,
                        _ => go && b.block.track.is_some_and(|tr| tracks.contains(&tr)),
                    };
                    if set {
                        *w |= 1 << 30;
                    }
                }
            }
        }
        for &(t, k) in P31_SET {
            if b.block.track == Some(t) {
                if let Some(w) = &mut b.words[k as usize] {
                    *w |= 1 << 30;
                }
            }
        }
    }
    out
}

/// Fill the minor cycles a sheet leaves empty from another image's block for
/// the same track (the library routines on track 15/0, which the sheet only
/// names). A minor cycle written on both is an error.
pub fn merge_library(blocks: &[BlockImage], library: &[BlockImage]) -> Result<Vec<BlockImage>, String> {
    let mut out = blocks.to_vec();
    for lib in library {
        let t = lib.block.track.ok_or("library block without a track")?;
        let b = out.iter_mut().find(|b| b.block.track == Some(t)).ok_or(format!(
            "no sheet block for track {}/{}",
            t / 16,
            t % 16
        ))?;
        for k in 0..32 {
            if let Some(w) = lib.words[k] {
                if b.words[k].is_some() {
                    return Err(format!(
                        "track {}/{} m.c. {k} is on both the sheet and the library",
                        t / 16,
                        t % 16
                    ));
                }
                b.words[k] = Some(w);
                b.instr_form |= lib.instr_form & (1 << k);
            }
        }
    }
    Ok(out)
}

/// The basic pack: every block with card numbers, in card-number order.
pub fn basic_pack(blocks: &[BlockImage]) -> Vec<Card> {
    let mut numbered: Vec<&BlockImage> = blocks.iter().filter(|b| b.card_numbers().0.is_some()).collect();
    numbered.sort_by_key(|b| b.card_numbers().0);
    numbered.iter().flat_map(|b| b.cards()).collect()
}

/// The one-card programs on a block without card numbers (sheet 22 block
/// 3): the first card holds rows Y, X, 0, 1 and m.c. 0-7; a second card,
/// if the block goes on, holds m.c. 8-19, and a third m.c. 20-31.
pub fn single_cards(block: &BlockImage) -> Vec<Card> {
    let mut b = block.clone();
    b.block.cards = Some("0-2".into());
    b.cards()
}

/// The G-program initial card: the first card of the block titled
/// "G-program initial card" (sheet 22, block 3).
pub fn gpic(blocks: &[BlockImage]) -> Option<Card> {
    blocks
        .iter()
        .find(|b| {
            b.card_numbers().0.is_none() && b.block.dl.is_none() && b.block.track.is_none() && b.words[7].is_some()
        })
        .map(|b| single_cards(b)[0])
}

/// Pack symbol bytes four to a row, twelve rows to a card, ending with the
/// code 14/15 (byte 0xFE). `order[k]` is the digit position (1, 9, 17 or 25)
/// of the k-th symbol of a row.
pub fn program_cards(bytes: &[u8], order: [u32; 4]) -> Vec<Card> {
    let mut all: Vec<u8> = bytes.to_vec();
    all.push(0xFE);
    let mut rows: Vec<u32> = Vec::new();
    for chunk in all.chunks(4) {
        let mut w = 0u32;
        for (k, &b) in chunk.iter().enumerate() {
            w |= u32::from(b) << (order[k] - 1);
        }
        rows.push(w);
    }
    rows.chunks(12)
        .map(|r| {
            let mut c = Card::blank();
            c.rows[..r.len()].copy_from_slice(r);
            c
        })
        .collect()
}

/// Boot the basic pack: initial input, run until the machine waits.
pub fn boot(m: &mut Machine, pack: Vec<Card>, limit: u64) -> Outcome {
    m.load_cards(pack);
    m.initial_input();
    m.run(limit)
}

/// What a run of a G-program produced.
#[derive(Debug)]
pub struct GRun {
    pub outcome: Outcome,
    pub punched: Vec<Card>,
    pub os: u32,
    pub count: u32,
    pub count_from: Option<(u8, u8)>,
    /// What the operator saw at the console (`operate`).
    pub console: Vec<Console>,
}

/// A machine with the basic pack on the drum, waiting for a G-program:
/// the sheets with `set_p31` and `correct` applied, read in by the
/// INITIAL INPUT key.
pub fn boot_george(blocks: &[BlockImage]) -> Result<Machine, String> {
    let mut config = super::Config::default();
    if let Ok(v) = std::env::var("GEORGE_READER_ROW") {
        config.reader.row_interval = v.parse::<u64>().map_err(|_| "GEORGE_READER_ROW")? * super::MAJOR;
    }
    if let Ok(v) = std::env::var("GEORGE_READER_CARD") {
        config.reader.card_interval = v.parse::<u64>().map_err(|_| "GEORGE_READER_CARD")? * super::MAJOR;
    }
    let mut m = Machine::new(config);
    let o = boot(&mut m, basic_pack(&correct(&set_p31(blocks))), 5_000_000);
    if o != Outcome::ReaderEmpty {
        return Err(format!("basic pack did not finish loading: {o:?}"));
    }
    Ok(m)
}

/// Present a G-program (initial card, program cards, data cards) to a
/// booted machine with the INITIAL INPUT key, and run it with the default
/// operator (single shot at `wait` only, I.D. clear).
pub fn run_booted(
    m: Machine,
    blocks: &[BlockImage],
    bytes: &[u8],
    data: Vec<Card>,
    limit: u64,
    trace: bool,
) -> Result<(Machine, GRun), String> {
    run_operated(m, blocks, bytes, data, limit, trace, &Operator::default())
}

/// As `run_booted`, with the operator `op` at the console.
pub fn run_operated(
    mut m: Machine,
    blocks: &[BlockImage],
    bytes: &[u8],
    data: Vec<Card>,
    limit: u64,
    trace: bool,
    op: &Operator,
) -> Result<(Machine, GRun), String> {
    let gpic = gpic(blocks).ok_or("no G-program initial card on the sheets")?;
    let mut deck = vec![gpic];
    deck.extend(program_cards(bytes, [1, 9, 17, 25]));
    deck.extend(data);
    if trace {
        m.trace_on();
    }
    m.load_cards(deck);
    m.initial_input();
    let (outcome, console) = operate(&mut m, op, limit);
    let run = GRun {
        outcome,
        console,
        punched: m.punched().to_vec(),
        os: m.os,
        count: m.count,
        count_from: m.count_from,
    };
    Ok((m, run))
}

/// What the operator does at the console while a G-program runs.
#[derive(Clone, Debug, Default)]
pub struct Operator {
    /// Numbers set on the I.D. keys, one for each `I` obeyed (then 0). IA's
    /// `I` does not stop: its keyword `0-16` (flow diagrams p.9) pushes the
    /// I.D. word as it stands, so each number is set as a GEORGE floating
    /// word (exponent P1-10, mantissa P11-32), just before the keyword.
    pub keys: Vec<f64>,
    /// Also give a single shot at the stops the manual calls resumable
    /// (section 2.4), not only at `wait`.
    pub single_shot: bool,
}

/// Something the operator saw and acted on, with the number of cards
/// punched before it.
#[derive(Clone, Debug, PartialEq)]
pub enum Console {
    /// The `wait` stop 1 16-29x: the O.S. word, given a single shot.
    Lights { os: u32, punched: usize },
    /// A resumable stop given a single shot (with `Operator::single_shot`).
    Resumed {
        stop: &'static crate::machine::StopKind,
        indication: String,
        punched: usize,
    },
}

/// The indication "NIS S-D[x]" of an instruction.
pub fn indication(count: u32) -> String {
    let i = super::Instr::decode(count);
    format!("{} {}-{}{}", i.nis, i.s, i.d, if i.go { "" } else { "x" })
}

/// Is the machine on the `wait` stop: 7_18 "16-29 X" on track 12/14 (flow
/// diagrams p.23), NIS 1?
pub fn on_wait(m: &Machine) -> bool {
    indication(m.count) == "1 16-29x"
}

/// The stop-table entry (manual section 2.4) for a resumable arithmetic or
/// translation stop the machine is on, if it is one. IA's 1 30-21x is
/// either log's argument stop (resumable) or the accumulator stop
/// (non-resumable, from 1_14 of track 12/1, the accumulator routines in
/// DL1); it is told apart by the DL the stopper is in.
pub fn resumable_stop(m: &Machine) -> Option<&'static crate::machine::StopKind> {
    use crate::machine::{ACCUMULATOR, ALL_STOPS, LINK_LIST};
    let ind = indication(m.count);
    if ind == "1 30-21x" && m.count_from.is_some_and(|(dl, _)| dl == 1) {
        return None;
    }
    ALL_STOPS.iter().find(|s| {
        s.resume.is_some()
            && **s != ACCUMULATOR
            && **s != LINK_LIST
            && s.indication
                .split(", ")
                .filter(|i| !i.ends_with("(II)"))
                .any(|i| i.split(' ').take(2).collect::<Vec<_>>().join(" ") == ind)
    })
}

/// Is TS COUNT the keyword of `I` (`0-16`, flow diagrams p.9), obeyed from
/// a keyword track (position 10) in DL2?
fn on_keys(m: &Machine) -> bool {
    let i = super::Instr::decode(m.count);
    i.s == 0 && i.d == 16 && m.count_from.is_some_and(|(dl, _)| dl == 2) && m.dl_origin[2].is_some_and(|t| t / 16 == 10)
}

/// How long the operator takes to press the single-shot key after a stop:
/// a second (31250 m.c.). Anything over 66 m.c. lets a division started
/// before the stop (÷ by zero stops while the divider runs, flow diagrams
/// p.20) finish during the stop instead of overwriting DS21 afterwards.
pub const OPERATOR_DELAY: u64 = 31_250;

/// Run a booted, loaded machine as the operator would: set the keys for each
/// `I`, give a single shot at `wait` (and at resumable stops if asked), and
/// stop at anything else.
pub fn operate(m: &mut Machine, op: &Operator, limit: u64) -> (Outcome, Vec<Console>) {
    let mut keys = op.keys.iter().copied();
    let mut seen = Vec::new();
    let end = m.obeyed + limit;
    loop {
        if m.obeyed >= end {
            return (Outcome::Limit, seen);
        }
        if !op.keys.is_empty() && on_keys(m) {
            let x = keys.next().unwrap_or(0.0);
            m.id = crate::number::Num::from_f64(x).map(|n| n.to_word()).unwrap_or(0);
        }
        match m.step() {
            None => {}
            Some(Outcome::AwaitSingleShot) if on_wait(m) => {
                seen.push(Console::Lights {
                    os: m.os,
                    punched: m.punched().len(),
                });
                m.single_shot_after(OPERATOR_DELAY);
                // Obey the stopper now, so that it is not seen twice.
                if let Some(o) = m.step() {
                    return (o, seen);
                }
            }
            Some(Outcome::AwaitSingleShot) if op.single_shot && resumable_stop(m).is_some() => {
                seen.push(Console::Resumed {
                    stop: resumable_stop(m).unwrap(),
                    indication: indication(m.count),
                    punched: m.punched().len(),
                });
                m.single_shot_after(OPERATOR_DELAY);
                if let Some(o) = m.step() {
                    return (o, seen);
                }
            }
            Some(o) => return (o, seen),
        }
    }
}

/// Boot the basic pack, then present and run a G-program.
pub fn run_program(
    blocks: &[BlockImage],
    bytes: &[u8],
    data: Vec<Card>,
    limit: u64,
    trace: bool,
) -> Result<(Machine, GRun), String> {
    run_booted(boot_george(blocks)?, blocks, bytes, data, limit, trace)
}

/// GEORGE IA as the sheets and the library give it: the transcribed sheets
/// in `root/sources/coding-sheets-1958/transcription`, with the library
/// routines of `root/sources/deuce/library/*.txt` filled in.
pub fn george_blocks(root: &Path) -> Result<Vec<BlockImage>, String> {
    let blocks = read_sheets(&root.join("sources/coding-sheets-1958/transcription"))?;
    let libdir = root.join("sources/deuce/library");
    let mut library = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&libdir) {
        let mut names: Vec<_> = entries
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "txt"))
            .collect();
        names.sort();
        for p in names {
            let src = std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
            library.extend(parse_image(&src).map_err(|e| format!("{}: {e}", p.display()))?.blocks);
        }
    }
    merge_library(&blocks, &library)
}

/// The row index (0 = Y, 1 = X, 2 = row 0, ... 11 = row 9) of digit `d`.
fn digit_row(d: u32) -> usize {
    d as usize + 2
}

/// A data card in the manual's standard form (§2.3) on the DEUCE field:
/// GEORGE column *k* (1-20) is card column 20 + *k*, i.e. digit P*k* of each
/// row. Column 1: sign of the mantissa (Y +, X -); columns 2-10: the mantissa
/// d.dddddddd; column 11: sign of the exponent; columns 12-20: the exponent.
/// Every digit is punched, zeros in row 0, as GEORGE punches its results.
pub fn data_card(n: &crate::number::Card) -> Card {
    let mut c = Card::blank();
    let mut hole = |col: u32, row: usize| c.rows[row] |= 1 << (col - 1);
    hole(1, if n.negative { 1 } else { 0 });
    let mant = format!("{:06}000", n.digits);
    for (i, ch) in mant.chars().enumerate() {
        hole(2 + i as u32, digit_row(ch.to_digit(10).unwrap()));
    }
    hole(11, if n.exponent < 0 { 1 } else { 0 });
    let exp = format!("{:09}", n.exponent.unsigned_abs());
    for (i, ch) in exp.chars().enumerate() {
        hole(12 + i as u32, digit_row(ch.to_digit(10).unwrap()));
    }
    c
}

/// A number as GEORGE punched it, column by column.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Punched {
    pub negative: bool,
    /// The nine mantissa digits, d.dddddddd.
    pub mantissa: [u8; 9],
    pub exp_negative: bool,
    pub exponent: u32,
}

impl Punched {
    /// Read columns 1-20 of the DEUCE field. Each digit column must have one
    /// hole in rows 0-9; sign columns Y (+), X (-) or nothing (+).
    pub fn read(card: &Card) -> Result<Punched, String> {
        let rows_of = |col: u32| -> Vec<usize> { (0..12).filter(|&r| card.rows[r] >> (col - 1) & 1 == 1).collect() };
        let sign = |col: u32| -> Result<bool, String> {
            match rows_of(col).as_slice() {
                [] | [0] => Ok(false),
                [1] => Ok(true),
                other => Err(format!("column {col}: holes in rows {other:?}")),
            }
        };
        let digit = |col: u32| -> Result<u8, String> {
            match rows_of(col).as_slice() {
                [] => Ok(0),
                [r] if *r >= 2 => Ok((*r - 2) as u8),
                other => Err(format!("column {col}: holes in rows {other:?}")),
            }
        };
        let mut mantissa = [0u8; 9];
        for (i, m) in mantissa.iter_mut().enumerate() {
            *m = digit(2 + i as u32)?;
        }
        let mut exponent = 0u32;
        for col in 12..=20 {
            exponent = exponent * 10 + u32::from(digit(col)?);
        }
        Ok(Punched {
            negative: sign(1)?,
            mantissa,
            exp_negative: sign(11)?,
            exponent,
        })
    }

    pub fn value(&self) -> f64 {
        let m: f64 = self.mantissa.iter().fold(0.0, |a, &d| a * 10.0 + f64::from(d)) / 1e8;
        let e = if self.exp_negative {
            -(self.exponent as i32)
        } else {
            self.exponent as i32
        };
        let v = m * 10f64.powi(e);
        if self.negative {
            -v
        } else {
            v
        }
    }

    /// The card as punched: "+d.dddddddd e+n".
    pub fn columns(&self) -> String {
        let m: String = self.mantissa.iter().map(|d| char::from(b'0' + d)).collect();
        format!(
            "{}{}.{} {}{}",
            if self.negative { '-' } else { '+' },
            &m[..1],
            &m[1..],
            if self.exp_negative { '-' } else { '+' },
            self.exponent
        )
    }

    /// The first six figures as the interpreter's card, for comparison.
    pub fn six_figures(&self) -> crate::number::Card {
        let digits = self.mantissa[..6].iter().fold(0u32, |a, &d| a * 10 + u32::from(d));
        let e = if self.exp_negative {
            -(self.exponent as i32)
        } else {
            self.exponent as i32
        };
        if digits == 0 {
            return crate::number::Card {
                negative: false,
                digits: 0,
                exponent: 0,
            };
        }
        crate::number::Card {
            negative: self.negative,
            digits,
            exponent: e,
        }
    }
}
