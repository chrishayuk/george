# george

GEORGE, the General Order Generator, that Charles Hamblin wrote for the
English Electric DEUCE at the New South Wales University of Technology in
1957-58. It was rebuilt in Rust with no dependencies from his programming
manual, and it runs the manual's programs.

```
$ cat examples/subroutine.george
R R 6 ↓ (P) ;                       # main program
]                                   # end of main program
* 6 dup × rev dup × + .5 pow ]      # subroutine
$ george run examples/subroutine.george --input 3,4
5
```

GEORGE was programmed in "reverse Polish" notation, with no addresses, on a
"pseudo-machine" that the basic program made out of DEUCE. It had a running
accumulator of twelve cells used last-in-first-out, a link list of six for
subroutines and repetitions, and a store of 32 plain variables, 1024
singly-suffixed ones and 4096 doubly-suffixed ones. Numbers were floating
point with a 22-digit binary mantissa. This was more than ten years before
Forth.

## Build and run

```
cargo build --release
target/release/george run examples/squares.george
target/release/george run examples/mean.george --data examples/mean.in
cargo test
```

```
george run FILE [--data FILE] [--columns] [--input N,N,...] [--keys N,N,...]
               [--cards] [--trace] [--single-shot] [--limit N]
george deck FILE        the program as punched: symbol codes, sixteen to a card
george format FILE      the program in the manual's printed form
george table            the coding table (Appendix I)
george stops            the stops (section 2.4)
```

`run` prints each number punched, to the six figures the punch gave it.
`--cards` prints the twenty columns of the card instead (sign, mantissa
d.ddddd, sign, exponent). `--trace` shows each symbol obeyed and the
accumulator after it, which is roughly what GEORGE II's "slow motion" put on
the output lights:

```
$ george run examples/subroutine.george --input 3,4 --trace
   0  R        3
   1  R        3  4
   2  6        3  4  6
   3  ↓        3  4
   9  dup      3  4  4
  10  ×        3  16
  ...
```

When a program fails it stops as the machine did, with the indication from
the manual's table of stops:

```
$ george run examples/pairs.george --input 1,2,3,4
3
7
stop 1 11-6x, 1 7-16x: data card wanted
  at symbol 2 "R", line 4, column 5
```

## Editors and the playground

`lsp/` is a language server, `george-lsp`, in a workspace crate of its own so
that `george` keeps no dependencies. It reports the reader's mistakes as you
type and warns about numerals that run together, skips to missing marks and
marks set twice. Hover over a symbol to see its code in the coding table,
whether it is monadic or diadic, what it does to the accumulator, and the
manual's words. `editors/vscode` is a VS Code extension built on it, with
commands to run a program, trace it, and show its deck, the coding table and
the stops. See [editors/README.md](editors/README.md) for Neovim, Helix and
Emacs.

`web/` is a playground: the same interpreter compiled to WebAssembly, with the
examples, punched cards shown with their twenty columns, and a trace that
steps through the twelve-cell accumulator a symbol at a time.

```
cargo build --release -p george-lsp
node web/build.mjs && node web/test.mjs     # needs: rustup target add wasm32-wasip1
cd web && python3 -m http.server            # then open http://localhost:8000
cd editors/vscode && npm install && npm run package
```

## The tutorial

[`tutorial/`](tutorial) has fifteen chapters, from reverse Polish and the
running accumulator to the cards and codes, with every program in it run by
`python3 tutorial/check_tutorial.py`.

## The code

Programs are written as the manual prints them. Each symbol is one entry
in the coding table:

| | |
|---|---|
| `a` ... `z` (no `o`), `θ α β γ λ μ ν` | variables: put the value in the next cell |
| `(a)` | name: store the last cell in a, leaving it there |
| `;` | cancel the last cell |
| `+ - × ÷ max pow rem` | diadic operators |
| `neg mod log exp √ sin cos` | monadic operators |
| `dup rev` | duplicate the last cell; interchange the last two |
| `j\|a`, `i j\|\|a`, `j\|(a)` | suffixed variables and names |
| `R`, `(P)` | read a card; punch the last cell |
| `a b R\|(x)`, `i m j n R\|\|(a)`, `P\|`, `P\|\|` | read or punch a vector or matrix |
| `a b rep (x) ... ]` | repeat for x from a to b |
| `* n` | mark n, 0 to 31 |
| `n ↑`, `n ↓` | skip to mark n; skip to subroutine n |
| `]` | end of repetition, subroutine or program |
| `= >`, `~ & ∨` | relations and logic: a skip after one is conditional |
| `wait`, `I` | stop showing the last cell; take the input keys |

