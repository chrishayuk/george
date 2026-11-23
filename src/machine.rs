//! The GEORGE pseudo-machine (manual, page 3: "Block diagram of the GEORGE
//! pseudo-machine"): a running accumulator of 12 cells, a link list of six
//! entries, a mark list of 32, the store for unsuffixed variables (32
//! locations), the vector store (32 × 32) and the matrix store (4 × 32 × 32),
//! a reader and a punch.
//!
//! The program is obeyed a symbol at a time, as GEORGE II did it. GEORGE IA
//! first translated the symbols into keywords; the manual says the two
//! "accept the same programs and differ only in details of internal
//! operation".

use std::collections::VecDeque;

use crate::functions::{self, Fault};
use crate::number::{numeral_word, read_word, Card, Num, NumeralSym, Overflow};
use crate::symbols::Sym;
use crate::text::Program;

pub const ACCUMULATOR_CELLS: usize = 12;
pub const LINK_LIST_ENTRIES: usize = 6;
pub const SYMBOL_CAPACITY: usize = 512;

/// A stop from the table in section 2.4 of the manual.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StopKind {
    pub indication: &'static str,
    pub cause: &'static str,
    /// What a single shot does, if the stop is resumable.
    pub resume: Option<&'static str>,
}

macro_rules! stops {
    ($($name:ident = $ind:literal, $cause:literal, $resume:expr;)*) => {
        $(pub const $name: StopKind = StopKind { indication: $ind, cause: $cause, resume: $resume };)*
        pub const ALL_STOPS: &[StopKind] = &[$($name),*];
    };
}

