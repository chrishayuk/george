//! GEORGE's numbers: "floating-point binary form (22 binary digit signed
//! mantissa, 10 binary digit signed exponent; hence about six decimal digit
//! accuracy)" (section 1.2). In the word, "signed exponent in positions
//! 1-10 and signed mantissa in positions 11-32 (binary point between
//! positions 31 and 32)" (section 1.9 (vii)): the mantissa is a fraction
//! between -1 and 1, its sign digit in P32.
//!
//! Addition, subtraction, multiplication and division are done word for
//! word as Hamblin's routines do them (`add_words`, `mul_words`,
//! `div_words`, all ending in `standardise`), and so are the conversions:
//! numerals at translation time (`numeral_word`), data cards (`read_word`)
//! and the punch's six decimal figures (`punch_word`). The functions are
//! in `functions.rs`. What is still worked out in f64 (the count of a
//! repetition, the I.D. keys, the single shot after overflow) is brought
//! back to 22 digits as `ROUNDING` says; see DECISIONS.md.

/// Digits after the binary point of the mantissa.
pub const FRACTION_BITS: i32 = 21;
pub const EXP_MAX: i32 = 511;
pub const EXP_MIN: i32 = -512;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rounding {
    /// To the nearest 22-digit mantissa.
    Nearest,
    /// Dropping the digits that do not fit (towards minus infinity, as a
    /// two's-complement shift does).
    Truncate,
}

pub const ROUNDING: Rounding = Rounding::Nearest;

/// A number as GEORGE holds it: `mantissa × 2^(exponent - 21)`, the
/// mantissa standardised so that its leading digit differs from its sign
/// digit, or zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Num {
    pub mantissa: i32,
    pub exponent: i32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Overflow;

impl Num {
    pub const ZERO: Num = Num {
        mantissa: 0,
        exponent: 0,
    };

    pub fn from_f64(x: f64) -> Result<Num, Overflow> {
        if x == 0.0 || x.is_nan() {
            return Ok(Num::ZERO);
        }
        if !x.is_finite() {
            return Err(Overflow);
        }
        // x = f × 2^e with 0.5 <= |f| < 1
        let mut e = x.abs().log2().floor() as i32 + 1;
        let mut f = x / 2f64.powi(e);
        while f.abs() >= 1.0 {
            f /= 2.0;
            e += 1;
        }
        while f.abs() < 0.5 {
            f *= 2.0;
            e -= 1;
        }
        let scaled = f * (1u32 << FRACTION_BITS) as f64;
        let mut m = match ROUNDING {
            Rounding::Nearest => scaled.round(),
            Rounding::Truncate => scaled.floor(),
        } as i64;
        if m == 1 << FRACTION_BITS {
            m >>= 1;
            e += 1;
        }
        if m == -(1 << (FRACTION_BITS - 1)) {
            // -1/2 standardises as -1 with the exponent one less
            m = -(1 << FRACTION_BITS);
            e -= 1;
        }
        if e > EXP_MAX {
            return Err(Overflow);
        }
        if e < EXP_MIN {
            // Hamblin's code stops on 3 14-21x here too: `R 4 ÷` with 1e-154
            // read in does (sources/deuce/RUNNING-GEORGE.md).
            return Err(Overflow);
        }
        Ok(Num {
            mantissa: m as i32,
            exponent: e,
        })
    }

    pub fn to_f64(self) -> f64 {
        self.mantissa as f64 * 2f64.powi(self.exponent - FRACTION_BITS)
    }

    /// The 32-digit DEUCE word: exponent in P1-10, mantissa in P11-32,
    /// both in two's complement.
    pub fn to_word(self) -> u32 {
        ((self.mantissa as u32) & 0x3F_FFFF) << 10 | (self.exponent as u32) & 0x3FF
    }

    /// A word as a number. Words made by the logical operators need not be
    /// standardised, and are kept as they are.
    pub fn from_word(w: u32) -> Num {
        let exponent = ((w & 0x3FF) as i32) << 22 >> 22;
        let mantissa = (w as i32) >> 10;
        Num { mantissa, exponent }
    }

    /// `self + b` as Hamblin's add routine forms it, word for word (tracks
    /// 12/3-12/5; flow diagrams pp.18-19; sources/deuce/RUNNING-GEORGE.md,
    /// finding 15). Not rounded to nearest: see `add_words`.
    pub fn plus(self, b: Num) -> Result<Num, Overflow> {
        add_words(self.mantissa as i64, self.exponent, b.mantissa as i64, b.exponent)
    }

    /// `self × b` as Hamblin's multiply routine forms it, word for word
    /// (track 12/5; flow diagram p.20): see `mul_words`.
    pub fn times(self, b: Num) -> Result<Num, Overflow> {
        mul_words(self.mantissa as i64, self.exponent, b.mantissa as i64, b.exponent)
    }