ASCII spellings: `mul` ×, `/` ÷, `down` ↓, `up` ↑, `or` ∨, `not` ~, `and` &,
`sqrt` √, `alpha` and the other Greek names. `#` starts a comment.

## Examples

`examples/` has 17 programs. All but three are from the manual and are as
printed, apart from data and the punching of results where the manual
leaves them in store. Each says what is ours. They include squares, sums of
squares, factorials, Newton's square root (by repetition and by
convergence), the mean and root mean square of twenty numbers, a
matrix-vector product and quadratic form, Simpson's rule, subroutines that
call subroutines, the endless reading loop that ends when the cards run
out, and Knuth and Trabb Pardo's TPK (`tpk.george`), whose 79.7062 for
i = 4 is one more in the last figure than the value rounds to, from
Hamblin's add routine. Each has an `.out` file and, where it reads cards, an `.in` file.

## What running it found

- **Example 3 of the manual is out by a factor of four.** Its Simpson
  coefficients are 1/48, 4/48, 2/48 ... for a step of 1/4, where Simpson's
  rule needs 1/12, 4/12, 2/12 .... Run as printed it punches a quarter of
  each integral. See [ERRATA.md](ERRATA.md).
- Numerals written next to each other run together. `3, 4 14 ↓` calls
  subroutine 414 mod 32 = 30. The manual says to separate them with a comma,
  and this reconstruction does what the machine would.

## Hamblin's own GEORGE, running

There are two GEORGEs here. `george run` is an interpreter written from the
manual. `george deuce george` is Hamblin's own GEORGE IA: the 22 coding
sheets, transcribed minor cycle by minor cycle, loaded as a deck of 193
cards into a simulated DEUCE, which then translates and obeys the G-program
the way the machine did.

The two describe different versions: the manual the system as released, the
sheets the version before the last. Where the manual says something
specific, `george run` follows it; where it is silent, `george run` does
what Hamblin's code does, word for word; `george deuce george` runs the
sheets as transcribed, with the four corrections and one reconstruction
listed in [ERRATA.md](ERRATA.md) applied at run time (`GEORGE_AS_WRITTEN=1`
turns them off). See "Which source wins" in [DECISIONS.md](DECISIONS.md)
for the cases that fall under the first rule.

"Word for word" means agreement with the simulated sheets. The simulator is
built from English Electric's Programming Manual and Logical Design Manual,
and much of what the arithmetic depends on is documented there (the
multiplier step by step, DS21's carries, the shifts, including the odd-first
shift that loses one digit and doubles another, Logical Design Manual
p. 58). Some is chosen or inferred: the divider gives its result from its
definition, not step by step; the reader's and punch's timings are taken
from the middle of the manuals' ranges; and one class of P31 digits rests
on the program running. So the agreement shows the interpreter is faithful
to Hamblin's code as reconstructed here, under those assumptions, not
that it matches a DEUCE that no longer exists.

```
$ george deuce george examples/factorials.george
1
2
6
...
1.30767e12
end: 0 0-0x with the reader called (end of program)
```

The operator's part is played too: a `wait` stop (1 16-29x) is reported as
"wait: the output lights show N" and given a single shot, `--keys N,N,...`
sets the input keys for each `I`, and `--single-shot` resumes the stops
the manual calls resumable.

