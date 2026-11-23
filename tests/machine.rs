//! The pseudo-machine's limits and stops, as section 1.9 and the table of
//! section 2.4 describe them.

use george::machine::{self, Event, Machine, Options};
use george::number::Card;
use george::parse;

fn run(src: &str, options: &Options) -> george::Outcome {
    Machine::new().run(&parse(src).unwrap(), options)
}

fn punched(src: &str) -> Vec<String> {
    run(src, &Options::default())
        .punched()
        .iter()
        .map(|n| Card::of(*n).decimal())
        .collect()
}

fn stop_of(src: &str) -> &'static str {
    run(src, &Options::default()).stop.expect("should stop").kind.indication
}

#[test]
fn the_reverse_polish_examples_of_section_1_1() {
    // a=2 b=3 c=5
    let set = "2 (a) ; 3 (b) ; 5 (c) ; ";
    assert_eq!(punched(&format!("{set} a b + neg (P)")), ["-5"]);
    assert_eq!(punched(&format!("{set} a b + c × (P)")), ["25"]);
    assert_eq!(punched(&format!("{set} c a b + × (P)")), ["25"]);
    assert_eq!(punched(&format!("{set} a b c × + (P)")), ["17"]);
    assert_eq!(punched(&format!("{set} a b + dup × (P)")), ["25"]);
    // the example of rev: (a + b) + c/(a + b)
    assert_eq!(punched(&format!("{set} a b + dup c rev ÷ + (P)")), ["6"]);
}

#[test]
fn numerals_need_a_comma_between_them() {
    assert_eq!(punched("4, 3, 16 × + (P)"), ["52"]);
    assert_eq!(punched("4 3 (P)"), ["43"]);
    assert_eq!(punched("33.84 (P)"), ["33.84"]);
}

#[test]
fn remainder() {
    // "a 1 rem" gives the positive fractional part of a
    assert_eq!(punched("2.75 neg 1 rem (P)"), ["0.25"]);
    assert_eq!(punched("17, 5 rem (P)"), ["2"]);
}

#[test]
fn relations_and_logic() {
    assert_eq!(punched("2, 2 = (P)"), ["-1"]);
    assert_eq!(punched("2, 3 > (P)"), ["0"]);
    // ~ complements every digit of the word: not-false is all ones, a tiny
    // negative number (punched as 0, as Hamblin's punch does; negated, it
    // shows), and not-true is just under a half. Only the sign digit counts
    // as truth.
    assert_eq!(punched("2, 3 = ~ (P)"), ["0"]);
    assert_eq!(punched("2, 3 = ~ neg (P)"), ["2.38419e-7"]);
    assert_eq!(punched("2, 2 = ~ (P)"), ["0.5"]);
    assert_eq!(punched("7, 2, 3 = ~ 5 ↑ 8 * 5 (P)"), ["7"]);
    assert_eq!(punched("1, 1 = 2, 1 > & (P)"), ["-1"]);
    assert_eq!(punched("1, 2 = 2, 1 > ∨ (P)"), ["-1"]);
    // the truth indication and the mark number are both cancelled
    assert_eq!(punched("7, 1, 1 = 5 ↑ 8 * 5 (P)"), ["7"]);
    assert_eq!(punched("7, 1, 2 = 5 ↑ 8 * 5 (P)"), ["8"]);
}

#[test]
fn suffixes_spill_into_following_letters() {
    // "a32 represents the same number as b0"
    assert_eq!(punched("5, 32|(a) ; 0|b (P)"), ["5"]);
    // negative suffixes are interpreted modulo 1024
    assert_eq!(punched("6, 1 neg|(a) ; 31|ν (P)"), ["6"]);
    // double suffixes: the first spills through a, b, c, d
    assert_eq!(punched("9, 32, 3||(a) ; 0, 3||b (P)"), ["9"]);
}

#[test]
fn twelve_cells() {
    let eleven = "1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12 (P)";
    assert_eq!(punched(eleven), ["12"]);
    assert_eq!(stop_of(&format!("{eleven} 13")), machine::ACCUMULATOR.indication);
    assert_eq!(stop_of("+"), machine::ACCUMULATOR.indication);
    // "by omitting a cancel sign in a repetitive operation"
    assert_eq!(stop_of("1, 20 rep (j) j j × ]"), machine::ACCUMULATOR.indication);
}

