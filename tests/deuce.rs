//! Tests of the DEUCE simulator against the examples of the DEUCE
//! Programming Manual NS-y-16/5-56 ("PM") and Basic DEUCE Programming NS y 88
//! ("BP"), and unit tests of each source and destination.

use george::deuce::cards::{self, Card};
use george::deuce::loader::parse_image;
use george::deuce::word::{p, Instr};
use george::deuce::{divide, Config, Machine, Outcome};

fn machine(src: &str) -> Machine {
    let img = parse_image(src).unwrap_or_else(|e| panic!("{e}"));
    let mut m = Machine::default();
    m.trace_on();
    m.load_image(&img);
    m
}

/// A line of the image for an instruction coded by where it transfers and
/// where it goes next (both as m.c. numbers).
#[allow(clippy::too_many_arguments)]
fn at(mc: u8, nis: u8, s: u8, d: u8, c: u8, first: u8, next: u8, go: bool) -> String {
    let i = Instr::coded(mc, nis, s, d, c, first, next, go);
    format!(
        "{mc} {} {} {} {} {} {} {}\n",
        i.nis,
        i.s,
        i.d,
        i.c,
        i.w,
        i.t,
        if go { 1 } else { 0 }
    )
}

/// Simple instruction in DL1: transfer in the next-but-one m.c., then go to `next`.
fn i1(mc: u8, s: u8, d: u8, next: u8) -> String {
    at(mc, 1, s, d, 0, (mc + 2) % 32, next, true)
}

fn last_step(m: &Machine) -> george::deuce::Step {
    m.trace.as_ref().unwrap().last().unwrap().clone()
}

// ---- the instruction word and timing (PM §5) -----------------------------

#[test]
fn instruction_layout_bp_sheet_19() {
    // BP sheet 19 lays the fields out as P2-4 NIS, P5-9 S, P10-14 D,
    // P15-16 C, P17-21 W, P26-30 T, P32 go.
    let i = Instr::new(5, 13, 3, 0, 6, 10, true);
    let w = i.encode();
    assert_eq!(w, 5 * p(2) + 13 * p(5) + 3 * p(10) + 6 * p(17) + 10 * p(26) + p(32));
    assert_eq!(Instr::decode(w), i);
    assert_eq!(i.to_string(), "5, 13-3, 0, 6, 10");
}

#[test]
fn single_transfer_pm_p27() {
    // "5, 13-3, 6, 10" stored in 1_17: transfer TS13 to 3_25, next from 5_29.
    let mut m = machine("dl 1\n17 5 13 3 0 6 10 1\nts 13 = 1234\nstart 1 17\n");
    m.step();
    let s = last_step(&m);
    assert_eq!((s.transfer.0 % 32, s.transfer.1 % 32), (25, 25));
    assert_eq!(s.next - s.entered, 12);
    assert_eq!(s.next_from, Some((5, 29)));
    assert_eq!(m.dl[3][25], 1234);
}

#[test]
fn wait_greater_than_timing_pm_p28() {
    // "0, 4-16, 10, 1" stored in m.c. 27: transfers 4_7 to TS16 in m.c. 39
    // and calls its successor from 8_30 in m.c. 62.
    let mut m = machine("dl 2\n27 0 4 16 0 10 1 1\ndl 4\n7 = 99\nstart 2 27\n");
    let t0 = m.now;
    m.step();
    let s = last_step(&m);
    assert_eq!(s.transfer.0 - t0, 39 - 27);
    assert_eq!(s.next - t0, 62 - 27);
    assert_eq!(s.next_from, Some((8, 30)));
    assert_eq!(m.ts16, 99);
}

#[test]
fn double_transfer_pm_p28() {
    // "4, 20-17, 2, 1, 5" in 1_13: DS20 to QS17 in m.c. 16 and 17, next from 4_20.
    let mut m = machine("dl 1\n13 4 20 17 2 1 5 1\nds 20 2 = 11\nds 20 3 = 22\nstart 1 13\n");
    m.step();
    let s = last_step(&m);
    assert_eq!((s.transfer.0 % 32, s.transfer.1 % 32), (16, 17));
    assert_eq!(s.next_from, Some((4, 20)));
    assert_eq!(m.qs[0][0], 11); // m.c. 16 = 17_0
    assert_eq!(m.qs[0][1], 22);
    // "3, 21-11, 2, 0, 0" in 3_4: DS21 to DL11 in m.c. 6 and 7, next from 3_6
    // in the next major cycle (W = T with C = 2 adds 32 to T).
    let mut m = machine("dl 3\n4 3 21 11 2 0 0 1\nds 21 2 = 5\nds 21 3 = 6\nstart 3 4\n");
    let t0 = m.now;
    m.step();
    let s = last_step(&m);
    assert_eq!(s.next - t0, 34);
    assert_eq!(s.next_from, Some((3, 6)));
    assert_eq!((m.dl[11][6], m.dl[11][7]), (5, 6));
}

#[test]
fn long_transfer_pm_p28() {
    // "7, 15-25, 1, 29, 1" in 6_5 adds TS15 into TS13 five times, m.c. 36-40,
    // and takes the next instruction from 7_8.
    let mut m = machine("dl 6\n5 7 15 25 1 29 1 1\nts 15 = 3\nstart 6 5\n");
    let t0 = m.now;
    m.step();
    let s = last_step(&m);
    assert_eq!((s.transfer.0 - t0, s.transfer.1 - t0), (31, 35));
    assert_eq!(s.next_from, Some((7, 8)));
    assert_eq!(m.ts13, 15);
}

