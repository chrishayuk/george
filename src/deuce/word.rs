//! DEUCE words and the instruction format.
//!
//! A word is 32 binary digits, P1 (least significant, first in time) to P32
//! (most significant, the sign digit). Here P*n* is bit *n* - 1 of a `u32`.
//!
//! The instruction word (DEUCE Programming Manual NS-y-16/5-56, §5.3, p. 25;
//! NS y 88 *Basic Programming*, §10, sheet 19):
//!
//! | digits  | field                          | range |
//! |---------|--------------------------------|-------|
//! | P1      | not used                       |       |
//! | P2-P4   | NIS, next instruction source   | 0-7 (0 means DL8) |
//! | P5-P9   | S, source                      | 0-31  |
//! | P10-P14 | D, destination                 | 0-31  |
//! | P15-P16 | C, characteristic              | 0-3   |
//! | P17-P21 | W, wait number                 | 0-31  |
//! | P22-P25 | not used by Control ("J" here) | 0-15  |
//! | P26-P30 | T, timing number               | 0-31  |
//! | P31     | not used                       |       |
//! | P32     | G, go digit (0 = stopper)      | 0-1   |
//!
//! P22-P25 are ignored by Control (manual §5.3). The UTECOM coding sheets
//! for GEORGE write a number in brackets between W and T, e.g. `9 (1) 8`;
//! that is read here as the contents of P22-P25 and kept in `j`, so that it
//! survives in the word even though Control ignores it.

use std::fmt;

/// P*n*: a word with a single one in digit *n* (1-32).
pub const fn p(n: u32) -> u32 {
    1u32 << (n - 1)
}

/// A decoded instruction word.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Instr {
    pub nis: u8,
    pub s: u8,
    pub d: u8,
    pub c: u8,
    pub w: u8,
    pub t: u8,
    /// Go digit: true means obeyed at once, false means a stopper ("X").
    pub go: bool,
    /// P22-P25, ignored by Control.
    pub j: u8,
    /// P1 and P31, ignored by Control; kept so decode/encode round-trips.
    pub spare: u32,
}

impl Instr {
    pub fn new(nis: u8, s: u8, d: u8, c: u8, w: u8, t: u8, go: bool) -> Instr {
        Instr {
            nis,
            s,
            d,
            c,
            w,
            t,
            go,
            j: 0,
            spare: 0,
        }
    }

    pub fn decode(word: u32) -> Instr {
        Instr {
            nis: ((word >> 1) & 7) as u8,
            s: ((word >> 4) & 31) as u8,
            d: ((word >> 9) & 31) as u8,
            c: ((word >> 14) & 3) as u8,
            w: ((word >> 16) & 31) as u8,
            j: ((word >> 21) & 15) as u8,
            t: ((word >> 25) & 31) as u8,
            go: word >> 31 == 1,
            spare: word & (p(1) | p(31)),
        }
    }

    pub fn encode(self) -> u32 {
        (self.spare & (p(1) | p(31)))
            | (u32::from(self.nis & 7) << 1)
            | (u32::from(self.s & 31) << 4)
            | (u32::from(self.d & 31) << 9)
            | (u32::from(self.c & 3) << 14)
            | (u32::from(self.w & 31) << 16)
            | (u32::from(self.j & 15) << 21)
            | (u32::from(self.t & 31) << 25)
            | (u32::from(self.go) << 31)
    }

    /// Code an instruction stored in m.c. `at` that transfers first in m.c.
    /// `first` and takes its successor from m.c. `next` (all mod 32), as a
    /// coder would: W = first - at - 2, T = next - at - 2 (mod 32).
    #[allow(clippy::too_many_arguments)]
    pub fn coded(at: u8, nis: u8, s: u8, d: u8, c: u8, first: u8, next: u8, go: bool) -> Instr {
        let w = (u32::from(first) + 64 - u32::from(at) - 2) % 32;
        let t = (u32::from(next) + 64 - u32::from(at) - 2) % 32;
        Instr::new(nis, s, d, c, w as u8, t as u8, go)
    }