    /// `self ÷ b` as Hamblin's divide routine forms it, word for word: see
    /// `div_words`. The caller stops on a zero divisor.
    pub fn over(self, b: Num) -> Result<Num, Overflow> {
        div_words(self.mantissa as i64, self.exponent, b.mantissa as i64, b.exponent)
    }

    /// `self - b`: the subtract routine negates b's mantissa (as a whole
    /// word, so -1.0 becomes +1.0 without overflow) and joins add.
    pub fn minus(self, b: Num) -> Result<Num, Overflow> {
        add_words(self.mantissa as i64, self.exponent, -(b.mantissa as i64), b.exponent)
    }

    pub fn is_zero(self) -> bool {
        self.mantissa == 0
    }

    /// The sign digit: how a relation's "truth" is shown.
    pub fn is_negative(self) -> bool {
        self.mantissa < 0
    }

    /// The value on overflow when the operator resumes with a single shot:
    /// "S.S. resumes with exponent halved" (stop 3 14-21x).
    pub fn halved_exponent(x: f64) -> Num {
        let e = x.abs().log2().floor() as i32 + 1;
        Num::from_f64(x / 2f64.powi(e) * 2f64.powi(e / 2)).unwrap_or(Num::ZERO)
    }

    /// As an integer for "indexing" purposes (suffixes, skip numbers, ranges
    /// of repetition). Section 1.9 (v): these "are assumed to be integers of
    /// modulus less than 2^15, and use of numbers with fractional parts or
    /// outside this range may give incorrect results". They do, in definite
    /// ways: suffixes, skip numbers and ranges all go through the same "fetch
    /// integer" as rem's divisor; measured on the transcribed coding sheets
    /// (DECISIONS.md 4).
    pub fn to_index(self) -> i64 {
        // Hamblin's "fetch integer" (functions::fetch_integer): rounded,
        // halves up, for 1/2 <= |x| < 2^15, and something definite but odd
        // outside that range, as on his code.
        i64::from(crate::functions::fetch_integer(self.to_word()) as i32)
    }
}

/// Hamblin's floating add, word for word (tracks 12/3-12/5, flow diagrams
/// pp.18-19; m.c. as on the coding sheets). DS21 is taken as one 64-digit
/// register `R = high·2^32 + low`, the high word in units of the last
/// digit of the mantissa:
///
/// 1. The operand with the smaller exponent (a, on a tie) is in the high
///    word, the low word cleared; the other's exponent is the result's
///    (`4_4 20-26`, swapped at `4_11`...`4_19` if a's is the larger).
///    k is the difference of the exponents.
/// 2. If k > 0 and the operand with the larger exponent is the zero word
///    (`4_12 20-28`), the result is the other, unshifted, standardised
///    with its own exponent (`5_15 19-13`). Otherwise, k >= 24: the smaller
///    operand is dropped (`5_16`, `4_21 20-21`). 17 <= k <= 23: one exact
///    16-place shift (`5_25 22-21`, 32 m.c. starting even), k -= 16. Then
///    k places by the instruction built from 5_6 in TS13 (`5_3`, obeyed by
///    `3_28 13-0`): `22-21` for 2k m.c. ending in m.c. 8, so starting odd.
///    Starting odd, each pair of m.c. halves the high word first and then
///    the low word, whose new top digit is the last digit of the *new* high
///    word: the old high word's digit 0 is lost and its digit k appears
///    twice, so the low word holds digits 1..k where 0..k-1 belong.
/// 3. The larger operand's mantissa is added to the high word (`4_8 20-22`).
/// 4. Standardise (`3_30`...`3_31`): a zero register gives the zero word.
///    Otherwise P1 is added to the high word (`3_2 27-22`); while digits
///    P22 and P23 of the high word agree, the register is doubled
///    (`3_29 21-22`), P1 taken off the high word again (`3_3 27-23`) and the
///    exponent lowered; then the register is halved (`3_31 22-21`, exact)
///    and the exponent raised. The mantissa is the high word; the low word
///    is dropped (`3_10 15-21`). So the P1 acts as a half in the final
///    place: the result is the double-length sum rounded to nearest, halves
///    up, but of a sum whose low word holds the misplaced digits of step 2.
pub fn add_words(ma: i64, ea: i32, mb: i64, eb: i32) -> Result<Num, Overflow> {
    // the smaller exponent (DS19 after the swap at 4_11...4_19) and the larger (DS20)
    let (ms, es, ml, el) = if ea - eb > 0 {
        (mb, eb, ma, ea)
    } else {
        (ma, ea, mb, eb)
    };
    let mut k = el - es;
    let mut e = el;
    let mut r: i64 = if k >= 24 { 0 } else { ms << 32 };
    if k > 0 && ml == 0 && el == 0 {
        // the larger is the zero word (`4_12 20-28`): the smaller alone, unshifted
        r = ms << 32;
        e = es;
    } else if k < 24 {
        if k >= 17 {
            r >>= 16;
            k -= 16;
        }
        let mut high = (r >> 32) as i32;
        let mut low = r as u32;
        for _ in 0..k {
            high >>= 1;
            low = (low >> 1) | ((high as u32 & 1) << 31);
        }
        r = ((high as i64) << 32) | low as i64;
    }
    r = r.wrapping_add(ml << 32);
    standardise(r, e)
}