#[test]
fn characteristic_three_pm_p29() {
    // C = 3 with W = T: a 33 m.c. transfer, next at m + W + 34.
    let mut m = machine("dl 1\n0 1 27 25 3 4 4 1\nstart 1 0\n");
    m.step();
    let s = last_step(&m);
    assert_eq!(s.transfer.1 - s.transfer.0 + 1, 33);
    assert_eq!(s.next - s.entered, 38);
    assert_eq!(m.ts13, 33);
}

#[test]
fn time_of_instruction_pm_p29() {
    // T + 2 if T > W; T + 2 if T = W and C = 0 or 1; otherwise T + 34.
    for (c, w, t, time) in [
        (0, 3, 9, 11),
        (1, 3, 3, 5),
        (0, 3, 3, 5),
        (2, 3, 3, 37),
        (0, 9, 3, 37),
        (1, 9, 3, 37),
        (2, 2, 3, 5),
    ] {
        let mut m = machine(&format!("dl 1\n0 1 30 13 {c} {w} {t} 1\nstart 1 0\n"));
        m.step();
        let s = last_step(&m);
        assert_eq!(s.next - s.entered, time, "C={c} W={w} T={t}");
    }
}

#[test]
fn destination_zero_pm_p31() {
    // "5, 15-0, 1, 4" in m.c. 7 has no effect: the next comes from 5_13.
    let mut m = machine("dl 1\n7 5 15 0 0 1 4 1\nts 15 = 77\nstart 1 7\n");
    m.step();
    assert_eq!(last_step(&m).next_from, Some((5, 13)));
    // "x, 15-0, 1, 0, 4" in m.c. 7 takes its next instruction from TS15 in m.c. 13.
    let mut m = machine("dl 1\n7 5 15 0 1 0 4 1\nts 15 = 77\nstart 1 7\n");
    m.step();
    let s = last_step(&m);
    assert_eq!(s.next_from, None);
    assert_eq!(s.next % 32, 13);
    assert_eq!(m.count, 77);
    // "x, 15-0, 3, 3" in m.c. 7 takes its successor from TS15 in m.c. 12.
    let mut m = machine("dl 1\n7 5 15 0 0 3 3 1\nts 15 = 78\nstart 1 7\n");
    m.step();
    assert_eq!(last_step(&m).next % 32, 12);
    assert_eq!(m.count, 78);
}

#[test]
fn discrimination_pm_p33() {
    // "1, 13-27, 0, 2" in 1_13: positive -> 1_17, negative -> 1_18.
    for (v, next) in [(5u32, 17u8), (0, 17), (u32::MAX, 18), (p(32), 18)] {
        let mut m = machine(&format!("dl 1\n13 1 13 27 0 0 2 1\nts 13 = {}\nstart 1 13\n", v as i64));
        m.step();
        assert_eq!(last_step(&m).next_from, Some((1, next)), "13-27 of {v:#x}");
    }
    // "1, 0-28, 0, 30" in 1_0: zero -> 1_0, non-zero -> 1_1 (reads the I.D.).
    for (id, next) in [(0u32, 0u8), (p(7), 1)] {
        let mut m = machine(&format!("dl 1\n0 1 0 28 0 0 30 1\nid = {id}\nstart 1 0\n"));
        m.step();
        assert_eq!(last_step(&m).next_from, Some((1, next)));
    }
    // Long transfer: next is m+T+3 if any word is negative.
    let mut m = machine("dl 1\n0 1 2 27 1 0 4 1\ndl 2\n4 = -1\nstart 1 0\n");
    m.step();
    assert_eq!(last_step(&m).next_from, Some((1, 7)));
}

// ---- sources (PM §6.1, 6.2, 6.5, 6.7; Appendix 2) --------------------------

#[test]
fn constant_sources() {
    let m = Machine::default();
    assert_eq!(m.source(27, 0), 1);
    assert_eq!(m.source(28, 0), p(17));
    assert_eq!(m.source(29, 0), p(32));
    assert_eq!(m.source(30, 0), 0);
    assert_eq!(m.source(31, 0), u32::MAX);
}

#[test]
fn logical_sources_pm_p35() {
    // The worked example on p. 35, written least significant digit first:
    // only the first eight and last four digits are shown there.
    let from_back = |s: &str| {
        s.chars()
            .enumerate()
            .fold(0u32, |w, (i, c)| w | (u32::from(c == '1') << i))
    };
    let mut m = Machine::default();
    m.ts14 = from_back("11000111") | (0b1100 << 28); // ... 0011 (P29-P32 = 0,0,1,1)
    m.ts15 = from_back("10101010") | (0b0111 << 28); // ... 1110
    assert_eq!(m.source(23, 0) & 0x7f, from_back("1000111"));
    assert_eq!(m.source(23, 0) >> 28, 0b1110); // P32 repeated: ...00111 -> "0111" read backward
    assert_eq!(m.source(24, 0) & 0x1ff, from_back("011000111"));
    assert_eq!(m.source(25, 0) & 0xff, from_back("10000010"));
    assert_eq!(m.source(25, 0) >> 28, 0b0100);
    assert_eq!(m.source(26, 0) & 0xff, from_back("01101101"));
    assert_eq!(m.source(26, 0) >> 28, 0b1011);
    // signed halving and doubling (PM §2.3 examples)
    m.ts14 = -6i32 as u32;
    assert_eq!(m.source(23, 0) as i32, -3);
    assert_eq!(m.source(24, 0) as i32, -12);
    m.ts14 = -3i32 as u32;
    assert_eq!(m.source(23, 0) as i32, -2);
}

