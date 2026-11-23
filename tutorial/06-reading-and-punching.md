# 6. Reading and punching

## R and (P)

> The symbol "R" represents an instruction to read one number from a card
> and place it in the first vacant cell of the accumulator.
>
> The symbol "(P)" represents an instruction to punch out the number in
> the last occupied cell of the accumulator. It is written in brackets to
> indicate that it operates like a name-symbol; i.e. that it does not
> "cancel" the cell. (Section 1.5)

The manual's examples: "Read in the number x" is `R (x) ;`; "Punch out
the number x" is `x (P) ;`; "Read two numbers and punch out their sum" is
`R R + (P) ;`.

```george
R R + (P) ;                # read two numbers and punch their sum
```

The data cards came after the program cards in the reader, "in the order
in which it is required by the program" (section 2.1). `george` takes them
from a file named with `--data`, one number to a line:

```console
$ cat programs/06-read-two.in
3
4
```

```console
$ george run programs/06-read-two.george --data programs/06-read-two.in
7
```

or from the command line with `--input`, the numbers separated by commas:

```console
$ george run programs/06-read-two.george --input 10,-2.5
7.5
```

Here is a program that reads three coefficients and an argument and
works out e = ay² + by + c, with a = 2, b = -3, c = 1 and y = 4. The
formula is the example usually quoted for GEORGE.

```george
# e = a y² + b y + c.
R (a) ; R (b) ; R (c) ; R (y) ;
a y dup × × b y × + c + (e) (P) ;
```
```console
$ george run programs/06-polynomial.george --data programs/06-polynomial.in
21
```

It is not in the manual, but it is G-code as the manual describes it.

## Blocks of numbers

Numbers can go straight to and from the vector and matrix stores:

> To read in a sequence of numbers x_a, x_a+1, ... x_b write a b R|(x)
>
> To punch out the same sequence of numbers write a b P|(x)
>
> To read in a matrix of numbers a_i,j ... a_m,n write i m j n R||(a)
>
> To punch out this matrix write i m j n P||(a)
>
> A matrix is read or punched row by row. (Section 1.5)

`R|`, `P|`, `R||` and `P||` are single symbols in the coding table, and
must be written without a space before the bar; `R |a` would be a read
followed by a suffix. They take their range numbers out of the
accumulator, two for a vector and four for a matrix, and leave nothing
behind.

```george
1, 5 R|(x)                 # read x1 ... x5
1, 5 P|(x)                 # punch them again
3, 4 P|(x)                 # and x3, x4
```
```console
$ george run programs/06-vector.george --data programs/06-vector.in
2
3
5
7
11
5
7
```

The order of the four matrix numbers is first row, last row, first
column, last column:

```george
1, 2, 1, 3 R||(a)          # a(1,1) ... a(2,3), row by row
2, 2, 1, 3 P||(a)          # punch the second row
1, 2, 2, 2 P||(a)          # and the second column
```
```console
$ george run programs/06-matrix.george --data programs/06-matrix.in
21
22
23
12
22
```

## The cards

A data card held one number in the twenty columns described in [chapter
3](03-numerals.md). Section 2.3 allowed some slack for punching them by
hand:

> Input cards are essentially in this form, but some flexibility is
> allowed for ease of manual punching, as follows: (i) Plus signs and
> zeros need not be punched. (ii) Minus sign of the mantissa may be
> punched in any of cols. 1-10, and minus sign of the exponent in any of
> cols. 11-20. (The latter is often conveniently overpunched in col. 20.)
> Note that figures in cols. 8-10 and 12-17 are in any case treated as
> zero by the reader.

With `--columns`, `george` reads each line of the data file as the twenty
columns of a card, a minus sign as `-`:

```console
$ cat programs/06-columns.in
 314159265
 25       -        1
-1                 2
 5            -    3
```

```george
R (P) ; R (P) ; R (P) ; R (P) ;
```
```console
$ george run programs/06-columns.george --data programs/06-columns.in --columns
3.14159
0.25
-100
0.005
```

The first card is 3.14159265 with the last three figures in columns 8-10,
which the reader ignored. The second has its exponent sign in column 11
and the fourth in column 15; both are allowed. Blank columns read as
zeros.

A number in ordinary notation in a data file is taken as it would have
been punched: its first six significant figures, the rest dropped (not
rounded), because the reader ignored them anyway:

```george
R (P) ; R (P) ;
```
```console
$ george run programs/06-six-figures.george --data programs/06-six-figures.in
3.14159
0.000123456
```

`--cards` shows what the punch put on the cards that came out:

```george
R R ÷ (P) ;
```
```console
$ george run programs/06-cards-out.george --data programs/06-cards-out.in --cards
 314286000 000000000
```

## When the cards run out

A program that reads one card too many stops:

```george
R R + (P) ; R R + (P) ;
```
```console
$ george run programs/06-no-more.george --data programs/06-no-more.in
3
stop 1 11-6x, 1 7-16x: data card wanted
  at symbol 6 "R", line 1, column 15
```

`1 11-6x` and `1 7-16x` are the two indications the manual gives for
"data card wanted". The action it gives is "Supply card or emend
program". [Chapter 9](09-skips-and-conditions.md) has a program that
reads until it meets this stop, on purpose.

A number too large for GEORGE's exponent stops the reader too:

```george
R (P) ;
```
```console
$ george run programs/06-too-big.george --data programs/06-too-big.in
stop 4 9-24: data card with exponent too large
  at symbol 0 "R", line 1, column 1
```

The card's exponent fits in three digits, but 10^200 is beyond 2^511, the
largest number GEORGE can hold.

Next: [Repetition](07-repetition.md)
