//! Hamblin's GEORGE IA, from the transcribed coding sheets, run on the DEUCE
//! simulator: the manual's programs in `examples/` must punch what the
//! interpreter punches. See `sources/deuce/RUNNING-GEORGE.md`.

use std::path::Path;

use george::deuce::george as g;
use george::deuce::{Card, Outcome};
use george::number::Card as DataCard;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Run `examples/NAME.george` (with NAME.in if there is one) on a clone of a
/// booted machine; the punched numbers to six figures, and the outcome.
fn run(
    m: &george::deuce::Machine,
    blocks: &[george::deuce::loader::BlockImage],
    name: &str,
) -> (Vec<String>, Outcome, u32) {
    let src = std::fs::read_to_string(root().join(format!("examples/{name}.george"))).unwrap();
    let prog = george::parse(&src).unwrap();
    let bytes: Vec<u8> = prog.symbols.iter().map(|s| s.byte()).collect();
    let mut data: Vec<Card> = Vec::new();
    if let Ok(text) = std::fs::read_to_string(root().join(format!("examples/{name}.in"))) {
        for line in text.lines() {
            let line = line.split('#').next().unwrap().trim();
            if !line.is_empty() {
                data.push(g::data_card(&DataCard::from_decimal(line).unwrap()));
            }
        }
    }
    let (_, r) = g::run_booted(m.clone(), blocks, &bytes, data, 3_000_000, false).unwrap();
    let out = r
        .punched
        .iter()
        .map(|c| g::Punched::read(c).unwrap().six_figures().decimal())
        .collect();
    (out, r.outcome, r.count)
}

/// The indication "NIS S-D[x]" of the instruction in TS COUNT.
fn indication(count: u32) -> String {
    let i = george::deuce::Instr::decode(count);
    format!("{} {}-{}{}", i.nis, i.s, i.d, if i.go { "" } else { "x" })
}

#[test]
fn the_manuals_programs_on_hamblins_george() {
    let blocks = g::george_blocks(root()).expect("the transcribed sheets");
    let m = g::boot_george(&blocks).unwrap();
    // Programs that punch exactly what the interpreter punches, and how each
    // ends: "0 0-0x" with the reader called is the end of a G-program; pairs
    // runs out of data cards at "1 11-6x" (its .stop file).
    for name in [
        "squares",
        "sum-of-squares",
        "factorials",
        "nested",
        "newton",
        "converge",
        "subroutine",
        "polynomial",
        "any-zero",
        "mean",
        "mean-one-pass",
        "normalise",
        "matrix",
        "pairs",
    ] {
        let (out, outcome, count) = run(&m, &blocks, name);
        let want: Vec<String> = std::fs::read_to_string(root().join(format!("examples/{name}.out")))
            .unwrap()
            .lines()
            .map(String::from)
            .collect();
        assert_eq!(out, want, "{name}");
        assert_eq!(outcome, Outcome::ReaderEmpty, "{name}");
        let end = if name == "pairs" { "1 11-6x" } else { "0 0-0x" };
        assert_eq!(indication(count), end, "{name}");
    }
    // Simpson's rule, which differed in the sixth figure until + and - were
    // done word for word as Hamblin's add routine does them (RUNNING-GEORGE.md,
    // finding 15); Hamblin's values, pinned so that any change shows.
    for (name, want) in [
        (
            "simpson",
            ["0.0497078", "0.0675882", "0.0807849", "0.093649", "0.0988802"],
        ),
        (
            "simpson-corrected",
            ["0.198831", "0.270353", "0.323139", "0.374596", "0.395521"],
        ),
    ] {
        let (out, outcome, count) = run(&m, &blocks, name);
        assert_eq!(out, want, "{name}");
        let file: Vec<String> = std::fs::read_to_string(root().join(format!("examples/{name}.out")))
            .unwrap()
            .lines()
            .map(String::from)
            .collect();
        assert_eq!(out, file, "{name}");
        assert_eq!(
            (outcome, indication(count).as_str()),
            (Outcome::ReaderEmpty, "0 0-0x"),
            "{name}"
        );
    }
}

