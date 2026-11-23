# 11. When things go wrong

## Stops

When a GEORGE program failed, DEUCE stopped, and the lights on the
console showed why. The manual lists what each pattern meant:

> The following is a virtually complete list of the "stops" that can occur
> in the course of a program. Note that in the case of purely-arithmetical
> failures the program can usually be resumed by giving a single-shot,
> though of course incorrect answers will be obtained. (Section 2.4)

`george stops` prints the ones a program can cause:

```console
$ george stops
5 9-24                       more than 512 symbols in program
3 2-2x (IA), 0 2-2x (II)     mark number used twice; S.S. second entry written over first in mark list
1 30-21x (IA), 1 1-1x (II)   program uses too many accumulator cells or has instructed taking number from empty accumulator; S.S. (II) calls in next symbol
1 8-21x (IA), 1 3-3x (II)    too many entries in link list (nested subroutines or repeats to higher than sixth order); S.S. (II) calls in next symbol
7 25-29                      program calls for skip to non-existent mark
3 14-21x                     result of arithmetic operation exceeds permissible magnitude; S.S. resumes with exponent halved
1 4-24x                      divisor zero (÷) or argument negative (√); S.S. resumes giving result zero
1 30-16x                     divisor negative or zero (rem); S.S. resumes giving result zero
1 30-21x                     argument zero or negative (log); S.S. resumes giving result zero
7 30-21x                     result exceeds permissible magnitude (exp or pow); S.S. resumes giving result zero
5 30-21x                     fractional power of negative number (pow); S.S. resumes giving result zero
4 9-24                       data card with exponent too large
1 11-6x, 1 7-16x             data card wanted
```

The indication is what the operator looked up. `1 30-21x` has two
meanings: it is the stop for the logarithm of a number not positive, and
on GEORGE IA it is also the stop for a full or empty accumulator. The list
leaves out the stops that belong to the machine and
its cards rather than to the program: the initial stop, a reader fault,
the end of the program.

When `george` stops, it says which symbol it was obeying, and where that
symbol is in the file. Its exit status is 1.

## The single shot

The *single-shot* key on DEUCE's console let a stopped machine go on. At
a resumable stop it was how the operator told GEORGE to carry on
regardless. `--single-shot` does this at every resumable stop, and
reports each one:

```george
1, 0 ÷ (P) ;               # divisor zero
2 neg √ (P) ;              # argument negative
1 neg log (P) ;            # argument negative
1, 0 rem (P) ;             # divisor zero
1000 exp (P) ;             # result too large
2 neg, .5 pow (P) ;        # fractional power of a negative number
0, 0 ÷ (P) ;               # zero divided by zero: a stop too
```
```console
$ george run programs/11-single-shot.george --single-shot
stop 1 4-24x: divisor zero (÷) or argument negative (√)
  at symbol 3 "÷", line 1, column 6 (single shot: resumes giving result zero)
0
stop 1 4-24x: divisor zero (÷) or argument negative (√)
  at symbol 8 "√", line 2, column 7 (single shot: resumes giving result zero)
0
stop 1 30-21x: argument zero or negative (log)
  at symbol 13 "log", line 3, column 7 (single shot: resumes giving result zero)
0
stop 1 30-16x: divisor negative or zero (rem)
  at symbol 19 "rem", line 4, column 6 (single shot: resumes giving result zero)
0
stop 7 30-21x: result exceeds permissible magnitude (exp or pow)
  at symbol 26 "exp", line 5, column 6 (single shot: resumes giving result zero)
0
stop 5 30-21x: fractional power of negative number (pow)
  at symbol 34 "pow", line 6, column 11 (single shot: resumes giving result zero)
0
stop 1 4-24x: divisor zero (÷) or argument negative (√)
  at symbol 40 "÷", line 7, column 6 (single shot: resumes giving result zero)
0
```

Each function that failed gave zero, as the table says. `0, 0 ÷` stops
like any other zero divisor, because the table says "divisor zero" and
nothing more. Hamblin's own code, run on the DEUCE simulator, does not:
its divide sends a zero dividend straight on and gives 0 without looking
at the divisor. Where the manual is specific, `george` follows the manual
(DECISIONS.md, "Which source wins"). Overflow resumes
differently, "with exponent halved":