#[test]
fn store_sources_by_minor_cycle() {
    let mut m = Machine::default();
    for k in 0..32 {
        m.dl[7][k] = k as u32 + 700;
    }
    m.qs[1] = [10, 11, 12, 13];
    m.ds[0] = [190, 191];
    m.ts13 = 13;
    m.ts14 = 14;
    m.ts15 = 15;
    m.ts16 = 16;
    assert_eq!(m.source(7, 32 * 5 + 9), 709);
    assert_eq!(m.source(18, 6), 12);
    assert_eq!(m.source(19, 7), 191);
    assert_eq!(
        (m.source(13, 0), m.source(14, 0), m.source(15, 0), m.source(16, 0)),
        (13, 14, 15, 16)
    );
}

#[test]
fn source_22_pm_p45() {
    let mut m = Machine::default();
    // TCB off: one double-length number halved
    m.ds[2] = [0x0000_0003, 0x8000_0001];
    assert_eq!(m.source(22, 0), 0x8000_0001); // A2..A32, B1
    assert_eq!(m.source(22, 1), 0xc000_0000); // B2..B32, B32
    m.tcb = true;
    m.ds[2] = [0x8000_0002, 0x0000_0001];
    assert_eq!(m.source(22, 0), 0xc000_0001); // A2..A32, A32
}

#[test]
fn source_zero_is_the_id_when_reader_idle() {
    let mut m = machine("dl 1\n0 1 0 15 0 0 0 1\nid = P3 + P20\nstart 1 0\n");
    m.step();
    assert_eq!(m.ts15, p(3) | p(20));
}

// ---- destinations ----------------------------------------------------------

#[test]
fn ts13_add_and_subtract_pm_p13() {
    // 15-13, 16-25, 13-15 replaces TS15 with TS15 + TS16.
    let src = format!(
        "dl 1\n{}{}{}ts 15 = 40\nts 16 = 2\nstart 1 0\n",
        i1(0, 15, 13, 2),
        i1(2, 16, 25, 4),
        i1(4, 13, 15, 6)
    );
    let mut m = machine(&src);
    for _ in 0..3 {
        m.step();
    }
    assert_eq!((m.ts15, m.ts13, m.ts16), (42, 42, 2));
    // "14-26" subtracts TS14 from TS13; "13-25 (5 m.c.)" multiplies TS13 by 32.
    let mut m = machine("dl 1\n0 1 14 26 0 0 0 1\n2 1 13 25 1 0 4 1\nts 13 = 10\nts 14 = 7\nstart 1 0\n");
    m.step();
    assert_eq!(m.ts13, 3);
    m.step();
    assert_eq!(m.ts13, 96);
}

#[test]
fn ds21_add_subtract_tcb_pm_p44() {
    // TCB off, "20-22 (2 m.c. e,o)" adds a double-length number with carry.
    let mut m = machine(
        "dl 1\n0 1 20 22 2 0 4 1\n6 1 30 29 0 0 0 1\nds 20 2 = -1\nds 20 3 = 0\nds 21 2 = 1\nds 21 3 = 5\nstart 1 0\n",
    );
    m.step();
    m.step();
    assert_eq!(m.ds[2], [0, 6]);
    // TCB on: two independent accumulators, no carry.
    let mut m = machine("dl 1\n0 1 20 22 2 0 4 1\nds 20 2 = -1\nds 20 3 = 0\nds 21 2 = 1\nds 21 3 = 5\nstart 1 0\n");
    m.tcb = true;
    m.step();
    m.step();
    assert_eq!(m.ds[2], [0, 5]);
    // TCB off, a negative single-length number into the even m.c. is
    // extended: "14-22_2" with TS14 = -3*2^30 leaves 21_3 all ones (BP §17).
    let mut m = machine("dl 1\n0 1 14 22 0 0 0 1\n2 1 30 29 0 0 0 1\nts 14 = P31 + P32\nstart 1 0\n");
    m.step();
    m.step();
    assert_eq!(m.ds[2], [p(31) | p(32), u32::MAX]);
    // "29-23_2" with TCB off adds P32 into 21_2 (PM §6.9 note (l)).
    let mut m = machine("dl 1\n0 1 29 23 0 0 0 1\n2 1 30 29 0 0 0 1\nds 21 2 = 0\nds 21 3 = 7\nstart 1 0\n");
    m.step();
    m.step();
    assert_eq!(m.ds[2], [p(32), 7]);
    // "15-23_3": subtract into the odd m.c. alone.
    let mut m = machine("dl 1\n0 1 15 23 0 1 1 1\nts 15 = 2\nds 21 3 = 7\nstart 1 0\n");
    m.step();
    assert_eq!(m.ds[2], [0, 5]);
}

#[test]
fn s22_to_d21_halves_double_length_pm_p46() {
    // TCB off: "22-21 (2 m.c. e,o)" halves; "22-22 (2 m.c. e,o)" multiplies by 1.5.
    let mut m = machine("dl 1\n0 1 22 21 2 0 4 1\nds 21 2 = 1\nds 21 3 = 3\nstart 1 0\n");
    m.step();
    assert_eq!(m.ds[2], [p(32), 1]);
    let mut m = machine("dl 1\n0 1 22 22 2 0 4 1\nds 21 2 = 0\nds 21 3 = 4\nstart 1 0\n");
    m.step();
    m.step();
    assert_eq!(m.ds[2], [0, 6]);
}