    /// The delay line the next instruction comes from (NIS 0 means DL8).
    pub fn next_dl(self) -> usize {
        if self.nis == 0 {
            8
        } else {
            self.nis as usize
        }
    }
}

impl fmt::Display for Instr {
    /// The usual written form "N, S-D, C, W, T X" (manual §5.7, p. 27),
    /// always with the characteristic.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}-{}, {}, {}", self.nis, self.s, self.d, self.c, self.w)?;
        if self.j != 0 {
            write!(f, "({})", self.j)?;
        }
        write!(f, ", {}", self.t)?;
        if !self.go {
            write!(f, " X")?;
        }
        Ok(())
    }
}

/// Write a word as a sum of multiples of P-digits in the coding-sheet style,
/// e.g. `5P1 + 12P5`. Each term is a 5-digit group starting at P1, P5, P10,
/// P15, P17, P22, P26, P31 or P32 (the instruction fields), so that an
/// instruction-shaped constant reads naturally.
pub fn p_notation(word: u32) -> String {
    if word == 0 {
        return "0".into();
    }
    // field starts and widths: P1(1) P2-4(3) P5(5) P10(5) P15(2) P17(5) P22(4) P26(5) P31(1) P32(1)
    const FIELDS: [(u32, u32); 10] = [
        (1, 1),
        (2, 3),
        (5, 5),
        (10, 5),
        (15, 2),
        (17, 5),
        (22, 4),
        (26, 5),
        (31, 1),
        (32, 1),
    ];
    let mut terms = Vec::new();
    for (start, width) in FIELDS {
        let v = (word >> (start - 1)) & ((1u32 << width) - 1);
        if v != 0 {
            terms.push(if v == 1 {
                format!("P{start}")
            } else {
                format!("{v}P{start}")
            });
        }
    }
    terms.join(" + ")
}

/// Parse a constant as the UTECOM coding sheets write it, or in plainer
/// forms. A constant is a sum of terms joined by `+` or `-` (modulo 2^32):
///
/// * `nPk` or `Pk`: n times P*k*;
/// * `Pa,b-c,...`: the digits a and b to c set (the sheets' subscript list,
///   e.g. `P17,22-32`, `P1-7`);
/// * `g0,g1,g2,g3,g4,g5.h`: six base-32 digits least significant first
///   (P1-5, P6-10, ..., P26-30) and h (0-3) in P31-P32, e.g.
///   `26,12,30,4,8,13.1` = 1518500250 (inferred from the GEORGE flow
///   diagrams, READING.md);
/// * a decimal integer (signed, units of P1), `0x...` hex, `b...` binary
///   (most significant digit first).
pub fn parse_constant(text: &str) -> Result<u32, String> {
    let s: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    if s.is_empty() {
        return Err("empty constant".into());
    }
    let b = s.as_bytes();
    let mut total: u32 = 0;
    let mut i = 0;
    let mut first = true;
    while i < b.len() {
        let mut sign = 1;
        if b[i] == b'+' || b[i] == b'-' {
            sign = if b[i] == b'-' { -1 } else { 1 };
            i += 1;
        } else if !first {
            return Err(format!("expected + or - in {text:?}"));
        }
        first = false;
        // a term runs to the next + or -, except that within a P-list a "-"
        // followed by a digit is a range
        let start = i;
        let is_plist = b.get(i).is_some_and(|c| c.eq_ignore_ascii_case(&b'P'))
            || (b[i..].iter().take_while(|c| c.is_ascii_digit()).count() > 0
                && b.get(i + b[i..].iter().take_while(|c| c.is_ascii_digit()).count())
                    .is_some_and(|c| c.eq_ignore_ascii_case(&b'P')));
        let mut seen_p = false;
        while i < b.len() {
            let c = b[i];
            if c == b'P' || c == b'p' {
                seen_p = true;
            }
            if c == b'+' {
                break;
            }
            if c == b'-' && i > start {
                let range = is_plist && seen_p && b.get(i + 1).is_some_and(|d| d.is_ascii_digit());
                if !range {
                    break;
                }
            }
            i += 1;
        }
        let term = &s[start..i];
        let v = parse_term(term).ok_or_else(|| format!("cannot read term {term:?} in {text:?}"))?;
        total = if sign > 0 {
            total.wrapping_add(v)
        } else {
            total.wrapping_sub(v)
        };
    }
    Ok(total)
}

