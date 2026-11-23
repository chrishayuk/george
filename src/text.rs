//! Reading a G-program written out as text.
//!
//! A G-program went to the machine as cards punched with symbol codes. The
//! text form here is the way the manual prints programs, so that its
//! examples can be typed as they stand:
//!
//! ```text
//! 1, 20 rep (j) j j × (P) ; ]
//! ```
//!
//! Where the printed symbol is not on a keyboard there is an ASCII
//! spelling: `mul` for ×, `/` for ÷, `down` and `up` for ↓ and ↑, `or` for
//! ∨, `not` for ~, `and` for &, `sqrt` for √, and `alpha`, `beta`, `gamma`,
//! `lambda`, `mu`, `nu` and `theta` for the Greek letters. `#` starts a
//! comment that runs to the end of the line.
//!
//! A run of letters that is not a word of the code is read as one variable
//! per letter, as the manual writes `ab+` for `a b +`. Words win: to write
//! the variables d, u, p one after another, separate them.
//!
//! The block read and punch symbols are written `R|`, `R||`, `P|` and `P||`
//! with nothing between the letter and the bar. `R |` with a space is the
//! read symbol followed by a bar sign.

use crate::symbols::{Sym, GREEK_ASCII, LETTERS, POINT};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
}

impl std::fmt::Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "line {}, column {}", self.line, self.col)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TextError {
    pub pos: Pos,
    pub message: String,
}

impl std::fmt::Display for TextError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.pos, self.message)
    }
}

/// A program: its symbols, and where each was written.
#[derive(Clone, Debug, Default)]
pub struct Program {
    pub symbols: Vec<Sym>,
    pub positions: Vec<Pos>,
}

const WORDS: [(&str, Sym); 21] = [
    ("wait", Sym::Wait),
    ("neg", Sym::Neg),
    ("mod", Sym::Mod),
    ("max", Sym::Max),
    ("dup", Sym::Dup),
    ("rev", Sym::Rev),
    ("rep", Sym::Rep),
    ("log", Sym::Log),
    ("exp", Sym::Exp),
    ("pow", Sym::Pow),
    ("rem", Sym::Rem),
    ("sin", Sym::Sin),
    ("cos", Sym::Cos),
    ("sqrt", Sym::Sqrt),
    ("not", Sym::Not),
    ("and", Sym::And),
    ("or", Sym::Or),
    ("mul", Sym::Mul),
    ("div", Sym::Div),
    ("down", Sym::Down),
    ("up", Sym::Up),
];

fn letter(s: &str) -> Option<u8> {
    LETTERS
        .iter()
        .position(|l| *l == s)
        .map(|v| v as u8)
        .or_else(|| GREEK_ASCII.iter().find(|(name, _)| *name == s).map(|(_, v)| *v))
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_lowercase() || "θαβγλμν".contains(c)
}

