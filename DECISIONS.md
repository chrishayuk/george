# Decisions

Where the manual does not settle how GEORGE behaves, this is what the
reconstruction does and why. Each decision is a candidate for checking
against the flow diagrams and coding sheets (`sources/`), which describe the
penultimate version of GEORGE IA.

## Which source wins

The manual and the coding sheets describe different versions. The manual
is the system as released, GEORGE IA and II with decimal program cards. The
sheets are the version before the last, and they are known to predate some
fixes: the keyword for `wait` on sheet 8 is the value the flow diagram shows
Hamblin correcting by hand (ERRATA.md). So:

- **Where the manual says something specific, the interpreter follows the
  manual**, even when the sheets disagree, because the manual is the better
  evidence of what GEORGE did. Two cases: `max` (the sheets give the wrong
  sign when the second number is larger; the flow diagram carries an
  amendment) and the single shot after `√` of a negative number (the stop
  table says it "resumes giving result zero"; the sheets give 0.5, while the
  other routines the table says the same of, ÷ rem log exp and pow, all
  give 0 on the sheets too, which makes √ look like a slip of this version).
- **Where the manual is silent, the interpreter follows Hamblin's code**,
  word for word, because it is the only evidence: rounding in every
  operation, the punch's conversion, numerals and data cards, the fetch of
  indexing numbers, `0 ÷ 0`, copying from an empty accumulator, sin and cos
  of large arguments, the all-ones word.
- **The sheets exactly, slips and all, are `george deuce george`**: Hamblin's
  translator on the simulated DEUCE is the record of the penultimate
  version, and no rule above changes it.

Two cases sit between the first two rules, and follow the code: the
multiplication that loses its sign just beyond minus a power of two
(0.609211 × -0.820734 punches +0.5), and a numeral that rounds up into the
sign digit (`4194303` is stored as -4194304). The manual promises "about six
decimal digits" and numbers "in the usual way", which these break, but only
in general terms, and both come from the standardising shared by every
operation, which very probably survived into the final version.

## The decisions

1. **Symbol by symbol.** Programs are obeyed one symbol at a time, as
   GEORGE II did. The manual says IA and II "accept the same programs and
   differ only in details of internal operation". Where the two versions
   stop on different indications, both are shown.

