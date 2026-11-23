//! GEORGE's functions word for word, as Hamblin's routines (tracks
//! 15/1-15/13, flow diagrams pp.32-38) form them: the same registers, the
//! same shifts and the same rounding, so that the stored word is the one
//! his code stores. Each was traced instruction by instruction on the
//! simulator (`tests/function_words.rs`, `trace_function`) and is written
//! down in sources/deuce/RUNNING-GEORGE.md (findings 18-24). m.c. here are
//! the coding sheets'.
//!
//! Registers are modelled as the machine holds them: DS21 as a high (odd)
//! and a low (even) 32-digit word, TS16 as a word. A GEORGE word is
//! exponent in P1-10 and mantissa in P11-32.

use crate::number::Num;

/// How a function ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The word stored in the accumulator.
    Word(u32),
    /// The routine stopped; `resumed` is the word it stores if the operator
    /// gives a single shot.
    Stop { kind: Fault, resumed: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// √ of a negative number: 1 4-24x.
    SqrtNegative,
    /// log of zero or a negative number: 1 30-21x.
    LogArgument,
    /// exp's result out of range: 7 30-21x.
    ExpRange,
    /// The exponent out of range in standardise: 3 14-21x.
    Overflow,
    /// pow of a negative number to a fractional power: 5 30-21x.
    FractionalPower,
    /// rem's divisor not positive: 1 30-16x (`7_31 30-16x`).
    RemDivisor,
}

/// The divider (PM §6.10): 2^31 A = QB + R. Only the quotient is used here.
fn quotient(a: u32, b: u32) -> u32 {
    crate::deuce::divide(a, b).0
}

/// Unpack one (12/3, flow diagrams p.18): the mantissa sign-extended in a
/// word (units of its last digit), and the exponent.
fn unpack(w: u32) -> (i32, i32) {
    let n = Num::from_word(w);
    (n.mantissa, n.exponent)
}

/// √ (track 15/11, sheet 21 block 1, flow diagrams p.37).
///
/// - Unpack; 20_2 := e·2^21 (`3_20 23-21`, kept in DS20 by `21-20`).
/// - `7_9 25-28` tests the exponent's last digit (TS14 = e·2^22 against
///   P23 from 7_5): even, `7_12 21-22(18)` doubles DS21 nine times; odd,
///   `7_13 21-22(16)` eight times. N := the high word, the mantissa as a
///   fraction at 30 binary places (P31 = 1.0), halved for an odd exponent;
///   `7_3 21-20` keeps it in 20_3.
/// - x := P31 (= 1.0, from 7_8 by `7_6 7-16`). Loop (`7_7 1-24`, the
///   divider: A = 21_3 = N, B = TS16 = x):
///   - `7_1 20-27`: N negative stops on `7_11 4-24x` (1 4-24x).
///   - `7_10 22-21`: q/2 into 21_2 (TCB on after a division: arithmetic
///     single-length shift); `7_14 16-23` takes x off; `7_16 27-22` adds P1.
///   - `7_18 22-28`: t = (q/2 - x + 1)/2 (arithmetic); zero ends the loop.
///   - Otherwise `7_21 22-21` (21_2 := t), `7_24 16-22` (+x), `7_26 21-16`
///     (x := x + t), `7_28 20-21` (21_3 := N again) and round to 7_7.
/// - At the end `7_20 16-21` (21_3 := x; 21_2 is 0 or 1), `7_23 4-24`
///   (TCB off), `7_25 22-21(18)` (DS21 halved nine times, exactly),
///   `7_15 29-23` (the double word plus 2^31: rounded, halves up),
///   `7_22 20-21` (21_2 := e·2^21), `7_31 21-22(20)` (DS21 up ten places:
///   the top ten digits of 21_2 come into P1-10), `7_27 29-23` (plus 2^31
///   again, so the exponent is e/2 rounded halves up; for e = -1 the carry
///   goes on into the mantissa), `7_30 21-16`: the word is 21_3, stored
///   without standardising.
/// - A negative argument resumed by a single shot stores TS16, which still
///   holds P31: 0.5.
pub fn sqrt(w: u32) -> Outcome {
    let (m, e) = unpack(w);
    let n: u32 = if e & 1 == 0 { (m << 9) as u32 } else { (m << 8) as u32 };
    if (n as i32) < 0 {
        return Outcome::Stop {
            kind: Fault::SqrtNegative,
            resumed: 1 << 30,
        };
    }
    let mut x: u32 = 1 << 30;
    let low = loop {
        let q = quotient(n, x);
        let half = (q >> 1) | (q & 1 << 31);
        let low = half.wrapping_sub(x).wrapping_add(1);
        let t = ((low as i32) >> 1) as u32;
        if t == 0 {
            break low;
        }
        x = t.wrapping_add(x);
    };
    let mut r: i64 = ((x as i32 as i64) << 32 | low as i64) >> 9;
    r = r.wrapping_add(1 << 31);
    let high = (r >> 32) as u32;
    let exp = (e << 21) as u32;
    let mut r: u64 = (u64::from(high) << 32 | u64::from(exp)) << 10;
    r = r.wrapping_add(1 << 31);
    Outcome::Word((r >> 32) as u32)
}