/// Hamblin's multiply, word for word (track 12/5, flow diagram p.20;
/// RUNNING-GEORGE.md, finding 26): the signed double-length product of the
/// two mantissas (the hardware's unsigned product of TS16 and 21_3 with the
/// sign corrections), shifted up 11 places (`21-22(22)`) so that the high
/// word is in units of the last digit, and standardised with the sum of the
/// exponents. Rounded to nearest, halves up, but for `standardise`'s two
/// exceptions.
pub fn mul_words(ma: i64, ea: i32, mb: i64, eb: i32) -> Result<Num, Overflow> {
    standardise((ma * mb) << 11, ea + eb)
}

/// Hamblin's divide, word for word (tracks 12/4-12/5, flow diagram p.20;
/// RUNNING-GEORGE.md, finding 27). The dividend's mantissa A goes in 21_3
/// and twice the divisor's, B, in TS16 (`24-16`); the divider gives
/// Q = floor(2^31 A / B) (2^31 A = QB + R, B <= R < 0 for negative B) in
/// 21_2. Q halved twice (`22-14` with TCB on, `23-21`) goes to the high
/// word and Q stays in the low word, and the register is shifted down 7
/// places (`22-21`): the high word is Q/2^9, the last digit's unit, with
/// Q's next seven digits at the top of the low word and Q's own digits
/// 7-31 below them. Standardised with the difference of the exponents.
/// For standardised operands that is the quotient rounded to nearest; a
/// dividend with few digits (an unstandardised word) gives a quotient with
/// few digits. A zero dividend gives zero before the divisor is looked at.
/// The divider is the simulator's (MACHINE.md §7): for |A| >= |B| (only an
/// unstandardised divisor such as the all-ones word) it is its 32-digit
/// wrap-round, not a result measured on a DEUCE.
pub fn div_words(ma: i64, ea: i32, mb: i64, eb: i32) -> Result<Num, Overflow> {
    if ma == 0 {
        return Ok(Num::ZERO);
    }
    let a = ma as i128;
    let b = 2 * mb as i128;
    let x = a << 31;
    let mut q = x.div_euclid(b);
    if b < 0 && x - q * b >= 0 {
        q -= 1;
    }
    let q = q as u32 as i32;
    let high = (q >> 1) >> 1;
    let r = ((high as i64) << 32) | (q as u32 as i64);
    standardise(r >> 7, ea - eb)
}