2. **Rounding.** **Numerals and data cards are converted word for word as
   Hamblin's translator and read translate convert them**
   (`number::numeral_word`, `number::read_word`; RUNNING-GEORGE.md
   findings 28 and 29). A numeral's digits are an exact integer, normalised
   and divided by 10 once for each digit after the point, each quotient
   rounded half up at 30 or 31 digits, and the result rounded half up to
   22 digits with no re-standardising: so `939.9729` is one unit above the
   nearest (about one numeral in 1,600 with a fractional part is), and
   `4194303` and `.999999999` carry into the sign digit and are stored as
   -4194304 and -1. A data card is scaled by 10^X through the punch's
   series (S06 and M12) and fixed-point constants and standardised with
   its own round-off: 1.5 % of cards read one unit away from the nearest,
   and a few negative values just beyond minus a power of two lose their
   sign (-3.91032e-149 reads as +3.91032e-149). Agreement in the stored
   word: 200,145 of 200,145 random numerals, 180,105 of 180,105 random
   cards, the stop 4 9-24 included (`tests/numeral_words.rs`). (Until
   these were traced, both went through f64 and were rounded to the
   nearest; what is still worked out in f64, the repetition count and the
   I.D. keys, goes to the nearest word as `number::ROUNDING` says.)
   **The functions √ log exp pow sin cos rem neg
   and mod are done word for word as Hamblin's routines do them**
   (`src/functions.rs`, findings 18-24): √ by Newton's iteration with the
   divider, log and exp by the library series S06 with M12's rounded
   products, pow as exp of b times the unstandardised log2 |a|, sin and cos
   by a polynomial in quarter periods at 31 binary places, rem by
   non-restoring division of the whole a by "fetch integer" of b. Agreement
   in the stored word, and in the stop and the word a single shot leaves,
   on 60,000 random arguments per function (including unstandardised and
   arbitrary 32-digit words): all of them (`tests/function_words.rs`). **+ - × ÷ and the punch are done word for word as Hamblin's add routine
   does them** (`number::add_words`). *Settled by tracing the routine on the
   simulator* (sources/deuce/RUNNING-GEORGE.md, finding 15): the operand
   with the smaller exponent is shifted right in DS21 by an instruction
   that starts in an odd minor cycle, which loses its last digit and puts
   its digits 1..k below the last place where 0..k-1 belong; the larger is
   added; standardise adds P1 to the high word and normalises, so the
   double-length sum is rounded half up, and the low word is dropped.
   Operands 24 or more places apart give the larger unchanged, and a zero
   with the larger exponent gives the other operand unshifted. The
   interpreter agrees with Hamblin's code in the stored word for all of
   200,324 random sums and differences (`tests/arith_words.rs`). So
   `1, 2097152 ÷ 1 + 1 - 2097152 ×` punches 0, as it does there, and the
   Simpson programs (Example 3) now punch Hamblin's sixth figures. (This
   was round to nearest for + and - too until the routine was traced.)
   **× and ÷** (`number::mul_words`, `div_words`; findings 26 and 27) end
   in the same standardise, which adds its rounding half *before* finding
   the leading digit. So × rounds to nearest except just below a power of
   two, where `2097151, 1 ×` gives 2097152, and just beyond minus one,
   where the 22-digit mantissa wraps round and the sign is lost:
   0.609211 × -0.820734 gives +0.5. ÷ rounds to nearest for standardised
   operands; an unstandardised dividend gives a quotient with as few digits
   as it has. Agreement in the stored word: 122,576 of 122,576 products,
   112,819 of 112,819 quotients. **The punch** (`number::punch_word`,
   finding 25) does not round the exact value: it finds the decimal
   exponent with log10 2, scales the mantissa by 10^frac from a series
   (library routines S06 and M12) and rounds that approximation by adding
   5·10^-6 (or 5·10^-7) at 32 binary places. Nearly all exact halves in the
   seventh figure therefore go down (747234.5 punches 747234, 87147.75
   87147.7, 292.9375 292.937), a few go up (2^-10 punches 9.76563e-4), and
   1.7 % of random numbers punch a sixth figure other than the nearest.
   The all-ones word punches `-000000000-000000001`, which reads as 0.
   All twenty columns agree on 160,056 of 160,056 random words. (Until
   these were traced, × and ÷ were rounded to nearest and the punch
   rounded the exact value halves up.)

3. **Overflow and underflow.** A result whose exponent would exceed +511
   gives stop 3 14-21x. So does one whose exponent would fall below -512:
   the manual has no stop for underflow, but Hamblin's code gives 3 14-21x
   for `R 4 ÷` with 1e-154 read in, and 7 30-21x for `1000 neg exp`, so
   these do too. (This was zero until the independent review.)

4. **Indexing numbers** (suffixes, skip numbers, ranges of repetition) go
   through Hamblin's "fetch integer" (`functions::fetch_integer`, RUNNING-
   GEORGE.md finding 23), the routine rem uses for its divisor. For
   1/2 <= |x| < 2^15 that is the nearest integer, halves up: `1, 2.5|a` is
   a(3), `1.6 ↑` skips to mark 2, `3.5, 5 rep` runs twice, -1.5 is -1.
   Outside that range its exponent is in effect taken mod 16, so 2^15 is 1
   and `0, .4 rep` runs for a very long time. The manual says only that
   indexing numbers "are assumed to be integers of modulus less than 2^15,
   and use of numbers with fractional parts or outside this range may give
   incorrect results"; this is what the incorrect results are. *Settled by
   running Hamblin's code* (this was rounding down until the tutorial's
   programs were run both ways, then rounding to nearest everywhere until
   the routine was traced).

