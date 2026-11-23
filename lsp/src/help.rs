//! What each symbol of G-code does, for hover help: the coding-table entry
//! (from the interpreter's own `Sym::code`), whether it is monadic or
//! diadic, what it does to the running accumulator, and a few words of the
//! 1958 manual.

use george::machine::{self, StopKind};
use george::symbols::Sym;

pub struct Help {
    /// What the symbol is, in a few words.
    pub title: &'static str,
    /// "monadic operator", "diadic operator", "control" and so on.
    pub kind: &'static str,
    /// The accumulator before and after, last cell on the right.
    pub effect: &'static str,
    /// Words of the manual, and where they are.
    pub manual: &'static str,
    pub section: &'static str,
    /// Other ways of writing it in the text form.
    pub spellings: &'static str,
    /// A stop it can give.
    pub stop: Option<StopKind>,
}

const fn h(
    title: &'static str,
    kind: &'static str,
    effect: &'static str,
    manual: &'static str,
    section: &'static str,
) -> Help {
    Help {
        title,
        kind,
        effect,
        manual,
        section,
        spellings: "",
        stop: None,
    }
}

impl Help {
    const fn spelt(mut self, spellings: &'static str) -> Help {
        self.spellings = spellings;
        self
    }
    const fn stops(mut self, stop: StopKind) -> Help {
        self.stop = Some(stop);
        self
    }
}

const TABLE: &str = "1.1 and its table";