#[test]
fn triggers_tca_tcb_alarm_os() {
    // 3-24 TCA on, 5-24 TCB on, 7-24 alarm on, 29 lights the O.S. (OR), 8-24 clears it.
    let src = format!(
        "dl 1\n{}{}{}{}{}ts 15 = P4\nts 14 = P2\n",
        i1(0, 3, 24, 2),
        i1(2, 5, 24, 4),
        i1(4, 7, 24, 6),
        i1(6, 15, 29, 8),
        i1(8, 14, 29, 10)
    ) + "start 1 0\n";
    let mut m = machine(&src);
    for _ in 0..5 {
        m.step();
    }
    assert!(m.tca && m.tcb && m.alarm);
    assert_eq!(m.os, p(2) | p(4));
    let src = format!(
        "dl 1\n{}{}{}start 1 0\n",
        i1(0, 4, 24, 2),
        i1(2, 6, 24, 4),
        i1(4, 8, 24, 6)
    );
    let mut m = machine(&src);
    m.tcb = true;
    m.alarm = true;
    m.os = 5;
    for _ in 0..3 {
        m.step();
    }
    assert!(!m.tcb && !m.alarm);
    assert_eq!(m.os, 0);
}

#[test]
fn tca_shifts_dl10_pm_p43() {
    // With TCA on, "16-10 (32 m.c.)" moves each word of DL10 on one m.c.
    let mut src = String::from("dl 1\n0 1 3 24 0 0 0 1\n2 1 16 10 1 0 31 1\ndl 10\n");
    for k in 0..32 {
        src += &format!("{k} = {}\n", 100 + k);
    }
    src += "start 1 0\n";
    let mut m = machine(&src);
    m.step();
    m.step();
    for k in 0..32 {
        assert_eq!(m.dl[10][k], 100 + ((k + 31) % 32) as u32, "10_{k}");
    }
    // a transfer to TS16 clears TCA
    let mut m = machine("dl 1\n0 1 3 24 0 0 0 1\n2 1 27 16 0 0 0 1\nstart 1 0\n");
    m.step();
    assert!(m.tca);
    m.step();
    assert!(!m.tca);
    assert_eq!(m.ts16, 1);
}

// ---- multiplier and divider (PM §6.8-6.10) ---------------------------------

#[test]
fn multiplication_pm_p46() {
    // The coding of PM p. 46: (a) 30-21_2, (b) 14-21_3, (c) 0-24 in m.c. 7,
    // (d) dummy 30-29, (e) 21-19 (2 m.c.) in m.c. 8 and 9.
    let src = "dl 1\n0 1 30 21 0 0 0 1\n2 1 14 21 0 1 1 1\n5 1 0 24 0 0 29 1\n4 1 30 29 0 1 0 1\n6 1 21 19 2 0 1 1\n\
               ts 14 = 123456789\nts 16 = 987654321\nds 21 2 = 55\nstart 1 0\n";
    let mut m = machine(src);
    for _ in 0..5 {
        m.step();
    }
    let prod = 123456789u64 * 987654321;
    assert_eq!(m.ds[0], [prod as u32, (prod >> 32) as u32]);
    assert!(m.warnings.is_empty(), "{:?}", m.warnings);
    // One m.c. earlier is too soon: the product is not there yet.
    let src = "dl 1\n0 1 30 21 0 0 0 1\n2 1 14 21 0 1 1 1\n5 1 0 24 0 0 29 1\n4 1 30 29 0 0 0 1\n6 1 21 19 2 0 1 1\n\
               ts 14 = 3\nts 16 = 5\nstart 1 0\n";
    let mut m = machine(src);
    for _ in 0..5 {
        m.step();
    }
    assert_ne!(m.ds[0], [15, 0]);
}

#[test]
fn unsigned_product_and_sign_correction_bp_sheet_34() {
    // The multiplier gives the unsigned product; "if either factor is
    // negative, the other is subtracted from the top half" (PM §6.9, BP
    // sheet 34). TS16 = x is negative here, so y (TS14) is sent to D23 odd.
    let (x, y) = (-7i32, 9i32);
    let src = format!(
        "dl 1\n0 1 30 21 0 0 0 1\n2 1 14 21 0 1 1 1\n5 1 0 24 0 0 29 1\n4 1 30 29 0 1 0 1\n6 1 14 23 0 1 1 1\n9 1 30 29 0 0 0 1\n\
         ts 14 = {y}\nts 16 = {x}\nstart 1 0\n"
    );
    let mut m = machine(&src);
    for _ in 0..6 {
        m.step();
    }
    let signed = ((u64::from(m.ds[2][1]) << 32) | u64::from(m.ds[2][0])) as i64;
    assert_eq!(signed, i64::from(x) * i64::from(y));
}

#[test]
fn divider_definition_pm_p50() {
    // 2^31 A = QB + R, 0 <= R < B (B > 0), B <= R < 0 (B < 0); r = 2(2R - B).
    for (a, b) in [
        (1i32, 3i32),
        (-1, 3),
        (1, -3),
        (-1, -3),
        (5, 7),
        (-6, 7),
        (6, -7),
        (3, 6),
        (3, -6),
        (-7, 7),
    ] {
        let (q, r) = divide(a as u32, b as u32);
        let q = i128::from(q as i32);
        let big = i128::from(a) << 31;
        let rem = big - q * i128::from(b);
        if b > 0 {
            assert!((0..i128::from(b)).contains(&rem), "{a}/{b}");
        } else {
            assert!(rem >= i128::from(b) && rem < 0, "{a}/{b}");
        }
        assert_eq!(i128::from(r as i32), 2 * (2 * rem - i128::from(b)), "{a}/{b}");
    }
    // exact quotient reached only for positive divisor
    assert_eq!(divide(3, 6).0, 1 << 30);
    assert_eq!(divide(3, -6i32 as u32).0 as i32, -(1 << 30) - 1);
}