5. **`a b rep (x)`** takes a and b from the accumulator, sets x to a, and
   enters a link with a count-down of b - a held in P1-16. Each `]` with the
   count above zero adds 1 to x and goes back to the symbol after
   `rep (x)`. At zero the link is removed and the program carries on, with x
   left at b if the body did not change it. *Settled by the flow diagrams, pp. 23, 26-27:* there is no test
   on entry, so the body is always obeyed at least once. When a > b the
   16-digit count wraps round to a repetition of about 65536 - (a - b)
   passes (medium confidence).

6. **Truth.** *Settled by the flow diagrams, pp. 22 and 25:* `=` and `>`
   leave the word with only the sign digit (-1) for truth and zero for
   falsehood, and a conditional skip tests only the sign digit. `~` is the
   complement of every digit of the word, and `&` and `∨` work digit by
   digit. So `~` of falsehood is all ones (-2^-22, punched as 0 as Hamblin's punch does) and
   `~` of truth is just under a half (0.5 when punched). Both still act
   correctly in a skip. The DEUCE sources involved (25 AND, 26 exclusive or,
   29 P32) were worked out from the pages and have since been confirmed
   against the DEUCE Programming Manual (`sources/deuce/MACHINE.md` §5).
   `=` compares the two words digit by digit, as p. 22 has it, which
   differs from comparing values only for the unstandardised words `~`, `&`
   and `∨` make (`0 ~ 0 + 0 ~ =` is false, on Hamblin's code and here).

7. **Block read and punch** (`a b R|(x)`, `i m j n R||(a)` and the punches)
   take their range numbers out of the accumulator.

8. **The end of the symbols** ends the program, as `]` with nothing in the
   link list does. The translator puts an end-of-program keyword at the end
   of every program (Appendix II).

9. **`wait`** shows the number on the output lights, reported on the error
   stream, and goes on as if the operator gave a single shot. **`I`** takes
   the next number given with `--keys`, or zero, which is what clear keys
   would give.

10. **Resumable stops** end the run unless `--single-shot` is given. With it,
    the program carries on as the stop table says a single shot would.

11. **Data cards.** A number given in ordinary notation is punched as the
    standard card would hold it: six significant figures, the rest dropped,
    because "figures in cols. 8-10 and 12-17 are in any case treated as zero
    by the reader". With `--columns` the twenty card columns are read
    directly.

12. **The text form.** The manual's printed notation, with ASCII spellings
    for symbols not on a keyboard. `R|`, `R||`, `P|` and `P||` must be
    written without a space; `R |a` is the read symbol followed by a bar
    sign. A run of letters that is not a word of the code is a sequence of
    variables (the manual writes `ab+`). `#` starts a comment.

13. **Double suffixes** are refused on letters other than a, b, c and d,
    the only letters the manual allows them on. More than nine digits in a
    numeral is refused too. In both cases the manual does not say what the
    translator did.

Settled by running Hamblin's own code. The transcribed coding sheets run on
the DEUCE simulator (`george deuce george FILE`, `sources/deuce/RUNNING-GEORGE.md`),
so a question can be answered by running a G-program both ways:

- **`rem` rounds its divisor** to a whole number, halves up, and then tests
  its sign: `7, 3.6 rem` gives 3 (divisor 4), `7, 4.5 rem` gives 2 (divisor
  5). The routine ("fetch integer", 12/6) shifts by the exponent mod 16, so
  this holds only from one half up to 2^15: `7, .4 rem` gives 7 because .4
  is taken as 26 (finding 23; this was "divisor 0, no stop" until rem was
  traced). The manual's note says only that n and b are integers. Adopted,
  word for word.
- **`dup` after `;`** duplicates the number below, not the one cancelled:
  `5, 9 ; dup` gives 5, 5 on both. The TS16 reading of the flow diagrams was
  wrong, as the decision not to adopt it assumed.
- **`=` and `>`** agree on every case tried; `>` is strict, and `=` compares
  whole words (decision 6).
- **`rep`**: `5, 5 rep` runs once and `3, 1 rep` wraps to 65535 passes on
  both, as decision 5 says.
- **`~`** gives the same word on both. Hamblin's punch prints the all-ones
  word (`~` of falsehood, -2^-22) as 0, and every other word the logical
  operators make, standardised or not, as its value (`5 ~`, `5, 3 &`,
  `0 ~ 2 ×` and others agree). The punch here now does the same, word for
  word (decision 2): the all-ones word's one digit, scaled by 10, is below
  the sixth figure, so its card is minus, 000000, exponent -1.
- **Copying from an empty accumulator gives 0, with no stop**: `(P)`, a
  name and `dup` read the cell below the first, which holds 0. Only taking a
  number away stops on 1 30-21x, which is what the stop table says: "has
  instructed taking number from empty accumulator". Adopted. Found by running
  random programs both ways.
- **`0 ÷ 0` is 0, with no stop.** Flow diagram p. 20 sends a zero dividend
  straight to standardise before the divisor is looked at, and Hamblin's
  code does that; `1 ÷ 0` still stops on 1 4-24x. Adopted. Found by running
  random programs both ways.
- **sin and cos of large arguments** are 0 on Hamblin's code outside
  -2^21 <= x < 2^21, the range of a 22-digit signed fixed-point word: 2097151
  sin is -0.320808 there and 2097152 sin is 0, while -2097152 sin is not 0.
  Below that, his routine's reduction to quarter periods loses figures as x
  grows (100000 sin: 0.0357600 there; the exact value is 0.0357488). Found
  by running random programs both ways (`scripts/fuzz_routes.py`). Both
  now come out of the word-for-word routine (finding 19), as does `0 sin`
  = -2^-25 and sin x = x for |x| < 2^-11.
- **`pow` of minus a power of two** (-1, -2, -4, -0.5) stops on 1 30-21x,
  the log stop, on Hamblin's code, and now here too (finding 22): |a| is
  made by negating the mantissa, and -1.0 negated does not fit. Resumed, it
  gives 1. Other negative bases work.
- **√ of a negative number, resumed by a single shot**, gives 0.5 on
  Hamblin's code (TS16 still holds the first approximation) and 0 here, as
  the manual's stop table says ("resumes giving result zero"). The model
  in `functions::sqrt` gives Hamblin's 0.5; `machine.rs` keeps the
  manual's 0, by the rule in "Which source wins" above. (`max` differs
  too, for the same reason: see ERRATA.md.)

Still open:

- ~~Two last-figure differences found while settling + and - (finding 15):
  `2097151, 1 ×` and exact halves at the punch.~~ Traced (findings 25-27,
  decision 2), and the interpreter now does both as Hamblin's code does.
- ~~Numeral conversion: `939.9729` one unit higher in the last binary
  place on Hamblin's code, the last difference `scripts/fuzz_routes.py`
  found.~~ Traced (findings 28-29, decision 2), with read translate, and
  the interpreter now converts numerals and data cards word for word.
- The flow diagrams and coding sheets are GEORGE IA, the penultimate
  version. Nothing describes GEORGE II. Margin notes mention variants "1B"
  and "1C".

Not decisions, but easy to get wrong:

- Two numerals in a row run together into one number. `3, 4 14 ↓` calls
  subroutine 414 mod 32 = 30, not 14. The manual says numbers written
  consecutively "should be separated by a comma".
- Subroutine numbers are marks. `* 6` at the head of a subroutine and
  `* 6` as a place marker are the same thing, in one mark list of 32;
  `↓` skips to a mark and links back, and `↑` skips without the link.
- A skip is conditional when `=` or `>` has been obeyed since the previous
  skip, whether or not that skip was taken. A conditional skip takes both the
  mark number and the truth value out of the accumulator.