/// Standardise (12/3, flow diagrams p.18) of the double register
/// `r = high·2^32 + low`, the high word in units of the mantissa's last
/// digit, with exponent `e`, as `number::add_words` does it: a zero register
/// gives the zero word; otherwise P1 is added to the high word, the register
/// doubled (taking P1 off again) while P22 and P23 of the high word agree,
/// then halved; the mantissa is the high word. An exponent out of range
/// stops (3 14-21x, `3_8 26-27`), and is `Err` with the word stored on a
/// single shot: `3_11 14-21` puts e·2^21 where e·2^22 belongs, so the
/// exponent is halved.
fn standardise(mut r: i64, mut e: i32) -> Result<u32, u32> {
    const HIGH: i64 = 1 << 32;
    if r == 0 {
        return Ok(0);
    }
    r = r.wrapping_add(HIGH);
    loop {
        let h = (r >> 32) as i32;
        if (h >> 22) & 1 != (h >> 21) & 1 {
            break;
        }
        r = r.wrapping_mul(2).wrapping_sub(HIGH);
        e -= 1;
    }
    e += 1;
    r >>= 1;
    let high = (r >> 32) as u32;
    if !(crate::number::EXP_MIN..=crate::number::EXP_MAX).contains(&e) {
        return Err(high << 10 | ((e << 21) as u32) >> 22);
    }
    Ok(high << 10 | (e as u32 & 0x3ff))
}

/// The multiplier (`0-24`, LDM ch. 10): the unsigned 64-digit product of
/// 21_3 (`a`) and TS16 (`b`), as (high, low). The routines make it signed
/// themselves: the word in 21_2 three m.c. after `0-24` is TS16 if the
/// multiplier's top digit is 1 and 0 otherwise (LDM p.64), and they read it
/// there and take it off the high word once the product is complete. A
/// `21-23` from 21_2 to itself clears the low word and, by the sign
/// extension of a negative word, adds its top digit to the high word: the
/// product rounded to the high word, halves up.
fn mult_unsigned(a: u32, b: u32) -> (u32, u32) {
    let p = u64::from(a) * u64::from(b);
    ((p >> 32) as u32, p as u32)
}

/// sin/cos's Horner coefficients, DL8 (track 15/13) m.c. 13, 7, 5, 3, 1 and
/// 31 (= P31, 1.0), at 30 binary places: cos(πy/2) = Σ c_k y^2k.
const SINCOS: [u32; 6] = [
    0xffff_9c24, // -0.0000238083
    985_355,     // +0.0009176834
    (-22_401_135i32) as u32,
    272_375_348,
    (-1_324_675_860i32) as u32,
    1 << 30,
];