#[test]
fn division_pm_p51() {
    // Divide TS14 by TS16, quotient to 30 b.p. rounded into TS16:
    // 14-21_3, 30-21_2, 1-24 (odd m.c.), dummy, 27-22_2, 22_2-16.
    let src = format!(
        "dl 1\n{}{}{}{}{}{}{}ts 14 = P29\nts 16 = P31\nstart 1 0\n",
        at(0, 1, 14, 21, 0, 3, 4, true),
        at(4, 1, 30, 21, 0, 6, 7, true),
        at(7, 1, 1, 24, 0, 9, 10, true),    // division starts in m.c. 9, done in 9 + 66
        at(10, 1, 30, 29, 0, 13, 12, true), // dummy, 34 m.c.
        at(12, 1, 27, 22, 0, 12, 14, true), // in m.c. 76 (12 of the third major cycle)
        at(14, 1, 22, 16, 0, 16, 16, true),
        at(16, 1, 7, 24, 0, 18, 16, true),
    );
    let mut m = machine(&src);
    assert_eq!(m.run(100), Outcome::Loop { alarm: true });
    assert!(m.warnings.is_empty(), "{:?}", m.warnings);
    assert!(m.tcb, "TCB is set by division");
    // 2^28 / 2^30 = 1/4: 2^29 to 31 b.p., rounded and halved to 2^28 (30 b.p.)
    assert_eq!(m.ts16, 1 << 28);
}

// ---- programs from the manuals ---------------------------------------------

/// The successive-squares programme of PM pp. 20-21 (second version): twenty
/// squares punched, then the punch is cleared and the alarm sounded.
fn squares_dl1() -> String {
    let mut s = String::from("dl 1\n");
    s += &at(0, 1, 1, 21, 0, 2, 3, true); //  '20' (1_2) - 21_2
    s += "2 = 20\n";
    s += &i1(3, 27, 15, 5);
    s += &i1(5, 15, 25, 7);
    s += &i1(7, 13, 15, 9);
    s += &i1(9, 16, 25, 11);
    s += &i1(11, 13, 16, 13);
    s += &i1(13, 10, 24, 15);
    s += &at(15, 1, 16, 29, 0, 17, 17, false); // 16-29 X
    s += &at(17, 1, 27, 23, 0, 20, 22, true); // 27-23_2 (even m.c. 20)
    s += &at(22, 1, 21, 28, 0, 24, 26, true); // 21_2-28: zero 26, non-zero 27
    s += &i1(26, 9, 24, 28);
    s += &at(28, 1, 7, 24, 0, 30, 28, true); // 7-24, loop on itself
    s += &at(27, 1, 1, 13, 0, 29, 5, true); //  '2' (1_29) - 13
    s += "29 = 2\n";
    s
}

#[test]
fn successive_squares_pm_p21() {
    let mut m = machine(&(squares_dl1() + "start 1 0\n"));
    let out = m.run(100_000);
    assert_eq!(out, Outcome::Loop { alarm: true });
    let rows: Vec<u32> = m.punched().iter().flat_map(|c| c.rows).collect();
    let mut want: Vec<u32> = (1..=20).map(|n| n * n).collect();
    want.resize(24, 0);
    assert_eq!(rows, want);
    assert_eq!(m.punched().len(), 2);
}

#[test]
fn programmed_division_pm_p35() {
    let src = format!(
        "dl 1\n{}{}{}{}{}{}{}{}{}{}{}{}{}ts 13 = {}\nts 16 = {}\nstart 1 0\n",
        i1(0, 27, 15, 2),   // (a)
        i1(2, 27, 14, 4),   // (b)
        i1(4, 24, 14, 6),   // (c)
        i1(6, 14, 27, 8),   // (d) + 8, - 9
        i1(8, 16, 26, 10),  // (e)
        i1(10, 13, 27, 12), // (f) + 12, - 13
        i1(12, 26, 14, 14), // (g)
        i1(13, 16, 25, 14), // (h)
        i1(14, 13, 25, 4),  // (i)
        i1(9, 26, 14, 11),  // (j)
        i1(11, 29, 15, 15), // (k)
        i1(15, 26, 14, 17), // (l)
        at(17, 1, 7, 24, 0, 19, 17, true),
        3 << 20,
        1 << 22
    );
    let mut m = machine(&src);
    assert_eq!(m.run(10_000), Outcome::Loop { alarm: true });
    // 0.75 with 30 binary places, the P1 digit forced to 1
    assert_eq!(m.ts14, (3 << 28) | 1);
}

#[test]
fn programmed_square_root_pm_p37() {
    for x in [9u32 << 20, 2, 1000, (1 << 30) - 1, 12345678] {
        let src = format!(
            "dl 1\n{}{}{}{}{}{}{}{}{}{}{}{}{}2 = P29\nts 13 = {x}\nstart 1 0\n",
            i1(0, 30, 15, 3),               // (a)
            at(3, 1, 1, 14, 0, 2, 5, true), // (b) P29 (1_2) - 14
            i1(5, 26, 26, 7),               // (c)
            i1(7, 13, 27, 9),               // (d) + 9, - 10
            i1(9, 24, 14, 11),              // (f)
            i1(10, 26, 25, 15),             // (e)
            i1(11, 26, 15, 13),             // (g)
            i1(13, 23, 14, 15),             // (h)
            i1(15, 13, 25, 17),             // (i)
            i1(17, 23, 28, 19),             // (j) zero 19, non-zero 20
            i1(20, 23, 14, 5),              // (k)
            i1(19, 26, 14, 21),             // (l)
            at(21, 1, 7, 24, 0, 23, 21, true),
        );
        let mut m = machine(&src);
        assert_eq!(m.run(100_000), Outcome::Loop { alarm: true }, "x = {x}");
        // "the answer (sqrt(2^30 x) to the nearest odd integer)"
        let exact = ((x as f64) * 2f64.powi(30)).sqrt();
        let got = m.ts14 as f64;
        assert_eq!(m.ts14 % 2, 1, "x = {x}");
        assert!((got - exact).abs() <= 1.0, "x = {x}: {got} against {exact}");
    }
}