All 16 example programs punch exactly the same numbers both ways. Example
3 (Simpson's rule, as printed and corrected) did not, in the sixth figure,
until the interpreter's + and - were made to work word for word as
Hamblin's add routine does (below). Small
programs aimed at each symbol settled several questions the manual leaves
open (see [DECISIONS.md](DECISIONS.md)), and found behaviour of this version
that the manual does not describe.

Beyond the examples, `scripts/fuzz_routes.py` writes random G-programs
(arithmetic, functions, suffixes, repetitions, conditional skips, logic and
subroutines), runs each both ways, and compares everything punched, every
wait and resumed stop, and the stop each run ends on, by its indication. Of
5,000 (seed 2027), 4,993 are the same in every respect, and the other 7 are
all `0 ÷ 0`, where the interpreter stops as the manual says and Hamblin's
code does not; with `max` in the mix, `max` accounts for the rest.
(Before the arithmetic, the punch, the functions and the numerals were
traced and done word for word, the comparison was looser and the numbers
worse: with + and - rounded to nearest, 1,833 of 2,000 the same and 140
differing in the sixth figure.)
Running these and the tutorial's 100 programs both ways found four behaviours the manual does not describe, which the
interpreter now copies from Hamblin's code: copying from an
empty accumulator gives 0, indexing numbers go through the same
"fetch integer" as `rem` (nearest, halves up, within the manual's range),
sin and cos are 0 beyond ±2^21, and the all-ones word punches as 0 (since traced to
the punch routine, finding 25). A fifth, `0 ÷ 0` giving 0 with no stop, was
copied too until it was seen that the manual's stop table does describe it
("divisor zero (÷)"): the interpreter now stops there, as the manual says
([DECISIONS.md](DECISIONS.md), "Which source wins").

The three documents, all on David Green's DEUCE archive
([sources/README.md](sources/README.md)):

| | | |
|---|---|---|
| Programming and operation manual | 23 pp. | read: the interpreter |
| Flow diagrams | 52 pp., handwritten (DEUCE instruction listings) | indexed: [READING.md](sources/flowcharts-1958/READING.md) |
| Coding sheets | 22 pp., handwritten DEUCE code | transcribed, about 1,830 rows, 20 marked doubtful: [transcription](sources/coding-sheets-1958/transcription) |

The archive says the flow diagrams and coding sheets are "the only extant
copy" and "correspond to the penultimate version of GEORGE IA". What it took
to run them is in [RUNNING-GEORGE.md](sources/deuce/RUNNING-GEORGE.md): how
this version took its program cards (four 8-bit codes to a row, not the
manual's decimal pairs), the DEUCE library routines S06 and M12 that the
sheets only name, a digit the sheets cannot show, and four places where a
sheet as written contradicts the flow diagram, one of them a keyword the flow diagram shows Hamblin amending by hand ([ERRATA.md](ERRATA.md)). The
DEUCE simulator, built from the English Electric programming manual and
course notes, is described in [MACHINE.md](sources/deuce/MACHINE.md).

sin and cos needed more than the programming manuals give. Hamblin's
routine reads the multiplier's double store part-way through a
multiplication, to find the term that makes the unsigned product signed.
What that store holds at each minor cycle is described in the DEUCE
Logical Design Manual (chapter 10), and with the multiplier simulated digit
by digit from it, sin and cos agree with `george run`.

The functions were then traced the same way (findings 18-24) and are now
done word for word in `george run` too (`src/functions.rs`): √ by Newton's
iteration with the divider, log and exp by the library series routine S06
with its rounded products, pow as exp of b times log2 |a| taken from the
double register, sin and cos by a polynomial in quarter periods, rem by
non-restoring division by "fetch integer" of b, neg and mod through
subtract. So `0 sin` is -2^-25 both ways, `2 neg, 3 pow` stops on 1 30-21x
both ways, and log is a unit or two low in the last place where Hamblin's
is. The interpreter agrees with Hamblin's code in the stored word, the stop
and the word a single shot leaves, on 60,000 random arguments to each
function.

Addition and subtraction do not round to nearest on Hamblin's code, and
for a while no rule fitted the last binary digits they leave. Traced
instruction by instruction on the simulator, the add routine shifts the
smaller operand with an instruction that starts in an odd minor cycle,
which drops its last digit and puts the rest one place out below the
mantissa; standardise then rounds the double-length sum half up. The
interpreter now does the same word for word, and agrees with Hamblin's
code in the stored word for every one of 200,324 random sums and
differences ([RUNNING-GEORGE.md](sources/deuce/RUNNING-GEORGE.md), finding
15). That removed Example 3's sixth-figure differences.

Multiplication, division and the punch's conversion to decimal were traced
the same way (findings 25-27) and are now done word for word too. Standardise
adds its rounding half before it looks for the leading digit, so × is
rounded to nearest except just below a power of two (`2097151, 1 ×` gives
2097152) and just beyond minus one, where the sign is lost (0.609211 ×
-0.820734 gives +0.5). The punch never rounds the exact value: it scales the
mantissa by a power of ten made from log10 2 and a series (the library
routines S06 and M12), and rounds that. So 747234.5 punches 747234, and of
the 1,234,717 exact halves in the seventh figure a word can hold, all but
43 go down. The interpreter agrees with Hamblin's code on 122,576 random
products, 112,819 quotients and all twenty columns of 160,056 punched cards.

## Sources

C. L. Hamblin, *GEORGE IA and II: A semi-translation programming scheme for
DEUCE. Programming and Operation Manual*, University of New South Wales,
c. 1958. Scans supplied by Robin Vowels to David Green's DEUCE archive.
