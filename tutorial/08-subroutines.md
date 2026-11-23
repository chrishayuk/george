# 8. Subroutines

## Numbered subroutines

> Subroutines written in G-code may be simply attached to the end of a
> program. Each subroutine is introduced by the symbol "*" followed by a
> number, and terminated by the bracket-symbol "]". A subroutine may be
> called in in the main program by giving its number, followed by the
> symbol "↓". When the subroutine is complete, the main program will
> automatically be resumed from the point of interruption. (Section 1.7)

The manual's example is a subroutine, number 6, that takes two numbers
from the accumulator and leaves the square root of the sum of their
squares:

```text
* 6 dup × rev dup × + .5 pow ]
```

Follow it with 3 and 4 in the last two cells: `dup ×` makes 3, 16; `rev`
makes 16, 3; `dup × +` makes 25; `.5 pow` makes 5. It takes two numbers
and leaves one, so

> "6 ↓" can be considered here simply as a diadic operator. (Section 1.7)

The subroutine works on whatever is in the accumulator when it is called.
There are no parameters to declare and no result to return: what it
leaves is the result.

The manual's complete program reads two numbers and punches the answer:

```george
R R 6 ↓ (P) ;                       # main program
]                                   # end of main program
* 6 dup × rev dup × + .5 pow ]      # subroutine
```
```console
$ george run programs/08-hypot.george --data programs/08-hypot.in
5
```

## The bracket's three uses

The `]` on the second line matters.

> When subroutines are attached to a program, the program itself must be
> terminated by a bracket-symbol "]" so that it will not carry on into
> the subroutines. The bracket-symbol in fact has three uses: (i) to
> return a subroutine to the main program; (ii) to restart or terminate a
> repetitive operation; and (iii) to terminate a program. It fulfils the
> third function provided no subroutine or repetitive operation is in
> progress. (Section 1.7)

Leave it out, and the main program runs on into the subroutine with an
empty accumulator:

```george
R R 6 ↓ (P) ;
* 6 dup × rev dup × + .5 pow ]
```
```console
$ george run programs/08-no-bracket.george --data programs/08-no-bracket.in
5
stop 1 30-21x (IA), 1 1-1x (II): program uses too many accumulator cells or has instructed taking number from empty accumulator (accumulator empty)
  at symbol 9 "×", line 2, column 9
```

The 5 was punched correctly, `;` cancelled it, and then `* 6` was passed
over, as a mark is. `dup` on an empty accumulator copies a 0 without
complaint, as Hamblin's own code does; the stop comes at `×`, the first
symbol that has to take two numbers away.

## A subroutine as an operator

Because `6 ↓` takes two numbers and leaves one, it can be used in a
formula like `+`. The hypotenuse of 3 and 4, and then of that and 12, is
`3, 4, 6 ↓ 12, 6 ↓`:

```george
# 6 ↓ used as a diadic operator: the hypotenuse of 3, 4 and then of
# that and 12, written a b 6 ↓ c 6 ↓.
3, 4, 6 ↓ 12, 6 ↓ (P) ;
]
* 6 dup × rev dup × + .5 pow ]
```
```console
$ george run programs/08-operator.george
13
```

## Subroutines that call subroutines

"Subroutines may themselves involve reference to other subroutines."
Subroutine 14 divides a pair of numbers by the square root of the sum of
their squares, using subroutine 6:

```george
3, 4, 14 ↓ (P) ; (P) ; ]
* 14 (β) rev (α) 6 ↓ (γ) α rev ÷ β γ ÷ ]
* 6 dup × rev dup × + .5 pow ]
```
```console
$ george run programs/08-normalise.george
0.8
0.6
```

It names its two arguments β and α, the last first, and calls 6 on the
pair, which is still in the accumulator. It names the result γ, and
leaves it there too, so that `α rev ÷` divides α by it. `β γ ÷` is the
other quotient. The main program's first `(P) ;` punches the second of
the pair, 4/5, because it is in the last occupied cell.

The number after `*` "must be represented by a single numeral symbol.
(Besides the symbols 0 -- 9 there are however also symbols 10, 11, ...
31: see coding table in Appendix I.)" So `* 14` is one symbol on the card,
not two. [Chapter 13](13-cards-and-codes.md) shows how it was punched.
`george` reads `* 14` as that one symbol, and refuses a number over 31.

## The number of a subroutine is a number

"The main program may specify the subroutine number in any manner." The
number before `↓` is taken out of the accumulator like any other operand,
so it can be worked out, or read from a card. This program reads pairs of
cards, a number and which subroutine to apply to it:

```george
# The subroutine number is read from a card: 1 squares, 2 cubes.
1, 4 rep (i) R R ↓ (P) ; ]
]
* 1 dup × ]
* 2 dup dup × × ]
```
```console
$ george run programs/08-computed.george --data programs/08-computed.in
25
125
9
27
```

And because it is an ordinary number, it is subject to the rule of the
comma. The manual writes the call to subroutine 14 as `3, 4, 14 ↓`. With
one comma fewer:

```george
3, 4 14 ↓ (P) ; (P) ; ]
* 14 (β) rev (α) 6 ↓ (γ) α rev ÷ β γ ÷ ]
* 6 dup × rev dup × + .5 pow ]
```
```console
$ george run programs/08-run-together.george
stop 7 25-29: program calls for skip to non-existent mark (mark number 30)
  at symbol 5 "↓", line 1, column 9
```

`4 14` is the numeral 414, and "Skip numbers are interpreted modulo 32"
(section 1.9 (v)): 414 is 12 × 32 + 30. There is no mark 30, and the
machine stopped. Had there been a subroutine 30, it would have been
called, with the accumulator holding 3 alone, and nothing would have
stopped.

## Locations used by subroutines

> Sometimes a subroutine will use locations in the number store. In this
> case some care must be taken that the locations concerned are not also
> in use in the main program when the subroutine is called. It is
> generally convenient to reserve Greek letters for use in subroutines.
> (Section 1.7)

There is only one store, and subroutines share it with the main program.
Subroutine 5 here adds up the squares of 1 to n, and counts with x. So
does nobody else, its author thought:

```george
# Subroutine 5 adds up the squares of 1 ... n, and uses x to count.
# The main program uses x for its own purposes. The values clash.
7 (x) ;
3, 5 ↓ (P) ;
x (P) ;
]
* 5 (n) ; 0, 1 n rep (x) x x × + ] ]
```
```console
$ george run programs/08-clash.george
14
3
```

The main program's x was 7 and is now 3. With Greek letters:

```george
# The same, with the subroutine's variables Greek letters.
7 (x) ;
3, 5 ↓ (P) ;
x (P) ;
]
* 5 (ν) ; 0, 1 ν rep (λ) λ λ × + ] ]
```
```console
$ george run programs/08-greek.george
14
7
```

## The link list again

A call takes an entry in the six-entry link list, and so does every
repetition in progress. A subroutine called from inside two repetitions,
which calls another, uses four. [Chapter 9](09-skips-and-conditions.md)
has a subroutine that calls itself, and shows what happens at the
seventh.

Next: [Marks, skips and conditions](09-skips-and-conditions.md)