/// sin and cos (tracks 15/12-15/13, sheet 21 blocks 2-3, flow diagrams
/// p.38). Both are cos(πy/2) for y = 2x/π quarter-periods, sin with one
/// quarter-period taken off.
///
/// - `7_4 29-19` (sin) or `7_6 30-19` (cos) sets the flag 19_2 to P32 or 0;
///   unpack one.
/// - TS13 := (e + 10)·P17 (`7_5`, `7_9`). sin: if negative (e < -10),
///   `7_25 19-27` sends sin to `7_23`, which stores TS16, the argument word
///   as it was fetched: sin x = x. cos goes on.
/// - `7_24` takes 32·P17 off: (e - 22)·P17. `7_27 7-16`, `7_31 0-24`: the
///   mantissa m (21_3) times 2/π (15/12 m.c. 29, 0xa2f9836e, unsigned at 32 b.p.).
///   `7_2 21-15` reads 21_2 three m.c. on (TS16 if m < 0), and `7_16 15-23`
///   takes it off the high word: P = m·K, signed. `7_12 13-27`: e >= 22
///   goes to `7_15 30-16` and stores 0.
/// - `7_3`/`7_8`/`7_11` shift DS21 down 16 places at a time while the
///   counter is negative, `7_10`/`7_13` build the rest of the shift and
///   `3_28 13-0` obeys it: R = P >> (22 - e), exact (every shift starts in
///   an even m.c.). R is y at 31 binary places: the low word the fraction,
///   the high word whole quarter-periods.
/// - `8_29 19-22` (sin): R := R - 2^31 (P32 added double-length).
/// - `8_0 22-14`, `8_2 24-16`, `8_6 24-21`: TS14 := R >> 1 (low word, with
///   the high word's last digit h0 on top), s := TS16 := 21_3 := the low
///   word with its last digit cleared, a signed fraction at 31 b.p.
///   `8_9 0-24`, `8_12 21-13`, `8_16 13-25`, `8_22 21-23`, `8_24 13-23`:
///   z = s², at 30 b.p., the low word rounded in.
/// - Horner (`8_11` loads c from m.c. 13; `8_15 0-24`, `8_18 21-15`,
///   `8_17 15-23`, `8_20 21-22(4)`, `8_25 21-23`, then `Q 8-22` with the
///   next coefficient): acc := round(acc·z / 2^30) + c, the product's high
///   word after a 2-place shift, rounded by the low word's top digit.
/// - `8_14 30-13` (exponent 0), `8_19 24-15`, `8_23 26-27`: if h0 differs
///   from the top digit of the low word (the quadrant), `8_27 21-14`,
///   `8_28 24-23` negate acc. `8_26 22-21(18)` shifts DS21 down 9 places
///   (exact) and `7_17` enters standardise with exponent 0.
pub fn sin_cos(w: u32, sin: bool) -> Outcome {
    let (m, e) = unpack(w);
    if sin && e + 10 < 0 {
        return Outcome::Word(w);
    }
    if e - 22 >= 0 {
        return Outcome::Word(0);
    }
    const K: u32 = 0xa2f9_836e;
    let p: i64 = i64::from(m) * i64::from(K);
    let mut n = 22 - e;
    let mut r = p;
    while n > 16 {
        r >>= 16;
        n -= 16;
    }
    r >>= n;
    if sin {
        r = r.wrapping_sub(1 << 31);
    }
    let lo = r as u32;
    let h0 = (r >> 32) as u32 & 1;
    let u = lo & !1;
    let (mut z, l) = mult_unsigned(u, u);
    z = z.wrapping_add(l >> 31);
    if u >> 31 == 1 {
        z = z.wrapping_sub(u.wrapping_mul(2));
    }
    let mut acc = SINCOS[0];
    for &c in &SINCOS[1..] {
        let (mut h, l) = mult_unsigned(acc, z);
        if acc >> 31 == 1 {
            h = h.wrapping_sub(z);
        }
        let r2 = (u64::from(h) << 32 | u64::from(l)) << 2;
        let h2 = ((r2 >> 32) as u32).wrapping_add(r2 as u32 >> 31);
        acc = h2.wrapping_add(c);
    }
    if h0 != lo >> 31 {
        acc = 0u32.wrapping_sub(acc);
    }
    let r = i64::from(acc as i32) << 32 >> 9;
    Outcome::Word(standardise(r, 0).expect("sin/cos within range"))
}