/// Run G-program text on a clone of a booted machine: what it punches, and
/// the outcome with TS COUNT.
fn run_text(
    m: &george::deuce::Machine,
    blocks: &[george::deuce::loader::BlockImage],
    src: &str,
) -> (Vec<String>, Outcome, u32) {
    let prog = george::parse(src).unwrap();
    let bytes: Vec<u8> = prog.symbols.iter().map(|s| s.byte()).collect();
    let (_, r) = g::run_booted(m.clone(), blocks, &bytes, vec![], 3_000_000, false).unwrap();
    let out = r
        .punched
        .iter()
        .map(|c| g::Punched::read(c).unwrap().six_figures().decimal())
        .collect();
    (out, r.outcome, r.count)
}

/// The interpreter's punched numbers for the same text.
fn interpret(src: &str) -> Vec<String> {
    let prog = george::parse(src).unwrap();
    let mut im = george::Machine::new();
    let o = im.run(&prog, &george::Options::default());
    o.punched().iter().map(|n| DataCard::of(*n).decimal()).collect()
}

#[test]
fn symbols_agree_with_the_interpreter() {
    let blocks = g::george_blocks(root()).expect("the transcribed sheets");
    let m = g::boot_george(&blocks).unwrap();
    for src in [
        "3, 4 + (P) ; 3, 4 - (P) ; 3, 4 × (P) ; 3, 4 ÷ (P) ;",
        "2 sqrt (P) ; 2 log (P) ; 1 exp (P) ; 0, 3 - exp (P) ;",
        "2, .5 pow (P) ; 2, 3 pow (P) ; 3 neg, 2 pow (P) ; 3 neg, 3 pow (P) ;",
        "7, 3 rem (P) ; 7, 2.5 rem (P) ; 7.5, 2 rem (P) ;",
        "3, 3 = (P) ; 4, 3 > (P) ; 3, 4 > (P) ; 3, 3 = 4, 4 = & (P) ; 3, 3 = 3, 4 = ∨ (P) ;",
        "3, 4 = 5 ↑ 1 (P) ; * 5 2 (P) ;",
        ".2 (P) ; .05 (P) ; .000123 (P) ; 12345678 (P) ; 0, .05 - (P) ;",
        "7, 5 max (P) ; 7, 7 max (P) ; 0, 3 -, 0, 7 - max (P) ;",
        "1, 2, 1, 2 P||(a)",
        // sin and cos need the multiplier digit by digit (RUNNING-GEORGE.md, finding 14)
        ".5 sin (P) ; 1 sin (P) ; 1.5 sin (P) ; 3 sin (P) ; 10 sin (P) ;",
        "0 cos (P) ; 1 cos (P) ; 2 cos (P) ;",
    ] {
        let (out, outcome, _) = run_text(&m, &blocks, src);
        assert_eq!(out, interpret(src), "{src}");
        assert_eq!(outcome, Outcome::ReaderEmpty, "{src}");
    }
}

#[test]
fn where_hamblins_george_differs() {
    // Behaviour of this (penultimate) GEORGE IA that the interpreter does not
    // share; see RUNNING-GEORGE.md, "Results".
    let blocks = g::george_blocks(root()).expect("the transcribed sheets");
    let m = g::boot_george(&blocks).unwrap();
    // max(a, b) with b > a gives -b.
    assert_eq!(run_text(&m, &blocks, "2, 5 max (P) ;").0, vec!["-5"]);
    // A negative base that is minus a power of two: the standardised
    // mantissa -1 negates to +1, which log takes as negative: stop 1 30-21x.
    let (out, outcome, count) = run_text(&m, &blocks, "2 neg, 2 pow (P) ;");
    assert!(out.is_empty());
    assert_eq!(outcome, Outcome::AwaitSingleShot);
    let ins = george::deuce::Instr::decode(count);
    assert_eq!((ins.nis, ins.s, ins.d, ins.go), (1, 30, 21, false));
    // sin 0 is not quite zero: the series leaves -2^-25.
    assert_eq!(run_text(&m, &blocks, "0 sin (P) ;").0, vec!["-2.98023e-8"]);
    // "~" of falsehood (all ones) punches as 0.
    assert_eq!(run_text(&m, &blocks, "2, 3 = ~ (P) ;").0, vec!["0"]);
}