/// Hamblin's standardise (track 12/3, flow diagram p.18), which + - × ÷
/// all end in: DS21 as one 64-digit register `r = high·2^32 + low`, the
/// high word in units of the last digit of the mantissa, and the exponent
/// `e`. A zero register gives the zero word (`3_30 21-28(2)`). Otherwise P1
/// is added to the high word (`3_2 27-22`); while digits P22 and P23 of the
/// high word agree, the register is doubled (`3_29 21-22`), P1 taken off
/// the high word again (`3_3 27-23`) and the exponent lowered; then the
/// register is halved (`3_31 22-21`, exact) and the exponent raised. The
/// mantissa is the high word; the low word is dropped (`3_10 15-21`).
///
/// So the P1 is a half in the final place: the result is `r` rounded to
/// nearest, halves up, *except* that the leading digit is tested with the
/// half already added, one place before the final halving
/// (RUNNING-GEORGE.md, finding 26). Where the register, scaled to the
/// final place, lies in [2^21 - 1, 2^21 - 1/2) units (positive, just below
/// a power of two), the half carries into the test digit and the loop stops
/// one doubling early: the result is 2^21 units, one unit high (2097151 × 1
/// gives 2097152). Where it lies in [-2^21 - 1, -2^21 - 1/2) units
/// (negative, just beyond minus a power of two), the half hides the test
/// digit and the loop goes one doubling too far: the mantissa is -2^21 - 1,
/// which does not fit in 22 digits and is stored as +2^21 - 1, so the sign
/// is lost (0.609211 × -0.820734 gives +0.5).
pub fn standardise(mut r: i64, mut e: i32) -> Result<Num, Overflow> {
    const HIGH: i64 = 1 << 32;
    if r == 0 {
        return Ok(Num::ZERO);
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
    let mantissa = ((r >> 32) as i32) << 10 >> 10;
    if !(EXP_MIN..=EXP_MAX).contains(&e) {
        return Err(Overflow);
    }
    Ok(Num { mantissa, exponent: e })
}

/// The coefficients of the series punch translate sums with the library
/// routine S06 (track 15/0) for 2^f/2, highest power first, as fetched from
/// DL4: 31 binary places, in powers of 2x where x = f - 1/2.
const PUNCH_SERIES: [i32; 8] = [
    0xb6, 0xe56, 0xf727, 0xdedab, 0xa0c1d0, 0x56f8a65, 0x1f5e4653, 0x5a82799a,
];

/// The library routine M12, "signed multiplication of two numbers with
/// shift of two places": the signed double-length product (the unsigned
/// product corrected by subtracting each negative factor's partner from
/// the top half), doubled twice (`21-22`), P32 subtracted from the bottom
/// half (`29-23`, so 2^31 added to the double word), the top half kept: the
/// product × 4 / 2^32 rounded to nearest, halves up.
fn m12(s: i32, x: i32) -> i32 {
    ((s as i64 * x as i64).wrapping_mul(4).wrapping_add(1 << 31) >> 32) as i32
}

/// Hamblin's punch translate, word for word (tracks 14/4-14/7 with S06 and
/// M12 on 15/0; flow diagrams pp.47-48; sources/deuce/RUNNING-GEORGE.md,
/// finding 25). The decimal exponent and the six figures come from the
/// stored word by fixed-point arithmetic, with these roundings:
///
/// 1. M = |mantissa| at 30 binary places (`23-13` or `23-26`: the word
///    halved, negated if negative); the sign goes to column 1. E = the
///    binary exponent + 1.
/// 2. E × log10 2 (0x4d104d42, 32 b.p.), signed double length: the top
///    half is the decimal exponent D, the bottom half the fraction F.
/// 3. F × log2 10 (0xd49a784c, 30 b.p.), unsigned; the top half H (30 b.p.)
///    is split into its integer part i (P31-32) and fraction g; the double
///    word, i taken off, is doubled and 1/2 taken from the top half:
///    x = g - 1/2 at 31 b.p., the next digit of the bottom half carried in.
/// 4. S06 sums the series for 2^(x + 1/2)/2 at x by Horner's rule with
///    M12, each product rounded.
/// 5. Y = M12(S, M), at 30 b.p.: the mantissa × 10^frac / 2. In the bottom
///    half of DS21 (top half clear) it is doubled 1 + i times: Z/2^32 is the
///    number divided by 10^D.
/// 6. Z = 0 punches the zero card. Otherwise the round-off 21475 (5·10^-6
///    × 2^32) is added. If the top half is still 0, the number is below
///    10^D: D is lowered by 1, 9/10 of the round-off (19327) taken off
///    again and Z multiplied by 10. The six figures are the top half, then
///    five times the top half cleared and the rest multiplied by 10.
/// 7. The exponent's three figures are exact.
///
/// The arithmetic is not exact, so a number exactly half-way between two
/// six-figure decimals can go either way: Z is a few units of 2^-32 out.
/// The all-ones word (mantissa -2^-21, exponent -1) gives "-0.00000 e-1".
pub fn punch_word(w: u32) -> Card {
    const LOG10_2: i64 = 0x4d10_4d42;
    const LOG2_10: u64 = 0xd49a_784c;
    const ROUND_OFF: u64 = 21475;
    const NINE_TENTHS_ROUND_OFF: u64 = 19327;
    let top = (w & 0xffff_fc00) as i32;
    let negative = top < 0;
    let m = if negative {
        ((top >> 1) as i64).wrapping_neg() as i32
    } else {
        top >> 1
    };
    let e = ((w & 0x3ff) as i32) << 22 >> 22;
    let p = (e as i64 + 1) * LOG10_2;
    let mut d = (p >> 32) as i32;
    let f = p as u32 as u64;
    let q = f * LOG2_10;
    let h = (q >> 32) as u32;
    let i = h >> 30;
    let g = h & 0x3fff_ffff;
    let x = (2 * g + ((q >> 31) & 1) as u32).wrapping_sub(1 << 30) as i32;
    let s = PUNCH_SERIES[1..]
        .iter()
        .fold(PUNCH_SERIES[0], |s, &c| m12(s, x).wrapping_add(c));
    let y = m12(s, m) as u32 as u64;
    let mut z = y << (1 + i);
    if z == 0 {
        return Card {
            negative: false,
            digits: 0,
            exponent: 0,
        };
    }
    z += ROUND_OFF;
    if z >> 32 == 0 {
        d -= 1;
        z = (z - NINE_TENTHS_ROUND_OFF) * 10;
    }
    let mut digits = 0;
    for k in 0..6 {
        if k > 0 {
            z = (z & 0xffff_ffff) * 10;
        }
        digits = digits * 10 + (z >> 32) as u32;
    }
    Card {
        negative,
        digits,
        exponent: d,
    }
}

/// One symbol of a numeral sequence: a digit (column 2 of the coding
/// table, its row) or a decimal point (any symbol of column 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumeralSym {
    Digit(u8),
    Point,
}

