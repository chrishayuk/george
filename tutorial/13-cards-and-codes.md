# 13. The cards and the codes

## The coding table

A G-program did not go to the machine as text. Each symbol was a pair of
numbers punched on a card. Appendix I of the manual, the "GEORGE Coding
Table", gives them:

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

> The code of each symbol is its row number followed by its column number.
> For example "+" is 5/0, and "n" is 13/4. Columns 9-14 are not used.
> Following an asterisk, symbols of column 3 represent numbers 16-31 as
> shown: otherwise any symbol of column 3 represents decimal point.
> (Appendix I)

Some things to notice:

- Column 2 holds the numerals 0 to 15, and column 3 the numerals 16 to
  31, which is why a mark number up to 31 is a single symbol. The numerals
  10 to 31 are written with a bar over them in the manual (shown here as
  `10̄` and so on). The manual uses them only after an asterisk.
  Anywhere else, any symbol of column 3 is a decimal point.
- Columns 4 and 5 are the 32 letters, and columns 6 and 7 the 32 names.
  A name is a letter's code with 2 added to the column.
- `R|`, `P|`, `R||` and `P||` are single symbols, in column 15.
- The code 0/0 is not a symbol: "Code 0/0 is ignored by read".

Inside the machine a code was eight binary digits. Appendix II says "the
'number' of a variable (or name) is given by the five least significant
digits of its binary symbol-code". That works for every letter only if the
column is the high half of the byte and the row the low half: n, 13/4, is
then 0100 1101, whose last five digits are 01101, 13. ν, 15/5, is
0101 1111, number 31.

## Punching a program

> Each program symbol is represented, in the internal operation of the
> machine, by a combination of 8 binary digits. For convenience, however,
> symbol-codes are punched in decimal as pairs of numbers in the range
> 0--15 [...] Symbol-codes of a program are punched consecutively up to
> sixteen per card, the first number of each code in an odd column and the
> second in the following even column. Code numbers 10, 11, ... 15 are
> punched as 0--5 with an overpunching in X-row (i.e. X counts as 10).
> (Section 2.2)

A card has a row for each digit 0 to 9, and two more above them, the X
and Y rows. A column with a hole in row 4 is the digit 4; a column with
holes in rows X and 4 is 14. Sixteen codes fill 32 columns. The manual
calls these "the Deuce field of a card", and "blank columns are read as
zeros, whence unused portions of the Deuce field of a card may be simply
left blank".

> The last card of a G-program should be punched in Y-row in two
> successive columns: this stops program read-in when all the symbols on
> the card have been read. Y-row is otherwise ignored.

`george deck` prints a program as it was punched, a line to a card. Here
is the manual's subroutine program:

```george
R R 6 ↓ (P) ;                       # main program
]                                   # end of main program
* 6 dup × rev dup × + .5 pow ]      # subroutine
```

```console
$ george deck programs/08-hypot.george
card  1: 0/15 0/15 6/2 6/1 1/15 2/0 5/1 3/0 6/2 12/0 7/0 13/0 12/0 7/0 5/0 0/3
card  2: 5/2 2/8 5/1   (Y-row: last card)
```

The first card begins 0/15 0/15: R, R. Column 1 is punched 0, and column
2 is punched 5 with an X overpunch, for 15. Then 6/2, the numeral 6; 6/1,
↓; 1/15, (P); 2/0, the cancel; and so on. `.5` is two symbols, 0/3 and
5/2: the first symbol of column 3 standing for the point, and the numeral
5. The mark `* 6` is 3/0 6/2. Nineteen symbols take two cards, and the
second has the Y-row punches.

Laid out column by column, the first six codes of the first card were
punched like this:

```text
column    1  2  3  4  5  6  7  8  9 10 11 12
punched   0  5  0  5  6  2  6  1  1  5  2  0
X-row        X     X                 X
symbol    R     R     6     ↓     (P)   ;
```

A mark above 15 is a symbol of column 3. This program calls subroutine
20:

```george
1, 2 + 20 ↓ (P) ; ]
* 20 dup × .5 + ]
```
```console
$ george run programs/13-codes.george
9.5
```

```console
$ george deck programs/13-codes.george
card  1: 1/2 1/0 2/2 5/0 2/2 0/2 6/1 1/15 2/0 5/1 3/0 4/3 12/0 7/0 0/3 5/2
card  2: 5/0 5/1   (Y-row: last card)
```

The `20` before the `↓` is an ordinary numeral, two symbols, 2/2 0/2.
The `20` after the `*` is one symbol, 4/3.

## The printed form

`george format` prints a program as the manual would, which is a way to
check what `george` made of it. It shows the symbols, not the spelling you
used. A program in ASCII spellings:

```george
R R 6 down (P) ; ]
* 6 dup mul rev dup mul + .5 pow ]
```
```console
$ george run programs/13-ascii.george --data programs/13-ascii.in
13
```

```console
$ george format programs/13-ascii.george
R R 6 ↓ (P); ] * 6 dup × rev dup × + .5 pow ]
```

`down` and `mul` have become ↓ and ×, and the line break has gone, since
only the symbols were on the cards.

## The rest of the deck

A complete job, in the order it went into the reader (section 2.1), was:

1. "A special 'G-program initial card'."
2. The program cards, the last with its Y-row punches.
3. "Cards containing data, if any, in the order in which it is required by
   the program."

The basic program itself, GEORGE IA or GEORGE II, was read in first and
kept on the magnetic drum, and stayed there while program after program
was run:

> If a second G-program (with its own initial card) is stacked behind the
> first it will be read in without interruption as soon as the first is
> finished. (Section 2.1)

There was room for 512 symbols. A longer program stops `5 9-24` when it is
read, and "programs can in cases of doubt be sectionalised", the later
sections using the store left by the earlier.

Data cards are described in [chapter 6](06-reading-and-punching.md).

Next: [GEORGE and what came after](14-after-george.md)
