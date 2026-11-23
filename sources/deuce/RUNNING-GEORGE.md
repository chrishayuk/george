# Running Hamblin's GEORGE IA on the simulator

A log of what it took to run G-programs through the transcribed coding
sheets (`sources/coding-sheets-1958/transcription/`) on the DEUCE simulator,
with evidence for each finding. Command:

```
george deuce george examples/squares.george [--data FILE] [--full] [--trace]
                   [--keys N,N,...] [--single-shot]
```

It does what an operator did (manual §2.1):

1. Puts the basic pack in the reader. The basic pack is every block on the
   sheets that has card numbers, in card-number order: 193 cards, 0-192.
2. Presses INITIAL INPUT and runs until the pack is on the drum and the
   machine waits for the reader (`ReaderEmpty`).
3. Puts in the G-program initial card, the program cards and the data
   cards, and presses INITIAL INPUT again.
4. Runs to the end. The end is the stop `0, 0-0 X` with the reader called
   (manual §2.1: a second G-program "will be read in without interruption").
5. Decodes the punched cards.

## How a G-program is presented (this penultimate version)

- **G-program initial card (GPIC).** This is sheet 22 block 3, rows Y-9 of
  its first card. The rows are:
  - X: `1, 30-1, 1, 0, 31 X` (clear DL1);
  - 1: `13-31` (read heads to position 13);
  - 5: `14-30` (read 13/14, the clock-track read-in, into DL11);
  - 7: `11-8` (to DL8);
  - 9: `12-24`, with column 54 punched.

  The block's second card (m.c. 8-19, "Diagnostic card") and the rows below
  it are not used.
- **Program cards.** These read the symbols as **8-bit codes, four to a
  row**, all twelve rows of each card used. The first symbol of a row is in
  P1-8 (columns 21-28), then P9-16, P17-24 and P25-32. The code byte is the
  column of the coding table in the high four digits and the row in the low
  four (`Sym::byte`). Code 14/15 (byte 0xFE) ends the read. Codes 0/0 and
  15/15 are ignored (flow diagrams p.5).
  - Evidence: a test deck of bytes 0x41, 0x42, ... was stored on track 11/0
    in that order, one symbol per m.c. in P17-24, as p.5 says. The
    read-in's own masks are P17-24 and "P18-24 code#" (byte 0xFE).
- **Data cards** are one number per card, in the manual's standard form
  (§2.3) on the DEUCE field: GEORGE column *k* is card column 20+*k*.
  - column 21: sign of the mantissa (Y +, X −);
  - columns 22-30: the mantissa, d.dddddddd;
  - column 31: sign of the exponent;
  - columns 32-40: the exponent.

  `pairs` reads its data this way and stops on "1 11-6x, data card wanted"
  when the cards run out, as the manual says.
- **Punched results** are in the same form. GEORGE punches every column,
  zeros in row 0, and Y for plus. The mantissa comes out as six significant
  figures followed by 000 (e.g. `+1.41421000 +0`), so this version already
  rounds or truncates to six figures, as the manual says of the final one.

## Findings

1. **Sheet 1 block 2, card row 1: T = 27, not 29.** The transcriber
   recorded "29 or 27". With 29 the loader obeys the constant `15P5` as an
   instruction and the pack loads wrongly; with 27 all 193 cards load. The
   coordinator applied this to the transcription.

2. **Sheet 22 block 3 lacked section markers.** The row `1  1 13 31` is
   card row 1 (right-margin label 1, above the grid), not m.c. 1. That is
   the same layout as sheet 1 block 1. Read as m.c. 1, the head shift falls
   one row late and the GPIC loses its `11-8` row. I added `# card rows` /
   `# mc rows` comments to the transcription, with a note. No field was
   changed.

3. **Track 15/0 is the DEUCE library routines S06 and M12, coded for DL6.**
   Sheet 17 block 2 says only "[Deuce subroutines S06 and M12, version 6,
   with additions as shown]" and writes m.c. 10 and 28. The other 28 words
   are now in `sources/deuce/library/m12-s06-dl6.txt`, transcribed from the
   library sheets S6/10012 (M12) and S6/10013 (S06) with A = B = 6. They
   fill only m.c. that the sheet leaves empty, and the two never collide.
   Without them, `(P)` falls into a blank word at 6_18 (the punch's
   decimal conversion uses S06).

4. **P31, the "unused" digit, matters, and the sheets cannot show it.**
   Control ignores P31, but GEORGE adds and subtracts constants to and from
   instruction words, and a carry or borrow through P31 reaches the go
   digit P32.
   - **The flow diagrams say it outright** (p.9, foot): "Note: keywords
     coded with P31 and (if necessary) P22 to obviate spill on adjustment of
     wait and timing numbers." The translator subtracts from a keyword's T,
     and without P31 the borrow clears the go digit. The keyword comes out
     as a stopper (`4, 2-14, 0, 0, 28 X`).
   - **The store quasi.** The accumulator's "convert to 16-10" constant
     10_29, written `0 6 26 3 0(4) 30 -`, must be 6P5 + 26P10 + 3P15 +
     4P22 − 2P26, i.e. with P31 = P32 = 1. Only then is the pair 10_29 +
     10_15 exactly P17 + P22 ("convert back, add P17,22", p.16). 10_15 as
     written (`0 26 5 - 0(13) 1 x`) is then reproduced bit for bit,
     including its go digit 0.
   - **The rule adopted, class by class.** The sheets cannot show P31,
     so the simulator decides it per word (`george::set_p31`).
     - *Keywords* (the translator's keyword templates; p.9's note): P31 = 1.
       **Documented.**
     - *Constants that stand for a negative field*, like 10_29 (T = 30
       meaning −2): P31 = P32 = 1. **Shown by arithmetic** for 10_29 (the
       pair 10_29 + 10_15 comes out exactly). No other such constant has
       been worked through.
     - *Spill counters whose T carries into P31*: P31 = 0. These are the
       next-keyword templates `2, 21-17, 0, 0` (12/12 m.c. 30, 12/15 m.c.
       8, 13/10 m.c. 10 and 30) and the read-in's store template (13/12
       m.c. 9, finding 7). **Shown by arithmetic**: with P31 = 1 their
       counting flips the go digit (`george::P31_CLEAR`).
     - *Every other word written with a go digit*: P31 = 1. For words that
       are only ever obeyed, P31 makes no difference. Some must have it,
       because keywords are built on several tracks besides the 13/4
       keyword table, and "keyword tracks only" left some keywords as
       stoppers. But I have not identified that set word by word.
       **This class rests on "it makes GEORGE run"**, together with the
       guess that the go column was punched in both columns 51 and 52. A
       word wrongly given P31 = 1 would show up as a spurious stop only if
       arithmetic carries through it.
   - `GEORGE_P31=none`, `all`, or a list of tracks overrides the rule, for
     experiments.

10. **Sheet 21 block 3 (DL8, 15/13) m.c. 21 is struck and overwritten**
    (layers `0 29 27 2 26 26` and `0 21 23 - 30 6`). It cannot be read
    securely, and the transcription keeps it as struck and doubtful. 8_18
    (T 1) leads to it. Flow diagram p.38 (sin/cos) has `8_21 28-26 (2)
    (waste time)` between `8_18 21_2-15` and `8_17 15-23_3`.
    `0 28 26 2 26 26` (C 2, W = T = 26) does exactly that. It is close to
    the row's lower layer and to the struck line noted at m.c. 20. This is
    a **reconstruction, not a reading**. It is supplied at run time as
    `george::RECONSTRUCTIONS`, kept apart from the corrections of cells
    that *are* read. The independent review agreed it cannot be read
    securely.

11. **Reader timing.** `R||` (block matrix read) stops on `5 9-24`. The
    manual's table gives that indication both for "S.S. missed (reader
    fault), data read" and for "more than 512 symbols". Here it is the
    first: between cards, `R||` does more than 81 major cycles of
    computation, and at the reader's minimum card interval (80) it misses
    the first row of the next card. The manual gives 80-100. The simulator
    now uses the middle of each reader range (rows 18 MC, cards 90 MC), and
    `matrix` runs; it works from 82 MC up. `GEORGE_READER_ROW` and
    `GEORGE_READER_CARD` (major cycles) override the timings. The command
    now lists every meaning of an ambiguous indication: 1 30-21x, 5 9-24.

