# 15. Reference

G-code as the 1958 manual describes it and `george` obeys it. "Takes"
is the number of cells taken from the accumulator, last occupied first;
"leaves" is the number put back. A stop on an empty or full accumulator
(`1 30-21x` on GEORGE IA, `1 1-1x` on GEORGE II) is possible wherever a
symbol takes or leaves a cell, and is not listed again.

## Numbers and the store

| symbol | code | takes | leaves | effect |
|---|---|---|---|---|
| `0` ... `9` | 0/2 ... 9/2 | 0 | 1 | a numeral: a run of numeral symbols, up to nine digits, is one number |
| `.` | 0/3 (any of column 3) | | | decimal point, part of a numeral |
| `,` | 1/0 | 0 | 0 | separates two numerals; otherwise nothing |
| `a` ... `ν` | 0/4 ... 15/5 | 0 | 1 | the value of the unsuffixed variable |
| `(a)` ... `(ν)` | 0/6 ... 15/7 | 0 | 0 | name: copies the last cell into the variable; does not cancel |
| `;` | 2/0 | 1 | 0 | cancel the last cell |
| `dup` | 12/0 | 0 | 1 | copy the last cell into the next |
| `rev` | 13/0 | 2 | 2 | interchange the last two cells |
| `s\|a` | 0/1 with the letter | 1 (s) | 1 | a(s), from the vector store |
| `s\|(a)` | 0/1 with the name | 1 (s) | 0 | name a(s): copies the last cell, does not cancel |
| `i j\|\|a` | 1/1 with the letter | 2 (i, j) | 1 | a(i, j), from the matrix store; a, b, c, d only |
| `i j\|\|(a)` | 1/1 with the name | 2 (i, j) | 0 | name a(i, j); a, b, c, d only |

The letters in order, columns 4 and 5 of the coding table:

| row | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| column 4 | a | b | c | d | e | f | g | h | i | j | k | l | m | n | θ | p |
| column 5 | q | r | s | t | u | v | w | x | y | z | α | β | γ | λ | μ | ν |

A single suffix outside 0-31 spills into the following (or preceding)
letters, modulo 1024. Of a double suffix, the first is taken modulo 128
and spills through a, b, c and d; the second is taken modulo 32. Suffixes
are rounded down to whole numbers.

## Arithmetic and functions

| symbol | code | | takes | leaves | result | stop |
|---|---|---|---|---|---|---|
| `+` | 5/0 | diadic | 2 | 1 | a + b | `3 14-21x` |
| `-` | 6/0 | diadic | 2 | 1 | a - b | `3 14-21x` |
| `×` | 7/0 | diadic | 2 | 1 | a × b | `3 14-21x` |
| `÷` | 8/0 | diadic | 2 | 1 | a / b | `1 4-24x` if b = 0; `3 14-21x` |
| `neg` | 9/0 | monadic | 1 | 1 | -a | |
| `mod` | 10/0 | monadic | 1 | 1 | \|a\| | |
| `max` | 11/0 | diadic | 2 | 1 | the greater of a and b | |
| `log` | 0/8 | monadic | 1 | 1 | logₑ a | `1 30-21x` if a ≤ 0 |
| `exp` | 1/8 | monadic | 1 | 1 | eᵃ | `7 30-21x` if too large |
| `pow` | 2/8 | diadic | 2 | 1 | aᵇ; 0 if a = 0 | `7 30-21x` if too large; `5 30-21x` if a < 0 and b fractional |
| `rem` | 3/8 | diadic | 2 | 1 | x with nb + x = a, n whole, 0 ≤ x < b | `1 30-16x` if b ≤ 0 |
| `√` | 4/8 | monadic | 1 | 1 | √a | `1 4-24x` if a < 0 |
| `sin` | 5/8 | monadic | 1 | 1 | sin a, a in radians | |
| `cos` | 6/8 | monadic | 1 | 1 | cos a, a in radians | |

In a diadic operator, b is the last occupied cell and a the one before.

## Relations and logic

| symbol | code | takes | leaves | result |
|---|---|---|---|---|
| `=` | 14/0 | 2 | 1 | truth (-1) if a = b, falsehood (0) otherwise; makes the next skip conditional |
| `>` | 15/0 | 2 | 1 | truth if a > b; makes the next skip conditional |
| `~` | 2/1 | 1 | 1 | every digit of the word changed: not-truth punches as 0.5; not-falsehood is -2⁻²² but punches as 0, as Hamblin's punch did |
| `&` | 3/1 | 2 | 1 | digit by digit "and" of the two words |
| `∨` | 4/1 | 2 | 1 | digit by digit "or" of the two words |

Truth is any word with its sign digit set, that is any negative number.

## Control