fn parse_term(term: &str) -> Option<u32> {
    if let Some(h) = term.strip_prefix("0x").or_else(|| term.strip_prefix("0X")) {
        return u32::from_str_radix(h, 16).ok();
    }
    if let Some(b) = term.strip_prefix('b') {
        return u32::from_str_radix(b, 2).ok();
    }
    let upper = term.to_ascii_uppercase();
    if let Some(i) = upper.find('P') {
        let mult: u64 = if i == 0 { 1 } else { upper[..i].parse().ok()? };
        let list = &upper[i + 1..];
        if list.contains(',') || list.contains('-') {
            if mult != 1 {
                return None;
            }
            let mut w = 0u32;
            for item in list.split(',') {
                let (a, z) = match item.split_once('-') {
                    Some((a, z)) => (a.parse::<u32>().ok()?, z.parse::<u32>().ok()?),
                    None => {
                        let a = item.parse::<u32>().ok()?;
                        (a, a)
                    }
                };
                if !(1..=32).contains(&a) || !(a..=32).contains(&z) {
                    return None;
                }
                for k in a..=z {
                    w |= p(k);
                }
            }
            return Some(w);
        }
        let k: u32 = list.parse().ok()?;
        if !(1..=32).contains(&k) {
            return None;
        }
        return Some((mult.wrapping_mul(1u64 << (k - 1))) as u32);
    }
    if term.contains(',') {
        let (groups, top) = term.split_once('.').unwrap_or((term, "0"));
        let groups: Vec<u32> = groups.split(',').map(|g| g.parse().ok()).collect::<Option<_>>()?;
        let top: u32 = top.parse().ok()?;
        if groups.len() > 6 || groups.iter().any(|&g| g > 31) || top > 3 {
            return None;
        }
        let mut w = top << 30;
        for (i, g) in groups.iter().enumerate() {
            w |= g << (5 * i);
        }
        return Some(w);
    }
    let v: i64 = term.parse().ok()?;
    Some(v as u32)
}

/// The digits of a word least significant first, as DEUCE wrote them
/// ("backward binary", manual §2.7, p. 8), e.g. for the trace.
pub fn backward_binary(word: u32) -> String {
    (0..32).map(|i| if word >> i & 1 == 1 { '1' } else { '0' }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        for w in [0u32, 0xffff_ffff, 0x1234_5678, 0x8000_0001, 0xdead_beef] {
            assert_eq!(Instr::decode(w).encode(), w);
        }
    }

    #[test]
    fn constants() {
        assert_eq!(parse_constant("5P1 + 12P5").unwrap(), 5 + 12 * 16);
        assert_eq!(parse_constant("P32").unwrap(), 0x8000_0000);
        assert_eq!(parse_constant("-1").unwrap(), u32::MAX);
        assert_eq!(parse_constant("7P22").unwrap(), 7 << 21);
        assert_eq!(parse_constant("20").unwrap(), 20);
        assert_eq!(parse_constant("P17-P1").unwrap(), (1 << 16) - 1);
        assert_eq!(parse_constant("P1-7").unwrap(), 127);
        assert_eq!(parse_constant("P17,22-32").unwrap(), p(17) | (0x7ff << 21));
        assert_eq!(parse_constant("P2,5,22,27").unwrap(), p(2) | p(5) | p(22) | p(27));
        assert_eq!(parse_constant("26,12,30,4,8,13.1").unwrap(), 1_518_500_250);
        assert_eq!(parse_constant("15P1 + 15P5 + P32").unwrap(), 15 + 15 * 16 + p(32));
        assert_eq!(parse_constant("P5,22 - P1").unwrap(), p(5) + p(22) - 1);
    }
}
