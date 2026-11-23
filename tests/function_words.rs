//! Hamblin's functions (√, log, exp, pow, sin, cos, rem; tracks 15/1-15/13,
//! flow diagrams pp.32-38) against the interpreter's `functions`, word for
//! word, and the stop and the word a single shot leaves. See
//! sources/deuce/RUNNING-GEORGE.md, findings 18-24.
//!
//!     cargo test --release --test function_words              300 of each
//!     CASES=20000 SEED=101 cargo test --release --test function_words
//!     SRC="0.5 sin (a) ;" cargo test --release --test function_words \
//!         trace_function -- --ignored --nocapture

use std::path::Path;

use george::deuce::george as g;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Not a check: the instruction trace of tracks TRACKS (default 15/x, as
/// the high nibble 15), and of instructions obeyed from TS13 (Q), with
/// TS13-16 and DS19-21 (high:low) after each. SRC is the G-program.
#[test]
#[ignore]
fn trace_function() {
    let blocks = g::george_blocks(root()).unwrap();
    let mut m = g::boot_george(&blocks).unwrap();
    let src = std::env::var("SRC").unwrap_or("0.5 √ (a) ;".into());
    let groups: Vec<u8> = std::env::var("TRACKS")
        .unwrap_or("15".into())
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect();
    let prog = george::parse(&src).unwrap();
    let bytes: Vec<u8> = prog.symbols.iter().map(|s| s.byte()).collect();
    let mut deck = vec![g::gpic(&blocks).unwrap()];
    deck.extend(g::program_cards(&bytes, [1, 9, 17, 25]));
    m.trace_on();
    m.load_cards(deck);
    m.initial_input();
    let mut on = false;
    for _ in 0..5_000_000 {
        if let Some(o) = m.step() {
            println!("stop {o:?}");
            break;
        }
        let st = m.trace.as_mut().unwrap().pop().unwrap();
        let fun = st.track.is_some_and(|t| groups.contains(&(t / 16)));
        if fun {
            on = true;
        }
        if on && (fun || st.from.is_none()) {
            println!("{st} [t {}]", st.transfer.0);
            println!(
                "      13={:08x} 14={:08x} 15={:08x} 16={:08x} 19={:08x}:{:08x} 20={:08x}:{:08x} 21={:08x}:{:08x}",
                m.ts13, m.ts14, m.ts15, m.ts16, m.ds[0][1], m.ds[0][0], m.ds[1][1], m.ds[1][0], m.ds[2][1], m.ds[2][0]
            );
        }
        if on && !fun && st.from.is_some() && st.track.is_some_and(|t| t / 16 == 12) {
            // back in the arithmetic tracks: show those too
            println!("{st} [t {}]", st.transfer.0);
            println!(
                "      13={:08x} 14={:08x} 15={:08x} 16={:08x} 19={:08x}:{:08x} 20={:08x}:{:08x} 21={:08x}:{:08x}",
                m.ts13, m.ts14, m.ts15, m.ts16, m.ds[0][1], m.ds[0][0], m.ds[1][1], m.ds[1][0], m.ds[2][1], m.ds[2][0]
            );
        }
    }
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

/// A random GEORGE word: mostly standardised numbers over the whole
/// exponent range (weighted towards small exponents), with mantissas at the
/// ends of the range, zero, and now and then any 32 digits at all.
fn random_word(r: &mut Rng, emin: i32, emax: i32) -> u32 {
    let m: i32 = match r.below(10) {
        0 => 1 << 20,
        1 => (1 << 21) - 1 - r.below(4) as i32,
        2 => (1 << 20) + r.below(4) as i32,
        _ => (1 << 20) + r.below(1 << 20) as i32,
    };
    let m = if r.below(2) == 0 {
        m
    } else if m == 1 << 20 {
        -(1 << 21)
    } else {
        -m
    };
    let e = emin + r.below((emax - emin + 1) as u64) as i32;
    match r.below(40) {
        0 => 0,
        1 => r.next() as u32,
        _ => george::Num {
            mantissa: m,
            exponent: e,
        }
        .to_word(),
    }
}

/// A machine with the G-program `wait <src>` translated and stopped at the
/// wait, so that the number store can be set before each run.
struct Rig {
    m: george::deuce::Machine,
}

impl Rig {
    fn new(src: &str) -> Rig {
        let blocks = g::george_blocks(root()).unwrap();
        let mut m = g::boot_george(&blocks).unwrap();
        let prog: Vec<u8> = george::parse(&format!("0 wait ; {src}"))
            .unwrap()
            .symbols
            .iter()
            .map(|s| s.byte())
            .collect();
        let mut deck = vec![g::gpic(&blocks).unwrap()];
        deck.extend(g::program_cards(&prog, [1, 9, 17, 25]));
        m.load_cards(deck);
        m.initial_input();
        loop {
            match m.step() {
                None => {}
                Some(george::deuce::Outcome::AwaitSingleShot) => break,
                Some(o) => panic!("{o:?} before the wait"),
            }
        }
        Rig { m }
    }

    /// Set a, b, ... (DL12 m.c. 26 on) to `w` and go on from the wait; the
    /// words of the next three variables after them, and the stops resumed
    /// by a single shot.
    fn run(&self, w: &[u32]) -> (Vec<u32>, Vec<String>, george::deuce::Outcome) {
        let mut m = self.m.clone();
        for (i, &x) in w.iter().enumerate() {
            m.dl[12][26 + i] = x;
        }
        for i in 26 + w.len()..32 {
            m.dl[12][i] = 0x5555_5555;
        }
        m.single_shot_after(g::OPERATOR_DELAY);
        m.step();
        let op = g::Operator {
            single_shot: true,
            ..Default::default()
        };
        let (o, seen) = g::operate(&mut m, &op, 3_000_000);
        let stops = seen
            .iter()
            .filter_map(|c| match c {
                g::Console::Resumed { indication, .. } => Some(indication.clone()),
                _ => None,
            })
            .collect();
        (m.dl[12][26 + w.len()..].to_vec(), stops, o)
    }
}

/// The model's outcome against GEORGE's: the same word with no stop, or
/// exactly one stop, the one the model names (its indication in the stop
/// table), and the same word stored after the single shot.
fn agrees(want: george::functions::Outcome, got: u32, seen: &[String]) -> bool {
    use george::functions::Outcome as F;
    match want {
        F::Word(x) => seen.is_empty() && got == x,
        F::Stop { kind, resumed } => {
            seen.len() == 1
                && george::machine::indication_is(&george::machine::fault_stop(kind), &seen[0])
                && got == resumed
        }
    }
}

fn check_unary(name: &str, src: &str, emin: i32, emax: i32, model: impl Fn(u32) -> george::functions::Outcome) {
    let rig = Rig::new(src);
    let cases: usize = std::env::var("CASES").ok().and_then(|s| s.parse().ok()).unwrap_or(300);
    let seed: u64 = std::env::var("SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0x9e3779b97f4a7c15);
    let mut r = Rng(seed);
    let (mut agree, mut differ, mut stops) = (0, 0, 0);
    for _ in 0..cases {
        let w = random_word(&mut r, emin, emax);
        let (out, seen, o) = rig.run(&[w]);
        assert_eq!(o, george::deuce::Outcome::ReaderEmpty, "{w:08x}");
        let got = out[0];
        let stopped = !seen.is_empty();
        let want = model(w);
        let ok = agrees(want, got, &seen);
        if stopped {
            stops += 1;
        }
        if ok {
            agree += 1;
        } else {
            differ += 1;
            if differ <= 30 {
                println!(
                    "{name} {w:08x} ({:?}): GEORGE {} model {want:x?}",
                    george::Num::from_word(w),
                    if stopped {
                        format!("stop {seen:?} {got:08x}")
                    } else {
                        format!("{got:08x} {:?}", george::Num::from_word(got))
                    },
                );
            }
        }
    }
    println!("{name}: agree {agree} differ {differ} (stops {stops})");
    assert_eq!(differ, 0);
}

#[test]
fn sqrt_word_for_word() {
    check_unary("√", "a √ (b) ;", -512, 511, george::functions::sqrt);
}

#[test]
fn sin_word_for_word() {
    check_unary("sin", "a sin (b) ;", -30, 25, |w| george::functions::sin_cos(w, true));
}

#[test]
fn cos_word_for_word() {
    check_unary("cos", "a cos (b) ;", -40, 25, |w| george::functions::sin_cos(w, false));
}

#[test]
fn log_word_for_word() {
    check_unary("log", "a log (b) ;", -512, 511, george::functions::log);
}

#[test]
fn exp_word_for_word() {
    check_unary("exp", "a exp (b) ;", -40, 10, george::functions::exp);
}

const FIXED_POW: &[(f64, f64)] = &[
    (1.0, 2.5),
    (1.0, -3.0),
    (-1.0, 3.0),
    (-1.0, 2.0),
    (-2.0, 3.0),
    (-0.5, 4.0),
    (3.0, 0.0),
    (-3.0, 0.0),
    (0.0, 2.0),
    (0.0, 0.0),
    (0.0, -2.0),
    (-3.0, 0.25),
    (-3.0, -0.25),
    (-3.0, 1e-3),
    (-3.0, 4194304.0),
    (-3.0, 3145728.0),
    (-1.5, 3e7),
    (-3.0, -3e7),
    (2.0, 511.0),
    (2.0, 512.0),
    (2.0, -512.0),
    (2.0, -513.0),
    (10.0, -154.0),
    (-3.0, 1e-30),
    (-3.0, -1e-30),
    (-3.0, 1e-7),
    (-3.0, 1e-6),
    (-3.0, 3e-6),
    (-4.0, 0.5),
];

fn check_binary(
    name: &str,
    src: &str,
    fixed: &[(u32, u32)],
    operands: impl Fn(&mut Rng) -> (u32, u32),
    model: impl Fn(u32, u32) -> george::functions::Outcome,
) {
    let rig = Rig::new(src);
    let cases: usize = std::env::var("CASES").ok().and_then(|s| s.parse().ok()).unwrap_or(300);
    let seed: u64 = std::env::var("SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0x9e3779b97f4a7c15);
    let mut r = Rng(seed);
    let (mut agree, mut differ, mut stops) = (0, 0, 0);
    for i in 0..cases + fixed.len() {
        let (a, b) = if i < fixed.len() { fixed[i] } else { operands(&mut r) };
        let (out, seen, o) = rig.run(&[a, b]);
        assert_eq!(o, george::deuce::Outcome::ReaderEmpty, "{a:08x} {b:08x}");
        let got = out[0];
        let stopped = !seen.is_empty();
        let want = model(a, b);
        let ok = agrees(want, got, &seen);
        stops += stopped as usize;
        if ok {
            agree += 1;
        } else {
            differ += 1;
            if differ <= 30 {
                println!(
                    "{name} {a:08x} {b:08x} ({:?} {:?}): GEORGE {} model {want:x?}",
                    george::Num::from_word(a),
                    george::Num::from_word(b),
                    if stopped {
                        format!("stop {seen:?} {got:08x}")
                    } else {
                        format!("{got:08x} {:?}", george::Num::from_word(got))
                    },
                );
            }
        }
    }
    println!("{name}: agree {agree} differ {differ} (stops {stops})");
    assert_eq!(differ, 0);
}

#[test]
fn pow_word_for_word() {
    let w = |x: f64| george::Num::from_f64(x).unwrap().to_word();
    let fixed: Vec<(u32, u32)> = FIXED_POW.iter().map(|&(a, b)| (w(a), w(b))).collect();
    check_binary(
        "pow",
        "a b pow (c) ;",
        &fixed,
        |r| {
            let a = random_word(r, -30, 30);
            let b = match r.below(3) {
                // a whole number, for negative bases
                0 => george::Num::from_f64(r.below(40) as f64 - 20.0).unwrap().to_word(),
                _ => random_word(r, -12, 8),
            };
            (a, b)
        },
        george::functions::pow,
    );
}

#[test]
fn rem_word_for_word() {
    let w = |x: f64| george::Num::from_f64(x).unwrap().to_word();
    let fixed: Vec<(u32, u32)> = [
        (7.0, 2.5),
        (7.75, 3.0),
        (-7.75, 3.0),
        (-1e-9, 3.0),
        (5.0, 0.3),
        (5.0, 0.5),
        (5.0, -0.5),
        (1e30, 7.0),
        (-1e30, 7.0),
        (6.0, 1e10),
        (0.0, 3.0),
        (3.0, 3.0),
        (-1e-9, 3.3),
        (-1e-9, 3.0),
        (-0.5, 33554432.0),
        (-0.25, 3.0),
        (0.25, 3.0),
    ]
    .iter()
    .map(|&(a, b)| (w(a), w(b)))
    .collect();
    check_binary(
        "rem",
        "a b rem (c) ;",
        &fixed,
        |r| {
            let a = random_word(r, -30, 60);
            let b = match r.below(3) {
                0 => george::Num::from_f64(r.below(40) as f64 - 5.0).unwrap().to_word(),
                1 => random_word(r, -3, 20),
                _ => random_word(r, -40, 40),
            };
            (a, b)
        },
        george::functions::rem,
    );
}

/// Not a check: the whole number "fetch integer" (12/6) makes of b in
/// `a b rem`: the word `7_1 21-19` stores in 19_3, for words from B (comma
/// separated hex) or a sweep of exponents.
#[test]
#[ignore]
fn fetch_integer_words() {
    let rig = Rig::new("a b rem (c) ;");
    let words: Vec<u32> = match std::env::var("B") {
        Ok(s) => s.split(',').map(|x| u32::from_str_radix(x, 16).unwrap()).collect(),
        Err(_) => {
            let mut v = vec![];
            for e in -40..60 {
                for m in [0x100000, 0x1abcde, 0x1fffff, -0x100001, -0x1abcde, -0x200000] {
                    v.push(
                        george::Num {
                            mantissa: m,
                            exponent: e,
                        }
                        .to_word(),
                    );
                }
            }
            v
        }
    };
    for b in words {
        let mut m = rig.m.clone();
        m.dl[12][26] = 0x60000003; // 6
        m.dl[12][27] = b;
        m.single_shot_after(g::OPERATOR_DELAY);
        m.trace_on();
        let mut got = None;
        for _ in 0..200_000 {
            if m.step().is_some() {
                break;
            }
            let st = m.trace.as_mut().unwrap().pop().unwrap();
            if st.from == Some((7, 27)) && st.track == Some(15 * 16 + 1) {
                got = Some(st.value);
                break;
            }
        }
        let n = george::Num::from_word(b);
        println!(
            "{b:08x} {:>9} {:>5} {:e} -> {:?}",
            n.mantissa,
            n.exponent,
            n.to_f64(),
            got.map(|v| v as i32)
        );
    }
}

#[test]
fn neg_word_for_word() {
    check_unary("neg", "a neg (b) ;", -512, 511, george::functions::neg);
}

#[test]
fn mod_word_for_word() {
    check_unary("mod", "a mod (b) ;", -512, 511, george::functions::modulus);
}

/// The repetition increment (`]` recalling `rep`, tracks 12/12-12/13, flow
/// diagrams p.27) against `machine::rep_increment`: `0, 1 rep (k) k (b) ;
/// a (k) ; ]` stores the increment of a in b on the second pass. Every
/// exponent 0-511 with random mantissas, and negative exponents, where
/// Hamblin's code goes astray (it must not come to the end of the program).
/// A (comma separated hex) gives the words instead, and prints them.
#[test]
fn rep_increment_word_for_word() {
    let rig = Rig::new("0, 1 rep (k) k (b) ; a (k) ; ]");
    let seed: u64 = std::env::var("SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0x9e3779b97f4a7c15);
    let mut r = Rng(seed);
    let words: Vec<u32> = match std::env::var("A") {
        Ok(s) => s.split(',').map(|x| u32::from_str_radix(x, 16).unwrap()).collect(),
        Err(_) => {
            let mut v = vec![0, 0x8000_0000, 0x7fff_fc00, 0x8000_0001];
            for e in -12..512 {
                let m = match r.below(4) {
                    0 => 0x1f_ffff,
                    1 => -0x20_0000,
                    _ => (0x10_0000 + r.below(0x10_0000) as i32) * if r.below(2) == 0 { 1 } else { -1 },
                };
                v.push(
                    george::Num {
                        mantissa: m,
                        exponent: e,
                    }
                    .to_word(),
                );
            }
            v
        }
    };
    let (mut agree, mut differ, mut astray) = (0, 0, 0);
    for a in words {
        let (out, seen, o) = rig.run(&[a]);
        let resumed = seen.len();
        let model = george::machine::rep_increment(a);
        let ok = match model {
            Ok(x) => o == george::deuce::Outcome::ReaderEmpty && resumed == 0 && out[0] == x,
            Err(Some(x)) => {
                o == george::deuce::Outcome::ReaderEmpty
                    && resumed == 1
                    && george::machine::indication_is(&george::machine::OVERFLOW, &seen[0])
                    && out[0] == x
            }
            Err(None) => {
                astray += 1;
                o != george::deuce::Outcome::ReaderEmpty
            }
        };
        if std::env::var("A").is_ok() || !ok {
            println!(
                "{a:08x} {:?} -> {o:?} {:08x} {:?} model {model:x?}",
                george::Num::from_word(a),
                out[0],
                george::Num::from_word(out[0]),
            );
        }
        if ok {
            agree += 1;
        } else {
            differ += 1;
        }
    }
    println!("rep increment: agree {agree} differ {differ} (astray {astray})");
    assert_eq!(differ, 0);
}

/// Addition, subtraction, multiplication and division near the ends of the
/// exponent range, with the single shot:
/// where standardise stops on 3 14-21x (overflow or underflow), the word
/// it stores on the single shot ("resumes with exponent halved") must be
/// `Overflow`'s word, as the interpreter resumes with it.
#[test]
fn arithmetic_resumes_word_for_word() {
    use george::functions::{Fault, Outcome as F};
    use george::number::Overflow;
    fn outcome(r: Result<george::Num, Overflow>) -> F {
        match r {
            Ok(n) => F::Word(n.to_word()),
            Err(Overflow(n)) => F::Stop {
                kind: Fault::Overflow,
                resumed: n.to_word(),
            },
        }
    }
    fn edge(r: &mut Rng) -> u32 {
        let e = match r.below(4) {
            0 => 511 - r.below(12) as i32,
            1 => -512 + r.below(12) as i32,
            2 => 255 + r.below(4) as i32 - (r.below(2) as i32) * 512,
            _ => r.below(1024) as i32 - 512,
        };
        let w = random_word(r, e, e);
        // no zero mantissa: a zero divisor stops on 1 4-24x instead
        if w >> 10 == 0 {
            1 << 30
        } else {
            w
        }
    }
    let n = |w: u32| george::Num::from_word(w);
    let w = |mantissa: i32, exponent: i32| george::Num { mantissa, exponent }.to_word();
    let fixed: Vec<(u32, u32)> = vec![
        (w(0x10_0000, -511), w(0x10_0000, -511)),
        (w(0x1f_ffff, 511), w(0x1f_ffff, 511)),
        (w(-0x20_0000, 511), w(-0x20_0000, 511)),
        (w(0x10_0000, -512), w(0x10_0000, 1)),
        (w(0x1f_ffff, -300), w(0x1f_ffff, -300)),
    ];
    check_binary(
        "+",
        "a b + (c) ;",
        &fixed,
        |r| (edge(r), edge(r)),
        |a, b| outcome(n(a).plus(n(b))),
    );
    check_binary(
        "-",
        "a b - (c) ;",
        &fixed,
        |r| (edge(r), edge(r)),
        |a, b| outcome(n(a).minus(n(b))),
    );
    check_binary(
        "×",
        "a b × (c) ;",
        &fixed,
        |r| (edge(r), edge(r)),
        |a, b| outcome(n(a).times(n(b))),
    );
    check_binary(
        "÷",
        "a b ÷ (c) ;",
        &fixed,
        |r| (edge(r), edge(r)),
        |a, b| outcome(n(a).over(n(b))),
    );
}