/// Hamblin's numeral conversion at translation time, word for word
/// (translator tracks 13/6-13/7, flow diagram p.12;
/// sources/deuce/RUNNING-GEORGE.md, finding 28). m.c. as on the sheets.
///
/// 1. The digits are accumulated as an integer N in TS13, point or no
///    point (`7_5 13-16`, `7_7 16-25(9)`, `8_17 25-25`: N = 10N + d, in
///    32 digits). A point sets the counter DS19 to P32 (`7_18 29-19`);
///    every later numeral symbol halves it (`8_9 19-21`, `8_11 22-19`),
///    so it counts k, the digits after the *last* point.
/// 2. N = 0 gives the zero word (`7_14 13-28`). Otherwise T = N is
///    normalised in TS14 with the exponent count e = 31 in TS13
///    (`8_20 13-14`, `7_16 [31P17]`): while P31 of T is 0 (`7_22 24-27`),
///    T doubled (`8_24 24-14`) and e lowered (`8_26 28-26`). Now the
///    number is T·2^(e-31), T in [2^30, 2^31).
/// 3. k times (`7_27 19-14`, `7_29 24-28`, `7_6 24-19`): T halved into
///    21_3 (`8_25 23-21`, the last digit lost) and divided by 10 at P28
///    (`7_0 [10P28]`, `7_9 1-24`): Q = floor(16 (T div 2) / 10); a unit
///    added (`8_14 27-22`) and halved (`8_16 22-14`): T = (Q + 1) div 2,
///    e lowered by 2 (`7_8 28-26(2)`), and normalised again as in 2.
/// 4. T halved into 21_3 and DS21 shifted down 9 places (`7_31 22-21`),
///    P32 subtracted from 21_2 (`7_19 29-23`, which adds 2^31 to the
///    double word): the mantissa m = (T + 2^9) div 2^10, rounded half up
///    from T's last ten digits. e at P23 (`8_22 13-25(6)`) in 21_2 (`8_29
///    13-21`), and DS21 shifted up 10 places (`8_0 21-22`): the word is
///    m at P11-32 and e mod 2^10 at P1-10.
///
/// So each division by 10 is rounded to nearest (halves up) at 30 or 31
/// digits, and the last rounding to 22 digits is half up. Where T rounds up
/// to 2^31 (the integer 4194303, or 2^22 - 1 divided down) the mantissa
/// carries into the sign digit and the number is stored as -2^e: there is
/// no re-standardising. With at most nine digits e lies between -29 and
/// 30, so there is no overflow.
pub fn numeral_word(syms: impl IntoIterator<Item = NumeralSym>) -> u32 {
    let mut n: u32 = 0;
    let mut k: Option<u32> = None;
    for s in syms {
        k = k.map(|k| k + 1);
        match s {
            NumeralSym::Digit(d) => n = n.wrapping_mul(10).wrapping_add(u32::from(d)),
            NumeralSym::Point => k = Some(0),
        }
    }
    if n == 0 {
        return 0;
    }
    let normalise = |t: &mut u32, e: &mut i32| {
        while *t & (1 << 30) == 0 {
            *t <<= 1;
            *e -= 1;
        }
    };
    let mut t = n;
    let mut e: i32 = 31;
    normalise(&mut t, &mut e);
    for _ in 0..k.unwrap_or(0) {
        let a = u64::from(t >> 1);
        let q = (a << 31) / (10 << 27);
        t = ((q + 1) >> 1) as u32;
        e -= 2;
        normalise(&mut t, &mut e);
    }
    let m = (t >> 1).wrapping_add(1 << 8) >> 9;
    m << 10 | (e as u32 & 0x3ff)
}

