//! A simulator of the English Electric DEUCE (Mark I), built from the DEUCE
//! Programming Manual NS-y-16/5-56 ("PM") and *Basic DEUCE Programming*,
//! NS y 88, 2nd issue, 1958 ("BP"). See `sources/deuce/MACHINE.md` for the
//! reading of the machine that this follows, with page references, and for
//! what is inferred.
//!
//! Time is counted in minor cycles (m.c., 32 microseconds) from an arbitrary
//! origin; the minor cycle *number* of time τ is τ mod 32, and words of a
//! DL, QS or DS are addressed by τ mod 32, 4 or 2.
//!
//! An instruction that enters TS COUNT at time m is obeyed by the rules of
//! PM §5.5-5.14 (pp. 25-33):
//!
//! * set-up in m + 1;
//! * C = 0: transfer in m+W+2, next instruction enters at m+T+2, where T is
//!   increased by 32 if W > T;
//! * C = 1: transfer from m+W+2 to m+T+2 inclusive (T + 32 if W > T);
//! * C = 2: transfer in m+W+2 and m+W+3, next at m+T+2 (T + 32 if W >= T);
//! * C = 3: as C = 1, except that W = T gives a 33 m.c. transfer ending at
//!   m+W+34, where the next instruction enters;
//! * the next instruction is the word in DL NIS (0 = DL8) at the m.c. it
//!   enters, or one m.c. later after a discrimination that succeeded
//!   (D27 negative, D28 non-zero, 2-24 TIL);
//! * a transfer to D0 that is still in progress when the next instruction
//!   enters supplies that instruction instead.

pub mod cards;
pub mod george;
pub mod loader;
pub mod word;

use std::collections::VecDeque;
use std::fmt;

pub use cards::Card;
pub use word::{p, Instr};

/// One major cycle: 32 minor cycles.
pub const MAJOR: u64 = 32;

/// Timing of a Hollerith machine, in minor cycles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HollerithTiming {
    /// From the call (12-24 or 10-24) on an idle machine to the first single shot.
    pub first_shot: u64,
    /// Between single shots of rows of the same card.
    pub row_interval: u64,
    /// From the single shot of row 9 of one card to row Y of the next.
    pub card_interval: u64,
    /// Reader: how long after its single shot source 0 gives a row.
    /// Punch: how long after its single shot the row is punched from the O.S.
    pub row_window: u64,
}

/// Choices the documents leave open, and timings.
#[derive(Clone, Debug)]
pub struct Config {
    pub reader: HollerithTiming,
    pub punch: HollerithTiming,
    /// A drum read or write: 13 major cycles (PM §6.4, p. 42).
    pub drum_transfer: u64,
    /// A head shift: PM §6.4 says 64 major cycles; BP §18 "about 35 ms".
    pub drum_shift: u64,
    /// Multiplication: 65 m.c. (PM §6.8, p. 46).
    pub mult_time: u64,
    /// Division: 66 m.c. (PM §6.10, p. 49).
    pub div_time: u64,
    /// The STOP key: every instruction behaves as a stopper (PM §4.19).
    pub stop_key: bool,
    /// The Automatic Instruction Modifier on 17-0 and 18-0 (PM appendix,
    /// pp. 82-84). Not fitted to a Mark I as delivered; off by default.
    pub aim: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            // PM Appendix 1, pp. 73-75: call to first S.S. >= 130 MC (read),
            // 180 (punch); rows 15-21 MC apart (read), >= 38 (punch); card
            // to card 80-100 MC (read), >= 116 (punch). For the reader the
            // middle of each range is taken (18 and 90 MC): GEORGE's matrix
            // read R|| needs more than 81 MC between cards (RUNNING-GEORGE.md).
            // For the punch, the minimum.
            // Source 0 gives the row for >= 2 MC and gives zeros from 6 MC;
            // 4 MC is taken. A row is punched if sent to D29 within 4 MC of
            // its stop instruction ("Successful read or punch": 2 / 4 MC).
            reader: HollerithTiming {
                first_shot: 130 * MAJOR,
                row_interval: 18 * MAJOR,
                card_interval: 90 * MAJOR,
                row_window: 4 * MAJOR,
            },
            punch: HollerithTiming {
                first_shot: 180 * MAJOR,
                row_interval: 38 * MAJOR,
                card_interval: 116 * MAJOR,
                row_window: 4 * MAJOR,
            },
            drum_transfer: 13 * MAJOR,
            drum_shift: 35 * MAJOR,
            mult_time: 65,
            div_time: 66,
            stop_key: false,
            aim: false,
        }
    }
}

/// Why `run` returned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// A stopper is in TS COUNT and neither the reader nor the punch is
    /// running: it waits for the single-shot key (`Machine::single_shot`).
    AwaitSingleShot,
    /// A stopper waits for the reader, which has no cards left.
    ReaderEmpty,
    /// An instruction that leads to itself and changes nothing, with no
    /// peripheral running: the machine is idling ("dynamic stop"). If the
    /// alarm is on, this is the usual end of a program (PM §4.12).
    Loop { alarm: bool },
    /// The instruction limit was reached.
    Limit,
}

