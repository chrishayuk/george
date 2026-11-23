//! Hamblin's numeral conversion (translator, tracks 13/6-13/7, flow
//! diagram p.12) and read translate (tracks 14/11-14/14 with S06 and M12,
//! pp.44-46) against the interpreter's `numeral_word` and `read_word`,
//! word for word. See sources/deuce/RUNNING-GEORGE.md, findings 28-29.
//!
//!     cargo test --release --test numeral_words        400 numerals, 300 cards
//!     CASES=40000 SEED=7 cargo test --release --test numeral_words
//!     SRC="939.9729 (a) ;" TRACKS=13/6,13/7 cargo test --release \
//!         --test numeral_words trace_numeral -- --ignored --nocapture
//!     SRC="R (a) ;" DATA=939.973 TRACKS=14/11,14/12,14/13,14/14,15/0 \
//!         cargo test --release --test numeral_words trace_numeral -- --ignored --nocapture
//!     NUMS="1..5 12345678901" cargo test --release --test numeral_words \
//!         show_numerals -- --ignored --nocapture

use std::path::Path;

use george::deuce::george as g;
use george::deuce::loader::BlockImage;
use george::number::{numeral_word, Card, Num, NumeralSym};

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn track_list(s: &str) -> Vec<u8> {
    s.split(',')
        .map(|t| {
            let (a, b) = t.split_once('/').unwrap();
            a.parse::<u8>().unwrap() * 16 + b.parse::<u8>().unwrap()
        })
        .collect()
}

/// Not a check: the instruction trace of tracks TRACKS (default 13/6,13/7),
/// with the registers after each. SRC is the G-program, DATA optional data
/// cards in ordinary notation, one per comma.
#[test]
#[ignore]
fn trace_numeral() {
    let blocks = g::george_blocks(root()).unwrap();
    let mut m = g::boot_george(&blocks).unwrap();
    let src = std::env::var("SRC").unwrap_or("939.9729 (a) ;".into());
    let tracks = track_list(&std::env::var("TRACKS").unwrap_or("13/6,13/7".into()));
    let prog = george::parse(&src).unwrap();
    let bytes: Vec<u8> = prog.symbols.iter().map(|s| s.byte()).collect();
    let mut deck = vec![g::gpic(&blocks).unwrap()];
    deck.extend(g::program_cards(&bytes, [1, 9, 17, 25]));
    if let Ok(d) = std::env::var("DATA") {
        for x in d.split(',') {
            deck.push(g::data_card(&george::number::Card::from_decimal(x).unwrap()));
        }
    }
    m.trace_on();
    m.load_cards(deck);
    m.initial_input();
    for _ in 0..30_000_000 {
        if let Some(o) = m.step() {
            println!("stop {o:?}");
            break;
        }
        let st = m.trace.as_mut().unwrap().pop().unwrap();
        if st.track.is_some_and(|t| tracks.contains(&t)) {
            println!("{st}");
            println!(
                "      13={:08x} 14={:08x} 15={:08x} 16={:08x} 19={:08x}:{:08x} 20={:08x}:{:08x} 21={:08x}:{:08x}",
                m.ts13, m.ts14, m.ts15, m.ts16, m.ds[0][1], m.ds[0][0], m.ds[1][1], m.ds[1][0], m.ds[2][1], m.ds[2][0]
            );
        }
    }
}