/// M12, the DEUCE library's "signed multiplication of two numbers with shift
/// of two places" (track 15/0 m.c. 12-27, as GEORGE uses it from S06): the
/// signed 64-digit product of `a` and `b` (the unsigned product with each
/// operand taken off the high word when the other is negative, `6_16`-
/// `6_19`), doubled twice (`6_22 21-22(4)`), plus 2^31 (`6_27 29-23`): the
/// high word.
fn m12(a: u32, b: u32) -> u32 {
    let p = (i64::from(a as i32) * i64::from(b as i32)) as u64;
    (p.wrapping_mul(4).wrapping_add(1 << 31) >> 32) as u32
}

/// S06, the DEUCE library's "sum a series" (track 15/0 m.c. 0-31), as log
/// and exp call it: c[n] + u(c[n-1] + u(... + u·c[0]...)), Horner's rule
/// with M12 for each product, the coefficients at 30 binary places. The
/// coefficient fetched last (count 0) is added into TS14 only: the sum
/// (TS14) and the high word of the last product (21_3) are both returned,
/// and the caller's link (planted in 1_31) takes one or the other.
fn s06(coefficients: &[u32], u: u32) -> (u32, u32) {
    let mut acc = coefficients[0];
    let mut high = 0;
    for &c in &coefficients[1..] {
        high = m12(acc, u);
        acc = c.wrapping_add(high);
    }
    (acc, high)
}

/// log's series (DL11, track 15/5 m.c. 20 down to 12), log2(1 + u) at 30
/// binary places for 0 <= u < √2 - 1; the last is 0.
const LOG2_SERIES: [u32; 9] = [
    (-45_590_540i32) as u32,
    137_965_020,
    (-228_613_849i32) as u32,
    303_274_750,
    (-386_390_743i32) as u32,
    516_292_912,
    (-774_538_357i32) as u32,
    1_549_081_965,
    0,
];