#[test]
fn program_card_layout() {
    // four 8-bit codes to a row, the first in P1-8, ending with 14/15
    let cards = g::program_cards(&[0x21, 0x01, 0xF1, 0x02, 0x15], [1, 9, 17, 25]);
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].rows[0], 0x02F1_0121);
    assert_eq!(cards[0].rows[1], 0xFE15);
}

#[test]
fn data_and_result_cards_round_trip() {
    for x in ["1", "-2.5", "0.000123", "1.23457e7"] {
        let c = DataCard::from_decimal(x).unwrap();
        let p = g::Punched::read(&g::data_card(&c)).unwrap();
        assert_eq!(p.six_figures(), c, "{x}");
    }
}

/// Not a check: prints, for each expression, the word GEORGE stores and
/// how its mantissa compares with the exact value (truncated or rounded).
/// `cargo test --test deuce_george explore_arithmetic -- --ignored --nocapture`
#[test]
#[ignore]
fn explore_arithmetic() {
    let blocks = g::george_blocks(root()).unwrap();
    let m = g::boot_george(&blocks).unwrap();
    let exprs = std::env::var("EXPRS").unwrap_or("7, 3 ÷|1, 3 ÷|2, 3 ÷|5, 7 ×".into());
    for e in exprs.split('|') {
        let prog = george::parse(&format!("{e} (a) ;")).unwrap();
        let bytes: Vec<u8> = prog.symbols.iter().map(|s| s.byte()).collect();
        let (mm, _) = g::run_booted(m.clone(), &blocks, &bytes, vec![], 3_000_000, false).unwrap();
        let w = mm.dl[12].iter().copied().find(|&w| w != 0).unwrap_or(0);
        let n = george::number::Num::from_word(w);
        println!("{e}\t{w:08x}\t{}\t{}\t{}", n.mantissa, n.exponent, n.to_f64());
    }
}

/// GEORGE's stored 22-digit mantissa and exponent for `expr (a) ;`.
fn stored(m: &george::deuce::Machine, blocks: &[george::deuce::loader::BlockImage], expr: &str) -> (i32, i32) {
    let prog = george::parse(&format!("{expr} (a) ;")).unwrap();
    let bytes: Vec<u8> = prog.symbols.iter().map(|s| s.byte()).collect();
    let (mm, _) = g::run_booted(m.clone(), blocks, &bytes, vec![], 3_000_000, false).unwrap();
    let w = mm.dl[12].iter().copied().find(|&w| w != 0).unwrap_or(0);
    let n = george::number::Num::from_word(w);
    (n.mantissa, n.exponent)
}

#[test]
fn hamblins_arithmetic_at_the_last_digit() {
    // RUNNING-GEORGE.md, "Where Hamblin's arithmetic differs". Mantissas are
    // in units of 2^-21 of the binary fraction, with exponent e (x = m 2^(e-21)).
    let blocks = g::george_blocks(root()).expect("the transcribed sheets");
    let m = g::boot_george(&blocks).unwrap();
    for (expr, want) in [
        // ÷ and × round to nearest
        ("1, 7 ÷", (1198373, -2)),          // exact 1198372.57
        ("7, 3 ÷", (1223339, 2)),           // exact 1223338.67
        ("1, 7 ÷ 1, 7 ÷ ×", (1369570, -5)), // exact 1369569.63
        // numerals round to nearest
        ("0.1", (1677722, -3)), // exact 1677721.6
        // + and -: neither rounded nor truncated (exact sum of the stored operands in brackets)
        ("1, 1, 7 ÷ +", (1198372, 1)),      // 1198372.625
        ("1, 1, 3 ÷ +", (1398102, 1)),      // 1398101.25
        ("1, 1, 11 ÷ +", (1143902, 1)),     // 1143901.0625
        ("3, 1, 7 ÷ +", (1647762, 2)),      // 1647762.3125
        ("5, 1, 3 ÷ +", (1398102, 3)),      // 1398101.3125
        ("1, 3 ÷ 1, 7 ÷ +", (1997287, -1)), // 1997287.5
        ("1, 1, 9 ÷ +", (1165084, 1)),      // 1165084.4375
        ("1, 2, 3 ÷ +", (1747626, 1)),      // 1747626.5
        ("3, 1, 3 ÷ +", (1747626, 2)),      // 1747626.625
        ("3, 1, 3 ÷ -", (1398102, 2)),      // 1398101.375
        ("5, 1, 7 ÷ -", (1273270, 3)),      // 1273270.84375
        ("7, 1, 3 ÷ +", (1922390, 3)),      // 1922389.3125
        ("7, 1, 3 ÷ -", (1747626, 3)),      // 1747626.6875
        ("1.5, 1, 3 ÷ +", (1922390, 1)),    // 1922389.25
        ("100, 1, 3 ÷ +", (1643862, 7)),    // 1643861.33
        ("1, 7 ÷ 3, 7 ÷ +", (1198373, 0)),  // 1198372.75
        ("1, 1, 3 ÷ -", (1398101, 0)),      // 1398101.5
        ("1, 2097152 ÷ 1 + 1 -", (0, 0)),   // 2^-21 lost: the smaller operand's last digit
    ] {
        assert_eq!(stored(&m, &blocks, expr), want, "{expr}");
        // and the interpreter stores the same word
        let prog = george::parse(&format!("{expr} (P) ;")).unwrap();
        let n = george::Machine::new().run(&prog, &george::Options::default()).punched()[0];
        assert_eq!((n.mantissa, n.exponent), want, "{expr} (interpreter)");
    }
}

