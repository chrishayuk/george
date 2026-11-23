# 2. Reverse Polish and the running accumulator

## Operators after their operands

In ordinary notation an operator goes in front of its operand, as in
"-a", or between two, as in "a + b". In G-code it goes after:

> In reverse Polish notation the operator-signs, whether monadic or diadic,
> are written after the variables concerned [...] One result of this is
> that brackets are never needed: any of the above expressions may be used
> directly and without ambiguity as an element in a longer expression.
> (Section 1.1)

The manual calls an operator on one number *monadic* and an operator on
two *diadic*. So:

| ordinary | G-code |
|---|---|
| a + b | `a b +` |
| a - b | `a b -` |
| ab | `a b ×` |
| a/b | `a b ÷` |
| -a | `a neg` |
| \|a\| | `a mod` |
| -(a + b) | `a b + neg` |
| (a + b)c | `a b + c ×` |
| a + bc | `a b c × +` |

Letters stand for numbers kept in the store, and [chapter 4](04-variables.md)
is about them. Until then the programs use numbers written out. This is
(2 + 3) × 4:

```george
2, 3 + 4 × (P) ;
```
```console
$ george run programs/02-sum.george
20
```

The comma separates two numbers written one after the other. Without it,
`2 3` would be the number 23. [Chapter 3](03-numerals.md) explains why.

## The running accumulator

The manual asks the reader to keep a picture in mind:

> The pseudo-machine into which DEUCE is converted by the basic GEORGE
> program can be envisaged as equipped with a "running accumulator"; this
> is a set of storage locations (known as "cells") arranged in a linear
> order and operated on the "last-in-first-out" principle. When a variable
> occurs in a program its "value" (from an appropriate position in the
> main number-store) is placed in the first vacant cell of the
> accumulator, i.e. in cell 1 if this does not already contain a number,
> otherwise in cell 2 or etc.; and when an operator-symbol occurs, the
> specified operation is carried out on the contents of the last occupied
> cell or last two occupied cells, depending as the operator is monadic or
> diadic. (Section 1.1)

A number written in the program goes into the first vacant cell in the
same way. A diadic operator takes the last two occupied cells, works out
the result, and leaves it in the first of the two. So the program above
runs like this:

| symbol | cells after it |
|---|---|
| `2` | 2 |
| `3` | 2, 3 |
| `+` | 5 |
| `4` | 5, 4 |
| `×` | 20 |

The same product can be written with the 4 first, as 4 × (2 + 3):

```george
4, 2, 3 + × (P) ;
```
```console
$ george run programs/02-other-order.george
20
```

Here three cells are filled before any arithmetic is done. The manual's
words: "the result of the addition would go into cell 2, and the final
result as before into cell 1." `--trace` shows each symbol as it is
obeyed, with the cells after it, first cell on the left:

```george
# 4 × (2 + 3), traced.
4, 2, 3 + × (P) ;
```
```console
$ george run programs/02-trace.george --trace
   0  4        4
   1  ,        4
   2  2        4  2
   3  ,        4  2
   4  3        4  2  3
   5  +        4  5
   6  ×        20
20
   7  (P)      20
   8  ;
```

The left column is the number of the symbol in the program, counting from
0. The comma is a symbol too, and does nothing. A numeral of more than one
symbol is shown at its first. The trace goes to the error stream, in
step with what is punched: the 20 appears when `(P)` is obeyed.

The manual sums this up: "the overall effect of any calculation of a
number is to place the number concerned in the first vacant cell of the
accumulator." Whatever a formula is, once it has been obeyed it is one
number in one more cell, and it can be the operand of the next operator.
That is why no brackets are needed.

## Punching and cancelling

`(P)` punches the number in the last occupied cell. It leaves it there. It
is written in brackets for that reason: it behaves like a name, which
[chapter 4](04-variables.md) explains, and a name does not "cancel" the
cell either.

`;` cancels the last occupied cell: it throws the number away. So `(P) ;`
punches a result and clears it out of the way. Every program so far has
ended with it. A program that leaves numbers in the accumulator at the end
does no harm, but one that leaves them inside a repetition soon fills it
(chapter 7).

## dup and rev