#[test]
fn six_links() {
    let deep = "* 1 2 ↓ ] * 2 3 ↓ ] * 3 4 ↓ ] * 4 5 ↓ ] * 5 6 ↓ ] * 6 7 ↓ ] * 7 ]";
    assert_eq!(stop_of(&format!("1 ↓ ] {deep}")), machine::LINK_LIST.indication);
    let fine = "* 1 2 ↓ ] * 2 3 ↓ ] * 3 4 ↓ ] * 4 5 ↓ ] * 5 6 ↓ ] * 6 99 (P) ]";
    assert_eq!(punched(&format!("1 ↓ ] {fine}")), ["99"]);
}

#[test]
fn arithmetic_stops_and_the_single_shot() {
    assert_eq!(stop_of("1, 0 ÷"), machine::DIVISOR_ZERO.indication);
    assert_eq!(stop_of("2 neg √"), machine::DIVISOR_ZERO.indication);
    assert_eq!(stop_of("0 log"), machine::LOG_ARGUMENT.indication);
    assert_eq!(stop_of("2 neg .5 pow"), machine::FRACTIONAL_POWER.indication);
    assert_eq!(stop_of("1000 exp"), machine::EXP_RESULT.indication);
    assert_eq!(stop_of("5, 0 rem"), machine::REM_DIVISOR.indication);
    assert_eq!(stop_of("3 0 ↑"), machine::NO_MARK.indication);
    assert_eq!(stop_of("R"), machine::DATA_WANTED.indication);
    let o = run(
        "1, 0 ÷ 7 + (P)",
        &Options {
            single_shot: true,
            ..Options::default()
        },
    );
    assert!(o.stop.is_none());
    assert!(matches!(o.events[0], Event::Resumed(_)));
    assert_eq!(Card::of(o.punched()[0]).decimal(), "7");
}

#[test]
fn repetition_runs_at_least_once_and_leaves_x_at_b() {
    assert_eq!(punched("0, 5, 5 rep (x) 1 + ] (P) ; x (P)"), ["1", "5"]);
    assert_eq!(punched("0, 3, 3 rep (x) x + ] (P)"), ["3"]);
}

#[test]
fn a_mark_used_twice() {
    assert_eq!(stop_of("* 1 * 1"), machine::MARK_TWICE.indication);
}

#[test]
fn input_keys_and_wait() {
    let o = run(
        "I 2 × wait (P)",
        &Options {
            keys: vec![21.0],
            ..Options::default()
        },
    );
    assert_eq!(Card::of(o.punched()[0]).decimal(), "42");
    assert!(matches!(o.events[0], Event::Lights(Some(_))));
}

#[test]
fn programs_are_limited_to_512_symbols() {
    let src = "1 ; ".repeat(257);
    assert_eq!(stop_of(&src), machine::TOO_MANY_SYMBOLS.indication);
}

#[test]
fn a_single_shot_leaves_the_failing_symbol_undone() {
    let ss = Options {
        single_shot: true,
        ..Options::default()
    };
    // the 5 is still there after the failed +
    let o = run("5 + (P) ;", &ss);
    assert!(o.stop.is_none());
    assert_eq!(Card::of(o.punched()[0]).decimal(), "5");
    // a numeral that will not fit is skipped whole
    let twelve = "1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12";
    let o = run(&format!("{twelve}, 13 + + + + + + + + + + + (P)"), &ss);
    assert_eq!(o.events.iter().filter(|e| matches!(e, Event::Resumed(_))).count(), 1);
    assert_eq!(Card::of(o.punched()[0]).decimal(), "78");
}

/// Measured on Hamblin's own code (the transcribed coding sheets run on the
/// DEUCE simulator): the divisor of rem is rounded to a whole number, halves
/// up, after the sign test.
#[test]
fn rem_rounds_its_divisor_as_hamblins_code_does() {
    assert_eq!(punched("7, 3.6 rem (P)"), ["3"]);
    assert_eq!(punched("7, 4.5 rem (P)"), ["2"]);
    assert_eq!(punched("7, 4.4 rem (P)"), ["3"]);
    assert_eq!(punched("7, 2.5 rem (P)"), ["1"]);
    assert_eq!(punched("7, .4 rem (P)"), ["7"]);
    assert_eq!(punched("2.75 neg, 1 rem (P)"), ["0.25"]);
}