/// log (tracks 15/3-15/4, sheet 18 blocks 2-3, flow diagrams p.32), with
/// S06 and M12 (track 15/0).
///
/// - Unpack; `Q 21-13`: TS13 := e·2^21. `8_0 30-21`, `8_12 21-22(20)`:
///   M := the mantissa shifted up ten places, 2f at 30 b.p. for the
///   fraction f.
/// - `8_31 21-28`: zero stops; `8_4 21-27`: negative stops (1 30-21x).
/// - `8_7 8-23`: t := M - √2 (8_9, at 30 b.p.). `8_10 21-27`:
///   - t >= 0: `8_20 8-25` E := e·2^21 - 2^20 (e - 1/2), and
///     u := M12(t, 1/√2) (8_19; S06 entered at its M12 with the link
///     `21-16`): u = 2f/√2 - 1.
///   - t < 0: `8_15 8-25` E := (e - 1)·2^21, `8_21 8-22` u := t + (√2 - 1)
///     (8_23): u = 2f - 1.
/// - `8_26 13-18`: QS18 := E. S06 over LOG2_SERIES with u (count 8, from
///   `8_1`, `8_6`, `8_13`): y := the last M12, log2(1 + u) at 30 b.p.
/// - `1_31 30-21`, `8_2 22-21(18)`: DS21 = y·2^32 shifted down 9 places
///   (exact); `8_27 18-22`: E added to the high word. DS21 is now log2 x
///   with 21 binary places in the high word.
/// - `7_6 21-28(2)`: a zero register is the zero word (log 1).
/// - `7_2 7-13`: c := 9·2^21; while the top two digits of the high word
///   agree (`7_10`, `7_13`, `7_15`), c -= 2^21 (`7_17`) and DS21 is doubled
///   (`7_22 21-22(2)`, exact).
/// - `7_18 7-16` TS16 := ln 2 (7_20, at 31 b.p.); `7_21 21-23` rounds the
///   low word into the high; `7_24 0-24` multiplies; `7_0 14-27`: if the
///   high word was negative before the rounding, `7_26 16-23` takes ln 2
///   off the product's high word. `7_1 22-21(16)`: down 8 places (exact).
/// - `7_12` enters standardise with exponent c.
pub fn log(w: u32) -> Outcome {
    const LN2: u32 = 0x58b9_0bfc;
    let (m, e) = unpack(w);
    let mut r = match log2_double(m as u32, e) {
        Some(r) => r,
        None => {
            return Outcome::Stop {
                kind: Fault::LogArgument,
                resumed: 0,
            }
        }
    };
    if r == 0 {
        return Outcome::Word(0);
    }
    let mut c = 9;
    loop {
        let h = (r >> 32) as u32;
        if (h >> 31) != (h >> 30) & 1 {
            break;
        }
        c -= 1;
        r = r.wrapping_mul(2);
    }
    let h = (r >> 32) as u32;
    let negative = (h as i32) < 0;
    let y = h.wrapping_add(r as u32 >> 31);
    let (mut ph, pl) = mult_unsigned(y, LN2);
    if negative {
        ph = ph.wrapping_sub(LN2);
    }
    let r = (i64::from(ph as i32) << 32 | i64::from(pl)) >> 8;
    Outcome::Word(standardise(r, c).expect("log within range"))
}

/// The log subroutine (track 15/4), shared by log and pow: log2 of the
/// mantissa word `m` (sign-extended, as 21_3 holds it) with exponent `e`, as
/// the double register DS21 with 21 binary places in the high word; None
/// for the stop (zero or negative after the 10-place shift).
fn log2_double(m: u32, e: i32) -> Option<i64> {
    const ROOT2: u32 = 1_518_500_250;
    const INV_ROOT2: u32 = 759_250_125;
    const ROOT2_LESS_1: u32 = 444_758_426;
    let big = m << 10;
    if big == 0 || (big as i32) < 0 {
        return None;
    }
    let t = big.wrapping_sub(ROOT2);
    let mut ex = (e << 21) as u32;
    let u = if (t as i32) >= 0 {
        ex = ex.wrapping_sub(1 << 20);
        m12(t, INV_ROOT2)
    } else {
        ex = ex.wrapping_sub(1 << 21);
        t.wrapping_add(ROOT2_LESS_1)
    };
    let (_, y) = s06(&LOG2_SERIES, u);
    let r: i64 = (i64::from(y as i32) << 32) >> 9;
    Some(r.wrapping_add(i64::from(ex as i32) << 32))
}

/// exp's series (DL11, track 15/5 m.c. 28 down to 21, read by S06; the
/// same words as punch translate's): 2^(-1/2)·(ln 2 / 2)^n / n! at 31
/// binary places, n = 7 down to 0.
const EXP_SERIES: [u32; 8] = [
    182,
    3670,
    63_271,
    912_811,
    10_535_376,
    91_196_005,
    526_272_083,
    1_518_500_250,
];