Two operators move numbers about:

> There are two special operators which, although not strictly necessary,
> are frequently useful. These are: (i) "dup": this "duplicates" the
> contents of the last occupied cell in the next cell. [...] (ii) "rev":
> this "reverses" (i.e. interchanges) the contents of the last two
> occupied cells. (Section 1.1)

`dup` saves working a number out twice. `rev` puts two numbers the other
way round, which matters for `-`, `÷` and every other diadic operator
where the order counts:

```george
2, 3 + dup × (P) ;          # (2 + 3) squared
10, 4 - (P) ;               # 10 - 4
10, 4 rev - (P) ;           # 4 - 10
```
```console
$ george run programs/02-dup-rev.george
25
6
-6
```

The manual's example uses both: a + b + c/(a + b) is
`a b + dup c rev ÷ +`. The sum a + b is worked out once, and `dup` keeps a
copy for the end. With a = 1, b = 3 and c = 2:

```george
# a + b + c/(a + b), section 1.1, with a = 1, b = 3 and c = 2.
1, 3 + dup 2 rev ÷ + (P) ;
```
```console
$ george run programs/02-dup-rev-manual.george
4.5
```

## The manual's examples

Section 1.1 gives a table of formulae in both notations. Here they are,
with the letters a, b and c replaced by 2, -3 and 5. `3 neg` is how minus
three is written; there is no negative numeral.

```george
# The manual's examples of section 1.1, with a = 2, b = -3 and c = 5.
2, 3 neg + neg (P) ;               # -(a + b)
2, 3 neg + mod (P) ;               # |a + b|
2 mod 3 neg mod + (P) ;            # |a| + |b|
2, 3 neg + 5 + (P) ;               # a + b + c
2, 3 neg 5 + + (P) ;               # the same, as a b c + +
2, 3 neg + 5 × (P) ;               # (a + b)c
2, 3 neg, 5 × + (P) ;              # a + bc
2, 3 neg × 2, 3 neg ÷ + (P) ;      # ab + a/b
```
```console
$ george run programs/02-manual-table.george
1
1
5
4
4
-5
-13
-6.66666
```

The two forms of a + b + c agree, as they must. Note `3 neg 5`: no comma
is needed there, because `neg` stands between the numerals.

The last line is -6 - 2/3, which is -6.666666... and might be expected to
punch as -6.66667. It punches -6.66666 because GEORGE's addition does not
round its result to the nearest binary fraction, and here the sum it keeps
is a little smaller in size than the true one; Hamblin's own code, run on
the DEUCE simulator, punches the same. (The rule is in the repository's
`DECISIONS.md`, decision 2.)

## Twelve cells

The accumulator has twelve cells:

> The accumulator is limited to 12 cells. This is more than enough for all
> normal calculations; but a program which demands more cells may be
> written inadvertently, e.g. by omitting a "cancel" sign (";") in a
> repetitive operation. Such a program will give failure; as also will a
> program which instructs taking a number from an empty accumulator.
> (Section 1.9 (i))

Thirteen numbers before the first operator is one too many:

```george
1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13
+ + + + + + + + + + + + (P) ;
```
```console
$ george run programs/02-full.george
stop 1 30-21x (IA), 1 1-1x (II): program uses too many accumulator cells or has instructed taking number from empty accumulator (all 12 cells occupied)
  at symbol 27 "1", line 1, column 40
```

The machine stopped on the thirteenth number. The indication is the
pattern of lights the operator saw: `1 30-21x` on GEORGE IA and `1 1-1x`
on GEORGE II. [Chapter 11](11-when-things-go-wrong.md) is about stops.

An operator with too few operands stops in the same way:

```george
5 + (P) ;
```
```console
$ george run programs/02-empty.george
stop 1 30-21x (IA), 1 1-1x (II): program uses too many accumulator cells or has instructed taking number from empty accumulator (accumulator empty)
  at symbol 1 "+", line 1, column 3
```

The stop is for *taking* a number. `(P)`, a name or `dup` on an empty
accumulator only copy, and copy a 0 without stopping: that is what
Hamblin's own code does, run on the DEUCE simulator.

Next: [Numerals and the comma](03-numerals.md)