/// Findings of the independent review (docs/independent-review.md), each
/// checked against Hamblin's own code on the DEUCE simulator.
#[test]
fn findings_of_the_independent_review() {
    // rem is exact however large the dividend
    assert_eq!(punched("2, 100 pow 3 rem (P)"), ["1"]);
    assert_eq!(punched("2, 100 pow 7 rem (P)"), ["2"]);
    // too small a result stops, as too large a one does (2, 300 neg pow
    // already stops in pow: x log2 e is past 2^9, RUNNING-GEORGE.md finding 22)
    assert_eq!(
        stop_of("2, 200 neg pow 2, 200 neg pow × 2, 200 neg pow ×"),
        machine::OVERFLOW.indication
    );
    assert_eq!(stop_of("2, 300 neg pow"), machine::EXP_RESULT.indication);
    assert_eq!(stop_of("1000 neg exp"), machine::EXP_RESULT.indication);
    // = compares words: ~ of falsehood plus 0 is standardised, ~ of falsehood is not
    assert_eq!(punched("0 ~ 0 + 0 ~ = (P)"), ["0"]);
    // a tie at the sixth figure is punched up
    assert_eq!(punched("2, 10 neg pow (P)"), ["0.000976563"]);
    // only a, b, c and d take double suffixes, block punches included
    assert!(george::parse("1, 1, 1, 1 P||(e)").is_err());
    // the printed form reads back as the same program
    for src in ["R |a (P)", "* 1 2 (P)", "* 14 3, 4 (P)"] {
        let p = george::parse(src).unwrap();
        let again = george::parse(&george::text::format(&p.symbols)).unwrap();
        assert_eq!(again.symbols, p.symbols, "{src}");
    }
}

/// Measured on Hamblin's own code: indexing numbers round to nearest, halves up.
#[test]
fn indexing_numbers_round_as_hamblins_code_does() {
    assert_eq!(punched("7, 3|(a) ; 1, 2.5|a (P)"), ["7"]);
    assert_eq!(punched("9, 2|(a) ; 1, 1.6|a (P)"), ["9"]);
    assert_eq!(punched("5, 1 neg|(b) ; 1.5 neg|b (P)"), ["5"]);
    assert_eq!(punched("3.5, 5 rep (i) i (P) ; ]"), ["3.5", "4.5"]);
    assert_eq!(punched("1, 4.5 rep (i) i (P) ; ]"), ["1", "2", "3", "4", "5"]);
    assert_eq!(punched("5, 1.6 ↑ 9 * 1 (P) ; ] * 2 3 (P) ; ]"), ["3"]);
}

/// Measured on Hamblin's own code: sin and cos give 0 outside -2^21 <= x < 2^21.
#[test]
fn sin_and_cos_of_large_arguments_are_zero_as_on_hamblins_code() {
    // the last figures are those of Hamblin's code (the argument is taken to
    // 2/π with 2^-31 quarter-periods: RUNNING-GEORGE.md, finding 20)
    assert_eq!(punched("2097151 sin (P)"), ["-0.320808"]);
    assert_eq!(punched("2097152 sin (P) ; 2097152 cos (P)"), ["0", "0"]);
    assert_eq!(punched("2097152 neg sin (P) ; 2097153 neg sin (P)"), ["-0.623661", "0"]);
    assert_eq!(punched("0 sin (P)"), ["-2.98023e-8"]);
}

/// The manual's stop table lists "divisor zero (÷)", resumed giving zero,
/// and the manual wins over Hamblin's code (DECISIONS.md, "Which source
/// wins"): his divide sends a zero dividend to standardise before it looks
/// at the divisor (flow diagrams p.20), so on DEUCE 0 ÷ 0 gives 0 with no
/// stop. Here it stops like any zero divisor.
#[test]
fn zero_divided_by_zero_stops_as_the_manual_says() {
    assert_eq!(stop_of("0, 0 ÷ (P)"), machine::DIVISOR_ZERO.indication);
    assert_eq!(stop_of("1, 0 ÷"), machine::DIVISOR_ZERO.indication);
    let ss = Options {
        single_shot: true,
        ..Options::default()
    };
    let o = run("0, 0 ÷ (P)", &ss);
    assert!(o.stop.is_none());
    assert!(matches!(&o.events[0], Event::Resumed(s) if s.kind == machine::DIVISOR_ZERO));
    assert_eq!(Card::of(o.punched()[0]).decimal(), "0");
}

