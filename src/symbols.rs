//! The G-code symbols and their codes: Appendix I of the manual, the
//! "GEORGE Coding Table".
//!
//! "The code of each symbol is its row number followed by its column
//! number. For example "+" is 5/0, and "n" is 13/4." Inside the machine a
//! code is eight binary digits, and "the 'number' of a variable (or name) is
//! given by the five least significant digits of its binary symbol-code"
//! (Appendix II). That only comes out right for every letter if the column
//! is the high half of the byte, so that is how `Sym::byte` builds it.

/// The 32 letters of the GEORGE alphabet, in the order of their variable
/// numbers: column 4 of the coding table, then column 5. There is no "o".
pub const LETTERS: [&str; 32] = [
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "θ", "p", //
    "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "α", "β", "γ", "λ", "μ", "ν",
];

/// ASCII spellings accepted for the Greek letters.
pub const GREEK_ASCII: [(&str, u8); 7] = [
    ("theta", 14),
    ("alpha", 26),
    ("beta", 27),
    ("gamma", 28),
    ("lambda", 29),
    ("mu", 30),
    ("nu", 31),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sym {
    // column 0
    Comma,
    Cancel,
    Star,
    Wait,
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    Mod,
    Max,
    Dup,
    Rev,
    Eq,
    Gt,
    // column 1
    Bar,
    DoubleBar,
    Not,
    And,
    Or,
    Bracket,
    /// "↓", skip to subroutine
    Down,
    /// "↑", skip
    Up,
    Rep,
    /// "I", the number set on the input keys
    Keys,
    /// column 2: the numerals 0 to 9 and the single symbols 10 to 15
    Digit(u8),
    /// column 3: the numbers 16 to 31 after an asterisk, otherwise a decimal point
    High(u8),
    /// columns 4 and 5: a variable, by number 0 to 31
    Var(u8),
    /// columns 6 and 7: a name, "(a)" and so on
    Name(u8),
    // column 8
    Log,
    Exp,
    Pow,
    Rem,
    Sqrt,
    Sin,
    Cos,
    // column 15
    Read,
    Punch,
    ReadVector,
    PunchVector,
    ReadMatrix,
    PunchMatrix,
}

/// The symbol written for a decimal point. Any symbol of column 3 serves;
/// the lexer uses the first.
pub const POINT: Sym = Sym::High(16);

impl Sym {
    /// Row and column in the coding table.
    pub fn code(self) -> (u8, u8) {
        use Sym::*;
        match self {
            Comma => (1, 0),
            Cancel => (2, 0),
            Star => (3, 0),
            Wait => (4, 0),
            Add => (5, 0),
            Sub => (6, 0),
            Mul => (7, 0),
            Div => (8, 0),
            Neg => (9, 0),
            Mod => (10, 0),
            Max => (11, 0),
            Dup => (12, 0),
            Rev => (13, 0),
            Eq => (14, 0),
            Gt => (15, 0),
            Bar => (0, 1),
            DoubleBar => (1, 1),
            Not => (2, 1),
            And => (3, 1),
            Or => (4, 1),
            Bracket => (5, 1),
            Down => (6, 1),
            Up => (7, 1),
            Rep => (8, 1),
            Keys => (9, 1),
            Digit(d) => (d, 2),
            High(n) => (n - 16, 3),
            Var(v) => (v % 16, 4 + v / 16),
            Name(v) => (v % 16, 6 + v / 16),
            Log => (0, 8),
            Exp => (1, 8),
            Pow => (2, 8),
            Rem => (3, 8),
            Sqrt => (4, 8),
            Sin => (5, 8),
            Cos => (6, 8),
            Read => (0, 15),
            Punch => (1, 15),
            ReadVector => (8, 15),
            PunchVector => (9, 15),
            ReadMatrix => (10, 15),
            PunchMatrix => (11, 15),
        }
    }

    /// The eight binary digits of the code, column in the high half.
    pub fn byte(self) -> u8 {
        let (row, col) = self.code();
        col << 4 | row
    }

    pub fn from_code(row: u8, col: u8) -> Option<Sym> {
        ALL.iter().copied().find(|s| s.code() == (row, col))
    }

    /// The symbol as the manual prints it.
    pub fn text(self) -> String {
        use Sym::*;
        match self {
            Comma => ",".into(),
            Cancel => ";".into(),
            Star => "*".into(),
            Wait => "wait".into(),
            Add => "+".into(),
            Sub => "-".into(),
            Mul => "×".into(),
            Div => "÷".into(),
            Neg => "neg".into(),
            Mod => "mod".into(),
            Max => "max".into(),
            Dup => "dup".into(),
            Rev => "rev".into(),
            Eq => "=".into(),
            Gt => ">".into(),
            Bar => "|".into(),
            DoubleBar => "||".into(),
            Not => "~".into(),
            And => "&".into(),
            Or => "∨".into(),
            Bracket => "]".into(),
            Down => "↓".into(),
            Up => "↑".into(),
            Rep => "rep".into(),
            Keys => "I".into(),
            Digit(d) => d.to_string(),
            High(n) => format!("{n}̄"),
            Var(v) => LETTERS[v as usize].into(),
            Name(v) => format!("({})", LETTERS[v as usize]),
            Log => "log".into(),
            Exp => "exp".into(),
            Pow => "pow".into(),
            Rem => "rem".into(),
            Sqrt => "√".into(),
            Sin => "sin".into(),
            Cos => "cos".into(),
            Read => "R".into(),
            Punch => "(P)".into(),
            ReadVector => "R|".into(),
            PunchVector => "P|".into(),
            ReadMatrix => "R||".into(),
            PunchMatrix => "P||".into(),
        }
    }

    pub fn is_numeral(self) -> bool {
        matches!(self, Sym::Digit(_) | Sym::High(_))
    }
}

/// Every symbol of the coding table, in column order.
pub const ALL: [Sym; 134] = {
    use Sym::*;
    let mut all = [Comma; 134];
    let col0 = [
        Comma, Cancel, Star, Wait, Add, Sub, Mul, Div, Neg, Mod, Max, Dup, Rev, Eq, Gt,
    ];
    let col1 = [Bar, DoubleBar, Not, And, Or, Bracket, Down, Up, Rep, Keys];
    let col8 = [Log, Exp, Pow, Rem, Sqrt, Sin, Cos];
    let col15 = [Read, Punch, ReadVector, PunchVector, ReadMatrix, PunchMatrix];
    let mut n = 0;
    let mut i = 0;
    while i < col0.len() {
        all[n] = col0[i];
        n += 1;
        i += 1;
    }
    i = 0;
    while i < col1.len() {
        all[n] = col1[i];
        n += 1;
        i += 1;
    }
    i = 0;
    while i < 16 {
        all[n] = Digit(i as u8);
        n += 1;
        i += 1;
    }
    i = 16;
    while i < 32 {
        all[n] = High(i as u8);
        n += 1;
        i += 1;
    }
    i = 0;
    while i < 32 {
        all[n] = Var(i as u8);
        n += 1;
        i += 1;
    }
    i = 0;
    while i < 32 {
        all[n] = Name(i as u8);
        n += 1;
        i += 1;
    }
    i = 0;
    while i < col8.len() {
        all[n] = col8[i];
        n += 1;
        i += 1;
    }
    i = 0;
    while i < col15.len() {
        all[n] = col15[i];
        n += 1;
        i += 1;
    }
    all
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_examples_of_codes() {
        assert_eq!(Sym::Add.code(), (5, 0));
        assert_eq!(Sym::Var(13).code(), (13, 4)); // "n" is 13/4
        assert_eq!(LETTERS[13], "n");
    }

    #[test]
    fn variable_number_is_five_low_digits_of_the_code() {
        for v in 0..32u8 {
            assert_eq!(Sym::Var(v).byte() & 31, v);
            assert_eq!(Sym::Name(v).byte() & 31, v);
        }
    }

    #[test]
    fn codes_are_distinct_and_round_trip() {
        for (i, a) in ALL.iter().enumerate() {
            for b in &ALL[i + 1..] {
                assert_ne!(a.code(), b.code(), "{a:?} {b:?}");
            }
            let (r, c) = a.code();
            assert_eq!(Sym::from_code(r, c), Some(*a));
        }
    }
}