#[test]
fn punch_dl10_by_destination_zero_pm_p31() {
    // 1_0 1, 1-13, 0, 1; 1_1 1, 28-25, 0, 2; 1_2 (1, 10-29, 23, 24 X);
    // 1_3 1, 10-24, 0, 0; 1_5 (0) 13-0, 1, 0, 0.
    let mut src = String::from(
        "dl 1\n0 1 1 13 0 0 1 1\n1 1 28 25 0 0 2 1\n2 1 10 29 0 23 24 x\n3 1 10 24 0 0 0 1\n5 0 13 0 1 0 0 1\ndl 10\n",
    );
    for k in 0..32 {
        src += &format!("{k} = {}\n", 1000 + k);
    }
    src += "start 1 0\n";
    let mut m = machine(&src);
    m.run(2_000);
    let rows: Vec<u32> = m.punched().iter().flat_map(|c| c.rows).collect();
    assert!(rows.len() >= 24);
    for (k, r) in rows.iter().enumerate().take(24) {
        assert_eq!(*r, 1000 + k as u32, "row {k}");
    }
}

// ---- cards and initial input (PM §6.3, §8) ---------------------------------

#[test]
fn card_columns_bp_sheet_20() {
    let mut c = Card::blank();
    c.rows[0] = p(1) | p(32);
    c.col54 = 1;
    let cols = c.row_columns(0);
    assert_eq!(cols.len(), 80);
    let holes: Vec<usize> = cols
        .char_indices()
        .filter(|&(_, ch)| ch == '1')
        .map(|(i, _)| i + 1)
        .collect();
    assert_eq!(holes, vec![21, 52, 54]);
}

#[test]
fn deck_text_round_trip() {
    let deck = cards::parse_deck(
        "card\nY: .\nX: 2 0 2 1 26 25 X\n0: = 5P1 + 12P5\n9: |10000000000000000000000000000001| /54\n",
    )
    .unwrap();
    assert_eq!(deck.len(), 1);
    assert_eq!(deck[0].rows[1], Instr::new(2, 0, 2, 1, 26, 25, false).encode());
    assert_eq!(deck[0].rows[2], 5 + 12 * 16);
    assert_eq!(deck[0].rows[11], p(1) | p(32));
    assert_eq!(deck[0].col54, 1 << 11);
    let again = cards::parse_deck(&cards::format_deck(&deck)).unwrap();
    assert_eq!(again, deck);
}

#[test]
fn reading_a_card_bp_sheet_29() {
    // 12-24; 0-15 X (top row); 9-24.
    let src = format!(
        "dl 1\n{}{}{}{}start 1 0\n",
        i1(0, 12, 24, 2),
        at(2, 1, 0, 15, 0, 4, 4, false),
        i1(4, 9, 24, 6),
        at(6, 1, 7, 24, 0, 8, 6, true)
    );
    let mut m = machine(&src);
    let mut c = Card::blank();
    c.rows[0] = 4321;
    c.rows[1] = 99;
    m.load_cards([c, Card::blank()]);
    assert_eq!(m.run(100), Outcome::Loop { alarm: true });
    assert_eq!(m.ts15, 4321);
    assert_eq!(m.cards_read(), 1);
    assert_eq!(m.cards_left(), 1);
}

#[test]
fn reading_all_rows_and_column_54() {
    // Read rows into successive m.c. of DL9 by "0-9 X" with W advanced by
    // P17 each time, as PM p. 31 does for punching; the reader stops on a
    // hole in column 54 of row 3 and the next stopper waits for a single shot.
    let src = "dl 1\n0 1 1 13 0 0 1 1\n1 1 28 25 0 0 2 1\n2 1 0 9 0 23 24 x\n3 1 12 24 0 0 0 1\n5 0 13 0 1 0 0 1\nstart 1 0\n";
    let mut m = machine(src);
    let mut c = Card::blank();
    for r in 0..12 {
        c.rows[r] = 50 + r as u32;
    }
    c.col54 = 1 << 5;
    m.load_cards([c]);
    assert_eq!(m.run(1000), Outcome::AwaitSingleShot);
    assert_eq!(&m.dl[9][0..6], &[50, 51, 52, 53, 54, 55]);
    assert!(!m.reader_running());
}

#[test]
fn reader_out_of_cards() {
    let src = format!(
        "dl 1\n{}{}start 1 0\n",
        i1(0, 12, 24, 2),
        at(2, 1, 0, 15, 0, 4, 2, false)
    );
    let mut m = machine(&src);
    m.load_cards([Card::blank()]);
    assert_eq!(m.run(1000), Outcome::ReaderEmpty);
}

#[test]
fn stopper_waits_for_the_key() {
    let src = format!(
        "dl 1\n{}{}start 1 0\n",
        at(0, 1, 27, 25, 0, 2, 2, false),
        at(2, 1, 7, 24, 0, 4, 2, true)
    );
    let mut m = machine(&src);
    assert_eq!(m.run(10), Outcome::AwaitSingleShot);
    assert_eq!(m.ts13, 0);
    m.single_shot();
    assert_eq!(m.run(10), Outcome::Loop { alarm: true });
    assert_eq!(m.ts13, 1);
}