/// exp (tracks 15/6-15/7, sheet 19 blocks 2-3, flow diagrams p.33), with
/// S06 and M12 (track 15/0).
///
/// - Unpack; `Q 23-14(4)`, `8_5 23-13`: TS13 := e·2^17 (one unit of e is
///   P18). `8_7 21-14`: TS14 := m.
/// - `8_9 8-16` TS16 := log2 e (8_12, unsigned at 31 b.p.); `8_23 0-24`;
///   `7_2 14-27`: for m < 0, `8_1 16-23` takes it off the high word:
///   P = m·log2 e (m signed).
/// - `7_0`, `7_3 24-14(10)`, `7_14`, `7_16`: if digits 21 and 20 of the
///   high word differ, `7_19 22-21(2)` halves DS21 (exact); otherwise
///   `7_18 28-26(2)` takes one unit off e instead. Call the count e'.
///   `7_23 22-21(20)` halves DS21 ten times more.
/// - `7_13 7-26`: e' - 9. `7_17 13-27`: e' >= 9 stops on `7_20 30-21(2)x`
///   (7 30-21x). Otherwise `7_21 7-25` adds 16; while that is negative,
///   `7_29 22-21(32)` shifts 16 places and `7_31` adds 16 (`8_24`); then
///   `8_26 8-25` builds the rest of the shift, obeyed from TS13. In all,
///   DS21 = P >> (20 - e), exact: the high word is n = floor(x log2 e),
///   the low word the fraction f at 32 b.p.
/// - `8_31 27-22(2)`: P1 added to both words (n + 1, and the fraction's
///   last digit). `8_3 21-20`: 20_3 := n + 1. `7_5 30-21` clears the high
///   word; `7_7 29-22`: P32 added double-length (f - 1/2); `7_10 22-16`:
///   TS16 := DS21 >> 1, low word: v = (f - 1/2) at 31 b.p., read by M12 as
///   2(f - 1/2) at 30 b.p.
/// - S06 over EXP_SERIES with v (count 7, `7_12`, `8_15`, `8_18`): the sum
///   s = 2^(f - 1) at 31 b.p.; the link `14-21` (1_31) puts it in 21_3.
/// - `7_1 30-21`, `8_4 22-21(20)`: DS21 = s·2^32 >> 10 (exact);
///   `8_25 20-13`, `8_30 13-25(21)`: TS13 := (n + 1)·2^21; `7_22` enters
///   standardise with exponent n + 1 (for n + 1 = 512 that stops, 3 14-21x).
pub fn exp(w: u32) -> Outcome {
    const LOG2E: u32 = 0xb8aa_3b2a;
    let (m, e) = unpack(w);
    let (mut ph, pl) = mult_unsigned(m as u32, LOG2E);
    if m < 0 {
        ph = ph.wrapping_sub(LOG2E);
    }
    exp_of_product(ph, pl, e, false)
}

/// exp from `7_0` on, for the product x·log2 e = (ph:pl)·2^(e - 52), as
/// exp forms it and as pow joins it; `negate` is pow's odd-index flag 1_1
/// (`1_0 1-27`: `5_2 21-20(2)`, `5_26 30-21(2)`, `5_30 20-23(2)` negate
/// DS21 before standardise).
fn exp_of_product(ph: u32, pl: u32, e: i32, negate: bool) -> Outcome {
    // the count e' (one less when no halving is needed); the shift is the
    // same either way
    let e1 = if (ph >> 21) & 1 == (ph >> 20) & 1 { e - 1 } else { e };
    if e1 >= 9 {
        return Outcome::Stop {
            kind: Fault::ExpRange,
            resumed: 0,
        };
    }
    let p = i64::from(ph as i32) << 32 | i64::from(pl);
    let mut n = 20 - e;
    let mut r = p;
    while n > 16 {
        r >>= 16;
        n -= 16;
    }
    r >>= n;
    let r = (r as u64).wrapping_add(0x1_0000_0001);
    let n1 = (r >> 32) as u32;
    let lo = r as u32;
    let v = ((i64::from(lo) - (1 << 31)) >> 1) as u32;
    let (s, _) = s06(&EXP_SERIES, v);
    let mut r = (i64::from(s as i32) << 32) >> 10;
    if negate {
        r = r.wrapping_neg();
    }
    match standardise(r, n1 as i32) {
        Ok(x) => Outcome::Word(x),
        Err(resumed) => Outcome::Stop {
            kind: Fault::Overflow,
            resumed,
        },
    }
}