/// Not a check: numerals in NUMS (space separated; any number of digits
/// and points) stored by Hamblin's code, with `numeral_word`'s word.
#[test]
#[ignore]
fn show_numerals() {
    let blocks = g::george_blocks(root()).unwrap();
    let m = g::boot_george(&blocks).unwrap();
    let nums = std::env::var("NUMS").unwrap_or("939.9729 1 0 .5 5. 999999999".into());
    for t in nums.split_whitespace() {
        // the numeral's codes directly, so that the text form's checks
        // (one point, nine digits) do not apply
        let mut prog: Vec<u8> = numeral_syms(t)
            .iter()
            .map(|s| match s {
                NumeralSym::Digit(d) => george::Sym::Digit(*d).byte(),
                NumeralSym::Point => george::symbols::POINT.byte(),
            })
            .collect();
        prog.extend(george::parse("(a) ;").unwrap().symbols.iter().map(|s| s.byte()));
        let (mm, run) = g::run_booted(m.clone(), &blocks, &prog, vec![], 3_000_000, false).unwrap();
        let w = mm.dl[12][26];
        let n = george::number::Num::from_word(w);
        let model = numeral_word(numeral_syms(t));
        println!(
            "{t}: {w:08x} {:?} {} ({:?} {}); model {model:08x}",
            n,
            n.to_f64(),
            run.outcome,
            g::indication(run.count)
        );
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

fn numeral_syms(t: &str) -> Vec<NumeralSym> {
    t.chars()
        .map(|c| match c {
            '.' => NumeralSym::Point,
            d => NumeralSym::Digit(d.to_digit(10).unwrap() as u8),
        })
        .collect()
}

/// A random numeral: one to nine digits, the point (or none) anywhere,
/// zeros favoured, sometimes all nines or the largest nine digits.
fn random_numeral(r: &mut Rng) -> String {
    let n = 1 + r.below(9) as usize;
    let mut digits: Vec<char> = (0..n)
        .map(|_| match r.below(10) {
            0..=1 => '0',
            2 => '9',
            _ => char::from(b'0' + r.below(10) as u8),
        })
        .collect();
    match r.below(10) {
        0 => digits.iter_mut().for_each(|d| *d = '9'),
        1 => digits.iter_mut().for_each(|d| *d = '0'),
        _ => {}
    }
    let mut s: String = digits.into_iter().collect();
    match r.below(4) {
        0 => {}
        _ => {
            let at = r.below(n as u64 + 1) as usize;
            s.insert(at, '.');
        }
    }
    s
}

/// The words Hamblin's translator makes of the numerals `nums`, sixteen to
/// a G-program, each stored in one of the variables a..p (`8 (b)`), which
/// are on DL12 m.c. 26-31 and 0-9.
fn hamblin_words(m: &george::deuce::Machine, blocks: &[BlockImage], nums: &[String]) -> Vec<u32> {
    let mut out = Vec::new();
    for chunk in nums.chunks(16) {
        let src: String = chunk
            .iter()
            .enumerate()
            .map(|(i, t)| format!("{t} ({}) ; ", george::symbols::LETTERS[i]))
            .collect::<String>();
        let prog: Vec<u8> = george::parse(&src).unwrap().symbols.iter().map(|s| s.byte()).collect();
        let (mm, run) = g::run_booted(m.clone(), blocks, &prog, vec![], 5_000_000, false).unwrap();
        assert_eq!(
            run.outcome,
            george::deuce::Outcome::ReaderEmpty,
            "{src} {}",
            g::indication(run.count)
        );
        for i in 0..chunk.len() {
            out.push(mm.dl[12][(26 + i) % 32]);
        }
    }
    out
}

/// Hamblin's numeral conversion against `numeral_word`, word for word, on
/// random numerals of one to nine digits with the point anywhere (or
/// none), and fixed ones. CASES (default 400) and SEED from the
/// environment.
#[test]
fn numerals_word_for_word() {
    let blocks = g::george_blocks(root()).unwrap();
    let m = g::boot_george(&blocks).unwrap();
    let cases: usize = std::env::var("CASES").ok().and_then(|s| s.parse().ok()).unwrap_or(400);
    let seed: u64 = std::env::var("SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0x2545f4914f6cdd1d);
    let mut r = Rng(seed);
    let fixed = [
        "0",
        ".",
        ".0",
        "0.",
        "000000000",
        "1",
        "5.",
        ".5",
        "939.9729",
        "999999999",
        "99999999.9",
        ".999999999",
        "999.999999",
        ".000000001",
        "000000001",
        "4194303",
        "4194304",
        "2097151",
        "2097152",
        "8388607",
        "123456789",
        "0.1",
        "1.1",
        "3.14159265",
        "100000000",
        "1.00000000",
        "0.00000001",
        ".123456789",
        "2.5",
    ];
    let mut nums: Vec<String> = fixed.iter().map(|s| s.to_string()).collect();
    nums.extend((0..cases).map(|_| random_numeral(&mut r)));
    let got = hamblin_words(&m, &blocks, &nums);
    let (mut agree, mut differ) = (0, 0);
    for (t, w) in nums.iter().zip(got) {
        let want = numeral_word(numeral_syms(t));
        if w == want {
            agree += 1;
        } else {
            differ += 1;
            println!(
                "{t}: GEORGE {w:08x} ({}) model {want:08x} ({})",
                Num::from_word(w).to_f64(),
                Num::from_word(want).to_f64()
            );
        }
    }
    println!("agree {agree} differ {differ}");
    assert_eq!(differ, 0);
}

/// A random data card: six figures (leading zeros allowed: the reader
/// takes columns 2-7 as they are punched), either sign, exponent -999..999,
/// mostly small.
fn random_card(r: &mut Rng) -> Card {
    let digits = match r.below(10) {
        0 => r.below(1_000_000) as u32,
        1 => [0, 1, 999_999, 100_000, 500_000, 999_990][r.below(6) as usize],
        _ => 100_000 + r.below(900_000) as u32,
    };
    let exponent = match r.below(10) {
        0 => r.below(1999) as i32 - 999,
        1..=2 => r.below(321) as i32 - 160,
        _ => r.below(41) as i32 - 20,
    };
    Card {
        negative: r.below(2) == 0,
        digits,
        exponent,
    }
}

/// What Hamblin's read translate makes of each card: the word, or the
/// indication it stopped on.
fn hamblin_read(m: &george::deuce::Machine, blocks: &[BlockImage], cards: &[Card]) -> Vec<Result<u32, String>> {
    let mut out = Vec::new();
    for chunk in cards.chunks(16) {
        let src: String = (0..chunk.len())
            .map(|i| format!("R ({}) ; ", george::symbols::LETTERS[i]))
            .collect();
        let prog: Vec<u8> = george::parse(&src).unwrap().symbols.iter().map(|s| s.byte()).collect();
        let data = chunk.iter().map(g::data_card).collect();
        let (mm, run) = g::run_booted(m.clone(), blocks, &prog, data, 20_000_000, false).unwrap();
        if run.outcome == george::deuce::Outcome::ReaderEmpty {
            out.extend((0..chunk.len()).map(|i| Ok(mm.dl[12][(26 + i) % 32])));
        } else {
            // one at a time, to see which card stopped it
            for c in chunk {
                let prog: Vec<u8> = george::parse("R (a) ;")
                    .unwrap()
                    .symbols
                    .iter()
                    .map(|s| s.byte())
                    .collect();
                let (mm, run) =
                    g::run_booted(m.clone(), blocks, &prog, vec![g::data_card(c)], 20_000_000, false).unwrap();
                out.push(if run.outcome == george::deuce::Outcome::ReaderEmpty {
                    Ok(mm.dl[12][26])
                } else {
                    Err(format!("{:?} {}", run.outcome, g::indication(run.count)))
                });
            }
        }
    }
    out
}

/// Hamblin's read translate against `read_word`, word for word, on random
/// data cards and fixed ones. CASES (default 300) and SEED.
#[test]
fn read_word_for_word() {
    let blocks = g::george_blocks(root()).unwrap();
    let m = g::boot_george(&blocks).unwrap();
    let cases: usize = std::env::var("CASES").ok().and_then(|s| s.parse().ok()).unwrap_or(300);
    let seed: u64 = std::env::var("SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0x2545f4914f6cdd1d);
    let mut r = Rng(seed);
    let mut cards: Vec<Card> = [
        "939.973", "0", "1", "-1", "0.5", "1e-5", "999999", "2.5", "3.14159", "1e150", "1e-150",
    ]
    .iter()
    .map(|s| Card::from_decimal(s).unwrap())
    .collect();
    // zero with any exponent and sign; the largest and smallest exponents
    // that fit and the first that do not; negative values that lose their
    // sign (finding 28)
    for (negative, digits, exponent) in [
        (true, 0, 0),
        (false, 0, 999),
        (true, 0, -999),
        (false, 999_999, 999),
        (false, 1, -999),
        (false, 670_390, 153),
        (false, 670_391, 153),
        (true, 670_390, 153),
        (false, 372_000, -155),
        (false, 374_000, -155),
        (true, 391_032, -149),
        (true, 831_633, -112),
        (true, 643_445, -86),
    ] {
        cards.push(Card {
            negative,
            digits,
            exponent,
        });
    }
    cards.extend((0..cases).map(|_| random_card(&mut r)));
    let got = hamblin_read(&m, &blocks, &cards);
    let (mut agree, mut differ) = (0, 0);
    let mut stops = std::collections::BTreeMap::new();
    for (c, w) in cards.iter().zip(got) {
        let want = george::number::read_word(*c);
        let same = match (&w, &want) {
            (Ok(a), Ok(b)) => a == b,
            (Err(_), Err(_)) => true,
            _ => false,
        };
        if same {
            agree += 1;
            if let Err(e) = &w {
                *stops.entry(e.clone()).or_insert(0) += 1;
            }
        } else {
            differ += 1;
            println!("{}: GEORGE {:x?} model {:x?}", c.columns(), w, want);
        }
    }
    println!("agree {agree} differ {differ}; stops (both ways) {stops:?}");
    assert_eq!(differ, 0);
}