#[test]
fn til_discrimination_on_the_last_row() {
    // Read rows with "0-15 X" then "2-24": TIL is present only on row 9,
    // where the program goes to 1_7 and stops the reader.
    let src = format!(
        "dl 1\n{}{}{}{}{}{}start 1 0\n",
        i1(0, 12, 24, 2),
        at(2, 1, 0, 25, 0, 4, 4, false),  // add the row into TS13
        at(4, 1, 2, 24, 0, 6, 8, true),   // no TIL: 1_8, TIL: 1_9
        at(8, 1, 30, 29, 0, 10, 2, true), // dummy, back to 1_2
        i1(9, 9, 24, 11),
        at(11, 1, 7, 24, 0, 13, 11, true),
    );
    let mut m = machine(&src);
    let mut c = Card::blank();
    for r in 0..12 {
        c.rows[r] = 1 << r;
    }
    m.load_cards([c]);
    assert_eq!(m.run(10_000), Outcome::Loop { alarm: true });
    assert_eq!(m.ts13, 0xfff);
}

#[test]
fn initial_input_with_initial_card_and_triad_pm_p56() {
    // Run the successive-squares programme in as the machine did: initial
    // card, then a triad for DL1 whose fourth row enters the programme at
    // 1_0, with a hole in column 54 of the last row.
    let img = parse_image(&squares_dl1()).unwrap();
    let words = img.blocks[0].values();
    let mut deck = vec![cards::initial_card()];
    let mut t = cards::triad(1, &words, Some((1, 0)));
    t[2].col54 = 1 << 11;
    deck.extend(t);
    let mut m = Machine::default();
    m.load_cards(deck);
    m.initial_input();
    let out = m.run(1_000_000);
    assert!(m.warnings.is_empty(), "{:?}", m.warnings);
    assert_eq!(out, Outcome::Loop { alarm: true });
    let rows: Vec<u32> = m.punched().iter().flat_map(|c| c.rows).collect();
    assert_eq!(&rows[..20], &(1..=20).map(|n| n * n).collect::<Vec<u32>>()[..]);
}

#[test]
fn initial_input_two_triads() {
    // Fill DL2 with data and DL1 with a programme that adds 2_5 into TS13.
    let mut data = [0u32; 32];
    data[5] = 777;
    let mut prog = [0u32; 32];
    prog[0] = Instr::coded(0, 1, 2, 25, 0, 5, 7, true).encode();
    prog[7] = Instr::coded(7, 1, 7, 24, 0, 9, 7, true).encode();
    let mut deck = vec![cards::initial_card()];
    deck.extend(cards::triad(2, &data, None));
    let mut t = cards::triad(1, &prog, Some((1, 0)));
    t[2].col54 = 1 << 11;
    deck.extend(t);
    let mut m = Machine::default();
    m.load_cards(deck);
    m.initial_input();
    assert_eq!(m.run(100_000), Outcome::Loop { alarm: true });
    assert_eq!(m.ts13, 777);
}

// ---- drum (PM §6.4) --------------------------------------------------------

#[test]
fn drum_write_and_read_back() {
    // shift write heads to 9; write DL11 on head 15 (track 9/15); clear DL11
    // by reading track 0/0 (empty); shift read heads to 9; read head 15.
    let mut src = String::from("dl 1\n");
    src += &at(0, 1, 9, 31, 1, 2, 2, true);
    src += &at(2, 1, 15, 30, 1, 4, 4, true);
    src += &at(4, 1, 0, 30, 0, 6, 6, true);
    src += &at(6, 1, 9, 31, 0, 8, 8, true);
    src += &at(8, 1, 15, 30, 0, 10, 10, true);
    src += &at(10, 1, 11, 13, 0, 17, 12, true); // 11_17 - 13 waits for the drum
    src += &at(12, 1, 7, 24, 0, 14, 12, true);
    src += "dl 11\n17 = 4242\nstart 1 0\n";
    let mut m = machine(&src);
    assert_eq!(m.run(100), Outcome::Loop { alarm: true });
    assert_eq!(m.drum[9 * 16 + 15][17], 4242);
    assert_eq!(m.ts13, 4242);
    // the interlock held the instructions: well over two head shifts and three transfers
    let cfg = Config::default();
    assert!(m.now > 2 * cfg.drum_shift + 3 * cfg.drum_transfer);
}

#[test]
fn coded_helper_matches_manual_examples() {
    // "5, 13-3, 6, 10" in 1_17 transfers in m.c. 25 and goes to 5_29.
    let i = Instr::coded(17, 5, 13, 3, 0, 25, 29, true);
    assert_eq!((i.w, i.t), (6, 10));
    // PM p. 30: "4, 10-24, 0, 8" in 20_7? stored in 2_7... next 4_17.
    let i = Instr::coded(7, 4, 10, 24, 0, 9, 17, true);
    assert_eq!((i.w, i.t), (0, 8));
}

// ---- the coding-sheet transcriptions ---------------------------------------