/// Hamblin's read translate, word for word (tracks 14/11-14/14 with S06
/// and M12 on 15/0; flow diagrams pp.44-46; sources/deuce/RUNNING-GEORGE.md,
/// finding 29). The card gives the six figures D (columns 2-7, taken as an
/// integer, leading zeros and all), the decimal exponent X (columns 18-20)
/// and the two signs. m.c. as on the sheets.
///
/// 1. D = 0 gives the zero word, whatever the exponent (`1_27 25-28`).
/// 2. The top half of DS21 is ±2|X| (`5_28 13-22` or, for a negative
///    exponent, `5_29 13-23`), multiplied by log2 10 at 30 b.p.
///    (`5_1 [0xd49a784c]`, `1_7 0-24`, sign correction `1_9 16-23`) and
///    doubled (`3_12 21-22`): the top half is i = floor(X log2 10)
///    (`3_15 21-19`), the bottom half its fraction f at 32 b.p. The top
///    half cleared (`3_17`), P32 added (`1_19 29-22`) and the double word
///    halved into TS16 (`4_22 22-16`): x = (f - 1/2)/2, signed, 31 b.p.
/// 3. S06 (`5_0`, `6_18`) sums the series of punch translate in x with
///    M12 (the same coefficients, DL4 m.c. 12-19): S = 2^f/2, 31 b.p.
/// 4. Y = M12(S, ±D·2^10) (`1_31 1-16`, `4_0`), and R = 4·Y·C, the
///    double-length product by C = 2^20·10^-5 at 26 b.p. (`1_5 [0x29f16b12]`,
///    `3_10`), M12's round-off taken back again (`1_30 29-22`). The top half
///    of R is the number ÷ 2^(i+5) at 31 b.p., with ten more digits below
///    the mantissa's last.
/// 5. Standardise, as `standardise` does but with the round-off 2^9 (half
///    the final place) in the top half: e = i + 5 (`5_5 19-13`, `5_7
///    27-25(5)`); R += 2^9·2^32 (`3_14 5-22`); while P31 and P32 of the
///    top half agree (`3_19`, `4_21 24-15`, `5_23 26-27`) R is doubled
///    (`5_27 21-22`), 2^9 taken off the top half (`1_6 5-23`) and e
///    lowered (`5_25 27-26`). There is no final halving.
/// 6. e + 512 outside 0..1023 (`5_26 [2^9]`, `5_24 23-14`, `3_6 23-28`)
///    stops on 4 9-24, "data card with exponent too large". Otherwise the
///    word is the top half with P1-10 replaced by e (`5_19 [P1-10]`,
///    `5_8`...`5_18 26-25`).
///
/// So the six figures are not rounded to nearest: S, Y and the
/// fixed-point constants carry their own errors, and 1.5 % of cards read
/// one unit away in the last place. As in standardise, a negative R that
/// rounds to just beyond minus a power of two is doubled once too often
/// and loses its sign (-3.91032e-149 reads as +3.91032e-149).
pub fn read_word(card: Card) -> Result<u32, Overflow> {
    const LOG2_10: i64 = 0xd49a_784c;
    const SCALE: i64 = 0x29f1_6b12; // 2^20·10^-5 at 26 b.p.
    const HALF: i64 = 1 << 41; // 2^9 in the high word
    let d = i64::from(card.digits);
    if d == 0 {
        return Ok(0);
    }
    let p = 2 * (2 * i64::from(card.exponent)) * LOG2_10;
    let i = (p >> 32) as i32;
    let frac = p & 0xffff_ffff;
    let x = ((frac - (1 << 31)) as i32) >> 1;
    let s = PUNCH_SERIES[1..]
        .iter()
        .fold(PUNCH_SERIES[0], |s, &c| m12(s, x).wrapping_add(c));
    let dm = if card.negative { -(d << 10) } else { d << 10 } as i32;
    let y = i64::from(m12(s, dm));
    let mut r = (4 * y * SCALE).wrapping_add(HALF);
    let mut e = i + 5;
    loop {
        let h = (r >> 32) as i32;
        if (h >> 31) & 1 != (h >> 30) & 1 {
            break;
        }
        r = r.wrapping_mul(2).wrapping_sub(HALF);
        e -= 1;
    }
    if !(EXP_MIN..=EXP_MAX).contains(&e) {
        return Err(Overflow);
    }
    let h = (r >> 32) as u32;
    Ok(h & !0x3ff | (e as u32 & 0x3ff))
}

/// A number as punched: "floating decimal, with sign of mantissa in col. 1,
/// mantissa (<10) in cols. 2-10 with decimal point between cols. 2 and 3;
/// sign of exponent in col. 11, exponent (integer) in cols. 12-20. ... The
/// mantissa is however rounded to six significant figures, and the exponent
/// cannot exceed three digits: hence cols. 8-10 and 12-17 will in any case
/// have zeros." (section 2.3)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Card {
    pub negative: bool,
    /// six significant figures, 100000 to 999999, or 0
    pub digits: u32,
    pub exponent: i32,
}

impl Card {
    /// The card Hamblin's punch translate makes of a number: `punch_word`
    /// of its word.
    pub fn of(n: Num) -> Card {
        punch_word(n.to_word())
    }

