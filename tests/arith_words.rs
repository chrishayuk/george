//! Hamblin's + and - (tracks 12/3-12/5, flow diagrams pp.18-19) against the
//! interpreter's `Num::plus` and `Num::minus`, and his × and ÷ (p.20)
//! against `Num::times` and `Num::over`, word for word: the stored 22-digit
//! mantissa and the exponent must be the same. See
//! sources/deuce/RUNNING-GEORGE.md, findings 15, 26 and 27.
//!
//! The operands are built in the G-program by exact operations only (an
//! integer numeral, divisions by powers of two, `neg`), and the words
//! GEORGE stores for them are checked before the sums are compared.
//!
//!     cargo test --release --test arith_words                   2000 pairs
//!     CASES=20000 SEED=5 cargo test --release --test arith_words
//!     SRC="1, 3 ÷ 1 + (a) ;" cargo test --release --test arith_words \
//!         trace_registers -- --ignored --nocapture

use std::path::Path;

use george::deuce::george as g;
use george::number::Num;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Not a check: the instruction trace of the arithmetic tracks (12/x), and
/// of instructions obeyed from TS13 (Q), with TS13-16 and DS19-21 (each as
/// odd:even, that is high:low) after each.
#[test]
#[ignore]
fn trace_registers() {
    let blocks = g::george_blocks(root()).unwrap();
    let mut m = g::boot_george(&blocks).unwrap();
    let src = std::env::var("SRC").unwrap_or("1, 3 ÷ 1 + (a) ;".into());
    let prog = george::parse(&src).unwrap();
    let bytes: Vec<u8> = prog.symbols.iter().map(|s| s.byte()).collect();
    let mut deck = vec![g::gpic(&blocks).unwrap()];
    deck.extend(g::program_cards(&bytes, [1, 9, 17, 25]));
    m.trace_on();
    m.load_cards(deck);
    m.initial_input();
    for _ in 0..3_000_000 {
        if m.step().is_some() {
            break;
        }
        let st = m.trace.as_mut().unwrap().pop().unwrap();
        if st.from.is_none() || st.track.is_some_and(|t| t / 16 == 12) {
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

/// G-program text whose value is exactly `n`: the mantissa as an integer
/// numeral, scaled by exact powers of two, negated with `neg`.
fn exact(n: Num) -> String {
    if n.mantissa == 0 {
        return "0".into();
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
    while e != 0 {
        let k = e.abs().min(20);
        // × of the mantissa 2^21 - 1 by a power of two is not exact on
        // Hamblin's code (it gives 2^21), so scale up by dividing by 2^-k
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

fn random_operands(r: &mut Rng) -> (Num, Num) {
    let mant = |r: &mut Rng| -> i32 {
        let m = match r.below(8) {
            0 => 1 << 20,
            1 => (1 << 21) - 1 - r.below(4) as i32,
            2 => (1 << 20) + r.below(4) as i32,
            _ => (1 << 20) + r.below(1 << 20) as i32,
        };
        if r.below(2) == 0 {
            m
        } else if m == 1 << 20 && r.below(2) == 0 {
            -(1 << 21)
        } else {
            -m
        }
    };
    let ea = r.below(41) as i32 - 20;
    let d = match r.below(6) {
        0 => 0,
        1 => r.below(3) as i32,
        2 => 21 + r.below(3) as i32,
        3 => 15 + r.below(10) as i32,
        _ => r.below(30) as i32,
    } * if r.below(2) == 0 { 1 } else { -1 };
    let a = Num {
        mantissa: mant(r),
        exponent: ea,
    };
    let mut b = Num {
        mantissa: mant(r),
        exponent: ea + d,
    };
    match r.below(12) {
        // equal magnitudes and near-cancellation
        0 => {
            b = Num {
                mantissa: -a.mantissa,
                exponent: ea,
            }
        }
        1 => b = a,
        2 => {
            b = Num {
                mantissa: -a.mantissa + r.below(5) as i32 - 2,
                exponent: ea,
            }
        }
        3 => {
            b = Num {
                mantissa: (r.below(2) as i32 * 2 - 1) * (a.mantissa.abs() + r.below(3) as i32 - 1),
                exponent: ea + 1,
            }
        }
        4 => b = Num::ZERO,
        _ => {}
    }
    if b.mantissa != 0 && b.mantissa != -(1 << 21) && !(1 << 20..1 << 21).contains(&b.mantissa.abs()) {
        let m = if b.mantissa.abs() < 1 << 20 {
            (1 << 20) + 5
        } else {
            (1 << 21) - 1
        };
        b.mantissa = if b.mantissa < 0 { -m } else { m };
    }
    // -1/2 standardises as -1 with the exponent one less
    let std = |n: Num| {
        if n.mantissa == -(1 << 20) {
            Num {
                mantissa: -(1 << 21),
                exponent: n.exponent - 1,
            }
        } else {
            n
        }
    };
    let (a, b) = (std(a), std(b));
    if r.below(20) == 0 {
        return (b, a);
    }
    (a, b)
}

/// Hamblin's + and - against `Num::add` and `Num::sub`, word for word, on
/// random operands, and on every pair of a few unstandardised words.
/// CASES (default 2000) and SEED from the environment.
#[test]
fn add_and_subtract_word_for_word() {
    let blocks = g::george_blocks(root()).unwrap();
    let m = g::boot_george(&blocks).unwrap();
    let cases: usize = std::env::var("CASES").ok().and_then(|s| s.parse().ok()).unwrap_or(2000);
    let seed: u64 = std::env::var("SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0x9e3779b97f4a7c15);
    let mut r = Rng(seed);
    let (mut agree, mut differ, mut bad) = (0, 0, 0);
    // [zero, exponent down (cancellation), same, up (carry)], and |exponent difference| > 23
    let mut kinds = [0; 5];
    // unstandardised words from the logical operators, and truth
    let fixed = [
        "0 ~",
        "2, 3 = ~",
        "3, 3 =",
        "3, 3 = ~",
        "5, 6 &",
        "5, 6 ∨",
        "0.75 ~",
        "1000 ~",
        "0.001, 3 ∨",
    ];
    for i in 0..cases + fixed.len() * fixed.len() {
        let (a_src, b_src, intended) = if i < cases {
            let (a, b) = random_operands(&mut r);
            (exact(a), exact(b), Some((a, b)))
        } else {
            let j = i - cases;
            (
                fixed[j / fixed.len()].to_string(),
                fixed[j % fixed.len()].to_string(),
                None,
            )
        };
        let src = format!("{a_src} (a) ; {b_src} (b) ; a b + (c) ; a b - (d) ;");
        let prog: Vec<u8> = george::parse(&src).unwrap().symbols.iter().map(|s| s.byte()).collect();
        let (mm, _) = g::run_booted(m.clone(), &blocks, &prog, vec![], 3_000_000, false).unwrap();
        let w = |i: usize| Num::from_word(mm.dl[12][i]);
        let (a, b) = intended.unwrap_or((w(26), w(27)));
        if w(26) != a || w(27) != b {
            bad += 1;
            println!("operands not built: {src}: {:?} {:?} wanted {a:?} {b:?}", w(26), w(27));
            continue;
        }
        if (a.exponent - b.exponent).abs() > 23 {
            kinds[4] += 1;
        }
        for (got, want, op) in [(w(28), a.plus(b), "+"), (w(29), a.minus(b), "-")] {
            let want = want.unwrap_or(Num {
                mantissa: 0x7fff_ffff,
                exponent: 0,
            });
            let top = a.exponent.max(b.exponent);
            kinds[if got.is_zero() {
                0
            } else {
                (got.exponent - top).signum() + 2
            } as usize] += 1;
            if got == want {
                agree += 1;
            } else {
                differ += 1;
                println!("{a:?} {op} {b:?}: GEORGE {got:?} model {want:?}");
            }
        }
    }
    println!(
        "agree {agree} differ {differ} (operands not built {bad}); results zero {}, renormalised down {}, same exponent {}, up {}; pairs more than 23 apart {}",
        kinds[0], kinds[1], kinds[2], kinds[3], kinds[4]
    );
    assert_eq!((differ, bad), (0, 0));
}

/// Hamblin's × and ÷ against `Num::times` and `Num::over`, word for word,
/// on the same random operands as + and -, and on every pair of a few
/// unstandardised words. CASES (default 2000) and SEED from the environment.
#[test]
fn multiply_and_divide_word_for_word() {
    let blocks = g::george_blocks(root()).unwrap();
    let m = g::boot_george(&blocks).unwrap();
    let cases: usize = std::env::var("CASES").ok().and_then(|s| s.parse().ok()).unwrap_or(2000);
    let seed: u64 = std::env::var("SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0x9e3779b97f4a7c15);
    let mut r = Rng(seed);
    let (mut agree, mut differ, mut bad) = ([0; 2], [0; 2], 0);
    let fixed = [
        "0 ~",
        "2, 3 = ~",
        "3, 3 =",
        "3, 3 = ~",
        "5, 6 &",
        "5, 6 ∨",
        "0.75 ~",
        "1000 ~",
        "0.001, 3 ∨",
        "2097151",
        "2097151 neg",
        "1048577 neg",
    ];
    for i in 0..cases + fixed.len() * fixed.len() {
        let (a_src, b_src, intended) = if i < cases {
            let (a, b) = random_operands(&mut r);
            (exact(a), exact(b), Some((a, b)))
        } else {
            let j = i - cases;
            (
                fixed[j / fixed.len()].to_string(),
                fixed[j % fixed.len()].to_string(),
                None,
            )
        };
        let src = format!("{a_src} (a) ; {b_src} (b) ; a b × (c) ; b 0 = 1 ↑ a b ÷ (d) ; * 1 ;");
        let prog: Vec<u8> = george::parse(&src).unwrap().symbols.iter().map(|s| s.byte()).collect();
        let (mm, _) = g::run_booted(m.clone(), &blocks, &prog, vec![], 3_000_000, false).unwrap();
        let w = |i: usize| Num::from_word(mm.dl[12][i]);
        let (a, b) = intended.unwrap_or((w(26), w(27)));
        if w(26) != a || w(27) != b {
            bad += 1;
            println!("operands not built: {src}: {:?} {:?} wanted {a:?} {b:?}", w(26), w(27));
            continue;
        }
        let sentinel = Num {
            mantissa: 0x7fff_ffff,
            exponent: 0,
        };
        let mut checks = vec![(0, w(28), a.times(b).unwrap_or(sentinel), "×")];
        if !b.is_zero() {
            checks.push((1, w(29), a.over(b).unwrap_or(sentinel), "÷"));
        }
        for (k, got, want, op) in checks {
            if got == want {
                agree[k] += 1;
            } else {
                differ[k] += 1;
                println!("{a:?} {op} {b:?}: GEORGE {got:?} model {want:?}");
            }
        }
    }
    println!(
        "× agree {} differ {}; ÷ agree {} differ {} (operands not built {bad})",
        agree[0], differ[0], agree[1], differ[1]
    );
    assert_eq!((differ, bad), ([0, 0], 0));
}