16. **`wait`: the keyword table has the old keyword (sheet 8 block 1,
    DL5 13/4, m.c. 6).** `2 wait (P) ;` stopped on a blank word at 8_21.
    - *The trace.* The symbol code 4/0 is decoded through the DL4 ladder
      (13/3 m.c. 4, `5_6-25 -> 1_3 SNT`, flow diagrams p.9), which takes
      the keyword template from 5_6. The sheet has `3 15 30 - 0 6` there,
      i.e. `15-30 -> 3_8`, and SNT stores it as `3, 15-30, 0, 0, 2`. At
      run time this keyword reads track 12/15 (the skips, p.25) into
      DL11, 3_8 (`11-7(32)`, 12/3) moves it to DL7 and enters it at 7_9,
      which goes back to NK. The next keyword slot is empty, hence the
      blank word. (The trace labels minor cycles six lower than the
      sheets throughout, translator and calculation alike: the whole
      store is consistently rotated, so the trace's 4_30 is the sheet's
      4_4 and its 3_2 the sheet's 3_8.)
    - *The flow diagrams.* p.9's keyword for wait is written "14-30 ->
      1_3", with 14 written over 15 and 1_3 over 3_8. p.23 gives the wait
      routine as "k 14-30, 1_3 11-7(32), 7_4 8-24, 7_6 17_3-21_3, ...,
      7_5 16-29, 7_18 16-29 X -> 1_24 (NK)".
    - *The sheets agree with the amendment.* 12/14 m.c. 4-18 is p.23's
      routine word for word (NOTES-1-11: "7_18 16-29 x matches wait"), and
      12/1 m.c. 3 is `7 11 7 1 0 31`, `11-7(32)` on to 7_4. No other route
      reaches 12/14 at 7_4; `15-30 -> 3_8` leads nowhere that waits.
    - *The scan.* At 600 dpi the cell reads clearly `3 15 30 - 0 6`, with
      the checking tick after T and no overwriting. So it is not a
      misreading: the sheet keeps the keyword from before the amendment
      that p.9 records (presumably wait was moved from 12/15 to 12/14,
      where `rep`'s first entry is, and the keyword table was not
      repunched on this sheet).
    - *Fix.* `george::CORRECTIONS` sets 13/4 m.c. 6 to `1 14 30 - 0 1 -`
      (NIS 1, `14-30`, T 1: `-> 1_3` as Q_0, like `max`'s `7-30 -> 1_3`
      at m.c. 13). The transcription keeps the literal reading. SNT's
      subtraction from T borrows through P31, which the keyword rule
      (finding 4) sets. Now `wait` stops on `1 16-29x` with the top cell on
      the O.S. (`2` shows as `01000000000000000000000000000010`), and a
      single shot goes on to NK, as the manual's stop table says.
    - With an empty accumulator wait shows the word below the bottom cell,
      10_0, which holds the mask P1-10 (13/9 m.c. 0). The command reports
      that as "the empty-accumulator indication", as the interpreter does.

17. **Resuming after ÷ by zero needs the operator's delay.** Divide starts
    the divider (`4_15 1-24`, p.20) before it tests the divisor, so stop
    `1 4-24x` comes while the divider is still running. A single shot in
    the same minor cycle let the divider finish later and overwrite DS21
    in the middle of punch translate (garbage cards, then a blank word).
    A real operator takes at least a fraction of a second, and the divider
    takes 66 m.c., so the command presses the key a second (31 250 m.c.)
    after the stop (`george::OPERATOR_DELAY`, `Machine::single_shot_after`).
    Then `1, 0 ÷ (P)` punches 0, as the manual says.

## Operator facilities (`george deuce george`)

- **wait.** When the run stops on `1 16-29x`, the command prints the O.S.
  as a number, "wait: the output lights show N" on stderr (with `--full`,
  also the O.S. bits), and gives a single shot. Only this stop is resumed
  by default.
- **`--single-shot`** also resumes the stops the manual calls resumable
  (section 2.4): 3 14-21x, 1 4-24x, 1 30-16x, 1 30-21x (log), 7 30-21x,
  5 30-21x and 3 2-2x (mark used twice, at translation). Each is reported
  as the interpreter reports it. IA's 1 30-21x is also the accumulator
  stop, which the manual calls non-resumable in IA. It comes from 1_14 of
  12/1 (DL1), log's from DL8, and the command resumes only the latter.
  The link-list stop 1 8-21x is non-resumable in IA too.
- **`--keys N,N,...`.** In IA, `I` does not stop. The manual's stops for
  `I` (`2 0-16x`, `1 4-15x`) are "II only", and the keyword is `0-16 ->
  1_20` (p.9): the I.D. word goes to TS16 as it stands and is stored in
  the accumulator. So the keys must be set to the number as a GEORGE word
  (exponent P1-10, mantissa P11-32). The command sets the I.D. to the next
  number just before each `I` keyword is obeyed, one number per `I` (0
  when the list runs out), as the interpreter's `--keys` does. On the
  machine the operator would set them at a stop, usually a `wait` before
  the `I`.
- **The I.D. at read-in (sections 2.4, 2.6).** The command leaves the I.D.
  clear until the first `I`. Tested: with a digit of P1-31 set while the
  G-program is translated, the machine loops on `0 0-14` / `0 24-28` ("I.D.
  has entry in P1-31, clear and continue"); with P32 alone it stops on
  `7 30-28x` at the end of translation (the test facility).
- Tests: `wait_shows_the_top_cell_and_resumes`, `keys_are_read_by_i`,
  `id_must_be_clear_at_read_in`, `single_shot_resumes_arithmetic_stops`
  in `tests/deuce_george.rs`.

## Results

All 17 programs in `examples/` were run through `george deuce george`
against `george run`:

| | |
|---|---|
| Same output | squares, sum-of-squares, factorials, nested, newton, converge, subroutine, polynomial, any-zero, mean, mean-one-pass, normalise, pairs (ends on 1 11-6x, data card wanted, as its .stop says), matrix, simpson, simpson-corrected |

Until + and − were done word for word as Hamblin's add routine does them
(finding 15), the two Simpson programs differed in the sixth figure:

| program | GEORGE IA | interpreter |
|---|---|---|
| simpson | 0.0497078 0.0675882 0.0807849 0.093649 0.0988802 | 0.0497078 0.0675879 0.0807847 0.0936491 0.0988803 |
| simpson-corrected | 0.198831 0.270353 0.323139 0.374596 0.395521 | 0.198831 0.270352 0.323139 0.374597 0.395521 |

The interpreter now punches the GEORGE IA column.

### Where Hamblin's arithmetic differs (the rounding question in DECISIONS.md)

I read the 22-digit mantissa GEORGE stores (`explore_arithmetic` in
`tests/deuce_george.rs`) and compared it with the exact result of the
stored operands:

- **× and ÷ round to nearest** (but see findings 26 and 27: not always).
  Examples: 1/7 stored as 1198373 (exact
  1198372.57); 7/3 as 1223339 (exact .67); (1/7)·(1/7) as 1369570 (exact
  .63); √2·√3 as 1284238 (exact .80).
- **Numerals round to nearest** (0.1, 0.3, 0.7, 1.1, 3.14159), and so does
  √.
- **exp is correctly rounded** for 1, 2, 0.5 and −1.
- **log is 1-2 units low in the last place for some arguments** (log 3,
  log 10), and correct for others (log 2, 5, 7, 1.5, 0.75). Traced in
  finding 20: an economised series with every product rounded.
- **+ and − are neither rounded nor truncated.** 1 + 1/7 gives 1198372
  (exact 1198372.625: truncated). 1 − 1/3 gives 1398101 (exact .5).
  5 − 1/7 gives 1273270 (exact .84: truncated). But 1 + 1/3 gives 1398102
  (exact 1398101.25) and 5 + 1/3 gives 1398102 (exact .31), which are one
  unit *above* nearest. **Settled by finding 15:** the alignment shift
  misplaces the smaller operand's dropped digits, and standardise then
  rounds half up.
- **Punching** gives six figures, rounded (2/3 → 0.666667), with "000" in
  the last three mantissa columns. (Rounded from an approximation, not the
  exact value: finding 25.)

So the sixth-figure differences in Simpson came from addition and
subtraction. They do not come from × or ÷, which round
as the interpreter does. No difference was found in single × ÷ exp √
operations at six figures (170 random cases).

### Behaviour of this version that differs from the interpreter

- **`max`, when the second operand is the larger, gives its negative:**
  `2, 5 max` → −5, `5, 2 max` → 5, `3, 3.5 max` → −3.5.
  - The sheet (12/7 m.c. 14, `7 20 21 - 1 4`, 20_3 → 21_3) is read
    correctly. After "subtract", 20_3 holds the operand with the larger
    exponent, with the sign it has in a − b, so it is −b when b is the
    larger.
  - Flow diagram p.21 shows this step marked with crosses and a marginal
    "? 30-21_3 / 20-23_3" (that is, 0 − 20_3). That amendment fixes
    (2, 5) but breaks (7, 3), because there 20_3 holds +7. I tried it and
    then took it out.
  - Taken to be genuine behaviour of this version, with the fix not
    recoverable from these pages.
- **`pow` with a negative base that is minus a power of two** (−1, −2,
  −4, −0.5) stops on 1 30-21x, which is the log argument stop.
  - Other negative bases work: (−3)² = 9, (−3)³ = −27, (−1.5)² = 2.25.
  - pow takes |a| by `30-13, 19_3-26, 13-19_3` (p.34). The standardised
    mantissa of a negative power of two is −1, and negated it is +1.0. That
    is outside the positive range. log's normalising shift (`21-22(20)` at
    8_6) carries it into the sign digit, and the sign test stops.
  - Genuine behaviour of this code, as transcribed and as the flow diagram
    writes it. The interpreter now does the same (finding 22); resumed by a
    single shot, it gives 1.
- **`~` of falsehood punches as 0.** The interpreter gave −2.38419e-7.
  - `~ neg` and `~ 1000000 ×` agree, so the stored word is right.
  - Traced (finding 25): the all-ones word is mantissa −2⁻²¹, exponent −1.
    Punch translate scales it by 10⁰ and finds it below 1, so it lowers
    the decimal exponent to −1 and takes the six figures of 10 × 2⁻²²,
    which are all 0. The card is `-000000000-000000001`, read as 0. The
    interpreter now punches the same card.
- **`(P)` with an empty accumulator** punches 0 and then stops at the next
  symbol. The interpreter stops at `(P)`.
- **`rem` with a fractional divisor** (7, 2.5 rem → 1) agrees with the
  interpreter. Traced (finding 23): "fetch integer" (12/6) takes the
  divisor, rounding halves up, but with the exponent mod 16, so `7, .4 rem`
  divides by 26. The interpreter now does rem word for word.

- **`√` of a negative number, resumed with a single shot, gives 0.5**, not
  the zero the manual's stop table promises. p.37 branches on the sign to
  `7_11 4-24 X` and then to "1_18 store and NKD", which stores TS16; at
  that point TS16 still holds the first approximation P31 (`7_6 7_8-16
  [P31]`), which as a GEORGE word is 0.5. Genuine behaviour of this code,
  as transcribed and as the flow diagram writes it. Confirmed by finding
  18: `functions::sqrt` gives 0.5 as the word resumed, but the interpreter
  keeps the manual's 0 (a choice left open in DECISIONS.md).

### The tutorial programs (`tutorial/programs/`)

Every program run both ways, with its `.in` as `--data` and its `.args`
(`--keys`, `--single-shot` and `--columns` given to both; `--trace` and
`--cards` only change the presentation and were dropped; `--limit` counts
symbols in the interpreter and was given to it alone). Punched numbers
compared. Of the 100 programs, 94 punch the same (five of them are
rejected by the program reader before either route runs, and 11-endless
runs to the limit both ways). Where both stop, they stop on the same
indication (1 30-21x, 1 8-21x, 1 11-6x, 4 9-24, 7 25-29, 1 30-16x,
3 2-2x, 3 14-21x). Six differ:

| program | GEORGE IA | interpreter | why |
|---|---|---|---|
| 10-functions | `3, 7 max` → −7 | 7 | max sign (above) |
| 10-pow | stops on 1 30-21x at `2 neg, 3 pow`, nothing punched | −8 4 0.5 0 0 | pow of minus a power of two (above). Now the same both ways (finding 22); the tutorial's program uses −3 instead and shows −27 9 0.5 0 0 |
| 10-sines | sin 0 → −2.98023e-8; cos 90° → 2.85916e-7 | 0; 3.13916e-7 | last figures of sin near 0 and cos 90°. Now the same both ways (finding 19) |
| 11-single-shot | `2 neg √` → 0.5 | 0 | √ of a negative number resumes with 0.5 (above) |
| 11-accumulator-ss | stops on 1 30-21x, nothing punched | 78 | IA's accumulator stop is not resumable (manual §2.4: "(IA) Non-resumable, (II) S.S. calls in next symbol") |
| 07-backwards | 5, 6, ... on to the instruction limit | 5 ... 13, then `--limit 40` | the interpreter's `--limit` counts symbols; the numbers agree as far as both go (`5, 3 rep` counts down from 2¹⁶ − 2) |

`11-wait` and `11-keys` now agree, wait lines included. The Simpson
programs (12-simpson, 12-simpson-corrected, 12-simpson-vector) agree with
the interpreter as it now stands.

### sin and cos (findings 12 and 14): solved with the multiplier modelled digit by digit

- **Layout (finding 12).** The trace of `.5 sin` lands on every step of
  flow diagram p.38. The Horner coefficients are 15/13 m.c. 13, then 7, 5,
  3, 1, then 31 (= 1.0), as the sheet has them. p.38's "(m.c. 4)",
  "(m.c. 14)", "(m.c. 20)" are the minor cycles in which 21_2 is read, not
  coefficient addresses (READING.md had read them as addresses; the
  independent review agrees). 2/π at 15/12 m.c. 29 reads correctly. The
  doubtful rows 15/13 m.c. 20 and 30 fit the flow and the timing.
- **Why they were wrong (finding 14).** Every multiplication in sin/cos
  reads 21_2 three m.c. after `0-24`, part way through, and later subtracts
  that word from the completed product. (Corrected by finding 19: the read
  is part way through, every subtraction comes after the product is
  complete.) That depends on DS21's contents during the multiplication.
  The *DEUCE Logical Design Manual* Part I, ch. 10, pp. 60-65 (saved in
  `sources/deuce/ldm/`) describes them exactly:
  - the partial product and the remaining multiplier share DS21;
  - in each pair of m.c. both shift up one place, and TS16 is added in
    the even m.c. if the multiplier digit examined was 1.

  Three m.c. after the start, 21_2 holds TS16 if the multiplier's top
  (sign) digit was 1 and zero otherwise. That is exactly the term that
  must be subtracted from the top half to make an unsigned product signed:
  Hamblin captures it while the multiplication is still running and takes
  it off afterwards.
- **Result.** With the step-level multiplier (`Machine::mult_until`), sin
  and cos agree with the interpreter to six figures for 0.5, 1, 1.5, 3 and
  10 (sin) and 0, 1, 2 (cos). `0 sin` gives −2.98023e-8 (−2⁻²⁵), where the
  interpreter gives 0. Nothing else changed:
  - all the examples give the same output as before;
  - × and ÷ still round to nearest;
  - the multiplier gives the plain product when nothing touches DS21
    during it.
- **Tests.** In `tests/deuce_george.rs`: `symbols_agree_with_the_interpreter`
  (sin and cos) and `where_hamblins_george_differs` (`0 sin`). In
  `tests/deuce.rs`: `multiplier_step_by_step_ldm_p64`.

### + and − at the last digit (finding 13)

Stored 22-digit mantissas (units 2⁻²¹ of the fraction) against the exact
sum of the stored operands:

| expression | exact sum (aligned) | GEORGE | |
|---|---|---|---|
| 1 + 1/7 | 1198372.625 | 1198372 | floor |
| 1 + 1/3 | 1398101.25 | 1398102 | ceil |
| 1 + 1/9 | 1165084.4375 | 1165084 | floor |
| 1 + 1/11 | 1143901.0625 | 1143902 | ceil |
| 1 + 2/3 | 1747626.5 | 1747626 | floor |
| 3 + 1/7 | 1647762.3125 | 1647762 | floor |
| 3 + 1/3 | 1747626.625 | 1747626 | floor |
| 3 − 1/3 | 1398101.375 | 1398102 | ceil |
| 5 + 1/3 | 1398101.3125 | 1398102 | ceil |
| 5 − 1/7 | 1273270.84375 | 1273270 | floor |
| 7 + 1/3 | 1922389.3125 | 1922390 | ceil |
| 7 − 1/3 | 1747626.6875 | 1747626 | floor |
| 1.5 + 1/3 | 1922389.25 | 1922390 | ceil |
| 100 + 1/3 | 1643861.33 | 1643862 | ceil |
| 1/3 + 1/7 | 1997287.5 | 1997287 | floor |
| 1/7 + 3/7 | 1198372.75 (after a 1-place normalising shift) | 1198373 | ceil |
| 1 + 3/1024, 1 + 5/1024, 1 + 1/1024 | exact | exact | |

- **There is no clean rule.** It is not truncation, nor rounding to
  nearest, nor round-half-up of the aligned operand. 3 + 1/7 and 5 + 1/3
  have the same exponent difference (4), the same shifted-out bits (0101)
  and the same fraction (.3125). One is truncated and the other goes up.
  - "Make the truncated integer sum even" fits 14 of the 16 inexact cases,
    but not 1/3 + 1/7 or 1 − 1/3.
  - The result depends on more than the aligned sum. Presumably it depends
    on how the unpack/add/standardise sequence (pp.18-19, with its
    "27-22_3" adding P1 to the high word) carries the dropped digits
    through the double register.
  - This needs the add routine traced digit by digit, which I have not
    done.
- **Does it explain Simpson?** Yes. The Simpson programs sum many
  terms, and with errors of up to one unit either way per addition that is
  enough to move the sixth figure. × and ÷ round to nearest, as the
  interpreter does, so they are not the source. The rule is finding 15;
  every row of this table follows from it, and the test
  `hamblins_arithmetic_at_the_last_digit` checks the simulator and the
  interpreter against the table.

### + and − word for word (finding 15)

Traced instruction by instruction with the registers
(`trace_registers` in `tests/arith_words.rs`, which prints TS13-16 and
DS19-21 after each instruction of tracks 12/x and of instructions obeyed
from TS13). **m.c. here are the coding sheets'**; the simulator's trace
labels them six lower (finding 16), so the sheet's 3_2 is the trace's
3_28. Words are written high:low for a double store (odd:even m.c.).

`1, 3 ÷ 1 +` as an example: a = 1/3 (mantissa 1398101 = 0x155555,
exponent −1), b = 1 (0x100000, exponent 1).

1. **Unpack two** (p.18, 12/3, `3_23 30-21`, `3_0 22-21(20)`, `3_21`,
   `3_26 23-21`): each operand becomes a double word, mantissa in the
   high word (P1-22, sign extended), exponent ×2²¹ in the low word. The
   top cell (b) is in DS20, the one below (a) in DS19: DS19 =
   00155555:ffe00000, DS20 = 00100000:00200000. DS21 holds a again with
   its low word cleared (`3_13 30-21`): 00155555:00000000.
2. **Exponent difference** (p.19, 12/4): `4_2 19-13`, `4_4 20-26`: TS13 =
   (e_a − e_b)·2²¹. `1_6 13-28` tests it for zero and `4_9 13-27` for
   sign.
   - **Positive** (a has the larger exponent): `4_11 29-18`, then DS19 and
     DS20 are exchanged through DS21 (`4_13 20-21(2)`, `4_16 19-20(2)`,
     `4_19 21-19(2)`), `4_24 30-21` clears DS21's low word, and the
     routine starts again at 4_2. So the operand with the smaller exponent
     is always the one in DS19 and DS21, and its exponent is the one
     shifted away from.
   - **Zero:** straight to the add, `4_8`, with no shift.
   - **Negative:** `4_12 20-28(2)` tests DS20 (the operand with the larger
     exponent) for the zero word. If it is zero, `5_15 19-13` takes DS19's
     exponent and the routine goes to standardise with DS21 = the other
     operand unshifted. Otherwise:
3. **The shift count** (12/5, k = e_b − e_a > 0): `5_16 4-25` adds
   4_18 = 23P22 to TS13, giving (23 − k)·2²¹, and `6_18 13-27` sends
   k ≥ 24 to `4_21 20-21` (DS21's high word := b's mantissa; a is dropped)
   and on to `4_14`. Otherwise `4_20 5-26` subtracts 5_22 = 7P22, giving
   (16 − k)·2²¹, and `4_22 13-27` tests it:
   - negative (k ≥ 17): `5_25 22-21` (C 1, W 1, T 0: 32 m.c. starting in
     an even one, an exact double-length shift of 16 places). TS13 now
     holds −(k − 16)·2²¹.
   - otherwise: `5_24 5-26` subtracts 5_26 = 16P22, leaving −k·2²¹.

   Call what is left −k'·2²¹ (k' = k or k − 16, 1 to 16). It is shifted
   down four places (`5_27 13-14`, `4_29 23-14(1)` for three, `5_1
   23-13`) to −2k'·P17 and added to the template 5_6, `4, 22-21, 1,
   9(1), 8` (`5_3 5-25`): W becomes 9 − 2k' (borrowing from the bracketed
   digit when it goes below 0), and the instruction is `22-21` for 2k'
   m.c., ending in m.c. 8. `3_28 13-0` obeys it from TS13.
4. **The shift is odd-first.** 2k' m.c. ending in an even one start in an
   odd one. S22 in an odd m.c. gives the high word halved; in an even m.c.
   the low word halved with, as its top digit, the last digit of the high
   word *as it now is* (MACHINE.md §5, S22). So each pair halves the high
   word and then fills the low word's top from the new high word: the old
   digit 0 of the high word is lost and its digit k' is copied in twice.
   For a = 0x155555, k = 2: DS21 becomes 00055555:80000000, where an exact
   shift gives 00055555:40000000. The low word holds digits 1..k' of the
   operand where digits 0..k'−1 belong (after an exact 16-place shift,
   when there is one).
5. **Add** `4_8 20-22` (m.c. 11, odd): b's mantissa into DS21's high
   word, 00155555:80000000. `4_14 20-13`: TS13 := b's exponent. Then the
   normal link to standardise (p.18).
6. **Standardise** (p.18, 12/3):
   - `3_30 21-28(2)`: if the whole of DS21 is zero, the result is the
     zero word.
   - `3_2 27-22` (m.c. 5, odd): **P1 is added to the high word**, once.
     00155556:80000000.
   - Loop: `3_5 21-14`, `3_7 24-14(9)`, `3_18 24-15`, `3_20 26-27`: TS14
     and TS15 are the high word shifted up 9 and 10, so the sign of their
     exclusive or is P23 ≢ P22 of the high word. If they agree:
     `3_24 10-26` (exponent − 1), `3_29 21-22(2)` (DS21 doubled, exactly,
     even first), `3_3 27-23` (**P1 taken off the high word again**), and
     round again. With R = high·2³² + low, R = V + 2³² throughout, where V
     is the true double-length sum: the P1 stays a unit in the final place.
     002aaaad:00000000 → 002aaaac:00000000.
   - When they differ: `3_25 10-25` (exponent + 1), `3_31 22-21(2)`
     (halved exactly, even first): 00155556:00000000.
   - Pack: `3_6 24-15`, `3_10 15-21` (**the low word is replaced by the
     exponent**: dropped), `3_12 21-22(20)` (shifted up 10 places). The
     high word is the mantissa: 1398102.

As word-level pseudo-code (`number::add_words` is this, in Rust):

```
add(a, b):                       # mantissas m (22 digits, signed), exponents e
    if e_a > e_b: swap a, b      # b has the larger exponent (or they are equal)
    k = e_b - e_a; e = e_b
    if k > 0 and b is the zero word:  R = m_a·2^32; e = e_a
    elif k >= 24:                R = m_b·2^32
    else:
        R = m_a·2^32
        if k >= 17: R = R >> 16 (arithmetic); k -= 16
        repeat k times:          # odd-first: high, then low from the new high
            high = high >> 1 (arithmetic)
            low  = (low >> 1, logical) + (high & 1)·2^31
        R = R + m_b·2^32
    if R == 0: return the zero word
    R = R + 2^32
    while digit 22 of high == digit 21 of high (0-indexed):
        R = 2R - 2^32; e = e - 1
    R = R >> 1 (arithmetic); e = e + 1
    return mantissa = high (22 digits), exponent e   # stop 3 14-21x if out of range