stops! {
    TOO_MANY_SYMBOLS = "5 9-24", "more than 512 symbols in program", None;
    MARK_TWICE = "3 2-2x (IA), 0 2-2x (II)", "mark number used twice", Some("second entry written over first in mark list");
    ACCUMULATOR = "1 30-21x (IA), 1 1-1x (II)", "program uses too many accumulator cells or has instructed taking number from empty accumulator", Some("(II) calls in next symbol");
    LINK_LIST = "1 8-21x (IA), 1 3-3x (II)", "too many entries in link list (nested subroutines or repeats to higher than sixth order)", Some("(II) calls in next symbol");
    NO_MARK = "7 25-29", "program calls for skip to non-existent mark", None;
    OVERFLOW = "3 14-21x", "result of arithmetic operation exceeds permissible magnitude", Some("resumes with exponent halved");
    DIVISOR_ZERO = "1 4-24x", "divisor zero (÷) or argument negative (√)", Some("resumes giving result zero");
    REM_DIVISOR = "1 30-16x", "divisor negative or zero (rem)", Some("resumes giving result zero");
    LOG_ARGUMENT = "1 30-21x", "argument zero or negative (log)", Some("resumes giving result zero");
    EXP_RESULT = "7 30-21x", "result exceeds permissible magnitude (exp or pow)", Some("resumes giving result zero");
    FRACTIONAL_POWER = "5 30-21x", "fractional power of negative number (pow)", Some("resumes giving result zero");
    DATA_EXPONENT = "4 9-24", "data card with exponent too large", None;
    DATA_WANTED = "1 11-6x, 1 7-16x", "data card wanted", None;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Stop {
    pub kind: StopKind,
    /// The symbol being obeyed, if any.
    pub at: Option<usize>,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Punched(Num),
    /// "wait": the number in the last occupied cell on the output lights, or
    /// None for the special indication of an empty accumulator.
    Lights(Option<Num>),
    /// A resumable stop the operator got past with a single shot.
    Resumed(Stop),
}

#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Numbers set on the input keys, one taken for each "I".
    pub keys: Vec<f64>,
    /// Give a single shot at resumable stops instead of ending the run.
    pub single_shot: bool,
    /// Give up after this many symbols obeyed (0: no limit).
    pub limit: u64,
    /// Record every symbol obeyed and the accumulator after it.
    pub trace: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Outcome {
    pub events: Vec<Event>,
    /// None when the program came to its end.
    pub stop: Option<Stop>,
    pub obeyed: u64,
    pub trace: Vec<TraceLine>,
    pub limit_reached: bool,
}

impl Outcome {
    pub fn punched(&self) -> Vec<Num> {
        self.events
            .iter()
            .filter_map(|e| if let Event::Punched(n) = e { Some(*n) } else { None })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct TraceLine {
    pub at: usize,
    /// How many events (punches, lights, resumed stops) came before this
    /// symbol finished, so that a trace can be printed in order with them.
    pub events: usize,
    pub accumulator: Vec<Num>,
}

#[derive(Clone, Copy, Debug)]
enum Link {
    Subroutine { back: usize },
    Repeat { var: u8, body: usize, count: i64 },
}

pub struct Machine {
    accumulator: Vec<Num>,
    links: Vec<Link>,
    pub plain: [Num; 32],
    pub vector: Vec<Num>,
    pub matrix: Vec<Num>,
    pub cards: VecDeque<Card>,
}

type Step<T> = Result<T, Stop>;

impl Default for Machine {
    fn default() -> Self {
        Machine::new()
    }
}

impl Machine {
    pub fn new() -> Machine {
        Machine {
            accumulator: Vec::new(),
            links: Vec::new(),
            plain: [Num::ZERO; 32],
            vector: vec![Num::ZERO; 1024],
            matrix: vec![Num::ZERO; 4096],
            cards: VecDeque::new(),
        }
    }

    /// Location in the vector store. "When a single suffix exceeds 31, the
    /// storage location represented is one corresponding with a letter
    /// alphabetically following ... Negative suffixes are similarly
    /// interpreted modulo 1024" (section 1.9 (vi)).
    pub fn vector_location(var: u8, suffix: i64) -> usize {
        (var as i64 * 32 + suffix).rem_euclid(1024) as usize
    }

    /// Location in the matrix store: "the second suffix is interpreted
    /// modulo 32 and does not cause spill; but the first suffix is
    /// interpreted modulo 128 and causes spill through a, b, c and d".
    pub fn matrix_location(var: u8, first: i64, second: i64) -> usize {
        (var as i64 * 32 + first).rem_euclid(128) as usize * 32 + second.rem_euclid(32) as usize
    }

    /// Run a G-program. The accumulator is cleared first ("The accumulator
    /// is cleared between programs, but the number store is not").
    pub fn run(&mut self, program: &Program, options: &Options) -> Outcome {
        self.accumulator.clear();
        self.links.clear();
        let mut out = Outcome::default();
        let mut keys = options.keys.iter().copied();
        let r = self.obey(program, options, &mut keys, &mut out);
        if let Err(stop) = r {
            out.stop = Some(stop);
        }
        out
    }

    fn resumable(&self, stop: Stop, options: &Options, out: &mut Outcome) -> Step<()> {
        if options.single_shot && stop.kind.resume.is_some() {
            out.events.push(Event::Resumed(stop));
            Ok(())
        } else {
            Err(stop)
        }
    }

    fn pop(&mut self, at: usize) -> Step<Num> {
        self.accumulator.pop().ok_or(Stop {
            kind: ACCUMULATOR,
            at: Some(at),
            detail: Some("accumulator empty".into()),
        })
    }

    /// The last occupied cell, for the symbols that copy it without taking it
    /// away: (P), a name, dup. The stop is for "taking number from empty
    /// accumulator"; copying from an empty one gives 0 on Hamblin's code
    /// (`7 ; (P)` punches 0), so it does here.
    fn top(&self, _at: usize) -> Step<Num> {
        Ok(self.accumulator.last().copied().unwrap_or(Num::ZERO))
    }

    fn push(&mut self, n: Num, at: usize) -> Step<()> {
        if self.accumulator.len() == ACCUMULATOR_CELLS {
            return Err(Stop {
                kind: ACCUMULATOR,
                at: Some(at),
                detail: Some("all 12 cells occupied".into()),
            });
        }
        self.accumulator.push(n);
        Ok(())
    }

    /// A result back into GEORGE's form, with the overflow stop.
    fn result(&self, x: f64, at: usize, options: &Options, out: &mut Outcome) -> Step<Num> {
        self.checked(Num::from_f64(x), x, at, options, out)
    }

    /// A result already in GEORGE's form, or the overflow stop; `x` is the
    /// exact value, for the single-shot resumption with exponent halved.
    fn checked(&self, n: Result<Num, Overflow>, x: f64, at: usize, options: &Options, out: &mut Outcome) -> Step<Num> {
        match n {
            Ok(n) => Ok(n),
            Err(_) => {
                self.resumable(
                    Stop {
                        kind: OVERFLOW,
                        at: Some(at),
                        detail: None,
                    },
                    options,
                    out,
                )?;
                Ok(Num::halved_exponent(x))
            }
        }
    }

    fn failed(&self, kind: StopKind, at: usize, options: &Options, out: &mut Outcome) -> Step<Num> {
        self.resumable(
            Stop {
                kind,
                at: Some(at),
                detail: None,
            },
            options,
            out,
        )?;
        Ok(Num::ZERO)
    }

    /// The result of one of Hamblin's function routines, or its stop. On a
    /// single shot the word his routine stores is taken, except for √ of a
    /// negative number: his code stores 0.5 there (TS16 still holds the
    /// first approximation), the manual's stop table says zero, and this
    /// follows the manual (RUNNING-GEORGE.md).
    fn function(&self, f: functions::Outcome, at: usize, options: &Options, out: &mut Outcome) -> Step<Num> {
        match f {
            functions::Outcome::Word(w) => Ok(Num::from_word(w)),
            functions::Outcome::Stop { kind, resumed } => {
                let (kind, resumed) = match kind {
                    Fault::SqrtNegative => (DIVISOR_ZERO, 0),
                    Fault::LogArgument => (LOG_ARGUMENT, resumed),
                    Fault::ExpRange => (EXP_RESULT, resumed),
                    Fault::Overflow => (OVERFLOW, resumed),
                    Fault::FractionalPower => (FRACTIONAL_POWER, resumed),
                    Fault::RemDivisor => (REM_DIVISOR, resumed),
                };
                self.resumable(
                    Stop {
                        kind,
                        at: Some(at),
                        detail: None,
                    },
                    options,
                    out,
                )?;
                Ok(Num::from_word(resumed))
            }
        }
    }

    fn read_card(&mut self, at: usize) -> Step<Num> {
        let card = self.cards.pop_front().ok_or(Stop {
            kind: DATA_WANTED,
            at: Some(at),
            detail: None,
        })?;
        // Hamblin's read translate (`number::read_word`).
        read_word(card).map(Num::from_word).map_err(|_| Stop {
            kind: DATA_EXPONENT,
            at: Some(at),
            detail: None,
        })
    }

    fn marks(&self, s: &[Sym], options: &Options, out: &mut Outcome) -> Step<[Option<usize>; 32]> {
        let mut marks = [None; 32];
        for k in 0..s.len() {
            if s[k] == Sym::Star && (k == 0 || s[k - 1] != Sym::Star) {
                if let Some(Sym::Digit(n) | Sym::High(n)) = s.get(k + 1) {
                    let n = *n as usize;
                    if marks[n].is_some() {
                        let stop = Stop {
                            kind: MARK_TWICE,
                            at: Some(k),
                            detail: Some(format!("mark {n}")),
                        };
                        self.resumable(stop, options, out)?;
                    }
                    marks[n] = Some(k + 2);
                }
            }
        }
        Ok(marks)
    }

    fn obey(
        &mut self,
        program: &Program,
        options: &Options,
        keys: &mut impl Iterator<Item = f64>,
        out: &mut Outcome,
    ) -> Step<()> {
        let s = &program.symbols;
        if s.len() > SYMBOL_CAPACITY {
            return Err(Stop {
                kind: TOO_MANY_SYMBOLS,
                at: None,
                detail: Some(format!("{} symbols", s.len())),
            });
        }
        let marks = self.marks(s, options, out)?;
        let mut pc = 0;
        let mut conditional = false;
        while pc < s.len() {
            if options.limit > 0 && out.obeyed >= options.limit {
                out.limit_reached = true;
                return Ok(());
            }
            out.obeyed += 1;
            let at = pc;
            let sym = s[pc];
            pc += 1;
            let saved = if options.single_shot {
                self.accumulator.clone()
            } else {
                Vec::new()
            };
            // Some(true) when the program has come to its end.
            let step = (|| -> Step<bool> {
                match sym {
                    Sym::Digit(_) | Sym::High(_) => {
                        // A numeral sequence: digits and a decimal point, ended by
                        // the first symbol that is not a numeral, converted at
                        // translation time as Hamblin's translator converts it
                        // (`number::numeral_word`). With at most nine digits it
                        // cannot overflow.
                        let mut syms = Vec::new();
                        let mut k = at;
                        while k < s.len() && s[k].is_numeral() {
                            match s[k] {
                                Sym::Digit(d) if d < 10 => syms.push(NumeralSym::Digit(d)),
                                Sym::High(_) => syms.push(NumeralSym::Point),
                                other => {
                                    return Err(Stop {
                                        kind: ACCUMULATOR,
                                        at: Some(k),
                                        detail: Some(format!("{} is a numeral only after an asterisk", other.text())),
                                    })
                                }
                            }
                            k += 1;
                        }
                        pc = k;
                        self.push(Num::from_word(numeral_word(syms)), at)?;
                    }
                    Sym::Comma => {}
                    Sym::Star => pc += 1,
                    Sym::Cancel => {
                        self.pop(at)?;
                    }
                    Sym::Wait => out.events.push(Event::Lights(self.accumulator.last().copied())),
                    Sym::Add | Sym::Sub | Sym::Mul | Sym::Div | Sym::Max | Sym::Pow | Sym::Rem => {
                        let b = self.pop(at)?;
                        let a = self.pop(at)?;
                        let (x, y) = (a.to_f64(), b.to_f64());
                        let n = match sym {
                            // + and - word for word as Hamblin's add routine does
                            // them, not rounded to nearest (DECISIONS.md 2)
                            Sym::Add => self.checked(a.plus(b), x + y, at, options, out)?,
                            Sym::Sub => self.checked(a.minus(b), x - y, at, options, out)?,
                            // × and ÷ word for word as Hamblin's routines do them
                            // (DECISIONS.md 2)
                            Sym::Mul => self.checked(a.times(b), x * y, at, options, out)?,
                            // "A zero dividend goes straight to standardise" (flow diagrams
                            // p. 20), before the divisor is looked at: 0 ÷ 0 is 0, no stop.
                            Sym::Div if x == 0.0 => Num::ZERO,
                            Sym::Div if y == 0.0 => self.failed(DIVISOR_ZERO, at, options, out)?,
                            Sym::Div => self.checked(a.over(b), x / y, at, options, out)?,
                            Sym::Max => {
                                if x >= y {
                                    a
                                } else {
                                    b
                                }
                            }
                            // rem and pow word for word as Hamblin's routines form
                            // them (src/functions.rs; RUNNING-GEORGE.md, findings 18-24)
                            Sym::Rem => self.function(functions::rem(a.to_word(), b.to_word()), at, options, out)?,
                            _ => self.function(functions::pow(a.to_word(), b.to_word()), at, options, out)?,
                        };
                        self.push(n, at)?;
                    }
                    Sym::Neg | Sym::Mod | Sym::Log | Sym::Exp | Sym::Sqrt | Sym::Sin | Sym::Cos => {
                        let a = self.pop(at)?;
                        // word for word as Hamblin's routines form them
                        // (src/functions.rs; RUNNING-GEORGE.md, findings 18-24)
                        let w = a.to_word();
                        let f = match sym {
                            Sym::Neg => functions::neg(w),
                            Sym::Mod => functions::modulus(w),
                            Sym::Log => functions::log(w),
                            Sym::Exp => functions::exp(w),
                            Sym::Sqrt => functions::sqrt(w),
                            Sym::Sin => functions::sin_cos(w, true),
                            _ => functions::sin_cos(w, false),
                        };
                        let n = self.function(f, at, options, out)?;
                        self.push(n, at)?;
                    }
                    Sym::Dup => {
                        let a = self.top(at)?;
                        self.push(a, at)?;
                    }
                    Sym::Rev => {
                        let b = self.pop(at)?;
                        let a = self.pop(at)?;
                        self.push(b, at)?;
                        self.push(a, at)?;
                    }
                    Sym::Eq | Sym::Gt => {
                        let (b, a) = (self.pop(at)?, self.pop(at)?);
                        // `=` compares the two words digit by digit (flow diagrams
                        // p. 22), which differs from comparing values only for the
                        // unstandardised words the logical operators make.
                        let truth = if sym == Sym::Eq {
                            a.to_word() == b.to_word()
                        } else {
                            a.to_f64() > b.to_f64()
                        };
                        self.push(truth_value(truth), at)?;
                        conditional = true;
                    }
                    // The flow diagrams (p. 22) do these on the whole word: ~ is
                    // the complement of every digit, & and ∨ are digit by digit.
                    // Only the sign digit matters to a skip.
                    Sym::Not => {
                        let a = self.pop(at)?.to_word();
                        self.push(Num::from_word(!a), at)?;
                    }
                    Sym::And | Sym::Or => {
                        let b = self.pop(at)?.to_word();
                        let a = self.pop(at)?.to_word();
                        self.push(Num::from_word(if sym == Sym::And { a & b } else { a | b }), at)?;
                    }
                    Sym::Var(v) => self.push(self.plain[v as usize], at)?,
                    Sym::Name(v) => self.plain[v as usize] = self.top(at)?,
                    Sym::Bar => {
                        let suffix = self.pop(at)?.to_index();
                        let loc = match s[pc] {
                            Sym::Var(v) | Sym::Name(v) => Machine::vector_location(v, suffix),
                            _ => unreachable!("checked when read"),
                        };
                        if let Sym::Var(_) = s[pc] {
                            self.push(self.vector[loc], at)?;
                        } else {
                            self.vector[loc] = self.top(at)?;
                        }
                        pc += 1;
                    }
                    Sym::DoubleBar => {
                        let j = self.pop(at)?.to_index();
                        let i = self.pop(at)?.to_index();
                        let loc = match s[pc] {
                            Sym::Var(v) | Sym::Name(v) => Machine::matrix_location(v, i, j),
                            _ => unreachable!("checked when read"),
                        };
                        if let Sym::Var(_) = s[pc] {
                            self.push(self.matrix[loc], at)?;
                        } else {
                            self.matrix[loc] = self.top(at)?;
                        }
                        pc += 1;
                    }
                    Sym::Rep => {
                        let Sym::Name(var) = s[pc] else {
                            unreachable!("checked when read")
                        };
                        let b = self.pop(at)?;
                        let a = self.pop(at)?;
                        self.plain[var as usize] = a;
                        pc += 1;
                        // The count-down is held in P1-16 of the link (flow diagrams p. 26),
                        // so a > b wraps round to a very long repetition.
                        let count = (b.to_index() - a.to_index()).rem_euclid(1 << 16);
                        self.link(Link::Repeat { var, body: pc, count }, at)?;
                    }
                    Sym::Bracket => match self.links.pop() {
                        None => return Ok(true), // end of program
                        Some(Link::Subroutine { back }) => pc = back,
                        Some(Link::Repeat { var, body, count }) => {
                            if count > 0 {
                                let next = self.plain[var as usize].to_f64() + 1.0;
                                self.plain[var as usize] = self.result(next, at, options, out)?;
                                self.links.push(Link::Repeat {
                                    var,
                                    body,
                                    count: count - 1,
                                });
                                pc = body;
                            }
                        }
                    },
                    Sym::Down | Sym::Up => {
                        let n = self.pop(at)?.to_index().rem_euclid(32) as usize;
                        let take = if conditional { self.pop(at)?.is_negative() } else { true };
                        conditional = false;
                        if take {
                            let target = marks[n].ok_or(Stop {
                                kind: NO_MARK,
                                at: Some(at),
                                detail: Some(format!("mark number {n}")),
                            })?;
                            if sym == Sym::Down {
                                self.link(Link::Subroutine { back: pc }, at)?;
                            }
                            pc = target;
                        }
                    }
                    Sym::Keys => {
                        let x = keys.next().unwrap_or(0.0);
                        let n = self.result(x, at, options, out)?;
                        self.push(n, at)?;
                    }
                    Sym::Read => {
                        let n = self.read_card(at)?;
                        self.push(n, at)?;
                    }
                    Sym::Punch => out.events.push(Event::Punched(self.top(at)?)),
                    Sym::ReadVector | Sym::PunchVector => {
                        let Sym::Name(v) = s[pc] else {
                            unreachable!("checked when read")
                        };
                        pc += 1;
                        let last = self.pop(at)?.to_index();
                        let first = self.pop(at)?.to_index();
                        for k in first..=last {
                            let loc = Machine::vector_location(v, k);
                            if sym == Sym::ReadVector {
                                self.vector[loc] = self.read_card(at)?;
                            } else {
                                out.events.push(Event::Punched(self.vector[loc]));
                            }
                        }
                    }
                    Sym::ReadMatrix | Sym::PunchMatrix => {
                        let Sym::Name(v) = s[pc] else {
                            unreachable!("checked when read")
                        };
                        pc += 1;
                        let n = self.pop(at)?.to_index();
                        let j = self.pop(at)?.to_index();
                        let m = self.pop(at)?.to_index();
                        let i = self.pop(at)?.to_index();
                        for row in i..=m {
                            for col in j..=n {
                                let loc = Machine::matrix_location(v, row, col);
                                if sym == Sym::ReadMatrix {
                                    self.matrix[loc] = self.read_card(at)?;
                                } else {
                                    out.events.push(Event::Punched(self.matrix[loc]));
                                }
                            }
                        }
                    }
                }
                Ok(false)
            })();
            match step {
                Ok(true) => return Ok(()),
                Ok(false) => {}
                // GEORGE II: "S.S. calls in next symbol" after a stop on the
                // accumulator or the link list. The symbol is left undone.
                Err(stop) if options.single_shot && (stop.kind == ACCUMULATOR || stop.kind == LINK_LIST) => {
                    out.events.push(Event::Resumed(stop));
                    // The symbol is left undone: the accumulator is as it was
                    // before it, and a numeral is skipped as a whole.
                    self.accumulator = saved;
                    pc = at + 1;
                    if sym.is_numeral() {
                        while pc < s.len() && s[pc].is_numeral() {
                            pc += 1;
                        }
                    }
                }
                Err(stop) => return Err(stop),
            }
            if options.trace {
                out.trace.push(TraceLine {
                    at,
                    events: out.events.len(),
                    accumulator: self.accumulator.clone(),
                });
            }
        }
        Ok(())
    }

    fn link(&mut self, link: Link, at: usize) -> Step<()> {
        if self.links.len() == LINK_LIST_ENTRIES {
            return Err(Stop {
                kind: LINK_LIST,
                at: Some(at),
                detail: None,
            });
        }
        self.links.push(link);
        Ok(())
    }

    pub fn accumulator(&self) -> &[Num] {
        &self.accumulator
    }
}

/// Truth is the word with only the sign digit, P32 (flow diagrams p. 22),
/// which as a number is -1: "as constants "0" may be written to represent
/// "falsehood" and "-1" (i.e. "1 neg") for "truth"."
fn truth_value(t: bool) -> Num {
    if t {
        Num::from_word(1 << 31)
    } else {
        Num::ZERO
    }
}