| symbol | code | takes | leaves | effect |
|---|---|---|---|---|
| `* n` | 3/0, then n | 0 | 0 | mark n, 0 to 31; n is one numeral symbol (10-15 the barred numerals of column 2, 16-31 those of column 3) |
| `n ↓` | 6/1 | 1, or 2 if conditional | 0 | skip to mark n, entering a link to return by |
| `n ↑` | 7/1 | 1, or 2 if conditional | 0 | skip to mark n, with no link |
| `a b rep (x)` | 8/1, then the name | 2 | 0 | set x = a, enter a link, obey what follows to `]`, add 1 to x and repeat, b - a more times |
| `]` | 5/1 | 0 | 0 | end of the innermost repetition or subroutine; with none in progress, end of program |

A skip is conditional if `=` or `>` has been obeyed since the last skip.
Then it takes the mark number and, under it, the truth value, and skips
only on truth. A skip number is taken modulo 32. A missing mark stops on
`7 25-29`; a seventh link stops on `1 8-21x` (IA) or `1 3-3x` (II); a
mark used twice stops on `3 2-2x` (IA) or `0 2-2x` (II).

## Input and output

| symbol | code | takes | leaves | effect |
|---|---|---|---|---|
| `R` | 0/15 | 0 | 1 | read a number from a card |
| `(P)` | 1/15 | 0 | 0 | punch the last cell; does not cancel |
| `a b R\|(x)` | 8/15, then the name | 2 | 0 | read x(a) ... x(b) |
| `a b P\|(x)` | 9/15, then the name | 2 | 0 | punch x(a) ... x(b) |
| `i m j n R\|\|(a)` | 10/15, then the name | 4 | 0 | read a(i,j) ... a(m,n), row by row |
| `i m j n P\|\|(a)` | 11/15, then the name | 4 | 0 | punch a(i,j) ... a(m,n), row by row |
| `wait` | 4/0 | 0 | 0 | stop `1 16-29x`, showing the last cell on the output lights |
| `I` | 9/1 | 0 | 1 | the number on the input keys |

Reading stops on `1 11-6x, 1 7-16x` when there is no card, and on
`4 9-24` when the number is too large.

## The coding table

```console
$ george table
         0     1     2     3     4     5     6     7     8    15
   0     -     |     0   16̄     a     q   (a)   (q)   log     R
   1     ,    ||     1   17̄     b     r   (b)   (r)   exp   (P)
   2     ;     ~     2   18̄     c     s   (c)   (s)   pow
   3     *     &     3   19̄     d     t   (d)   (t)   rem
   4  wait     ∨     4   20̄     e     u   (e)   (u)     √
   5     +     ]     5   21̄     f     v   (f)   (v)   sin
   6     -     ↓     6   22̄     g     w   (g)   (w)   cos
   7     ×     ↑     7   23̄     h     x   (h)   (x)
   8     ÷   rep     8   24̄     i     y   (i)   (y)          R|
   9   neg     I     9   25̄     j     z   (j)   (z)          P|
  10   mod         10̄   26̄     k     α   (k)   (α)         R||
  11   max         11̄   27̄     l     β   (l)   (β)         P||
  12   dup         12̄   28̄     m     γ   (m)   (γ)
  13   rev         13̄   29̄     n     λ   (n)   (λ)
  14     =         14̄   30̄     θ     μ   (θ)   (μ)
  15     >         15̄   31̄     p     ν   (p)   (ν)

The code of each symbol is its row number followed by its column number.
Following an asterisk, symbols of column 3 represent numbers 16-31:
otherwise any symbol of column 3 represents decimal point.
```

## The stops

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

The indications are as the manual prints them, the IA and II forms both
given where they differ. `S.S.` is the single shot. A stop with an action
after `S.S.` is resumable: `george run --single-shot` carries on through
it. For the accumulator and link-list stops the action is GEORGE II's;
on GEORGE IA they were final.

## The george command

```text
george run FILE [options]   obey a program, printing what it punches
george deck FILE            the program as punched: codes, sixteen to a card
george format FILE          the program in the manual's printed form
george table                the coding table
george stops                the table of stops
```

| option | |
|---|---|
| `--data FILE` | data cards, one number to a line |
| `--columns` | read the data file as twenty card columns to a line |
| `--input N,N,...` | data cards on the command line |
| `--keys N,N,...` | the input keys, one number for each `I` |
| `--cards` | print punched numbers as twenty card columns |
| `--trace` | print each symbol obeyed and the accumulator after it |
| `--single-shot` | carry on through resumable stops |
| `--limit N` | give up after N symbols (default ten million) |

Exit status: 0 at the end of the program, 1 on a stop or at the limit, 2
if the program or the data cannot be read.

## ASCII spellings

| printed | ASCII |
|---|---|
| `×` | `mul` |
| `÷` | `/` or `div` |
| `↓` `↑` | `down` `up` |
| `~` `&` `∨` | `not` `and` `or` |
| `√` | `sqrt` |
| `θ α β γ λ μ ν` | `theta alpha beta gamma lambda mu nu` |
| `-` | `-` or `−` |

`#` starts a comment. Letters written together are separate variables,
unless together they make a word of the code.

Back to the start: [Introduction](01-introduction.md)