```george
10, 100 pow (P)            # 10 to the 100th: in range
dup × (P) ;                # 10 to the 200th: out of range
```
```console
$ george run programs/11-overflow.george
1e100
stop 3 14-21x: result of arithmetic operation exceeds permissible magnitude
  at symbol 9 "×", line 2, column 5
```

```george
# The same, carried on with a single shot.
10, 100 pow (P)
dup × (P) ;
```
```console
$ george run programs/11-overflow-ss.george --single-shot
1e100
stop 3 14-21x: result of arithmetic operation exceeds permissible magnitude
  at symbol 9 "×", line 3, column 5 (single shot: resumes with exponent halved)
5.71494e99
```

As GEORGE holds it, 10^200 is 0.653 × 2^665. Halving the exponent, 665 to
332, gives 0.653 × 2^332, which is the 5.71494 × 10^99 punched. It is
not a useful number, but the program goes on.

A mark used twice is resumable too. The second entry is "written over
first in mark list", so the later mark wins:

```george
1 ↑ ]
* 1 1 (P) ; ]
* 1 2 (P) ; ]
```
```console
$ george run programs/11-mark-twice.george
stop 3 2-2x (IA), 0 2-2x (II): mark number used twice (mark 1)
  at symbol 9 "*", line 3, column 1
```

```george
# The same, with a single shot at the stop.
1 ↑ ]
* 1 1 (P) ; ]
* 1 2 (P) ; ]
```
```console
$ george run programs/11-mark-twice-ss.george --single-shot
stop 3 2-2x (IA), 0 2-2x (II): mark number used twice (mark 1)
  at symbol 9 "*", line 4, column 1 (single shot: second entry written over first in mark list)
2
```

GEORGE noted the marks before it obeyed anything, so this stop comes
before the program starts. The `1 ↑` never ran without the single shot.

Two more stops were final on GEORGE IA but not on GEORGE II: the
accumulator (`1 1-1x` on II) and the link list (`1 3-3x`). For both, the
manual gives II the action "S.S. calls in next symbol": the symbol that
failed is left undone and the program goes on from the next. `--single-shot`
does the same. Here the thirteenth number never reaches the accumulator,
and the twelve that did are added up:

```george
# Thirteen numbers, with a single shot at the thirteenth.
1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 9
+ + + + + + + + + + + (P) ;
```
```console
$ george run programs/11-accumulator-ss.george --single-shot
stop 1 30-21x (IA), 1 1-1x (II): program uses too many accumulator cells or has instructed taking number from empty accumulator (all 12 cells occupied)
  at symbol 27 "9", line 2, column 40 (single shot: (II) calls in next symbol)
78
```

The rest of the stops cannot be got past by `--single-shot`.

A skip to a mark that is not there is final. "Mark number × P17 appears
on O.S." — the operator could read the number off the output
staticiser:

```george
R ↑                        # skip to the mark numbered on the card
* 1 1 (P) ; ]
* 2 2 (P) ; ]
```
```console
$ george run programs/11-no-mark.george --data programs/11-no-mark.in
stop 7 25-29: program calls for skip to non-existent mark (mark number 3)
  at symbol 1 "↑", line 1, column 3
```

## wait

> The symbol "wait" performs no operation, but holds up the calculation
> until a manual signal (single shot) is given, incidentally displaying on
> the output lights the number in the last occupied cell of the
> accumulator (or the special indication P1-10 if the accumulator is
> empty). (Section 1.9 (vii))

On the machine this was stop `1 16-29x`. The operator could read the
number in binary from the lights, or "call and cancel punch [...] to
obtain a copy on a card". `george` reports the number on the error stream
and goes on, as if the single shot had been given at once. Here `wait`
shows each step of Newton's method:

```george
2 (y) ;
1, 1, 3 rep (a) dup y rev ÷ + 2 ÷ wait ]
(P) ;
```
```console
$ george run programs/11-wait.george
wait: the output lights show 1.5
wait: the output lights show 1.41667
wait: the output lights show 1.41422
1.41422
```

## The input keys: I

> Symbol "I" operates to transfer a number set in on the input keys to the
> first vacant cell of the accumulator. This permits manual control of a
> program by means of discrimination: e.g. for "If the input lights are
> clear, skip to mark no. 4" write "I 0 = 4 ↑". It is generally advisable
> that the program should have "wait" at some preceding point to permit
> the input lights to be set as required. (Section 1.9 (viii))