subtract(a, b) = add(a, b with its mantissa negated as a whole word)
```

The standardise half on its own is "round the double-length sum to
nearest, halves up". What makes + and − look ruleless is step 4: the
digits below the last place are not the shifted-out digits of the smaller
operand but those digits moved one place along, with the lowest lost. In
the 22 cases of finding 13, 3 + 1/7 and 5 + 1/3 have the same exact
fraction .3125 but different digits 1..4: 1/7 (…100101) shifted 4 places
puts .125 in the low word and 1/3 (…010101) puts .625, so with the half
from P1 one stays at 1647762 and the other goes up to 1398102.

**Checked word for word** (`tests/arith_words.rs`): random operand pairs,
built in the G-program by exact operations only (an integer numeral,
divisions by powers of two, `neg`), both signs, mantissas at the ends of
the range, exponent differences 0-29 with 0, 1, 21-23 and 15-24 weighted,
equal and opposite operands, near-cancellation, carries, zero operands; and
every pair of nine unstandardised words (`0 ~`, truth, `&`, `∨`). The
words GEORGE stores for the operands are checked first, then `a b + (c)`
and `a b - (d)` against `Num::plus` and `Num::minus`. **200,324 of
200,324 sums and differences agree** (two runs of 50,000 pairs plus the
81 fixed pairs each); 2,000 pairs run in the normal `cargo test`. Of them
9 % cancel to zero, 22 % renormalise down and 26 % carry up a place, and
5 % of the pairs are more than 23 places apart.

Two things found on the way, followed up in findings 25 and 26:

- **× of the mantissa 2²¹ − 1 is not exact:** `2097151, 1 ×` stores
  2097152 (exponent 22), and `2097151, 2 ×` likewise. ÷ by a power of two
  is exact. The interpreter gives 2097151.
- **Punching an exact half in the seventh figure** goes down for 747234.5
  (`864.4266 dup × 1 +` punches 747234; the interpreter 747235) and
  87147.75 (`295.2081 dup ×`: 87147.7; the interpreter 87147.8), but up
  for 2⁻¹⁰ = 0.0009765625 (9.76563 both ways). The stored words are the
  same both ways.

### The functions word for word (findings 18-24)

Traced as finding 15 was, with `trace_function` in
`tests/function_words.rs` (`SRC="0.5 √ (a) ;"`; `TRACKS` picks the track
groups shown, 15 by default, and track 12 is shown once a function has
started). M.c. below are the sheets' (the trace's plus six, finding 16).
Words are written high:low for DS21; "b.p." is binary places, so P31 is
1.0 at 30 b.p. `unpack` leaves the mantissa m sign-extended in 21_3 (units
of its last digit), e·2^21 in 21_2 and e·2^22 in TS14. The multiplier
(`0-24`) gives the unsigned 64-digit product; every routine reads 21_2
three m.c. after `0-24` (TS16 if the multiplier's top digit is 1, else 0:
LDM p.64, finding 14) but takes that off the high word only after the
product is complete, so each product below is the plain signed (or, where
said, unsigned) product. "Round" means a `21-23` or `29-23` on 21_2, which
adds the low word's top digit (or 2^31) to the high word: halves up.
Standardise is finding 15's step 6; out of range it stops 3 14-21x, and a
single shot stores the word with e·2^21 where e·2^22 belongs (`3_11
14-21`): the exponent halved.

18. **√** (15/11, p.37). No standardise; the iteration is Newton's with the
    divider (PM §6.10, A = 21_3, B = TS16, quotient 2^31·A/B in 21_2, TCB
    left on, so `22` then shifts the low word alone, arithmetically).
    ```
    N = m << 9 if e even, m << 8 if e odd    # 7_9 25-28 (TS14 & P23), 7_12/7_13 21-22(18)/(16)
    if N < 0: stop 1 4-24x                   # 7_1 20-27, 7_11 4-24x; resumed: TS16 = P31, i.e. 0.5
    x = P31                                  # 7_6 7-16
    loop:                                    # 7_7 1-24
        t = (q/2 - x + 1) >> 1   (q = 2^31 N / x; q/2 arithmetic)   # 7_10, 7_14, 7_16, 7_18 22-28
        if t == 0: break
        x = x + t                            # 7_21, 7_24, 7_26 21-16; 7_28 20-21 puts N back
    R = (x : q/2 - x + 1) >> 9, + 2^31       # 7_20, 7_23 4-24, 7_25 22-21(18), 7_15 29-23
    R = (high(R) : e·2^21) << 10, + 2^31     # 7_22 20-21, 7_31 21-22(20), 7_27 29-23
    word = high(R)                           # 7_30 21-16, stored as it stands
    ```
    So the exponent is e/2 rounded halves up, and the mantissa x rounded at
    the ninth digit. Two quirks follow: for e = −1 the exponent field
    overflows (0x3ff + 1) and the carry adds a unit to the mantissa, and a
    mantissa that rounds up to 2^21 lands in the sign digit.

19. **sin and cos** (15/12-15/13, p.38). `7_4 29-19` (sin) or `7_6 30-19`
    (cos) sets the flag 19_2.
    ```
    if sin and e < -10: store the argument word (TS16) as it is   # 7_9 28-25(10), 7_20, 7_25 19-27, 7_23
    if e >= 22: store 0                                           # 7_24, 7_12 13-27, 7_15 30-16
    P = m · 0xa2f9836e   (2/π, unsigned at 32 b.p.; m signed)     # 7_27, 7_31 0-24, 7_2 21-15, 7_16 15-23
    R = P >> (22 - e), exact (16 at a time, then the rest)        # 7_3, 7_8, 7_11, 7_10, 7_13, 3_28 13-0
    if sin: R = R - 2^31                                          # 8_29 19-22 (P32 added double-length)
    h0 = bit 0 of high(R);  u = low(R) with bit 0 cleared         # 8_0 22-14, 8_2 24-16, 8_6 24-21
    z = round(u·u signed, to the high word)                       # 8_9 0-24, 8_12, 8_16, 8_22, 8_24
    s = c13;  for c in c7, c5, c3, c1, P31:                       # 8_11, then Q 8-22 from 8_8
        s = round(4·(s·z signed) to the high word) + c            # 8_15, 8_18, 8_17, 8_20 21-22(4), 8_25 21-23
    if h0 != top digit of low(R): s = -s                          # 8_14, 8_19 24-15, 8_23 26-27, 8_27, 8_28
    standardise((s : 0) >> 9, exponent 0)                         # 8_26 22-21(18), 7_17
    ```
    R is the argument in quarter periods, y, with 32 binary places after
    the point; u is y's fraction at 31 b.p. as a signed number in [−1, 1),
    and the result is ±cos(πu/2). The coefficients c13, c7, c5, c3, c1 are
    DL8 m.c. 13, 7, 5, 3, 1 (finding 12). This gives everything measured
    before: 0 at and beyond 2^21 (e >= 22), `0 sin` = −2^−25 (y = −1, u =
    −1, where the polynomial is −2^−25 short of 0), and the loss of figures
    for large x (100000 sin = 0.0357600; the exact value is 0.0357488).

20. **log** (15/3-15/4, p.32, with S06 and M12 on 15/0). **S06** (library
    sheet S6/10013) sums a series by Horner's rule with **M12** (S6/10012)
    for each product: M12(a, b) = high word of 4·a·b + 2^31 (signed, 30
    b.p.), as finding 25 found for the punch. The coefficients are fetched
    from DL11 by a `11-13` built from a count (`6_6 15-25`, `Q 11-13`,
    `6_26 21-25`, `6_30 13-14`); the caller's link at 1_31 picks the sum
    (TS14) or the last product (21_3).
    ```
    M = m << 10                              # 8_0, 8_12 21-22(20): 2f at 30 b.p.
    if M == 0 or M < 0: stop 1 30-21x        # 8_31 21-28, 8_4 21-27; resumed: 0
    t = M - √2                               # 8_7 8-23 (√2 at 30 b.p. = 1518500250, from 8_9)
    if t >= 0: E = e·2^21 - 2^20; u = M12(t, 1/√2)     # 8_20, S06's M12 with link 21-16
    else:      E = (e - 1)·2^21;  u = t + (√2 - 1)     # 8_15, 8_21 8-22
    y = S06(LOG2_SERIES, u)                  # 8_1, 8_6, 8_13: 8 terms + 0, log2(1+u) at 30 b.p.
    R = (y : 0) >> 9;  high(R) += E          # 1_31 30-21, 8_2 22-21(18), 8_27 18-22: log2 x
    if R == 0: the zero word                 # 7_6 21-28(2)
    c = 9; while P31 == P32 of high(R): R = 2R, c -= 1        # 7_2, 7_10 ... 7_22
    Y = high(R) + top digit of low(R)        # 7_21 21-23
    P = Y · ln 2 (0x58b90bfc, 31 b.p.), less ln 2 if high(R) was negative   # 7_18, 7_24, 7_0 14-27, 7_26 16-23
    standardise(P >> 8, exponent c)          # 7_1 22-21(16), 7_12
    ```
    The series is economised (NOTES-12-22: "coefficient table B"), and
    every M12 rounds; that is the 1-2 units low in the last place noted
    above (log 3, log 10).

21. **exp** (15/6-15/7, p.33, with S06 and M12).
    ```
    P = m · log2 e (0xb8aa3b2a, unsigned, 31 b.p.; m signed)  # 8_9, 8_23 0-24, 7_2 14-27, 8_1 16-23
    e' = e, or e - 1 if digits 21 and 20 of high(P) agree    # 7_0, 7_3, 7_14, 7_16 26-27, 7_18 28-26(2)
    if e' >= 9: stop 7 30-21x                                 # 7_13 7-26, 7_17 13-27, 7_20; resumed: 0
    R = P >> (20 - e), exact                                  # 7_19 or not, 7_23 22-21(20), 7_21 ... 8_26, Q
    R = R + 2^32 + 1                                          # 8_31 27-22(2): n + 1, and the fraction's last digit
    N = high(R); v = (low(R) - 2^31) >> 1                     # 8_3 21-20, 7_5 30-21, 7_7 29-22, 7_10 22-16
    s = S06(EXP_SERIES, v), the sum                           # 7_12, 8_15, 8_18; link 1_31 14-21
    standardise((s : 0) >> 10, exponent N)                    # 7_1, 8_4 22-21(20), 8_25, 8_30 13-25(21), 7_22
    ```
    high(P >> (20 − e)) is n = ⌊x log2 e⌋, its low word the fraction f;
    the series (the punch's, DL11 m.c. 21-28) gives 2^(f−1) at 31 b.p.
    The stop covers results too large and too small alike (|x| beyond
    about 354.9); N = 512 (x between 354.2 and 354.9) passes it and stops
    in standardise instead, 3 14-21x.

22. **pow** (15/8-15/10, pp.34-35): exp of b·log2|a|, taken at the double
    register, not through the stored log.
    ```
    if a == 0: store 0                        # 7_1 19-28, 7_5 30-21(2)
    if b == 0: store 1                        # 7_6 20-28
    odd = false
    if a < 0:                                 # 7_10 19-27
        m_a = -m_a (as a whole word)          # 7_14, 7_16 19-26, 7_19 13-19
        if e_b < 0: stop 5 30-21x             # 7_21 20-13, 7_24 13-27; resumed: 0
        if e_b < 22:
            R = (m_b · 2^33) >> (22 - e_b)    # 7_8, 7_11, 7_15, 7_30 ..., 7_17, 7_23: b's whole part in the high word
            if low(R) != 0: stop 5 30-21x     # 5_3 21-28
            odd = bit 0 of high(R)            # 5_6 22-27, 5_12 30-1 / 5_13 29-1 (flag 1_1)
    R = log2 of (m_a, e_a), as finding 20 up to 8_27      # 5_18, 5_22, into 15/4
        (a minus power of two: m_a = +1.0 overflows, the log stop 1 30-21x;
         resumed, the register is 0 and that gives 1)
    if R == 0: store 1                        # 8_11 21-28(2)
    c = e_b + 10; while P31 == P32 of high(R): R = 2R, c -= 1   # 8_5, 8_8, 8_15 ... 8_23
    Y = high(R) + top digit of low(R)         # 8_27 21-23, 8_30 21-15
    P = Y · m_b, signed both ways             # 8_24 20-16, 8_1 0-24, 8_4, 8_18, 8_2 13-23
    exp from 7_0 with P and exponent c (finding 21)   # 8_13 23-14(3), 8_9 23-13, to 15/7
    if odd: R = -R before standardise         # 1_0 1-27, 5_2 21-20(2), 5_26 30-21(2), 5_30 20-23(2)
    ```

23. **rem** (15/1-15/2, p.36) and **fetch integer** (12/6).
    ```
    fetch integer(w) = high((sext(w) << ((e mod 16) + 1)) + 2^31)   # 6_22 16-22, Q 21-22, 6_27 29-23
    b' = fetch integer(b)                      # 7_6, 7_1 21-19
    if 0 - b' >= 0: stop 1 30-16x              # 7_24, 7_26 19-26, 7_29 13-27; resumed: 0
    R = (a's word with P1-10 cleared) · 2, sign-extended: m · 2^11   # 7_2, 8_5, 7_7, 8_10, 7_12
    if e < 0:                                  # 7_8 13-27
        if m >= 0: store a as it is            # 7_11 21-27, 7_16
        R = R >> -e (exact); high(R) += b'     # 7_17, 8_19 ... Q, 8_17 19-22
    else:
        c = e
        loop:                                  # non-restoring division of the high word
            while high(R) >= 0: high(R) -= b'  # 7_10 21-27, 7_13 19-23
            if c == 0: high(R) += b'; break    # 7_14 13-28, 8_17 19-22
            R = 2R; c -= 1; high(R) += b'      # 8_18 21-22(2), 7_21 7-26, 8_23 19-22
    x = standardise(R, exponent 21)            # 7_20 8-13, 7_25
    store 0 if x == b's word, else x           # link 1_30 18-23, 8_1 21-28, 7_15 30-16
    ```
    The count of doublings in fetch integer is built into a `21-22` that
    runs for 2((e mod 32) + 1) m.c.; a transfer lasts at most 32, so it is
    (e mod 16) + 1 (measured over exponents −40 to 59 by
    `fetch_integer_words`). The number is rounded halves up from one half
    to 2^15, the range the manual allows; outside it, .4 is taken as 26 and
    2^15 as 1. b' = 0 needs b = 0 (a positive mantissa is at least a half
    before the shift). The interpreter's indexing (`Num::to_index`) rounds
    the same way inside that range; whether the indexing symbols use this
    routine (and so share its behaviour outside it) has not been checked.

24. **neg and mod** (p.19). neg is 0 − x through the subtract routine:
    whichever exponent is the larger, nothing is shifted (the zero word is
    either shifted right out of the way or, as the larger, skipped by
    `4_12 20-28`), so it is standardise(−m·2^32, e), with −m taken as a
    whole word (−1.0 becomes +1.0, exponent + 1); an unstandardised word
    comes out standardised. mod stores a word whose mantissa is not
    negative unchanged (`21_3-27`), and sends the others to neg.

**Checked word for word** (`tests/function_words.rs`). The argument
words are put straight into the number store (a, b: DL12 m.c. 26, 27) of
a machine stopped at a `wait` at the head of the program, cloned for each
case, and the program resumed. Exponents over the function's range (the
whole −512..511 for √, log, neg and mod), both signs, mantissas at the
ends of the range, the zero word, and one word in 40 any 32 digits at
all; for pow a third of the indices whole numbers, for rem a third of the
divisors; and fixed cases (minus powers of two, zero and fractional
indices, 2^±511, divisors that round to b). Compared: the stored word,
whether it stopped, and the word stored after the operator's single shot.
**All agree**, for each of √, sin, cos, log, exp, neg and mod 60,000 of
60,000, and for pow 60,087 and rem 60,051 (seeds 101-103 of 20,000 plus
the fixed cases); 300 of each run in the normal `cargo test`.
`src/functions.rs` is this, and `machine.rs` uses it for √, log, exp,
pow, sin, cos, rem, neg and mod (not `max`). The one place it departs on
purpose: after √ of a negative number a single shot gives the manual's 0,
not Hamblin's 0.5 (finding 18; DECISIONS.md).

### Punch translate, × and ÷ word for word (findings 25-27)

Traced as finding 15 was, with `trace_punch` in `tests/punch_words.rs`
(any tracks: `TRACKS=13,14,15` for the punch, `TRACKS=12` for the
arithmetic). Trace m.c. are six below the sheets' (finding 16); the m.c.
below are the sheets'. (Findings 18-24 are left to the functions √, log,
exp, pow, sin and cos, traced at the same time.)

25. **Punch translate** (pp.47-48; DL1 = 14/4, DL3 = 14/5, DL4 = 14/6,
    DL5 = 14/7, with S06 and M12 on 15/0). The decimal exponent and the six
    figures are computed from the word in fixed point, never exactly:
    1. `1_11 3_13-14 [P11-32]`, `1_13 25-14`: the mantissa. `1_17 14-27`
       puts its sign in column 1 (`27-11_0` or `27-11_1`), and `1_1 23-13`
       or `1_2 23-26` gives M = |mantissa| at 30 b.p. `1_7 26-21_3`,
       `1_10`…`1_22` sign-extend the exponent, `6_28 27-22_3` adds 1:
       E = e + 1.
    2. `3_0 3_2-16` [log₁₀2, 32 b.p.] = 0x4d104d42; `3_6 0-24` and the sign
       correction `1_9 16-23_3`: the signed product E·log₁₀2. Its top half
       is the decimal exponent D (`3_11 21_3-19_3`), its bottom half the
       fraction F (`3_14 21_2-16`).
    3. `3_16 3_19-21_3` [log₂10, 30 b.p., unsigned] = 0xd49a784c, `1_23
       0-24`: H = top half of F·log₂10, 30 b.p., 0 ≤ H < 3.33. `1_18`,
       `1_24`, `3_27 25-19_2`, `3_30 25-23_3` split off its integer part i
       (P31-32), `5_18 21-22(2)` doubles the double word (so the next digit
       of the bottom half comes up), `3_25 14-22_3` adds −½: x = H − i − ½ at
       31 b.p.
    4. `6_18` S06 with `3_1 [7P17]` sums an 8-term series in x by Horner's
       rule, each product by M12. The coefficients, fetched from DL4, are
       0xb6, 0xe56, 0xf727, 0xdedab, 0xa0c1d0, 0x56f8a65, 0x1f5e4653,
       0x5a82799a (the last is √2/2 at 31 b.p.): S ≈ 2^(x+½)/2.
       **M12** (library sheet S6/10012): the unsigned product of TS14 and
       TS16 with both sign corrections, `21-22` for two doublings, then
       `29-23` (P32 taken from the bottom half, which adds 2³¹ to the double
       word): the top half of 4·a·b + 2³¹, i.e. a·b/2³⁰ rounded, halves up.
    5. `Q31 1_6-16`, `6_12` M12: Y = M12(S, M) at 30 b.p. `4_13 21_3-16`,
       `3_15 16-21_2`, `3_18 30-21_3`: Y in the bottom half, the top half
       clear; `3_21 24-27`/`3_23`/`3_24`, `3_5 14-27`/`4_9` double it 1 + i
       times. Now the double word, as a fraction with the point between the
       halves, is the number ÷ 10^D, approximately: z.
    6. `3_17 21-28(2)`: zero punches the zero card (`4_20`, `5_1`,
       `3_31`). Otherwise `5_4 5_6-22_2` adds the round-off [r.o.] =
       21475 = 5·10⁻⁶·2³² to the bottom half. `4_17 21_3-28`: if the top
       half is still zero (z < 1 − 5·10⁻⁶), `5_20`…`4_29` lower D by 1 and
       `4_26 4_29-23_2` takes [9/10 × r.o.] = 19327 off again, so that
       5·10⁻⁷ is left, and the loop below starts with a ×10.
    7. The six figures (columns 2-7, selected by the mask at `3_9`): the
       top half is a figure (`5_7 21_3-13`, `5_9 13-25(16)`, `5_26`, `Q30
       (11-15)` and `Q30 (26-11)` put the hole in its row), then `5_17
       30-21_3` clears it and `5_22 21-20(2)`, `5_25 20-22(8)`, `5_3
       21-22(2)` multiply the double word by 10. The decimal exponent's
       three figures (columns 18-20) come the same way from (2|D| + 1) ×
       [10⁻², 31 b.p.] (`4_25 27-21_3`, `4_15`/`4_16`, `4_24 0-24`), which
       is exact; its sign goes in column 11.

    As word-level pseudo-code (`number::punch_word` is this, in Rust):

    ```
    punch(w):                        # all arithmetic in 32/64-digit words
        M = |mantissa(w)| · 2^9;  E = exponent(w) + 1
        P = E · 0x4d104d42 (signed);  D = P >> 32;  F = P mod 2^32
        Q = F · 0xd49a784c;  H = Q >> 32;  i = H >> 30
        x = 2(H mod 2^30) + digit 31 of Q − 2^30           # signed, 31 b.p.
        S = 0xb6;  for c in the other seven: S = M12(S, x) + c
        z = M12(S, M) << (1 + i)                            # 64 digits
        if z == 0: the zero card
        z += 21475
        if z < 2^32: D −= 1;  z = (z − 19327) · 10
        figures: z >> 32, then five times z = (z mod 2^32) · 10, z >> 32
        sign of w; D
    M12(a, b) = (4·a·b + 2^31) >> 32
    ```

    So the punch rounds z, not the number. z is within a few units of
    2⁻³² of the true quotient for moderate exponents; the error in the
    constants grows with |E| (at |e| ≈ 500, 9.99997e-151 punches
    1.00000e-150). In a sample of 200,000 random standardised words
    (exponents −300 to 300), 1.7 % punch differently from the exact value
    rounded to six figures, by at most 0.63 of a unit in the sixth figure.
    And nearly every exact half in the seventh figure goes **down**: of the
    1,234,717 that a word holds exactly, 1,234,674 punch the lower six
    figures (747234.5 → 747234, 87147.75 → 87147.7, 292.9375 → 292.937),
    and 43 the upper. Those 43 are numbers of a few binary digits, such as
    2⁻¹⁰ = 0.0009765625 (→ 9.76563e-4), 0.1015625 = 13/128 and 999999.5:
    for the rest z comes out a few units of 2⁻³² below the half.

    **Checked word for word** (`punch_word_for_word` in
    `tests/punch_words.rs`): random words, built in the G-program by exact
    operations (unstandardised ones by `&` and `∨` of two standardised
    words with the same exponent), punched by `(P)`, and all twenty columns
    of the card compared with `Card::of`. Exponents over the whole range
    −512..511, both signs, mantissas at the ends of the range, 20 % exact
    halves in the seventh figure, 20 % unstandardised (P31 = P32, including
    mantissa 0 with any exponent), the zero word, the all-ones word. **All
    160,056 of 160,056 cards agree** (eight seeds of 20,000 plus the fixed
    words); 300 run in the normal `cargo test`.

26. **× of the mantissa 2²¹ − 1, and standardise.** Multiply (p.20, 12/5)
    is plain: `1_24 20-16` and `3_7 30-21` put b's mantissa in TS16 and
    a's in 21_3, `5_28 0-24`, the exponents are added (`4_26 19-13`, `5_31
    20-25`), the sign corrections `5_3 19-27`, `5_11 20-27` (none for
    positive factors), `5_17 21-22(22)` shifts the 64-digit product up 11
    places so that the top half is in units of the last digit, and
    standardise (finding 15, step 6) finishes. The trouble is standardise:
    it adds P1 to the top half *before* the loop that finds the leading
    digit, and the loop tests the digit above the final place (P22 ≠ P23),
    then halves. With R the register in units of the final place:
    - if R (doubled as many times as standardising needs) is in
      [2²¹ − 1, 2²¹ − ½), the P1 carries into P22 and the loop stops one
      doubling early; the halving then rounds to 2²⁰ at the exponent one
      higher. `2097151, 1 ×`: R = 2097151 exactly, stored 2097152.
    - if it is in [−2²¹ − 1, −2²¹ − ½), the P1 turns P22 back to the sign
      and the loop doubles once too often; the mantissa −2²¹ − 1 does not
      fit in 22 digits and is stored as **+2²¹ − 1: the sign is lost.**
      `1277608, 2097152 ÷ 1721204, 2097152 ÷ neg ×` (0.609211 ×
      −0.820734 = −0.50000017) punches 0.5.

    Everywhere else this is round to nearest, halves up. `number::mul_words`
    forms the signed product, shifts it 11 places and calls the same
    `standardise` as `add_words`. The add routine passes the same loop,
    but its low word holds the misplaced digits of finding 15, and −0.5 −
    0.75·2⁻²² comes out as −0.5 there.

27. **÷** (p.20, 12/4-12/5) differs from round to nearest only for
    unstandardised operands. `4_20 20-14`, `4_4 24-16`: TS16 = twice the
    divisor's mantissa, 21_3 = the dividend's, `4_9 1-24`, the zero
    divisor test `3_3 14-28`, the exponents subtracted (`3_9 19-13`,
    `5_1 20-26`). Then `5_15 22-14` (an even m.c., TCB on, so the
    quotient halved with its sign), `4_25 23-21` (halved again, to the top
    half; the quotient stays in the bottom half) and `5_30 22-21` (seven
    places down): the top half is Q/2⁹, in units of the last digit, with
    Q's next seven digits at the top of the bottom half and Q's digits
    7-31 below them. Standardise. For standardised operands that rounds to
    nearest. A dividend with few digits gives a quotient with few: the
    all-ones word (−2⁻²²) ÷ 0.4999998 gives −2⁻²¹ where the nearest is
    −2⁻²¹ − 2⁻⁴². `number::div_words` does this. The divider is the
    simulator's (MACHINE.md §7), which for an out-of-range pair (here only
    an unstandardised divisor such as the all-ones word) gives its
    definition's 32-digit wrap-round, not a measured result.

    **Checked word for word** (`multiply_and_divide_word_for_word` in
    `tests/arith_words.rs`, the operands of finding 15 and every pair of
    twelve fixed words, unstandardised ones among them): **× 122,576 of
    122,576 and ÷ 112,819 of 112,819 agree** (seeds 3, 11 and 12 and the
    default; ÷ by zero left out); 2,144 and about 1,960 run in the normal
    `cargo test`. Random products almost never land in the two ranges of
    finding 26 (none in 200,000 tried on the model); of products within
    2⁻²⁰ of ½, 3 % do, and of those within 2⁻²⁰ of −½ (beyond it), 12 %.

### Numerals and read translate word for word (findings 28-29)

Traced as findings 15 and 25 were, with `trace_numeral` in
`tests/numeral_words.rs` (`TRACKS=13/6,13/7` for numerals,
`TRACKS=14/11,14/12,14/13,14/14,15/0` and `DATA=...` for read). Trace m.c.
are six below the sheets' (finding 16); the m.c. below are the sheets'.

28. **Numerals** (translator, flow diagram p.12; DL7 = 13/6, DL8 = 13/7,
    sheets 8-9). READING.md's reading of p.12 is right, and the "unit added
    to the low word" is the rounding of each quotient.
    1. The digits are accumulated as an integer N in TS13, ignoring the
       point (`7_5 13-16`, `7_7 16-25(9)`, `8_17 25-25`: N = 10N + d; the
       symbol is tested for column 2 or 3 at `8_2`...`8_7` and for the
       point, P21 = column 3, at `7_13 [P21]`, `8_15 25-28`). The point sets
       DS19 to P32 (`7_18 29-19`), and every later numeral symbol halves it
       (`8_9 19-21`, `8_11 22-19`): it counts k, the digits after the
       **last** point (a second point starts the count again; the text form
       refuses it anyway).
    2. At the first symbol that is not a numeral: N = 0 stores the zero word
       (`7_14 13-28`), so `0`, `.`, `.0` and `000` are all the zero word.
       Otherwise T = N in TS14 and e = 31 in TS13 (`8_20 13-14`, `7_16
       [31P17]`), and T is normalised: while P31 of T is 0 (`7_22 24-27`),
       T doubled (`8_24 24-14`), e lowered (`8_26 28-26`).
    3. While the counter shows digits after the point (`7_27 19-14`,
       `7_29 24-28`; `7_6 24-19` steps it): T halved into 21_3 (`8_25
       23-21`; its last digit is lost), divided by 10 at P28 (`7_0
       [10P28]`, `7_9 1-24`), e lowered by 2 (`7_8 28-26(2)`), a unit added
       to the quotient in 21_2 (`8_14 27-22`) and the whole halved into TS14
       (`8_16 22-14`): T = (Q + 1) div 2, the quotient rounded half up.
       TCB off (`7_20 4-24`) and normalise again.
    4. T halved into 21_3 (`8_25`), DS21 shifted down 9 places (`7_31
       22-21`) and P32 subtracted from 21_2 (`7_19 29-23`, which adds 2³¹
       to the double word, as in M12): the high word is T/2¹⁰ rounded half
       up. e is moved to P23 (`8_22 13-25(6)`) and into 21_2 (`8_29
       13-21`), and DS21 shifted up 10 places (`8_0 21-22`): mantissa at
       P11-32, e at P1-10. To the accumulator by `7_25 21-13`.

    As word-level pseudo-code (`number::numeral_word` is this, in Rust):

    ```
    numeral(symbols):                  # 32-digit words throughout
        N = 0;  k = none
        for s in symbols:
            k = k + 1 if k is not none
            if s is a point: k = 0  else: N = 10N + digit(s)
        if N == 0: the zero word
        T = N;  e = 31
        while P31 of T is 0: T = 2T;  e −= 1
        repeat k times:
            Q = floor(2^31 (T div 2) / (10·2^27))         # 1-24, 21_3 ÷ [10P28]
            T = (Q + 1) div 2;  e −= 2
            while P31 of T is 0: T = 2T;  e −= 1
        m = ((T div 2) + 2^8) div 2^9                     # 22-21(9), 29-23
        word = m·2^10 + (e mod 2^10)
    ```

    So each division by 10 rounds to nearest at 30 or 31 digits and the
    last step rounds half up to 22: the result is within a unit of the
    nearest, and is the nearest except in about one numeral in 1,600 with
    digits after the point (8,412 of 13,242,529 random ones; integers,
    6,757,465 of them, are always the nearest). `939.9729` is one: 1925065 × 2⁻¹¹ where the nearest is
    1925064 × 2⁻¹¹; `.43513` and `1.45892` are others. There is **no
    re-standardising** after the last rounding: where T rounds up to 2³¹
    the mantissa carries into the sign digit, and the number is stored as
    −2^e. `4194303` (2²² − 1) is stored as −4194304, `8388607` as
    −8388608, `.999999999` as −1 (6 of the 20 million random numerals). The exponent cannot overflow: with nine digits
    e lies between −29 (`.000000001`) and 30 (`999999999`). Beyond the
    manual's nine digits N wraps round modulo 2³² and the result follows
    the same steps (`12345678901` gives an unstandardised negative word,
    `4294967296` the zero word; `show_numerals` shows these), but the text
    form refuses more than nine digits, as before (decision 13).

    The divider is the simulator's (MACHINE.md §7), which takes the
    dividend from 21_3 alone. PM §6.10 asks for 21_2 clear, and this loop
    leaves the previous quotient (or, the first time, whatever was there) in
    21_2. If the real divider let that into the partial remainder, the last
    digit of a quotient could differ; the documents here do not settle it.

    **Checked word for word** (`numerals_word_for_word`): random numerals
    of one to nine digits, the point anywhere or nowhere (leading, trailing,
    between zeros), zeros and nines favoured, all nines and all zeros, and
    29 fixed ones (`0`, `.`, `.0`, `0.`, `000000000`, `999999999`,
    `.999999999`, `.000000001`, `4194303`, `939.9729`, ...), sixteen to a
    G-program, each stored with a name (`939.9729 (i) ;`) and the word
    compared with `numeral_word`'s. **All 200,145 of 200,145 agree** (five
    seeds of 40,000 plus the fixed ones); 429 run in the normal
    `cargo test`. Numerals of more than nine digits or with two points
    (`1..5`, `1.2.3.4`, `99999999999`) agree too, by `show_numerals`.

29. **Read translate** (pp.44-46; DL1 = 14/11, DL3 = 14/12, DL4 = 14/13,
    DL5 = 14/14, with S06 and M12 on 15/0). The card's six figures D
    (columns 2-7, as an integer: leading zeros are kept, columns 8-10 are
    not read), its exponent X (columns 18-20) and the two signs are
    assembled from the rows first (p.44); the conversion is the punch's
    run backwards, with the same series:
    1. D = 0 stores the zero word, whatever the exponent or signs
       (`1_27 25-28`, `3_28 30-13`).
    2. ±2|X| goes to the top half of DS21 (`5_28 13-22`, or `5_29 13-23`
       when the exponent's sign digit, P11 of the assembled exponent, is
       set: `1_24 [P11-20]`, `4_26 25-28`), and is multiplied by log₂10 at
       30 b.p. (`5_1 [0xd49a784c]`, `1_7 0-24`, sign correction `1_9
       16-23`) and doubled (`3_12 21-22`): the top half is i = floor(X
       log₂10) (kept in DS19, `3_15 21-19`), the bottom half the fraction
       f at 32 b.p. The top half cleared (`3_17 30-21`), P32 added (`1_19
       29-22`) and the double word halved into TS16 (`4_22 22-16`): x =
       (f − ½)/2, signed, 31 b.p.
    3. S06 (`4_28 [7P17]`, `5_0`, `6_18`) sums the punch's series in x
       with M12, from the copy of the coefficients at DL4 m.c. 12-19 (p.46's
       "Coefficients"): S ≈ 2^f / 2 at 31 b.p.
    4. Y = M12(S, ±D·2¹⁰) (`1_31 1-16`, `4_0` → `6_12`). Then M12 again
       with C = 2²⁰·10⁻⁵ at 26 b.p. (`1_5 [0x29f16b12]`, p.45's constant;
       `3_10` → `6_12`), and on return `1_30 29-22` takes M12's round-off
       back: R = 4·Y·C exactly, a double word whose top half is the number
       ÷ 2^(i+5) with ten digits below the last place of the mantissa.
    5. A standardise of its own: e = i + 5 (`5_5 19-13`, `5_7 27-25(5)`),
       2⁹ (half the final place) added to the top half (`3_14 5-22`), and
       while P31 and P32 of the top half agree (`3_19 21-14`, `4_21 24-15`,
       `5_23 26-27`): e lowered (`5_25 27-26`), DS21 doubled (`5_27
       21-22`) and 2⁹ taken off again (`1_6 5-23`). No final halving.
    6. e + 512 is tested for 0..1023 (`5_26 [2⁹]`, `5_22`, `5_24
       23-14(10)`, `3_6 23-28`): outside it, the loop on 4 9-24, "data card
       with exponent too large". So the range is that of the word: 2⁻⁵¹³
       (3.73e-155) to just under 2⁵¹¹ (6.70390e153 reads, 6.70391e153
       stops). Otherwise `4_8` takes 512 off, and the word is the top half
       with P1-10 replaced by e mod 2¹⁰ (`5_19 [P1-10]`, `5_8 25-13`,
       `5_13 25-15`, `5_18 26-25`), stored by `1_0 13-16`.

    As word-level pseudo-code (`number::read_word`):

    ```
    read(card):                        # D six figures, X exponent
        if D == 0: the zero word
        P = 2·(2X)·0xd49a784c                    # signed, 64 digits
        i = P >> 32;  f = P mod 2^32
        x = (f − 2^31) >> 1                      # signed, 31 b.p.
        S = 0xb6;  for c in the other seven: S = M12(S, x) + c
        Y = M12(S, ±D·2^10)
        R = 4·Y·0x29f16b12 + 2^9·2^32;  e = i + 5
        while P31 and P32 of R >> 32 agree:
            R = 2R − 2^9·2^32;  e −= 1
        if e + 512 not in 0..1023: stop 4 9-24
        word = (R >> 32 with P1-10 cleared) + (e mod 2^10)
    ```

    The constants and the series carry their own errors, so the result is
    not the nearest word to D·10^(X−5): **1.5 %** of cards read one unit
    away in the last place (1,190,151 of 79,285,344 with exponents −160 to
    160; never more than one unit). 939.973 reads as 1925065 × 2⁻¹¹, the
    word the numeral 939.9729 makes; the nearest to 939.973 is 1925064.
    And like `standardise` (finding 26), a negative R that rounds to just
    beyond minus a power of two is doubled once too often and **loses its
    sign**: −3.91032e-149, −8.31633e-112 and −6.43445e-86 read as
    positive numbers (11 cards of the 79 million).

    **Checked word for word** (`read_word_for_word`): random cards (six
    figures, leading zeros allowed, 10 % with zero or the extreme figures;
    either sign; exponents −999..999, mostly −20..20 or −160..160) and 24
    fixed ones (zero with exponent ±999 and a minus sign, the range limits,
    the three sign losses), sixteen to a G-program `R (a) ; R (b) ; ...`,
    a card that stops run alone. The stored word, or the stop, compared
    with `read_word`: **all 180,105 of 180,105 agree** (seeds 1, 2, 3, 11,
    12 and 13 of 30,000), about 9 % of them on the stop 4 9-24 both ways;
    324 run in the normal `cargo test`.

    The interpreter now converts numerals with `numeral_word` and data
    cards with `read_word` (before: decimal text → f64 → the nearest word).

### Still open

- ~~The exact rule for + and − (finding 13).~~ Found (finding 15), and the
  interpreter now does it.
- ~~The two observations at the end of finding 15 (× of 2²¹ − 1, punching
  of exact halves).~~ Traced (findings 25-27), and the interpreter now
  punches, multiplies and divides word for word.
- ~~Numeral conversion (the one difference left in `scripts/fuzz_routes.py`:
  939.9729 one unit higher on Hamblin's code).~~ Traced (finding 28), with
  read translate (finding 29), and the interpreter now converts both word
  for word.