#[test]
fn reads_coding_sheet_format() {
    let src = "# block 1: DL 5 track 12/5 cards 16-18\n# card rows (above the mc grid):\nY  = 5P1 + 12P5\n# mc rows:\n\
               0  1 0 30 - 0 1 -\n6  4 22 21 1 9 8 -   # W/T written \"9 (1) 8\": bracketed\n\
               29  3 3 22 3 30 9 x   # W/T written \"30 (15) 9\"\n3  7 21 27 - 0 6 - ?   # doubtful\n\
               5  0 0 0 - 0 1 +P1\n7  0 12 24 - 0 0 go+P34 ?\n24  -   # struck\n4  = P17,22-32\n\
               19  5 22 21 1 4(1) 3 -\n\
               # block 2: DL - track - cards 0\n# card rows (above the mc grid):\nX  0 30 8 1 1 0 x\n1  0 4 24 - 0 0 x\n# mc rows:\n1  0 8 24 - 0 0 x\n";
    let img = parse_image(src).unwrap();
    assert_eq!(img.blocks.len(), 2);
    let b = &img.blocks[0];
    assert_eq!((b.block.dl, b.block.track), (Some(5), Some(12 * 16 + 5)));
    assert_eq!(b.intro[0], Some(5 + 12 * 16));
    let i = Instr::decode(b.words[6].unwrap());
    assert_eq!(
        (i.nis, i.s, i.d, i.c, i.w, i.j, i.t, i.go),
        (4, 22, 21, 1, 9, 1, 8, true)
    );
    let i = Instr::decode(b.words[29].unwrap());
    assert_eq!((i.c, i.w, i.j, i.t, i.go), (3, 30, 15, 9, false));
    assert_eq!(b.words[5].unwrap(), Instr::new(0, 0, 0, 0, 0, 1, true).encode() | 1);
    assert_eq!(b.col54, 1 << 7);
    assert_eq!(b.words[24], None);
    assert_eq!(b.words[4], Some(p(17) | (0x7ff << 21)));
    assert_eq!(Instr::decode(b.words[19].unwrap()).j, 1);
    assert_eq!(b.doubts.len(), 2);
    let c = &img.blocks[1];
    assert_eq!(c.block.dl, None);
    assert_eq!(c.intro[1], Some(Instr::new(0, 30, 8, 1, 1, 0, false).encode()));
    assert_eq!(c.intro[3], Some(Instr::new(0, 4, 24, 0, 0, 0, false).encode()));
    assert_eq!(c.words[1], Some(Instr::new(0, 8, 24, 0, 0, 0, false).encode()));
    let cards = b.cards();
    assert_eq!(cards[0].rows[0], 5 + 12 * 16);
    assert_eq!(cards[0].rows[4], b.words[0].unwrap());
    assert_eq!(cards[0].col54, 1 << 11); // m.c. 7 is card 1 row 9
}

#[test]
fn all_transcribed_sheets_load() {
    // Every transcribed coding sheet loads; together they make one store
    // image (drum tracks, plus the blocks with no track).
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("sources/coding-sheets-1958/transcription");
    let Ok(entries) = std::fs::read_dir(&dir) else { return };
    let mut names: Vec<_> = entries
        .map(|e| e.unwrap().path())
        .filter(|p| {
            let n = p.file_name().unwrap().to_string_lossy();
            n.starts_with("sheet-") && n.ends_with(".txt")
        })
        .collect();
    names.sort();
    let mut m = Machine::default();
    let mut tracks = std::collections::BTreeMap::new();
    let mut blocks = 0;
    for path in &names {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let img = parse_image(&std::fs::read_to_string(path).unwrap()).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(!img.blocks.is_empty(), "{name}");
        for b in &img.blocks {
            if let Some(t) = b.block.track {
                if let Some(other) = tracks.insert(t, name.clone()) {
                    panic!("track {}/{} on both {other} and {name}", t / 16, t % 16);
                }
            }
            blocks += 1;
        }
        m.load_image(&img);
    }
    assert_eq!(names.len(), 22);
    assert!(blocks >= 60, "{blocks} blocks");
}

#[test]
fn division_out_of_range_warns_pm_p50() {
    // |A| = |B| is allowed only for A negative and B positive (PM p.50).
    for (a, b, warns) in [
        (1i32, 1i32, true),
        (-1, 1, false),
        (1, -1, true),
        (-1, -1, true),
        (1, 2, false),
        (3, 2, true),
    ] {
        let src = format!(
            "dl 1\n{}{}{}{}{}ts 14 = {a}\nts 16 = {b}\nstart 1 0\n",
            at(0, 1, 14, 21, 0, 3, 4, true),
            at(4, 1, 30, 21, 0, 6, 7, true),
            at(7, 1, 1, 24, 0, 9, 10, true),
            at(10, 1, 30, 29, 0, 13, 12, true),
            at(12, 1, 7, 24, 0, 14, 12, true),
        );
        let mut m = machine(&src);
        m.run(100);
        assert_eq!(
            m.warnings.iter().any(|w| w.contains("out of range")),
            warns,
            "{a}/{b}: {:?}",
            m.warnings
        );
    }
}

#[test]
fn multiplier_step_by_step_ldm_p64() {
    // The Logical Design Manual's example (p.64), 13 x 14, in the top four
    // digits: after 3, 5, 7 and 9 m.c. DS21 shows the partial products and
    // the remaining multiplier digits. Here with 32-digit words: multiplier
    // 13 * 2^28 in 21_3, multiplicand 14 in TS16; read DS21 just after each
    // step by long transfers 21-19 from the m.c. of 0-24.
    let src = "dl 1\n0 1 30 21 0 0 0 1\n2 1 14 21 0 1 1 1\n5 1 0 24 0 0 0 1\n7 1 21 19 2 0 1 1\n\
               10 1 30 29 0 0 30 1\nts 14 = 13P29\nts 16 = 14\nstart 1 0\n";
    let mut m = machine(src);
    for _ in 0..4 {
        m.step();
    }
    // 0-24 transferred in m.c. 7 (the multiplier's first m.c.); 21-19 (2)
    // in m.c. 9 and 10. In m.c. 9 the odd word emerges before that m.c.'s
    // shift, so it is still the multiplier; in m.c. 10 the even word emerges
    // as step 1 left it: the multiplicand, added because the top multiplier
    // digit is 1.
    assert_eq!(m.ds[0], [14, 13 << 28]);
    assert_eq!(m.run(100), Outcome::Loop { alarm: false });
    // and the finished product
    let prod = (13u64 << 28) * 14;
    assert_eq!(m.ds[2], [prod as u32, (prod >> 32) as u32]);
}