/// pow (tracks 15/8-15/10, sheet 20, flow diagrams pp.34-35), a b pow with
/// a the base: log (track 15/4) of |a|, times b, into exp (track 15/7).
pub fn pow(a: u32, b: u32) -> Outcome {
    let (ma, ea) = unpack(a);
    let (mb, eb) = unpack(b);
    if ma == 0 {
        return Outcome::Word(0);
    }
    if mb == 0 {
        return Outcome::Word(0x4000_0001);
    }
    let mut odd = false;
    let mut ma = ma as u32;
    if (ma as i32) < 0 {
        ma = ma.wrapping_neg();
        if eb < 0 {
            return Outcome::Stop {
                kind: Fault::FractionalPower,
                resumed: 0,
            };
        }
        if eb - 22 < 0 {
            let r = (i64::from(mb) << 33) >> (22 - eb);
            if r as u32 != 0 {
                return Outcome::Stop {
                    kind: Fault::FractionalPower,
                    resumed: 0,
                };
            }
            odd = (r >> 32) & 1 == 1;
        }
    }
    const ONE: u32 = 0x4000_0001;
    let mut r = match log2_double(ma, ea) {
        Some(r) => r,
        // resumed, the log subroutine leaves a zero register, and pow
        // takes that as log 1
        None => {
            return Outcome::Stop {
                kind: Fault::LogArgument,
                resumed: ONE,
            }
        }
    };
    if r == 0 {
        return Outcome::Word(ONE);
    }
    let mut c = eb + 10;
    loop {
        let h = (r >> 32) as u32;
        if (h >> 31) != (h >> 30) & 1 {
            break;
        }
        c -= 1;
        r = r.wrapping_mul(2);
    }
    let y = ((r >> 32) as u32).wrapping_add(r as u32 >> 31);
    let p = (i64::from(y as i32) * i64::from(mb)) as u64;
    exp_of_product((p >> 32) as u32, p as u32, c, odd)
}

/// "Fetch integer" (track 12/6, flow diagrams p.36 `6_23`): the whole
/// number a word stands for, as rem (and the indexing symbols) take it.
/// The word is added into 21_2 (`6_22 16-22`, sign-extended into 21_3),
/// and DS21 is doubled by `21-22` obeyed from TS13, built from the
/// template 6_20 less (e mod 32)·P17 (`6_30 24-14(17)`, `6_0 25-26` with the
/// mask 31·P17 at 10_30): 2((e mod 32) + 1) m.c., ending in m.c. 27. A
/// transfer lasts at most 32 m.c., so it is (e mod 16) + 1 doublings.
/// `6_27 29-23` adds 2^31 (round, halves up). The high word is the number:
/// for 1/2 <= |x| < 2^15 it is x rounded, halves up; otherwise the exponent
/// is in effect taken mod 16 (0.4 is taken as 26, 2^15 as 1). Measured over
/// exponents -40 to 59 (`fetch_integer_words`).
pub fn fetch_integer(w: u32) -> u32 {
    let k = (w & 15) + 1;
    let r = (i64::from(w as i32) << k).wrapping_add(1 << 31);
    (r >> 32) as u32
}