    pub fn value(self) -> f64 {
        let v = self.digits as f64 * 10f64.powi(self.exponent - 5);
        if self.negative {
            -v
        } else {
            v
        }
    }

    /// The twenty columns, a minus sign shown as "-" and a plus sign as a
    /// blank, since plus signs are not punched.
    pub fn columns(self) -> String {
        format!(
            "{}{:06}000{}000000{:03}",
            if self.negative { '-' } else { ' ' },
            self.digits,
            if self.exponent < 0 { '-' } else { ' ' },
            self.exponent.abs()
        )
    }

    /// The number in ordinary notation, to the six figures punched.
    pub fn decimal(self) -> String {
        if self.digits == 0 {
            return "0".into();
        }
        let sign = if self.negative { "-" } else { "" };
        let d = self.digits.to_string();
        let e = self.exponent;
        let body = if (-5..=6).contains(&e) {
            if e >= 5 {
                format!("{}{}", d, "0".repeat((e - 5) as usize))
            } else if e >= 0 {
                let (a, b) = d.split_at(e as usize + 1);
                format!("{a}.{b}")
            } else {
                format!("0.{}{}", "0".repeat((-e - 1) as usize), d)
            }
        } else {
            let (a, b) = d.split_at(1);
            format!("{a}.{b}e{e}")
        };
        let body = if body.contains('.') && !body.contains('e') {
            body.trim_end_matches('0').trim_end_matches('.').to_string()
        } else if let Some((m, ex)) = body.split_once('e') {
            format!("{}e{}", m.trim_end_matches('0').trim_end_matches('.'), ex)
        } else {
            body
        };
        format!("{sign}{body}")
    }

    /// Read a data card in the standard columns, with the flexibility the
    /// manual allows for manual punching: plus signs and zeros need not be
    /// punched, the minus sign of the mantissa may be in any of cols. 1-10
    /// and that of the exponent in any of cols. 11-20, and "figures in cols.
    /// 8-10 and 12-17 are in any case treated as zero by the reader".
    pub fn parse_columns(line: &str) -> Option<Card> {
        let cols: Vec<char> = line.chars().collect();
        if cols.len() > 20 || cols.iter().any(|c| !(c.is_ascii_digit() || *c == ' ' || *c == '-')) {
            return None;
        }
        let col = |n: usize| cols.get(n - 1).copied().unwrap_or(' ');
        let negative = (1..=10).any(|n| col(n) == '-');
        let neg_exp = (11..=20).any(|n| col(n) == '-');
        let digit = |n: usize| col(n).to_digit(10).unwrap_or(0);
        let digits = (2..=7).fold(0, |acc, n| acc * 10 + digit(n));
        let exp = (18..=20).fold(0, |acc, n| acc * 10 + digit(n)) as i32;
        // the mantissa as punched is d.ddddd; standardise for comparison
        let mut card = Card {
            negative,
            digits,
            exponent: if neg_exp { -exp } else { exp },
        };
        if digits == 0 {
            card = Card {
                negative: false,
                digits: 0,
                exponent: 0,
            };
        }
        Some(card)
    }