pub fn parse(src: &str) -> Result<Program, TextError> {
    let chars: Vec<char> = src.chars().collect();
    let mut program = Program::default();
    let mut i = 0;
    let (mut line, mut line_start) = (1, 0);
    let pos_of = |i: usize, line: usize, line_start: usize| Pos {
        line,
        col: i - line_start + 1,
    };

    while i < chars.len() {
        let c = chars[i];
        let pos = pos_of(i, line, line_start);
        let err = |message: String| TextError { pos, message };
        macro_rules! push {
            ($s:expr) => {{
                let sym = $s;
                program.symbols.push(sym);
                program.positions.push(pos);
            }};
        }
        if c == '\n' {
            line += 1;
            i += 1;
            line_start = i;
            continue;
        }
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == '#' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        let after_star = program.symbols.last() == Some(&Sym::Star);
        if c.is_ascii_digit() || (c == '.' && !after_star) {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let run: String = chars[start..i].iter().collect();
            if after_star {
                // "The number of a subroutine, as written at the beginning of
                // the subroutine itself, must be represented by a single
                // numeral symbol": 0 to 9, or the symbols 10 to 31.
                let n: u8 = run
                    .parse()
                    .ok()
                    .filter(|n| *n < 32)
                    .ok_or_else(|| err(format!("a mark is a single numeral symbol, 0 to 31, not {run}")))?;
                push!(if n < 16 { Sym::Digit(n) } else { Sym::High(n) });
            } else {
                for d in run.chars() {
                    push!(if d == '.' { POINT } else { Sym::Digit(d as u8 - b'0') });
                }
            }
            continue;
        }
        i += 1;
        let single = match c {
            ',' => Some(Sym::Comma),
            ';' => Some(Sym::Cancel),
            '*' => Some(Sym::Star),
            '+' => Some(Sym::Add),
            '-' | '−' => Some(Sym::Sub),
            '×' => Some(Sym::Mul),
            '÷' | '/' => Some(Sym::Div),
            '=' => Some(Sym::Eq),
            '>' => Some(Sym::Gt),
            '~' => Some(Sym::Not),
            '&' => Some(Sym::And),
            '∨' => Some(Sym::Or),
            ']' => Some(Sym::Bracket),
            '↓' => Some(Sym::Down),
            '↑' => Some(Sym::Up),
            '√' => Some(Sym::Sqrt),
            'I' => Some(Sym::Keys),
            _ => None,
        };
        if let Some(s) = single {
            push!(s);
            continue;
        }
        match c {
            '|' => {
                if chars.get(i) == Some(&'|') {
                    i += 1;
                    push!(Sym::DoubleBar);
                } else {
                    push!(Sym::Bar);
                }
            }
            'R' | 'P' => {
                let bars = if chars.get(i) == Some(&'|') && chars.get(i + 1) == Some(&'|') {
                    2
                } else if chars.get(i) == Some(&'|') {
                    1
                } else {
                    0
                };
                i += bars;
                push!(match (c, bars) {
                    ('R', 0) => Sym::Read,
                    ('R', 1) => Sym::ReadVector,
                    ('R', _) => Sym::ReadMatrix,
                    ('P', 1) => Sym::PunchVector,
                    ('P', 2) => Sym::PunchMatrix,
                    _ => return Err(err("P alone is not a symbol: punch is written (P)".into())),
                });
            }
            '(' => {
                let start = i;
                while i < chars.len() && chars[i] != ')' && chars[i] != '\n' {
                    i += 1;
                }
                if chars.get(i) != Some(&')') {
                    return Err(err(
                        "a name is a letter in brackets, and this bracket is not closed".into()
                    ));
                }
                let inner: String = chars[start..i].iter().collect::<String>().trim().to_string();
                i += 1;
                if inner == "P" {
                    push!(Sym::Punch);
                } else if let Some(v) = letter(&inner) {
                    push!(Sym::Name(v));
                } else {
                    return Err(err(format!(
                        "({inner}) is not a name: a name is one letter of the alphabet in brackets"
                    )));
                }
            }
            c if is_word_char(c) => {
                let start = i - 1;
                while i < chars.len() && is_word_char(chars[i]) {
                    i += 1;
                }
                let run: String = chars[start..i].iter().collect();
                if let Some((_, s)) = WORDS.iter().find(|(w, _)| *w == run) {
                    push!(*s);
                } else if let Some(v) = GREEK_ASCII.iter().find(|(w, _)| *w == run).map(|(_, v)| *v) {
                    push!(Sym::Var(v));
                } else {
                    for (k, l) in run.chars().enumerate() {
                        let p = Pos { line, col: pos.col + k };
                        match letter(&l.to_string()) {
                            Some(v) => {
                                program.symbols.push(Sym::Var(v));
                                program.positions.push(p);
                            }
                            None if l == 'o' => {
                                return Err(TextError {
                                    pos: p,
                                    message: "there is no letter o in the GEORGE alphabet".into(),
                                })
                            }
                            None => {
                                return Err(TextError {
                                    pos: p,
                                    message: format!("{run} is not a symbol of G-code"),
                                })
                            }
                        }
                    }
                }
            }
            _ => return Err(err(format!("{c} is not a symbol of G-code"))),
        }
    }
    check_marks(&program)?;
    Ok(program)
}