/// One instruction obeyed, for the trace.
#[derive(Clone, Debug)]
pub struct Step {
    /// The time (m.c. count) it entered TS COUNT, after any stop.
    pub entered: u64,
    /// Where it came from: (DL, m.c.), or None from D0 or a card row.
    pub from: Option<(u8, u8)>,
    /// The drum track the instruction's DL was last filled from, if known.
    pub track: Option<u8>,
    pub word: u32,
    /// First and last m.c. of transfer.
    pub transfer: (u64, u64),
    /// When the next instruction entered, and from where.
    pub next: u64,
    pub next_from: Option<(u8, u8)>,
    /// The (last) word sent to the destination.
    pub value: u32,
    /// The time it waited for a single shot or the drum, if it did.
    pub waited: Option<u64>,
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let loc = |l: Option<(u8, u8)>| match l {
            Some((d, m)) => format!("{d}_{m}"),
            None => "Q".to_string(),
        };
        let ins = Instr::decode(self.word);
        let place = match (self.from, self.track) {
            (Some(_), Some(t)) => format!("{}@{}/{}", loc(self.from), t / 16, t % 16),
            _ => loc(self.from),
        };
        write!(
            f,
            "{:>9} {:>11} {:<22} mc {:>2}-{:>2}  {:>11} -> {} at {}",
            self.entered,
            place,
            ins.to_string(),
            self.transfer.0 % 32,
            self.transfer.1 % 32,
            self.value as i32,
            loc(self.next_from),
            self.next,
        )?;
        if let Some(w) = self.waited {
            write!(f, "  (waited from {w})")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
struct Reader {
    deck: VecDeque<Card>,
    running: bool,
    /// Row of the current card to be read next (0 = Y).
    row: usize,
    current: Option<Card>,
    next_shot: Option<u64>,
    /// The last row read and the time of its single shot.
    last: Option<(u64, u32, usize)>,
    /// Cards that have passed the brushes.
    cards_read: usize,
}

#[derive(Clone, Debug, Default)]
struct Punch {
    running: bool,
    row: usize,
    card: [u32; 12],
    next_shot: Option<u64>,
    last_shot: Option<(u64, usize)>,
    /// Rows waiting to be punched: time at which the O.S. is punched.
    pending: Option<u64>,
    /// Whether the punch was cleared during the current card.
    stopping: bool,
    output: Vec<Card>,
}

#[derive(Clone, Copy, Debug)]
enum Arith {
    /// The multiplier, digit by digit (Logical Design Manual NS-y-37/11-57,
    /// ch. 10, pp. 60-65): started in odd m.c. `start`; `y` is the multiplier
    /// digit examined last; `carry` is what the even m.c. owes the odd one;
    /// `next` is the next m.c. to be processed.
    Mult {
        start: u64,
        b: u32,
        y: bool,
        carry: u32,
        next: u64,
    },
    Div {
        done: u64,
        a: u32,
        b: u32,
    },
}

#[derive(Clone, Copy, Debug)]
enum DrumOp {
    Read(u8),
    Write(u8),
    Shift,
}

/// The machine.
#[derive(Clone)]
pub struct Machine {
    pub config: Config,
    /// DL1-12 (index 0 unused).
    pub dl: [[u32; 32]; 13],
    pub ts13: u32,
    pub ts14: u32,
    pub ts15: u32,
    pub ts16: u32,
    /// QS17, QS18.
    pub qs: [[u32; 4]; 2],
    /// DS19, DS20, DS21: [even m.c., odd m.c.].
    pub ds: [[u32; 2]; 3],
    pub tca: bool,
    pub tcb: bool,
    pub alarm: bool,
    /// The output staticiser lamps.
    pub os: u32,
    /// The input dynamiciser keys.
    pub id: u32,
    /// 256 tracks of 32 words.
    pub drum: Vec<[u32; 32]>,
    pub read_heads: u8,
    pub write_heads: u8,
    /// TS COUNT: the instruction about to be obeyed.
    pub count: u32,
    /// Where it came from.
    pub count_from: Option<(u8, u8)>,
    /// For each DL, the drum track it was last filled from (by a whole-DL
    /// transfer from DL11 after a read), for the trace.
    pub dl_origin: [Option<u8>; 13],
    /// When it entered TS COUNT.
    pub now: u64,
    /// Instructions obeyed.
    pub obeyed: u64,
    /// Things the program did that the manual says it must not.
    pub warnings: Vec<String>,
    /// Steps obeyed, when tracing.
    pub trace: Option<Vec<Step>>,
    reader: Reader,
    punch: Punch,
    arith: Option<Arith>,
    drum_op: Option<(u64, DrumOp)>,
    /// DS21 odd word: carry and sign extension owed from the even m.c.
    ds21_owed: Option<(u64, u32)>,
    /// Last write to DL10: (time, m.c., old word), for TCA.
    dl10_write: Option<(u64, usize, u32)>,
    console_shot: bool,
    /// Minor cycles from the stop to the key being pressed.
    console_delay: u64,
    /// Bumped whenever anything in the machine changes.
    version: u64,
}

impl Default for Machine {
    fn default() -> Machine {
        Machine::new(Config::default())
    }
}

fn mc_of(t: u64, n: u64) -> usize {
    (t % n) as usize
}

impl Machine {
    pub fn new(config: Config) -> Machine {
        Machine {
            config,
            dl: [[0; 32]; 13],
            ts13: 0,
            ts14: 0,
            ts15: 0,
            ts16: 0,
            qs: [[0; 4]; 2],
            ds: [[0; 2]; 3],
            tca: false,
            tcb: false,
            alarm: false,
            os: 0,
            id: 0,
            drum: vec![[0; 32]; 256],
            read_heads: 0,
            write_heads: 0,
            count: 0,
            count_from: None,
            dl_origin: [None; 13],
            now: 0,
            obeyed: 0,
            warnings: Vec::new(),
            trace: None,
            reader: Reader::default(),
            punch: Punch::default(),
            arith: None,
            drum_op: None,
            ds21_owed: None,
            dl10_write: None,
            console_shot: false,
            console_delay: 0,
            version: 0,
        }
    }

    // ---- set-up -------------------------------------------------------

    /// Load a store image (see `loader`). Blocks with a track go to the
    /// drum, others to their DL. If the image names a start, TS COUNT is
    /// loaded from it.
    pub fn load_image(&mut self, img: &loader::Image) {
        for b in &img.blocks {
            let w = b.values();
            match (b.block.track, b.block.dl) {
                (Some(t), _) => self.drum[t as usize] = w,
                (None, Some(d)) => self.dl[d as usize] = w,
                (None, None) => {}
            }
        }
        for &(s, v) in &img.ts {
            self.set_ts(s, v);
        }
        for &((s, k), v) in &img.ds {
            self.ds[(s - 19) as usize][k as usize] = v;
        }
        for &((s, k), v) in &img.qs {
            self.qs[(s - 17) as usize][k as usize] = v;
        }
        if let Some(id) = img.id {
            self.id = id;
        }
        if let Some((d, m)) = img.start {
            self.start(d, m);
        }
    }

    fn set_ts(&mut self, s: u8, v: u32) {
        match s {
            13 => self.ts13 = v,
            14 => self.ts14 = v,
            15 => self.ts15 = v,
            _ => self.ts16 = v,
        }
    }

    /// Put the word in DL `dl`, m.c. `mc` into TS COUNT at the next time
    /// whose m.c. number is `mc`, as if an instruction had called it.
    pub fn start(&mut self, dl: u8, mc: u8) {
        let t = self.now + (u64::from(mc) + 32 - self.now % 32) % 32;
        self.now = t;
        self.count = self.dl[dl as usize][mc as usize];
        self.count_from = Some((dl, mc));
    }

    /// The INITIAL INPUT key (PM §8.1, p. 56): clear the high-speed store
    /// (DLs, TSs, DSs, QSs and TS COUNT; not the drum) and call the reader.
    pub fn initial_input(&mut self) {
        self.dl = [[0; 32]; 13];
        self.ts13 = 0;
        self.ts14 = 0;
        self.ts15 = 0;
        self.ts16 = 0;
        self.qs = [[0; 4]; 2];
        self.ds = [[0; 2]; 3];
        self.count = 0;
        self.count_from = None;
        self.arith = None;
        self.ds21_owed = None;
        self.version += 1;
        let t = self.now;
        self.call_reader(t);
    }

    /// Put cards in the reader hopper.
    pub fn load_cards(&mut self, cards: impl IntoIterator<Item = Card>) {
        self.reader.deck.extend(cards);
        if self.reader.running && self.reader.next_shot.is_none() && self.reader.current.is_none() {
            let t = self.now + self.config.reader.first_shot;
            self.reader.next_shot = Some(t);
        }
    }

    /// Cards still in the reader hopper.
    pub fn cards_left(&self) -> usize {
        self.reader.deck.len() + usize::from(self.reader.current.is_some())
    }

    pub fn cards_read(&self) -> usize {
        self.reader.cards_read
    }

    /// Cards that have come out of the punch.
    pub fn punched(&self) -> &[Card] {
        &self.punch.output
    }

    /// Press the single-shot key: a waiting stopper is obeyed.
    pub fn single_shot(&mut self) {
        self.console_shot = true;
        self.console_delay = 0;
    }

    /// Press the single-shot key `delay` minor cycles after the machine
    /// stopped, as an operator would: whatever runs on its own (the divider,
    /// the drum) finishes during the stop.
    pub fn single_shot_after(&mut self, delay: u64) {
        self.console_shot = true;
        self.console_delay = delay;
    }

    pub fn reader_running(&self) -> bool {
        self.reader.running
    }

    pub fn punch_running(&self) -> bool {
        self.punch.running
    }

    pub fn trace_on(&mut self) {
        self.trace = Some(Vec::new());
    }

    // ---- running ------------------------------------------------------

    /// Obey instructions until something stops the machine or `limit`
    /// instructions have been obeyed.
    pub fn run(&mut self, limit: u64) -> Outcome {
        let end = self.obeyed + limit;
        while self.obeyed < end {
            if let Some(o) = self.step() {
                return o;
            }
        }
        Outcome::Limit
    }

    /// Obey the instruction in TS COUNT. Returns `Some` if it cannot be
    /// obeyed yet, or if the machine has settled into a dynamic stop.
    pub fn step(&mut self) -> Option<Outcome> {
        let ins = Instr::decode(self.count);
        let mut m = self.now;
        let mut waited = None;

        // Control-magnetics interlock (PM §6.4, p. 42): an instruction using
        // S11, D11, D30 or D31 waits for the drum, "as though it were a stopper".
        let uses_drum = ins.s == 11 || matches!(ins.d, 11 | 30 | 31);
        if uses_drum {
            if let Some((done, _)) = self.drum_op {
                if done > m {
                    waited = Some(m);
                    m = align(done, m);
                }
            }
        }

        // Stoppers (PM §4.17-4.18): obeyed on a single shot, from the reader
        // or punch if either is running, otherwise from the key.
        if !ins.go || self.config.stop_key {
            self.advance(m.saturating_sub(1));
            let shot = if self.reader.running {
                match self.reader.next_shot {
                    Some(s) => s.max(m),
                    None => return Some(Outcome::ReaderEmpty),
                }
            } else if self.punch.running {
                self.punch.next_shot.expect("running punch has a next shot").max(m)
            } else if self.console_shot {
                self.console_shot = false;
                m + self.console_delay
            } else {
                return Some(Outcome::AwaitSingleShot);
            };
            self.advance(shot);
            if shot > m {
                waited.get_or_insert(m);
            }
            // Inferred: the stopped instruction is obeyed as if it had
            // entered TS COUNT at the first time after the shot with the
            // same m.c. number, so its W and T keep their meaning (PM §8.1,
            // p. 57: "2, 0-2, 30, 31 X" entering in m.c. m sends the next
            // row to 2_m).
            m = align(shot, m);
        }

        let (w, t) = (u64::from(ins.w), u64::from(ins.t));
        let (start, end, next) = match ins.c {
            0 => {
                let tt = if w > t { t + 32 } else { t };
                (m + w + 2, m + w + 2, m + tt + 2)
            }
            2 => {
                let tt = if w >= t { t + 32 } else { t };
                (m + w + 2, m + w + 3, m + tt + 2)
            }
            c => {
                if c == 3 && w == t {
                    (m + w + 2, m + w + 34, m + w + 34)
                } else {
                    let tt = if w > t { t + 32 } else { t };
                    (m + w + 2, m + tt + 2, m + tt + 2)
                }
            }
        };

        let version = self.version;
        let mut discrim = false;
        let mut to_count = None;
        let mut value = 0;
        for tau in start..=end {
            self.advance(tau - 1);
            let v = self.transfer(&ins, tau, tau == start, tau == end, &mut discrim);
            value = v;
            if ins.d == 0 && tau == next {
                to_count = Some(v);
            }
        }
        if (1..=12).contains(&ins.s) && (1..=12).contains(&ins.d) && ins.s != ins.d && end - start >= 31 {
            self.dl_origin[ins.d as usize] = self.dl_origin[ins.s as usize];
        }

        let (word, from, n) = match to_count {
            Some(v) => (v, None, next),
            None => {
                let n = next + u64::from(discrim);
                self.advance(n - 1);
                self.apply_owed(n);
                let dl = ins.next_dl();
                (self.dl[dl][mc_of(n, 32)], Some((dl as u8, mc_of(n, 32) as u8)), n)
            }
        };

        if let Some(tr) = &mut self.trace {
            let track = self.count_from.and_then(|(d, _)| self.dl_origin[d as usize]);
            tr.push(Step {
                track,
                entered: m,
                from: self.count_from,
                word: self.count,
                transfer: (start, end),
                next: n,
                next_from: from,
                value,
                waited,
            });
        }
        let same = from.is_some() && from == self.count_from && word == self.count;
        self.count = word;
        self.count_from = from;
        self.now = n;
        self.obeyed += 1;

        if same && self.version == version && self.idle() {
            return Some(Outcome::Loop { alarm: self.alarm });
        }
        None
    }

    fn idle(&self) -> bool {
        !self.reader.running
            && !self.punch.running
            && self.punch.pending.is_none()
            && self.arith.is_none()
            && self.drum_op.is_none()
    }

    // ---- one minor cycle of transfer -----------------------------------

    fn transfer(&mut self, ins: &Instr, tau: u64, first: bool, last: bool, discrim: &mut bool) -> u32 {
        if self.config.aim && first && ins.d == 0 && (ins.s == 17 || ins.s == 18) {
            self.aim(ins, tau);
        }
        let v = self.source(ins.s, tau);
        self.apply_owed(tau);
        self.mult_until(tau);
        self.destination(ins, v, tau, first, last, discrim);
        v
    }

    /// The word at source `s` in minor cycle `tau` (PM Appendix 2, p. 77;
    /// §6.1-6.2, 6.5, 6.7).
    pub fn source(&self, s: u8, tau: u64) -> u32 {
        match s {
            0 => self.source0(tau),
            1..=12 => self.dl[s as usize][mc_of(tau, 32)],
            13 => self.ts13,
            14 => self.ts14,
            15 => self.ts15,
            16 => {
                if self.tca {
                    // DL10 delayed by one m.c.: the word that came out of
                    // DL10 in the previous m.c. (before any write then).
                    let prev = tau.wrapping_sub(1);
                    match self.dl10_write {
                        Some((t, _, old)) if t == prev => old,
                        _ => self.dl[10][mc_of(prev, 32)],
                    }
                } else {
                    self.ts16
                }
            }
            17 | 18 => self.qs[(s - 17) as usize][mc_of(tau, 4)],
            19..=21 => self.ds[(s - 19) as usize][mc_of(tau, 2)],
            22 => {
                let [lo, hi] = self.ds[2];
                if tau.is_multiple_of(2) {
                    let x = if self.tcb { lo >> 31 } else { hi & 1 };
                    (lo >> 1) | (x << 31)
                } else {
                    ((hi as i32) >> 1) as u32
                }
            }
            23 => ((self.ts14 as i32) >> 1) as u32,
            24 => self.ts14 << 1,
            25 => self.ts14 & self.ts15,
            26 => self.ts14 ^ self.ts15,
            27 => p(1),
            28 => p(17),
            29 => p(32),
            30 => 0,
            _ => u32::MAX,
        }
    }

    /// Source 0: the row under the reading brushes while the reader is
    /// running, otherwise the I.D. keys (PM §4.8, §6.3; Appendix 1 (a)).
    fn source0(&self, tau: u64) -> u32 {
        if let Some((shot, row, _)) = self.reader.last {
            if tau >= shot && tau < shot + self.config.reader.row_window {
                return row;
            }
        }
        if self.reader.running {
            0
        } else {
            self.id
        }
    }

    fn bump<T: PartialEq>(version: &mut u64, slot: &mut T, v: T) {
        if *slot != v {
            *slot = v;
            *version += 1;
        }
    }

    fn destination(&mut self, ins: &Instr, v: u32, tau: u64, first: bool, last: bool, discrim: &mut bool) {
        let d = ins.d;
        if matches!(self.arith, Some(Arith::Div { .. })) && matches!(d, 16 | 21 | 22 | 23) {
            self.warnings
                .push(format!("m.c. {tau}: destination {d} used during division"));
        }
        if let Some(Arith::Mult { carry, .. }) = &mut self.arith {
            // During multiplication DS21 holds the waning multiplier and the
            // waxing partial product (LDM p.63); the long accumulator's adder
            // is in the circulation path, so D22/D23 add into the word being
            // processed and D21 replaces it. TCB is off throughout (LDM p.65).
            if matches!(d, 21..=23) {
                let k = mc_of(tau, 2);
                let w = self.ds[2][k];
                match d {
                    21 => self.ds[2][k] = v,
                    _ => {
                        let sub = d == 23;
                        let r = if sub { w.wrapping_sub(v) } else { w.wrapping_add(v) };
                        if k == 0 {
                            // carry or borrow, and Trigger X's sign extension
                            // when the transfer ends here (LDM pp.59-60)
                            let c = if sub {
                                0u32.wrapping_sub(u32::from(v > w))
                            } else {
                                u32::from(r < w)
                            };
                            let mut owed = c;
                            if last && v >> 31 == 1 {
                                owed = if sub {
                                    owed.wrapping_add(1)
                                } else {
                                    owed.wrapping_sub(1)
                                };
                            }
                            *carry = carry.wrapping_add(owed);
                        }
                        self.ds[2][k] = r;
                    }
                }
                self.version += 1;
                return;
            }
        }
        let ver = &mut self.version;
        match d {
            0 => {}
            1..=12 => {
                let k = mc_of(tau, 32);
                if d == 10 {
                    self.dl10_write = Some((tau, k, self.dl[10][k]));
                }
                Self::bump(ver, &mut self.dl[d as usize][k], v);
            }
            13 => Self::bump(ver, &mut self.ts13, v),
            14 => Self::bump(ver, &mut self.ts14, v),
            15 => Self::bump(ver, &mut self.ts15, v),
            16 => {
                // TCA is cleared at the beginning of any transfer to TS16 (PM §6.5).
                Self::bump(ver, &mut self.tca, false);
                Self::bump(ver, &mut self.ts16, v);
            }
            17 | 18 => Self::bump(ver, &mut self.qs[(d - 17) as usize][mc_of(tau, 4)], v),
            19..=21 => Self::bump(ver, &mut self.ds[(d - 19) as usize][mc_of(tau, 2)], v),
            22 | 23 => self.ds21_arith(v, d == 23, tau, last),
            24 => self.trigger(ins.s, tau, discrim),
            25 => {
                let x = self.ts13.wrapping_add(v);
                Self::bump(ver, &mut self.ts13, x);
            }
            26 => {
                let x = self.ts13.wrapping_sub(v);
                Self::bump(ver, &mut self.ts13, x);
            }
            27 => *discrim |= v >> 31 == 1,
            28 => *discrim |= v != 0,
            29 => {
                // Inferred: D29 sets lamps of the O.S. (an OR); it is cleared
                // only by 8-24, the key, starting the punch or punching a row.
                // PM §6.8 uses "30-29" as a dummy instruction.
                let x = self.os | v;
                Self::bump(ver, &mut self.os, x);
            }
            30 if first => self.drum_rw(ins, tau),
            31 if first => self.drum_shift(ins, tau),
            _ => {}
        }
    }

    /// D22 / D23: add or subtract into DS21 (PM §6.6, pp. 44-45; BP §17).
    fn ds21_arith(&mut self, v: u32, sub: bool, tau: u64, last: bool) {
        let op = |a: u32, b: u32| if sub { a.wrapping_sub(b) } else { a.wrapping_add(b) };
        let [lo, hi] = self.ds[2];
        if tau % 2 == 1 {
            self.ds[2][1] = op(hi, v);
        } else if self.tcb {
            self.ds[2][0] = op(lo, v);
        } else {
            // Double length: the carry (or borrow) from P32 of the even word
            // goes into P1 of the odd word in the next m.c.; if the transfer
            // ends on this even m.c., 32 copies of the sign digit of v are
            // added (subtracted) there too.
            let new = op(lo, v);
            let carry = if sub { u32::from(v > lo) } else { u32::from(new < lo) };
            let mut owed = if sub { 0u32.wrapping_sub(carry) } else { carry };
            if last && v >> 31 == 1 {
                owed = op(owed, u32::MAX);
            }
            self.ds[2][0] = new;
            if owed != 0 {
                self.ds21_owed = Some((tau + 1, owed));
            }
        }
        self.version += 1;
    }

    fn apply_owed(&mut self, tau: u64) {
        if let Some((t, owed)) = self.ds21_owed {
            if t <= tau {
                self.ds[2][1] = self.ds[2][1].wrapping_add(owed);
                self.ds21_owed = None;
            }
        }
    }

    /// D24, the destination triggers (PM §4.7 and Appendix 2).
    fn trigger(&mut self, s: u8, tau: u64, discrim: &mut bool) {
        match s {
            0 | 1 => {
                if self.arith.is_none() {
                    if tau.is_multiple_of(2) {
                        self.warnings
                            .push(format!("m.c. {tau}: {s}-24 obeyed in an even minor cycle"));
                    }
                    let (a, b) = (self.ds[2][1], self.ts16);
                    if s == 0 {
                        if self.ds[2][0] != 0 {
                            self.warnings
                                .push(format!("m.c. {tau}: multiplication started with 21_2 not empty"));
                        }
                        self.tcb = false;
                        let _ = a;
                        self.arith = Some(Arith::Mult {
                            start: tau,
                            b,
                            y: false,
                            carry: 0,
                            next: tau,
                        });
                        self.mult_until(tau);
                    } else {
                        if self.ds[2][0] != 0 {
                            self.warnings
                                .push(format!("m.c. {tau}: division started with 21_2 not empty"));
                        }
                        self.tcb = true;
                        self.arith = Some(Arith::Div {
                            done: tau + self.config.div_time,
                            a,
                            b,
                        });
                    }
                    self.version += 1;
                }
            }
            2 => *discrim |= self.til(tau),
            3 => Self::bump(&mut self.version, &mut self.tca, true),
            4 => Self::bump(&mut self.version, &mut self.tcb, false),
            5 => Self::bump(&mut self.version, &mut self.tcb, true),
            6 => Self::bump(&mut self.version, &mut self.alarm, false),
            7 => Self::bump(&mut self.version, &mut self.alarm, true),
            8 => Self::bump(&mut self.version, &mut self.os, 0),
            9 => self.clear_hollerith(tau),
            10 => self.call_punch(tau),
            12 => self.call_reader(tau),
            _ => self
                .warnings
                .push(format!("m.c. {tau}: trigger {s}-24 is not on a Mark I DEUCE")),
        }
    }

    // ---- multiplier and divider ----------------------------------------

    fn finish_arith(&mut self, upto: u64) {
        let Some(op) = self.arith else { return };
        let done = match op {
            Arith::Div { done, .. } => done,
            Arith::Mult { .. } => return,
        };
        // `done` is the first m.c. in which the result can be read: a
        // multiplication started in m.c. 7 holds MULT on to the end of m.c. 7
        // two major cycles later and "21-19" may transfer in m.c. 8 (PM §6.8 (c), (e)).
        if done > upto + 1 {
            return;
        }
        self.arith = None;
        self.version += 1;
        match op {
            Arith::Mult { .. } => {}
            Arith::Div { a, b, .. } => {
                let (q, r) = divide(a, b);
                // PM p.50: |A| <= |B|, and |A| = |B| only for A negative and
                // B positive (the quotient must be single-length).
                let (sa, sb) = (a as i32, b as i32);
                let (ma, mb) = (sa.unsigned_abs(), sb.unsigned_abs());
                if b == 0 || ma > mb || (ma == mb && !(sa < 0 && sb > 0)) {
                    self.warnings
                        .push(format!("division of {} by {}: out of range", a as i32, b as i32));
                }
                self.ds[2] = [q, r];
            }
        }
    }

    /// Run the multiplier through every m.c. up to and including `upto`
    /// (Logical Design Manual, ch. 10, pp. 62-65). In the first (odd) m.c.
    /// the top digit of the multiplier in 21_3 is examined. Then, in each
    /// pair of m.c., the even word is shifted up one place and TS16 added to
    /// it if the digit examined was 1, and the odd word is shifted up one
    /// place (taking the digit that left the top of the even word and any
    /// carry) and its new top digit examined. After 65 m.c., DS21 holds the
    /// 64-digit product, and in between it holds the partial product and the
    /// remaining multiplier digits as the manual's figure on p.64 shows.
    fn mult_until(&mut self, upto: u64) {
        let Some(Arith::Mult {
            start,
            b,
            mut y,
            mut carry,
            mut next,
        }) = self.arith
        else {
            return;
        };
        while next <= upto {
            let t = next;
            let [low, high] = self.ds[2];
            if t == start {
                y = high >> 31 == 1;
            } else if t % 2 == 0 {
                let lo2 = u64::from(low) << 1;
                let sum = (lo2 & 0xFFFF_FFFF) + if y { u64::from(b) } else { 0 };
                self.ds[2][0] = sum as u32;
                carry = carry.wrapping_add(((lo2 >> 32) + (sum >> 32)) as u32);
            } else {
                let h = (high << 1).wrapping_add(carry);
                self.ds[2][1] = h;
                carry = 0;
                y = h >> 31 == 1;
            }
            next += 1;
            if t == start + self.config.mult_time - 1 {
                self.arith = None;
                self.version += 1;
                return;
            }
        }
        self.arith = Some(Arith::Mult {
            start,
            b,
            y,
            carry,
            next,
        });
    }

    // ---- reader and punch ----------------------------------------------

    fn call_reader(&mut self, tau: u64) {
        if self.reader.running {
            return;
        }
        self.reader.running = true;
        self.version += 1;
        if self.reader.current.is_some() || !self.reader.deck.is_empty() {
            self.reader.next_shot = Some(tau + self.config.reader.first_shot);
        } else {
            self.reader.next_shot = None;
        }
    }

    fn call_punch(&mut self, tau: u64) {
        if self.punch.running {
            return;
        }
        self.punch.running = true;
        self.punch.stopping = false;
        self.os = 0;
        self.version += 1;
        if self.punch.next_shot.is_none() {
            self.punch.next_shot = Some(tau + self.config.punch.first_shot);
        }
    }

    /// 9-24: clear (stop) both reader and punch (PM §4.8, §6.3).
    fn clear_hollerith(&mut self, _tau: u64) {
        if self.reader.running {
            self.reader.running = false;
            self.reader.next_shot = None;
            // The rest of the card is not read (Inferred: the next call
            // starts on the next card).
            if self.reader.current.take().is_some() {
                self.reader.cards_read += 1;
            }
            self.reader.row = 0;
            self.version += 1;
        }
        if self.punch.running {
            self.punch.running = false;
            self.punch.stopping = true;
            self.version += 1;
        }
    }

    /// Twelfth Impulse Line: from 2 MC before the single shot of row 9 to
    /// 18 MC after it (PM Appendix 1 (d); minimum figures).
    fn til(&self, tau: u64) -> bool {
        let near = |next: Option<u64>, row: usize, last: Option<(u64, usize)>| {
            let before = matches!(next, Some(s) if row == 11 && tau + 2 * MAJOR >= s && tau < s);
            let after = matches!(last, Some((s, 11)) if tau >= s && tau < s + 18 * MAJOR);
            before || after
        };
        near(
            self.reader.next_shot,
            self.reader.row,
            self.reader.last.map(|(s, _, r)| (s, r)),
        ) || near(self.punch.next_shot, self.punch.row, self.punch.last_shot)
    }

    fn advance_reader(&mut self, upto: u64) {
        while let Some(s) = self.reader.next_shot {
            if s > upto {
                break;
            }
            if self.reader.current.is_none() {
                match self.reader.deck.pop_front() {
                    Some(c) => {
                        self.reader.current = Some(c);
                        self.reader.row = 0;
                    }
                    None => {
                        self.reader.next_shot = None;
                        break;
                    }
                }
            }
            let card = self.reader.current.expect("card");
            let r = self.reader.row;
            self.reader.last = Some((s, card.rows[r], r));
            self.version += 1;
            self.reader.row += 1;
            let stop54 = card.col54 >> r & 1 == 1;
            if self.reader.row == 12 {
                self.reader.current = None;
                self.reader.cards_read += 1;
                self.reader.row = 0;
                self.reader.next_shot = if self.reader.deck.is_empty() {
                    None
                } else {
                    Some(s + self.config.reader.card_interval)
                };
            } else {
                self.reader.next_shot = Some(s + self.config.reader.row_interval);
            }
            if stop54 {
                // A hole in column 54 stops the reader after this row's
                // single shot (PM §6.3, p. 39). Inferred: the rest of the
                // card is not read.
                self.reader.running = false;
                self.reader.next_shot = None;
                if self.reader.current.take().is_some() {
                    self.reader.cards_read += 1;
                }
                self.reader.row = 0;
            }
        }
    }

    fn advance_punch(&mut self, upto: u64) {
        loop {
            let pend = self.punch.pending.filter(|&p| p <= upto);
            let shot = self.punch.next_shot.filter(|&s| s <= upto && self.punch.running);
            match (pend, shot) {
                (Some(p), s) if s.is_none_or(|s| p <= s) => {
                    self.punch.pending = None;
                    let r = self.punch.row;
                    self.punch.card[r] = self.os;
                    self.os = 0;
                    self.punch.row += 1;
                    self.version += 1;
                    if self.punch.row == 12 || self.punch.stopping {
                        let mut c = Card::blank();
                        c.rows = self.punch.card;
                        self.punch.output.push(c);
                        self.punch.card = [0; 12];
                        self.punch.row = 0;
                        if self.punch.stopping {
                            self.punch.stopping = false;
                            self.punch.next_shot = None;
                        }
                    }
                }
                (_, Some(s)) => {
                    self.punch.last_shot = Some((s, self.punch.row));
                    self.punch.pending = Some(s + self.config.punch.row_window);
                    let interval = if self.punch.row == 11 {
                        self.config.punch.card_interval
                    } else {
                        self.config.punch.row_interval
                    };
                    self.punch.next_shot = Some(s + interval);
                    self.version += 1;
                }
                _ => break,
            }
        }
        if !self.punch.running && self.punch.pending.is_none() && self.punch.stopping {
            // Cleared between rows: the rest of the card runs out blank.
            if self.punch.row > 0 {
                let mut c = Card::blank();
                c.rows = self.punch.card;
                self.punch.output.push(c);
            }
            self.punch.card = [0; 12];
            self.punch.row = 0;
            self.punch.stopping = false;
            self.punch.next_shot = None;
        }
    }

    // ---- drum ----------------------------------------------------------

    fn drum_shift(&mut self, ins: &Instr, tau: u64) {
        let pos = ins.s & 15;
        if ins.s > 15 {
            self.warnings
                .push(format!("m.c. {tau}: head position {} taken as {pos}", ins.s));
        }
        let heads = if ins.c.is_multiple_of(2) {
            &mut self.read_heads
        } else {
            &mut self.write_heads
        };
        if *heads != pos {
            *heads = pos;
            self.drum_op = Some((tau + self.config.drum_shift, DrumOp::Shift));
            self.version += 1;
        }
    }

    fn drum_rw(&mut self, ins: &Instr, tau: u64) {
        let head = ins.s & 15;
        if ins.s > 15 {
            self.warnings
                .push(format!("m.c. {tau}: head {} taken as {head}", ins.s));
        }
        let op = if ins.c.is_multiple_of(2) {
            DrumOp::Read(self.read_heads * 16 + head)
        } else {
            DrumOp::Write(self.write_heads * 16 + head)
        };
        self.drum_op = Some((tau + self.config.drum_transfer, op));
        self.version += 1;
    }

    fn finish_drum(&mut self, upto: u64) {
        if let Some((done, op)) = self.drum_op {
            if done <= upto {
                match op {
                    DrumOp::Read(t) => {
                        self.dl[11] = self.drum[t as usize];
                        self.dl_origin[11] = Some(t);
                    }
                    DrumOp::Write(t) => self.drum[t as usize] = self.dl[11],
                    DrumOp::Shift => {}
                }
                self.drum_op = None;
                self.version += 1;
            }
        }
    }

    /// The Automatic Instruction Modifier (PM appendix pp. 82-83), applied
    /// to the QS word at the first m.c. of a 17-0 or 18-0 transfer.
    fn aim(&mut self, ins: &Instr, tau: u64) {
        let word = ins.encode();
        let n = word & 3;
        let mut add = [p(5), p(10), p(17), p(18)][n as usize];
        let mut subtract = false;
        if ins.c == 1 {
            if word & p(3) != 0 {
                add = add.wrapping_add(p(22));
            }
            subtract = word & p(4) != 0;
        }
        if ins.s == 18 {
            subtract = !subtract;
        }
        let slot = &mut self.qs[(ins.s - 17) as usize][mc_of(tau, 4)];
        *slot = if subtract {
            slot.wrapping_sub(add)
        } else {
            slot.wrapping_add(add)
        };
        self.version += 1;
    }

    /// Bring everything that happens on its own (cards, multiplier, drum)
    /// up to the end of minor cycle `upto`.
    fn advance(&mut self, upto: u64) {
        self.advance_reader(upto);
        self.advance_punch(upto);
        self.finish_arith(upto);
        self.mult_until(upto);
        self.finish_drum(upto);
        if let Some((t, _)) = self.ds21_owed {
            if t <= upto {
                self.apply_owed(t);
            }
        }
    }
}

/// The first time at or after `t` with the same minor-cycle number as `m`.
fn align(t: u64, m: u64) -> u64 {
    t + (m % 32 + 32 - t % 32) % 32
}

/// The divider (PM §6.10, p. 50): A = 21_3 and B = TS16 as signed integers;
/// 2^31 A = QB + R with 0 <= R < B for B > 0 and B <= R < 0 for B < 0; the
/// quotient Q goes to 21_2 and the "modified remainder" r = 2(2R - B) to
/// 21_3. Computed from the definition, not digit by digit.
pub fn divide(a: u32, b: u32) -> (u32, u32) {
    let a = i128::from(a as i32);
    let b = i128::from(b as i32);
    if b == 0 {
        return (0, 0);
    }
    let x = a << 31;
    let mut q = x.div_euclid(b);
    let mut r = x - q * b;
    if b < 0 {
        // div_euclid gives 0 <= r < |b|; we need b <= r < 0
        if r >= 0 {
            q -= 1;
            r += b;
        }
    }
    let rr = 2 * (2 * r - b);
    (q as u32, rr as u32)
}