/// rem (tracks 15/1-15/2, sheets 17-18, flow diagrams p.36): a b rem.
///
/// - `7_6`: b' := fetch integer of b (19_3, `7_1 21-19`). `7_24 30-13`,
///   `7_26 19-26`, `7_29 13-27`: -b' not negative stops (7 30-16x).
/// - `7_0 16-15`, `7_2 8-14` (P1-10), `8_5 25-14`: TS14 := a's exponent
///   field. `7_7 30-21(2)`, `8_10 26-22`, `7_12 26-22`: DS21 := twice a's
///   word with the exponent cleared, added into 21_2 double-length:
///   R = m·2^11. `8_14 24-14(22)`, `8_6 23-13`: the count c := e.
/// - `7_8 13-27`: ... Loop (non-restoring division of the high word by
///   b'): `7_10 21-27`: while the high word is not negative, `7_13 19-23`
///   takes b' off it; when it is negative, `7_14 13-28`: if c = 0,
///   `8_17 19-22` adds b' back and the loop ends; otherwise `8_18 21-22(2)`
///   doubles DS21, `7_21` c -= 1 and `8_23 19-22` adds b'.
/// - `7_20 8-13`: TS13 := 21·2^21, and standardise: the remainder, with
///   its fraction in the low word.
pub fn rem(a: u32, b: u32) -> Outcome {
    let bi = fetch_integer(b);
    if (0u32.wrapping_sub(bi) as i32) >= 0 {
        return Outcome::Stop {
            kind: Fault::RemDivisor,
            resumed: 0,
        };
    }
    let e = (((a & 0x3ff) << 22) as i32) >> 22;
    let mant = a & !0x3ff;
    let mut r: i64 = i64::from(mant as i32) << 1;
    if e < 0 {
        if (mant as i32) >= 0 {
            return Outcome::Word(a);
        }
        r >>= (-e).min(63);
        let h = ((r >> 32) as u32).wrapping_add(bi);
        r = (i64::from(h as i32) << 32) | (r & 0xffff_ffff);
        return rem_result(r, b);
    }
    let mut c = e;
    loop {
        let mut h = (r >> 32) as u32;
        while (h as i32) >= 0 {
            h = h.wrapping_sub(bi);
        }
        r = (i64::from(h as i32) << 32) | (r & 0xffff_ffff);
        if c <= 0 {
            h = h.wrapping_add(bi);
            r = (i64::from(h as i32) << 32) | (r & 0xffff_ffff);
            break;
        }
        r = r.wrapping_mul(2);
        c -= 1;
        let h = ((r >> 32) as u32).wrapping_add(bi);
        r = (i64::from(h as i32) << 32) | (r & 0xffff_ffff);
    }
    rem_result(r, b)
}

/// rem's end: `7_20 8-13` (exponent 21), standardise with the link
/// `1_30 18-23`: the word b as it was fetched (QS18) is taken off the
/// result, and `8_1 21-28` stores 0 (`7_15 30-16`) if they were equal, the
/// result otherwise.
fn rem_result(r: i64, b: u32) -> Outcome {
    match standardise(r, 21) {
        Ok(x) if x == b => Outcome::Word(0),
        Ok(x) => Outcome::Word(x),
        Err(x) => Outcome::Stop {
            kind: Fault::Overflow,
            resumed: x,
        },
    }
}

/// neg (flow diagrams p.19): 0 - x through the subtract routine
/// (`number::add_words`, finding 15). Whichever exponent is the larger, no
/// shift happens (the zero word is either the operand shifted right out of
/// the way or, as the larger, skipped by `4_12 20-28`), so it is the
/// negated mantissa (as a whole word: -1.0 becomes +1.0) standardised with
/// x's exponent; an unstandardised word comes out standardised. Out of
/// range it stops (3 14-21x) and resumes with the exponent halved.
pub fn neg(w: u32) -> Outcome {
    let (m, e) = unpack(w);
    match standardise(-i64::from(m) << 32, e) {
        Ok(x) => Outcome::Word(x),
        Err(x) => Outcome::Stop {
            kind: Fault::Overflow,
            resumed: x,
        },
    }
}

/// mod (flow diagrams p.19): unpack, `21_3-27`: a word whose mantissa is
/// not negative is stored again unchanged; a negative one goes to neg.
pub fn modulus(w: u32) -> Outcome {
    if (w as i32) >= 0 {
        Outcome::Word(w)
    } else {
        neg(w)
    }
}
