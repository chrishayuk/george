//! Hamblin's punch translate (tracks 14/4-14/7, with S06 and M12 on 15/0;
//! flow diagrams pp.47-48) against the interpreter's `Card::of`: random
//! stored words, standardised and not, punched both ways, all twenty card
//! columns compared. See sources/deuce/RUNNING-GEORGE.md, finding 25.
//!
//!     cargo test --release --test punch_words                    300 words
//!     CASES=20000 SEED=5 cargo test --release --test punch_words
//!     SRC="1494469, 2 ÷ (a) ; a (P) ;" TRACKS=13,14,15 cargo test --release \
//!         --test punch_words trace_punch -- --ignored --nocapture

use std::path::Path;

use george::deuce::george as g;
use george::number::{Card, Num};

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Not a check: the instruction trace of tracks TRACKS (default 13,14,15),
/// and of instructions obeyed from TS13 (Q), with the registers after each.
#[test]
#[ignore]
fn trace_punch() {
    let blocks = g::george_blocks(root()).unwrap();
    let mut m = g::boot_george(&blocks).unwrap();
    let src = std::env::var("SRC").unwrap_or("1494469, 2 ÷ (a) ; a (P) ;".into());
    let tracks: Vec<u8> = std::env::var("TRACKS")
        .unwrap_or("13,14,15".into())
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
    for _ in 0..30_000_000 {
        if m.step().is_some() {
            break;
        }
        let st = m.trace.as_mut().unwrap().pop().unwrap();
        if st.from.is_none() || st.track.is_some_and(|t| tracks.contains(&(t / 16))) {
            println!("{st}");
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

/// G-program text whose value is exactly the standardised `n` (as in
/// tests/arith_words.rs): the odd part of the mantissa as an integer
/// numeral, scaled by exact divisions by powers of two, negated with `neg`.
fn exact(n: Num) -> String {
    if n.mantissa == 0 {
        return "0".into();
    }
    if n.mantissa == -(1 << 21) && n.exponent == 511 {
        // -2^511: 2^511 is out of range, so -2^510 doubled
        return format!(
            "{} dup +",
            exact(Num {
                mantissa: -(1 << 21),
                exponent: 510
            })
        );
    }
    let mut m = n.mantissa as i64;
    let mut e = n.exponent - 21;
    let neg = m < 0;
    m = m.abs();
    while m % 2 == 0 {
        m /= 2;
        e += 1;
    }
    let mut s = format!("{m}");
    // 2^80 at a time (2^20 squared twice), then 2^20 at a time, always by
    // ÷: × of the mantissa 2^21 - 1 is not exact on Hamblin's code
    while e.abs() >= 80 {
        if e > 0 {
            s += ", 1, 1048576 dup × dup × ÷ ÷";
        } else {
            s += ", 1048576 dup × dup × ÷";
        }
        e -= 80 * e.signum();
    }
    while e != 0 {
        let k = e.abs().min(20);
        if e > 0 {
            s += &format!(", 1, {} ÷ ÷", 1u64 << k);
        } else {
            s += &format!(", {} ÷", 1u64 << k);
        }
        e -= k * e.signum();
    }
    if neg {
        s += " neg";
    }
    s
}

/// G-program text whose stored word is exactly `w`, standardised or not:
/// an unstandardised mantissa (P31 = P32) is made by `&` (both 0) or `∨`
/// (both 1) of two standardised words with the same exponent.
fn word_src(w: u32) -> String {
    let n = Num::from_word(w);
    let (p32, p31) = (w >> 31 & 1, w >> 30 & 1);
    if w == 0 {
        "0".into()
    } else if p32 != p31 {
        exact(n)
    } else if p32 == 0 {
        // 01... & 10...
        let a = Num::from_word(w | 1 << 30);
        let b = Num::from_word(w | 1 << 31);
        format!("{}, {} &", exact(a), exact(b))
    } else {
        // 10... ∨ 01...
        let a = Num::from_word(w & !(1 << 30));
        let b = Num::from_word(w & !(1 << 31));
        format!("{}, {} ∨", exact(a), exact(b))
    }
}

/// A number with exactly seven significant figures, the seventh a 5, that
/// is held exactly: K × 10^-t with K = 5^t × u, u odd.
fn half(r: &mut Rng) -> Option<Num> {
    let t = r.below(9) as u32;
    let p = 5u64.pow(t);
    let lo = 1_000_000u64.div_ceil(p);
    let hi = 9_999_999 / p;
    if lo > hi {
        return None;
    }
    let u = (lo + r.below(hi - lo + 1)) | 1;
    let k = u * p;
    if k % 10 != 5 || !(1_000_000..=9_999_999).contains(&k) {
        return None;
    }
    // value k / 10^t = u / 2^t: mantissa u (odd) must fit in 21 digits
    if u >= 1 << 21 {
        return None;
    }
    let bits = 64 - u.leading_zeros() as i32;
    let m = (u << (21 - bits)) as i32;
    let n = Num {
        mantissa: if r.below(2) == 0 { m } else { -m },
        exponent: bits - t as i32,
    };
    // -1/2 standardises as -1 with the exponent one less
    Some(if n.mantissa == -(1 << 20) {
        Num {
            mantissa: -(1 << 21),
            exponent: n.exponent - 1,
        }
    } else {
        n
    })
}

fn random_word(r: &mut Rng) -> u32 {
    loop {
        let kind = r.below(10);
        let e = match r.below(4) {
            0 => r.below(1024) as i32 - 512,
            1 => r.below(41) as i32 - 20,
            _ => r.below(200) as i32 - 100,
        };
        let w = match kind {
            // standardised
            0..=5 => {
                let m = match r.below(8) {
                    0 => 1 << 20,
                    1 => (1 << 21) - 1 - r.below(4) as i32,
                    2 => (1 << 20) + r.below(4) as i32,
                    _ => (1 << 20) + r.below(1 << 20) as i32,
                };
                let m = if r.below(2) == 0 {
                    m
                } else if m == 1 << 20 && r.below(2) == 0 {
                    -(1 << 21)
                } else if m == 1 << 20 {
                    continue;
                } else {
                    -m
                };
                Num {
                    mantissa: m,
                    exponent: e,
                }
                .to_word()
            }
            // exact halves in the seventh figure
            6 | 7 => match half(r) {
                Some(n) => n.to_word(),
                None => continue,
            },
            // unstandardised: P31 = P32, any other digits
            _ => {
                let low = r.next() as u32 & 0x3fff_ffff & !0x3ff;
                let low = match r.below(4) {
                    0 => low >> r.below(22),
                    1 => 0,
                    _ => low,
                } & !0x3ff;
                let top = if r.below(2) == 0 { 0 } else { 0xc000_0000 };
                top | low | (e as u32 & 0x3ff)
            }
        };
        if w == 0 {
            continue;
        }
        return w;
    }
}

/// Hamblin's punch translate against `Card::of`, all twenty columns, on
/// random stored words. CASES (default 300) and SEED from the environment.
#[test]
fn punch_word_for_word() {
    let blocks = g::george_blocks(root()).unwrap();
    let m = g::boot_george(&blocks).unwrap();
    let cases: usize = std::env::var("CASES").ok().and_then(|s| s.parse().ok()).unwrap_or(300);
    let seed: u64 = std::env::var("SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0x2545f4914f6cdd1d);
    let mut r = Rng(seed);
    let fixed = [
        0u32,
        u32::MAX,
        0x3ff,
        0x8000_0000,
        0x4000_0000,
        0xc000_0000,
        0x0000_0400,
    ];
    let (mut agree, mut differ, mut bad) = (0, 0, 0);
    let mut words: Vec<u32> = fixed.to_vec();
    words.extend((0..cases).map(|_| random_word(&mut r)));
    for w in words {
        let src = format!("{} (a) ; a (P) ;", word_src(w));
        let prog: Vec<u8> = george::parse(&src).unwrap().symbols.iter().map(|s| s.byte()).collect();
        let (mm, run) = g::run_booted(m.clone(), &blocks, &prog, vec![], 3_000_000, false).unwrap();
        if mm.dl[12][26] != w {
            bad += 1;
            println!("word not built: {src}: {:08x} wanted {w:08x}", mm.dl[12][26]);
            continue;
        }
        let got = g::Punched::read(&run.punched[0]).unwrap();
        let want = Card::of(Num::from_word(w));
        let got_cols = format!(
            "{}{}{}{:09}",
            if got.negative { '-' } else { ' ' },
            got.mantissa.iter().map(|d| char::from(b'0' + d)).collect::<String>(),
            if got.exp_negative { '-' } else { ' ' },
            got.exponent
        );
        if got_cols == want.columns() {
            agree += 1;
        } else {
            differ += 1;
            println!(
                "{w:08x} ({}): GEORGE {got_cols} model {}",
                Num::from_word(w).to_f64(),
                want.columns()
            );
        }
    }
    println!("agree {agree} differ {differ} (words not built {bad})");
    assert_eq!((differ, bad), (0, 0));
}