    /// The card a number written in ordinary notation would be punched as:
    /// the first six significant figures; the reader ignores the rest.
    pub fn from_decimal(s: &str) -> Option<Card> {
        let s = s.trim();
        let (negative, body) = match s.strip_prefix('-') {
            Some(b) => (true, b),
            None => (false, s.strip_prefix('+').unwrap_or(s)),
        };
        let (mant, exp) = match body.split_once(['e', 'E']) {
            Some((m, e)) => (m, e.parse::<i32>().ok()?),
            None => (body, 0),
        };
        if mant.is_empty() || mant.chars().any(|c| !(c.is_ascii_digit() || c == '.')) || mant.matches('.').count() > 1 {
            return None;
        }
        let (int, frac) = mant.split_once('.').unwrap_or((mant, ""));
        let all: String = format!("{int}{frac}");
        let first = match all.find(|c: char| c != '0') {
            Some(k) => k,
            None => {
                return Some(Card {
                    negative: false,
                    digits: 0,
                    exponent: 0,
                })
            }
        };
        let sig: String = all[first..].chars().chain(std::iter::repeat('0')).take(6).collect();
        let exponent = int.len() as i32 - 1 - first as i32 + exp;
        Some(Card {
            negative,
            digits: sig.parse().ok()?,
            exponent,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_of_simple_values() {
        for x in [1.0, -1.0, 0.5, -0.5, 3.0, 20.0, 338350.0, -0.75] {
            assert_eq!(Num::from_f64(x).unwrap().to_f64(), x);
        }
    }

    #[test]
    fn mantissa_is_standardised() {
        for x in [1.0, -1.0, 0.1, -0.1, 12345.678, -3.0, -0.5] {
            let n = Num::from_f64(x).unwrap();
            let m = n.mantissa;
            assert!(
                (1 << 20..1 << 21).contains(&m) || (-(1 << 21)..-(1 << 20)).contains(&m),
                "{x} {m}"
            );
        }
    }

    #[test]
    fn add_and_subtract_as_hamblins_code() {
        // RUNNING-GEORGE.md, finding 13's table, measured on the simulator
        let n = |m, e| Num {
            mantissa: m,
            exponent: e,
        };
        let third = n(1398101, -1); // 1/3, as GEORGE's ÷ leaves it
        let seventh = n(1198373, -2); // 1/7
        let one = n(1 << 20, 1);
        assert_eq!(one.plus(seventh), Ok(n(1198372, 1))); // exact 1198372.625
        assert_eq!(one.plus(third), Ok(n(1398102, 1))); // exact 1398101.25
        assert_eq!(n(1572864, 2).plus(seventh), Ok(n(1647762, 2))); // 3 + 1/7: exact 1647762.3125
        assert_eq!(one.minus(third), Ok(n(1398101, 0))); // exact 1398101.5
        assert_eq!(third.plus(seventh), Ok(n(1997287, -1))); // exact 1997287.5
        assert_eq!(one.minus(one), Ok(Num::ZERO));
        // the larger exponent belongs to zero: the other operand unshifted
        assert_eq!(n(1234702, -9).plus(Num::ZERO), Ok(n(1234702, -9)));
        // more than 23 places apart: the smaller is dropped
        assert_eq!(n(1 << 20, 30).plus(n(-(1 << 21), 6)), Ok(n(1 << 20, 30)));
    }

    #[test]
    fn multiply_and_divide_as_hamblins_code() {
        let n = |m, e| Num {
            mantissa: m,
            exponent: e,
        };
        // RUNNING-GEORGE.md, finding 26, measured on the simulator
        let big = n((1 << 21) - 1, 21); // 2097151
        assert_eq!(big.times(n(1 << 20, 1)), Ok(n(1 << 20, 22))); // 2097152
        assert_eq!(n(1277608, 0).times(n(-1721204, 0)), Ok(n((1 << 21) - 1, -1))); // +0.5
        assert_eq!(n(1 << 20, 2).over(n(1572864, 2)), Ok(n(1398101, 0))); // 2/3
        assert_eq!(n(1 << 20, 1).over(n(1835008, 3)), Ok(n(1198373, -2))); // 1/7

        // the all-ones word divided: its one digit is all the quotient has
        assert_eq!(n(-1, -1).over(n(2097151, -1)), Ok(n(-(1 << 21), -21)));
    }

    #[test]
    fn punch_as_hamblins_code() {
        // RUNNING-GEORGE.md, finding 25: exact halves at the seventh figure
        let c = |m, e| {
            Card::of(Num {
                mantissa: m,
                exponent: e,
            })
            .columns()
        };
        assert_eq!(c(1494469, 20), " 747234000 000000005"); // 747234.5
        assert_eq!(c(1394364, 17), " 871477000 000000004"); // 87147.75
        assert_eq!(c(1 << 20, -9), " 976563000-000000004"); // 2^-10

        // the all-ones word: mantissa -2^-21, exponent -1
        assert_eq!(Card::of(Num::from_word(u32::MAX)).columns(), "-000000000-000000001");
        assert_eq!(Card::of(Num::from_word(u32::MAX)).decimal(), "0");
    }

    #[test]
    fn six_figures() {
        let third = Num::from_f64(1.0 / 3.0).unwrap();
        assert_eq!(Card::of(third).columns(), " 333333000-000000001");
        assert_eq!(
            Card::of(Num::from_f64(338350.0).unwrap()).columns(),
            " 338350000 000000005"
        );
        assert_eq!(Card::of(Num::from_f64(338350.0).unwrap()).decimal(), "338350");
        assert_eq!(Card::of(Num::from_f64(-2.5).unwrap()).decimal(), "-2.5");
        assert_eq!(Card::of(Num::ZERO).columns(), " 000000000 000000000");
    }

    #[test]
    fn data_cards() {
        let c = Card::parse_columns(" 314159265 000000000").unwrap();
        assert_eq!((c.digits, c.exponent), (314159, 0));
        let c = Card::parse_columns(" 25       -000000001").unwrap();
        assert_eq!((c.digits, c.exponent, c.negative), (250000, -1, false));
        let c = Card::from_decimal("0.0025").unwrap();
        assert_eq!((c.digits, c.exponent), (250000, -3));
        let c = Card::from_decimal("3.14159265").unwrap();
        assert_eq!((c.digits, c.exponent), (314159, 0));
    }
}