/// The asterisk must be followed by a numeral symbol, and the members of a
/// pair or group must not be separated (section 1.9 (iv)).
fn check_marks(program: &Program) -> Result<(), TextError> {
    let s = &program.symbols;
    // A numeral sequence is every numeral symbol in a row, however it was
    // spaced in the text: "Sequences of these symbols (of not more than nine
    // digits in length) are treated as numbers in the usual way." Two
    // decimal points in one are refused too.
    let mut k = 0;
    while k < s.len() {
        if s[k].is_numeral() && !(k > 0 && s[k - 1] == Sym::Star) {
            let start = k;
            while k < s.len() && s[k].is_numeral() {
                k += 1;
            }
            let run = &s[start..k];
            let points = run.iter().filter(|x| matches!(x, Sym::High(_))).count();
            let digits = run.len() - points;
            let pos = program.positions[start];
            if digits > 9 {
                return Err(TextError {
                    pos,
                    message: format!("a numeral sequence has not more than nine digits, and this has {digits}"),
                });
            }
            if points > 1 {
                return Err(TextError {
                    pos,
                    message: "a numeral sequence has at most one decimal point".into(),
                });
            }
        } else {
            k += 1;
        }
    }
    for (k, sym) in s.iter().enumerate() {
        let next = s.get(k + 1).copied();
        let pos = program.positions[k];
        match sym {
            Sym::Star if !next.is_some_and(Sym::is_numeral) => {
                return Err(TextError {
                    pos,
                    message: "an asterisk must be followed by a numeral".into(),
                })
            }
            Sym::Rep if !matches!(next, Some(Sym::Name(_))) => {
                return Err(TextError {
                    pos,
                    message: "rep must be followed by a name, as in rep (x)".into(),
                })
            }
            Sym::ReadVector | Sym::PunchVector | Sym::ReadMatrix | Sym::PunchMatrix
                if !matches!(next, Some(Sym::Name(_))) =>
            {
                return Err(TextError {
                    pos,
                    message: format!("{} must be followed by a name, as in {} (a)", sym.text(), sym.text()),
                })
            }
            Sym::Bar | Sym::DoubleBar if !matches!(next, Some(Sym::Var(_) | Sym::Name(_))) => {
                return Err(TextError {
                    pos,
                    message: "a bar sign must be followed by a variable or a name".into(),
                })
            }
            // "the letters a, b, c and d with double suffixes" (section 1.3)
            Sym::DoubleBar | Sym::ReadMatrix | Sym::PunchMatrix if matches!(next, Some(Sym::Var(v) | Sym::Name(v)) if v > 3) => {
                return Err(TextError {
                    pos,
                    message: "only a, b, c and d take double suffixes".into(),
                })
            }
            _ => {}
        }
    }
    Ok(())
}

/// Write symbols back out in the printed form.
pub fn format(symbols: &[Sym]) -> String {
    let mut out = String::new();
    let mut prev: Option<Sym> = None;
    // The numeral that names a mark is a symbol of its own: a numeral after
    // it must not be printed against it.
    let mut prev_is_mark = false;
    for &s in symbols {
        let joined = match (prev, s) {
            (Some(a), b) if a.is_numeral() && b.is_numeral() => prev != Some(Sym::Star) && !prev_is_mark,
            // `R|` is the block read; the read symbol and a bar are written apart
            (Some(Sym::Read), Sym::Bar | Sym::DoubleBar) => false,
            (Some(Sym::Bar | Sym::DoubleBar), _) => true,
            (_, Sym::Bar | Sym::DoubleBar | Sym::Comma | Sym::Cancel) => true,
            _ => false,
        };
        if !out.is_empty() && !joined {
            out.push(' ');
        }
        if s.is_numeral() && prev != Some(Sym::Star) {
            match s {
                Sym::Digit(d) if d < 10 => out.push_str(&d.to_string()),
                Sym::High(_) => out.push('.'),
                _ => out.push_str(&s.text()),
            }
        } else if let Sym::High(n) | Sym::Digit(n) = s {
            out.push_str(&n.to_string());
        } else {
            out.push_str(&s.text());
        }
        prev_is_mark = prev == Some(Sym::Star) && s.is_numeral();
        prev = Some(s);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use Sym::*;

    #[test]
    fn polynomial() {
        let p = parse("a y dup × × b y × + c + (e)").unwrap();
        assert_eq!(
            p.symbols,
            vec![
                Var(0),
                Var(24),
                Dup,
                Mul,
                Mul,
                Var(1),
                Var(24),
                Mul,
                Add,
                Var(2),
                Add,
                Name(4)
            ]
        );
    }

    #[test]
    fn numerals_marks_and_suffixes() {
        let p = parse("* 14 (β) 20 ÷ .5 pow j|a i j||(b)").unwrap();
        assert_eq!(
            p.symbols,
            vec![
                Star,
                Digit(14),
                Name(27),
                Digit(2),
                Digit(0),
                Div,
                POINT,
                Digit(5),
                Pow,
                Var(9),
                Bar,
                Var(0),
                Var(8),
                Var(9),
                DoubleBar,
                Name(1)
            ]
        );
    }

    #[test]
    fn block_input_and_read_then_bar() {
        assert_eq!(parse("1, 20 R| (a)").unwrap().symbols[4], ReadVector);
        assert_eq!(parse("R |a").unwrap().symbols, vec![Read, Bar, Var(0)]);
        assert_eq!(parse("ab+").unwrap().symbols, vec![Var(0), Var(1), Add]);
    }

    #[test]
    fn no_letter_o() {
        assert!(parse("o").is_err());
    }

    #[test]
    fn round_trip() {
        let src = "1, 20 rep (j) j j × (P); ]";
        let p = parse(src).unwrap();
        assert_eq!(parse(&format(&p.symbols)).unwrap().symbols, p.symbols);
    }
}