/// Run G-program text with an operator at the console.
fn run_with(
    m: &george::deuce::Machine,
    blocks: &[george::deuce::loader::BlockImage],
    src: &str,
    op: &g::Operator,
) -> (Vec<String>, g::GRun) {
    let prog = george::parse(src).unwrap();
    let bytes: Vec<u8> = prog.symbols.iter().map(|s| s.byte()).collect();
    let (_, r) = g::run_operated(m.clone(), blocks, &bytes, vec![], 3_000_000, false, op).unwrap();
    let out = r
        .punched
        .iter()
        .map(|c| g::Punched::read(c).unwrap().six_figures().decimal())
        .collect();
    (out, r)
}

#[test]
fn wait_shows_the_top_cell_and_resumes() {
    // RUNNING-GEORGE.md, finding 16: the keyword for wait (13/4 m.c. 6) is
    // corrected to "14-30 -> 1_3" (flow diagrams pp.9, 23).
    let blocks = g::george_blocks(root()).expect("the transcribed sheets");
    let m = g::boot_george(&blocks).unwrap();
    let src = "2 (y) ; 1, 1, 3 rep (a) dup y rev ÷ + 2 ÷ wait ] (P) ;";
    let (out, r) = run_with(&m, &blocks, src, &g::Operator::default());
    assert_eq!(out, interpret(src));
    assert_eq!(r.outcome, Outcome::ReaderEmpty);
    assert_eq!(indication(r.count), "0 0-0x");
    let lights: Vec<String> = r
        .console
        .iter()
        .map(|c| match c {
            g::Console::Lights { os, punched } => {
                assert_eq!(*punched, 0);
                DataCard::of(george::Num::from_word(*os)).decimal()
            }
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(lights, ["1.5", "1.41667", "1.41422"]);
    // Without the operator, the run stops on 1 16-29x with the number on
    // the O.S.
    let prog = george::parse("2 wait (P) ;").unwrap();
    let bytes: Vec<u8> = prog.symbols.iter().map(|s| s.byte()).collect();
    let mut mm = m.clone();
    mm.load_cards(std::iter::once(g::gpic(&blocks).unwrap()).chain(g::program_cards(&bytes, [1, 9, 17, 25])));
    mm.initial_input();
    assert_eq!(mm.run(3_000_000), Outcome::AwaitSingleShot);
    assert_eq!(indication(mm.count), "1 16-29x");
    assert_eq!(george::Num::from_word(mm.os).to_f64(), 2.0);
    // An empty accumulator shows the word below the bottom cell, P1-10.
    let (_, r) = run_with(&m, &blocks, "wait 1 (P) ;", &g::Operator::default());
    assert_eq!(r.console, [g::Console::Lights { os: 0x3FF, punched: 0 }]);
}

#[test]
fn keys_are_read_by_i() {
    // IA's I does not stop: its keyword 0-16 pushes the I.D. word, which the
    // operator sets as a GEORGE floating word.
    let blocks = g::george_blocks(root()).expect("the transcribed sheets");
    let m = g::boot_george(&blocks).unwrap();
    let op = g::Operator {
        keys: vec![2.5, -3.0, 1e-3],
        ..g::Operator::default()
    };
    let (out, r) = run_with(&m, &blocks, "I (P) ; I (P) ; I, I + (P) ;", &op);
    assert_eq!(out, ["2.5", "-3", "0.001"]);
    assert_eq!(indication(r.count), "0 0-0x");
    // tutorial 11-keys: wait, then I tested for zero
    let src = "1, 3 rep (i) wait I 0 = 4 ↑ 1 (P) ; 5 ↑ * 4 2 (P) ; * 5 ]";
    let op = g::Operator {
        keys: vec![7.0, 0.0],
        ..g::Operator::default()
    };
    assert_eq!(run_with(&m, &blocks, src, &op).0, ["1", "2", "2"]);
}

#[test]
fn id_must_be_clear_at_read_in() {
    // Manual section 2.4: "0 0-14, 0 24-28 (IA only): I.D. has entry in
    // P1-31. Clear and continue." The translator tests the I.D. at the end.
    let blocks = g::george_blocks(root()).expect("the transcribed sheets");
    let mut m = g::boot_george(&blocks).unwrap();
    m.id = 1 << 10;
    let (out, r) = run_with(&m, &blocks, "2 (P) ;", &g::Operator::default());
    // It loops reading the I.D. and testing it for zero until it is cleared.
    assert!(out.is_empty());
    assert_eq!(r.outcome, Outcome::Limit);
    assert!(["0 0-14", "0 24-28"].contains(&indication(r.count).as_str()));
    // Section 2.6: P32 alone on the I.D. during translation gives the stop
    // 7 30-28x at the end of the translation.
    m.id = 1 << 31;
    let (out, r) = run_with(&m, &blocks, "2 (P) ;", &g::Operator::default());
    assert!(out.is_empty());
    assert_eq!(r.outcome, Outcome::AwaitSingleShot);
    assert_eq!(indication(r.count), "7 30-28x");
}

#[test]
fn single_shot_resumes_arithmetic_stops() {
    let blocks = g::george_blocks(root()).expect("the transcribed sheets");
    let m = g::boot_george(&blocks).unwrap();
    let ss = g::Operator {
        single_shot: true,
        ..g::Operator::default()
    };
    let src = "1, 0 ÷ (P) ; 1 neg log (P) ; 1, 0 rem (P) ; 1000 exp (P) ; 2 neg, .5 pow (P) ; 5 (P) ;";
    let (out, r) = run_with(&m, &blocks, src, &ss);
    assert_eq!(out, ["0", "0", "0", "0", "0", "5"]);
    let stops: Vec<String> = r
        .console
        .iter()
        .map(|c| match c {
            g::Console::Resumed { indication, .. } => indication.clone(),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(stops, ["1 4-24x", "1 30-21x", "1 30-16x", "7 30-21x", "5 30-21x"]);
    // √ of a negative number resumes with 0.5, not zero: TS16 still holds
    // the first approximation P31 at 7_11 (flow diagrams p.37).
    assert_eq!(run_with(&m, &blocks, "2 neg √ (P) ;", &ss).0, ["0.5"]);
    // Overflow resumes with the exponent halved.
    assert_eq!(
        run_with(&m, &blocks, "1e100 (P) ; 1e100, 1e100 × (P) ;", &ss).0,
        interpret_ss("1e100 (P) ; 1e100, 1e100 × (P) ;")
    );
    // Without --single-shot the run ends at the first stop.
    let (out, r) = run_with(&m, &blocks, src, &g::Operator::default());
    assert!(out.is_empty());
    assert_eq!(indication(r.count), "1 4-24x");
    // IA's accumulator stop 1 30-21x is not resumable (manual section 2.4).
    let (_, r) = run_with(&m, &blocks, "(P) ;", &ss);
    assert_eq!(r.outcome, Outcome::AwaitSingleShot);
    assert_eq!(indication(r.count), "1 30-21x");
    assert!(r.console.is_empty());
}

fn interpret_ss(src: &str) -> Vec<String> {
    let prog = george::parse(src).unwrap();
    let mut im = george::Machine::new();
    let o = im.run(
        &prog,
        &george::Options {
            single_shot: true,
            ..george::Options::default()
        },
    );
    o.punched().iter().map(|n| DataCard::of(*n).decimal()).collect()
}