pub fn of(sym: Sym) -> Help {
    use Sym::*;
    match sym {
        Comma => h(
            "comma: separates two numbers",
            "separator, omitted in translation",
            "… → …   (no change)",
            "When two numbers must be written consecutively they should be separated by a comma (special symbol, omitted in translation).",
            "1.2",
        ),
        Cancel => h(
            "cancel the last cell",
            "special symbol",
            "… a → …",
            "A name does not, however, \"cancel\" the last occupied cell of the accumulator. For this purpose a special symbol \";\" is used if required.",
            "1.4",
        )
        .stops(machine::ACCUMULATOR),
        Star => h(
            "mark: the head of a subroutine, or a place marker",
            "mark, omitted in translation",
            "… → …   (no change)",
            "A place marker, consisting of the symbol \"*\" followed by a single numeral symbol ... may be inserted anywhere in a program: it will be omitted by the translator, which however notes in a special list called the \"mark list\" the address of the next instruction following.",
            "1.8",
        )
        .stops(machine::MARK_TWICE),
        Wait => h(
            "wait for a single shot, showing the last cell",
            "control",
            "… → …   (no change)",
            "The symbol \"wait\" performs no operation, but holds up the calculation until a manual signal (single shot) is given, incidentally displaying on the output lights the number in the last occupied cell of the accumulator.",
            "1.9 (vii)",
        ),
        Add => h(
            "addition",
            "diadic operator",
            "… a b → … a+b",
            "Contents of last two occupied cells added together; cells cleared and result replaced.",
            "1.1",
        )
        .stops(machine::OVERFLOW),
        Sub => h("subtraction", "diadic operator", "… a b → … a−b", "diadic · subtraction · a − b · write a b −", TABLE)
            .spelt("`-` or `−`")
            .stops(machine::OVERFLOW),
        Mul => h("multiplication", "diadic operator", "… a b → … a×b", "diadic · multiplication · ab · write a b ×", TABLE)
            .spelt("`×` or `mul`")
            .stops(machine::OVERFLOW),
        Div => h("division", "diadic operator", "… a b → … a÷b", "diadic · division · a/b · write a b ÷", TABLE)
            .spelt("`÷`, `/` or `div`")
            .stops(machine::DIVISOR_ZERO),
        Neg => h("minus sign", "monadic operator", "… a → … −a", "monadic · minus sign · −a · write a neg", TABLE),
        Mod => h("modulus", "monadic operator", "… a → … |a|", "monadic · modulus · |a| · write a mod", TABLE),
        Max => h(
            "maximum",
            "diadic operator",
            "… a b → … max(a, b)",
            "diadic · maximum · max(a, b) · write a b max",
            TABLE,
        ),
        Dup => h(
            "duplicate the last cell",
            "special operator",
            "… a → … a a",
            "\"dup\": this \"duplicates\" the contents of the last occupied cell in the next cell. For example: for (a + b)² write a b + dup ×.",
            "1.1",
        )
        .stops(machine::ACCUMULATOR),
        Rev => h(
            "interchange the last two cells",
            "special operator",
            "… a b → … b a",
            "\"rev\": this \"reverses\" (i.e. interchanges) the contents of the last two occupied cells.",
            "1.1",
        ),
        Eq => h(
            "equal to: a relation",
            "diadic relational operator",
            "… a b → … truth   (−1 if a = b, else 0)",
            "\"a b =\" gives \"truth\" if a = b, \"falsehood\" otherwise. ... A skip (\"↑\" or \"↓\") is treated as conditional whenever one of the two relational operators has preceded it (since the previous skip).",
            "1.8",
        ),
        Gt => h(
            "greater than: a relation",
            "diadic relational operator",
            "… a b → … truth   (−1 if a > b, else 0)",
            "\"a b >\" gives \"truth\" if a > b, \"falsehood\" otherwise. ... A skip is treated as conditional whenever one of the two relational operators has preceded it (since the previous skip).",
            "1.8",
        ),
        Bar => h(
            "bar sign: a single suffix for the letter that follows",
            "suffix",
            "… j → … aⱼ   (j|a)      … v j → … v   (j|(a) stores v in aⱼ)",
            "The bar sign simply takes the number in the last occupied cell and treats it as a suffix of the letter which follows.",
            "1.3",
        ),
        DoubleBar => h(
            "double-bar sign: two suffixes for the letter that follows",
            "suffix",
            "… i j → … aᵢⱼ   (i j||a)      … v i j → … v   (i j||(a) stores v in aᵢⱼ)",
            "Similarly the double-bar sign takes the numbers in the last two occupied cells. ... the letters a, b, c and d with double suffixes.",
            "1.3",
        ),
        Not => h(
            "not: logical negation",
            "monadic logical operator",
            "… t → … ~t",
            "\"~\" (or \"not\") is monadic and gives logical negation: that is, it changes \"truth\" to \"falsehood\" or vice versa.",
            "1.8",
        )
        .spelt("`~` or `not`"),
        And => h(
            "and: logical conjunction",
            "diadic logical operator",
            "… s t → … s&t",
            "\"&\" (or \"and\") is diadic and gives logical conjunction; i.e. \"truth\" if both operands have \"truth\", \"falsehood\" otherwise.",
            "1.8",
        )
        .spelt("`&` or `and`"),
        Or => h(
            "or: logical disjunction",
            "diadic logical operator",
            "… s t → … s∨t",
            "\"∨\" (or \"or\") is diadic and gives logical disjunction; i.e. \"truth\" if either or both operands have \"truth\", \"falsehood\" only if both have \"falsehood\".",
            "1.8",
        )
        .spelt("`∨` or `or`"),
        Bracket => h(
            "bracket: end of a repetition, a subroutine or the program",
            "control",
            "… → …   (no change)",
            "The bracket-symbol in fact has three uses: (i) to return a subroutine to the main program; (ii) to restart or terminate a repetitive operation; and (iii) to terminate a program. It fulfils the third function provided no subroutine or repetitive operation is in progress.",
            "1.7",
        ),
        Down => h(
            "skip to subroutine",
            "control",
            "… n → …   (conditional: … t n → …)",
            "A subroutine may be called in in the main program by giving its number, followed by the symbol \"↓\". When the subroutine is complete, the main program will automatically be resumed from the point of interruption.",
            "1.7",
        )
        .spelt("`↓` or `down`")
        .stops(machine::NO_MARK),
        Up => h(
            "skip to a mark",
            "control",
            "… n → …   (conditional: … t n → …)",
            "A \"skip\" instruction, or instruction to resume the program from a point so marked, may be given by writing the number of the mark followed by the symbol \"↑\". This symbol operates in the same way as the \"skip to subroutine\" symbol \"↓\" except that it makes no provision for later returning to the point of interruption.",
            "1.8",
        )
        .spelt("`↑` or `up`")
        .stops(machine::NO_MARK),
        Rep => h(
            "repeat, for the name that follows from a to b",
            "control",
            "… a b → …   (x = a, then a+1, … at each ])",
            "To perform an operation for each value of x, in unit steps, from a to b, write a b rep (x) ...... ]",
            "1.6",
        )
        .stops(machine::LINK_LIST),
        Keys => h(
            "the number set on the input keys",
            "input",
            "… → … k",
            "Symbol \"I\" operates to transfer a number set in on the input keys to the first vacant cell of the accumulator. This permits manual control of a program by means of discrimination.",
            "1.9 (viii)",
        ),
        Digit(_) | High(_) => h(
            "numeral",
            "number",
            "… → … n",
            "Sequences of these symbols (of not more than nine digits in length) are treated as numbers in the usual way. ... In operation the number concerned is placed in the first vacant cell of the accumulator.",
            "1.2",
        ),
        Var(_) => h(
            "variable",
            "variable",
            "… → … a",
            "When a variable occurs in a program its \"value\" (from an appropriate position in the main number-store) is placed in the first vacant cell of the accumulator.",
            "1.1",
        )
        .stops(machine::ACCUMULATOR),
        Name(_) => h(
            "name: store the last cell, leaving it there",
            "name",
            "… v → … v   (and a = v)",
            "A name-symbol is written as the corresponding variable in brackets, e.g. \"(a)\" (treated as a single symbol). ... such a combination of symbols operates to transfer the number in the last occupied cell to the appropriate location in the number store.",
            "1.4",
        ),
        Log => h("natural logarithm", "monadic operator", "… a → … logₑ a", "monadic · natural logarithm · logₑ a · write a log", TABLE)
            .stops(machine::LOG_ARGUMENT),
        Exp => h("exponential", "monadic operator", "… a → … eᵃ", "monadic · exponential · eᵃ · write a exp", TABLE)
            .stops(machine::EXP_RESULT),
        Pow => h("power", "diadic operator", "… a b → … aᵇ", "diadic · power · aᵇ · write a b pow", TABLE)
            .stops(machine::FRACTIONAL_POWER),
        Rem => h(
            "remainder on division",
            "diadic operator",
            "… a b → … rem(a, b)",
            "The \"remainder of a on division by b\" is defined as a number x such that nb + x = a, where n is an integer and 0 ≤ x < b. Note that \"a 1 rem\" gives the positive fractional part of a.",
            TABLE,
        )
        .stops(machine::REM_DIVISOR),
        Sqrt => h("square root", "monadic operator", "… a → … √a", "monadic · square root · √a · write a √", TABLE)
            .spelt("`√` or `sqrt`")
            .stops(machine::DIVISOR_ZERO),
        Sin => h("sine", "monadic operator", "… a → … sin a", "monadic · sine (angle a in radians) · write a sin", TABLE),
        Cos => h("cosine", "monadic operator", "… a → … cos a", "monadic · cosine (angle a in radians) · write a cos", TABLE),
        Read => h(
            "read a data card",
            "input",
            "… → … x",
            "The symbol \"R\" represents an instruction to read one number from a card and place it in the first vacant cell of the accumulator.",
            "1.5",
        )
        .stops(machine::DATA_WANTED),
        Punch => h(
            "punch the last cell",
            "output",
            "… v → … v   (v punched on a card)",
            "The symbol \"(P)\" represents an instruction to punch out the number in the last occupied cell of the accumulator. It is written in brackets to indicate that it operates like a name-symbol; i.e. that it does not \"cancel\" the cell.",
            "1.5",
        ),
        ReadVector => h(
            "read a vector of cards into the name that follows",
            "block input",
            "… a b → …   (xₐ … x_b read)",
            "To read in a sequence of numbers xₐ, xₐ₊₁, ... x_b write a b R| (x).",
            "1.5",
        )
        .stops(machine::DATA_WANTED),
        PunchVector => h(
            "punch a vector from the name that follows",
            "block output",
            "… a b → …   (xₐ … x_b punched)",
            "To punch out the same sequence of numbers write a b P| (x).",
            "1.5",
        ),
        ReadMatrix => h(
            "read a matrix of cards into the name that follows",
            "block input",
            "… i m j n → …   (aᵢⱼ … aₘₙ read, row by row)",
            "To read in a matrix of numbers aᵢⱼ ... aₘₙ write i m j n R|| (a). ... A matrix is read or punched row by row.",
            "1.5",
        )
        .stops(machine::DATA_WANTED),
        PunchMatrix => h(
            "punch a matrix from the name that follows",
            "block output",
            "… i m j n → …   (aᵢⱼ … aₘₙ punched, row by row)",
            "To punch out this matrix write i m j n P|| (a). A matrix is read or punched row by row.",
            "1.5",
        ),
    }
}

/// The words of the text form, for completion: the spelling, and the symbol.
pub const WORDS: &[(&str, Sym)] = &[
    ("dup", Sym::Dup),
    ("rev", Sym::Rev),
    ("rep", Sym::Rep),
    ("neg", Sym::Neg),
    ("mod", Sym::Mod),
    ("max", Sym::Max),
    ("pow", Sym::Pow),
    ("rem", Sym::Rem),
    ("log", Sym::Log),
    ("exp", Sym::Exp),
    ("sin", Sym::Sin),
    ("cos", Sym::Cos),
    ("sqrt", Sym::Sqrt),
    ("wait", Sym::Wait),
    ("mul", Sym::Mul),
    ("div", Sym::Div),
    ("down", Sym::Down),
    ("up", Sym::Up),
    ("not", Sym::Not),
    ("and", Sym::And),
    ("or", Sym::Or),
];