/// rem by a whole divisor of 2^15 or more follows the manual's definition
/// ("nb + x = a, n and b being integers and 0 <= x < b"), not Hamblin's
/// fetch integer, which takes 32768 as 1 and gives 0 (DECISIONS.md, "Which
/// source wins"). A fractional divisor, which the manual excludes, is still
/// rounded as his code does.
#[test]
fn rem_by_a_large_whole_divisor_follows_the_manual() {
    assert_eq!(punched("7, 32768 rem (P)"), ["7"]);
    assert_eq!(punched("100000, 32768 rem (P)"), ["1696"]);
    assert_eq!(punched("7 neg 32768 rem (P)"), ["32761"]);
    assert_eq!(punched("2.5, 65536 rem (P)"), ["2.5"]);
    assert_eq!(punched("1048576 dup × 1594323 rem (P)"), ["1119730"]);
    assert_eq!(stop_of("7, 32768 neg rem"), machine::REM_DIVISOR.indication);
    // below 2^15 Hamblin's rem and the definition agree word for word
    assert_eq!(punched("7, 32767 rem (P)"), ["7"]);
    assert_eq!(punched("7, 3.6 rem (P)"), ["3"]);
}

/// `a b >` "gives truth if a > b" (manual), on the values: a word with a
/// zero mantissa and a non-zero exponent, as `&` can make, is zero. Hamblin's
/// code compares through the subtract routine and gives falsehood here
/// (DECISIONS.md, "Which source wins").
#[test]
fn greater_than_compares_values_as_the_manual_says() {
    // .000000001 2 3 neg &: mantissa 0, exponent of 2's word & -3's
    assert_eq!(punched(".000000001, 2, 3 neg & > (P)"), ["-1"]);
    assert_eq!(punched("2, 3 neg & 0 > (P)"), ["0"]);
    assert_eq!(punched("0, 2, 3 neg & > (P)"), ["0"]);
}

/// The repetition increment word for word (tracks 12/12-12/13): the stored
/// variable is stepped in its own scale, a value the body gave it included.
/// Below 1/2 (other than zero) Hamblin's code goes astray, and the
/// interpreter stops (RUNNING-GEORGE.md, finding 30).
#[test]
fn the_repetition_increment_as_hamblins_code_does_it() {
    assert_eq!(punched("1, 3 rep (i) i (P) ; 1.5 (i) ; ]"), ["1", "2.5", "2.5"]);
    assert_eq!(punched("1, 2 rep (i) i (P) ; .75 (i) ; ]"), ["1", "1.75"]);
    assert_eq!(punched("1, 2 rep (i) i (P) ; .75 neg (i) ; ]"), ["1", "0.25"]);
    // from 2^20 up to 2^30 they do not step
    assert_eq!(punched("1, 2 rep (i) i (P) ; 2097152 (i) ; ]"), ["1", "2097150"]);
    let o = run("1, 2 rep (i) i (P) ; .3 (i) ; ]", &Options::default());
    assert_eq!(o.punched().len(), 1);
    assert_eq!(o.stop.unwrap().kind, machine::REP_ASTRAY);
    assert_eq!(stop_of("1, 2 rep (i) .3 (i) ; ] ;"), machine::REP_ASTRAY.indication);
}

/// A single shot after 3 14-21x stores standardise's word with the exponent
/// field e·2^21 where e·2^22 belongs: the exponent halved, rounded down.
#[test]
fn overflow_resumes_with_hamblins_word() {
    let ss = Options {
        single_shot: true,
        keys: vec![],
        ..Options::default()
    };
    let o = Machine::new().run(&parse("R R × (P) ;").unwrap(), &ss);
    assert!(o.stop.is_some());
    let mut m = Machine::new();
    m.cards.extend([
        Card::from_decimal("1e-154").unwrap(),
        Card::from_decimal("1e-154").unwrap(),
    ]);
    let o = m.run(&parse("R R × (P) ;").unwrap(), &ss);
    assert!(o.stop.is_none());
    assert_eq!(Card::of(o.punched()[0]).decimal(), "6.70391e-155");
}

/// Hamblin's own code: copying from an empty accumulator gives 0; taking a
/// number away from it stops.
#[test]
fn copying_from_an_empty_accumulator_gives_zero() {
    assert_eq!(punched("7 ; (P)"), ["0"]);
    assert_eq!(punched("7 ; (a) a (P)"), ["0"]);
    assert_eq!(punched("7 ; dup (P) ; 5 (P)"), ["0", "5"]);
    assert_eq!(stop_of("7 ; ;"), machine::ACCUMULATOR.indication);
    assert_eq!(stop_of("7 ; neg"), machine::ACCUMULATOR.indication);
}