The input keys were a row of keys on the console, one for each of the 32
digits of a word. `--keys` gives the numbers set on them, one for each
`I` obeyed. When the list runs out,
the keys are clear, and `I` gives zero.

```george
# Three times: if the input keys are clear, skip to mark 4.
1, 3 rep (i)
wait I 0 = 4 ↑
1 (P) ; 5 ↑
* 4 2 (P) ;
* 5 ]
```
```console
$ george run programs/11-keys.george --keys 7,0
wait: the output lights show the empty-accumulator indication
1
wait: the output lights show the empty-accumulator indication
2
wait: the output lights show the empty-accumulator indication
2
```

The first time round the keys held 7, so the program did not skip; the
second time 0; the third time nothing was given, which is zero too.

On the machine the number was a binary word set by hand, and an operator
would set a pattern of switches rather than a decimal number. `--keys`
takes ordinary numbers. On GEORGE II, `I` itself stopped twice, on
`2 0-16x` to let the keys be set and on `1 4-15x` to let them be cleared,
so the `wait` was not needed there.

## Following a program: --trace

GEORGE II was the version for testing programs, and section 2.7,
"Facilities for manual control in GEORGE II", describes what it offered.
Every symbol's code was shown on the output staticiser as it was obeyed,
"this permits identification of any symbol leading to failure". With
switches on the input keys the operator could also have:

- **Stop**: the machine stops before every symbol, and a single shot
  obeys it.
- **Request stop**: the machine runs until the symbol about to be obeyed
  has a given code.
- **Slow motion**: "each symbol is 'held' for about [a fraction of a]
  second before being obeyed. The code of the symbol is displayed on the
  O.S. during this period."
- **Manual insertion**: at a test stop, symbols set on the input keys are
  obeyed one by one, as if they were in the program. "Manual insertion of
  symbols permits punching out of numbers from store or the accumulator;
  display of number from the accumulator on the O.S. (using 'wait');
  skipping to marked positions, etc.; as well as temporary alteration of
  the course of a faulty program pending re-punching."
- **Skipping** symbols of the stored program.
- **Final stop**, at the end of the program, where more symbols could be
  inserted, "e.g. for additional punching out of results".
- **Restart**: code 15/15 given by hand starts the program again, without
  clearing the store.
- **Looking in**: "The accumulator is in D.L.10, m.c. 1-12", and could be
  watched on a monitor. Unused cells were cleared at each test stop
  "to facilitate inspection".

`--trace` is the nearest thing `george` has. It prints each symbol as it
is obeyed, and the accumulator after it, as a programmer at the console in
slow motion would have watched it. Here is the manual's subroutine 6 at
work:

```george
R R 6 ↓ (P) ;
]
* 6 dup × rev dup × + .5 pow ]
```
```console
$ george run programs/11-trace.george --data programs/11-trace.in --trace
   0  R        3
   1  R        3  4
   2  6        3  4  6
   3  ↓        3  4
   9  dup      3  4  4
  10  ×        3  16
  11  rev      16  3
  12  dup      16  3  3
  13  ×        16  9
  14  +        25
  15  .5       25  0.5
  17  pow      5
  18  ]        5
5
   4  (P)      5
   5  ;
```

The left column is the symbol's number in the program, counting from 0.
After `↓` the trace jumps to 9, the first symbol after `* 6`, and after `]`
it comes back to 4. The punched 5 appears in its place, just before the
trace line for the `(P)` that punched it.

One detail differs from the machine. On GEORGE II, "numeral symbols do not
lead to the insertion of a number in the accumulator until after the first
following non-numeral symbol has been called in". The trace shows a
numeral's number in the accumulator at once, against the numeral's first
symbol.

GEORGE IA had a coarser facility: with a switch set during translation,
every keyword stopped, and "the calculation [could] be taken step-by-step
with the single-shot key" (section 2.6). There was also a diagnostic card
which, after a failure, punched five cards showing the link list, the
accumulator and the current track of the program (section 2.5).

## A program that does not stop

A program that goes round for ever is not a stop. `george` gives up after
ten million symbols, or after the number given with `--limit`:

```george
* 0 0 ↑
```
```console
$ george run programs/11-endless.george --limit 1000
george: gave up after 1000 symbols obeyed (--limit)
```

The operator of DEUCE had no such help. They watched the lights and
decided for themselves.

Next: [Worked examples](12-worked-examples.md)
