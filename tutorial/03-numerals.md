# 3. Numerals and the comma

## Numbers in the program

> Data required in a calculation will normally be read in separately from
> the program by means of special instructions. It is frequently
> convenient, however, to include numbers as part of the program itself.
> For this purpose the symbols 0--9 are provided, together with a symbol
> for decimal point. Sequences of these symbols (of not more than nine
> digits in length) are treated as numbers in the usual way. (Section 1.2)

A numeral goes into the first vacant cell, like a variable. So "constant
numbers may be included in a formula in the same way as variables." The
manual's examples are 3a + 4b, written `a 3 × b 4 × +`, and .14(a + 33.8),
written `a 33.8 + .14 ×`.

```george
7 (P) ;
.5 (P) ;
33.8 (P) ;
123456789 (P) ;
.000001 (P) ;
1000000 (P) ;
```
```console
$ george run programs/03-numerals.george
7
0.5
33.8
1.23457e8
1e-6
1000000
```

There is no minus sign in a numeral. Write `7 neg`.

A number in GEORGE is in "floating-point binary form (22 binary digit
signed mantissa, 10 binary digit signed exponent; hence about six decimal
digit accuracy)". So 123456789 is held only to about six figures, and is
punched to six. Numbers can run from about 10^-154 to 10^154.

## A numeral is one symbol per digit

Each digit, and the decimal point, is a symbol of its own. The program
`33.8` is four symbols: 3, 3, point, 8. The translator collects a run of
numeral symbols into one number. That is the whole reason for the comma:

> When two numbers must be written consecutively they should be separated
> by a comma (special symbol, omitted in translation). (Section 1.2)

Spaces are not symbols. They are not on the cards at all, only on the
page. So `2 3` and `23` are the same program, and both are the number 23:

```george
2 3 + (P) ;
```
```console
$ george run programs/03-run-together.george
stop 1 30-21x (IA), 1 1-1x (II): program uses too many accumulator cells or has instructed taking number from empty accumulator (accumulator empty)
  at symbol 2 "+", line 1, column 5
```

The `+` found only one number, and the machine stopped. With the comma:

```george
2, 3 + (P) ;
```
```console
$ george run programs/03-comma.george
5
```

The manual's example of the comma is 4.3/(16 - x), written
`4.3, 16 x - ÷`:

```george
# 4.3 / (16 - x), with x = 6: the manual's example of the comma.
6 (x) ;
4.3, 16 x - ÷ (P) ;
```
```console
$ george run programs/03-ratio.george
0.43
```

A comma is needed only between two numerals. A letter or an operator
between them separates them already: `a 3 ×`, `3 neg 5`.

This is the most common mistake in G-code, and it does not always stop
the machine. Often the run of digits is a perfectly good number, and the
program goes on with the wrong one. [Chapter 8](08-subroutines.md) shows
a subroutine call that goes to the wrong subroutine because of it.
`george format` prints a program as the machine sees it, with the digits
of each numeral run together, which shows the mistake at once:

```console
$ george format programs/08-run-together.george
3, 414 ↓ (P); (P); ] * 14 (β) rev (α) 6 ↓ (γ) α rev ÷ β γ ÷ ] * 6 dup × rev dup × + .5 pow ]
```

That `414` was meant to be 4 and 14.

## Nine digits

A numeral has not more than nine digits. `george` refuses a longer one
when it reads the program, before anything is run:

```george
1234567891 (P) ;
```
```console
$ george run programs/03-ten-digits.george
george: programs/03-ten-digits.george: line 1, column 1: a numeral sequence has not more than nine digits, and this has 10
```

The manual does not say what the translator did with a tenth digit.
Spaces make no difference, since they were never on the cards: `12345
67891` is the same ten-digit numeral, and is refused in the same way.

```george
12345 67891 (P) ;
```
```console
$ george run programs/03-spaced-digits.george
george: programs/03-spaced-digits.george: line 1, column 1: a numeral sequence has not more than nine digits, and this has 10
```

A numeral may have one decimal point, and no more:

```george
1.2.3 (P) ;
```
```console
$ george run programs/03-two-points.george
george: programs/03-two-points.george: line 1, column 1: a numeral sequence has at most one decimal point
```

## Six figures

Numbers are punched rounded to six significant figures. Inside the
machine they carry a little more: 22 binary digits is a relative precision
of about one part in two million. Squaring the square root of two shows
the last binary digit:

```george
2 √ (P)           # the square root of two
dup × (P)         # squared
2 - (P) ;         # less two
```
```console
$ george run programs/03-root-two.george
1.41421
2
-9.53674e-7
```

The square came out a little short of 2, by 2^-20, which rounds to 2 at
six figures. The difference is still there to be found by subtracting.

## The cards themselves

`--cards` prints each punched number as the card that carried it. Section
2.3 gives the layout:

> Numbers as data are punched one per card in floating decimal, with sign
> of mantissa in col. 1, mantissa (<10) in cols. 2-10 with decimal point
> between cols. 2 and 3; sign of exponent in col. 11, exponent (integer)
> in cols. 12-20. [...] The mantissa is however rounded to six
> significant figures, and the exponent cannot exceed three digits: hence
> cols. 8-10 and 12-17 will in any case have zeros.

```george
1, 3 ÷ (P) ;
338350 (P) ;
2.5 neg (P) ;
.000001 (P) ;
```
```console
$ george run programs/03-cards.george --cards
 333333000-000000001
 338350000 000000005
-250000000 000000000
 100000000-000000006
```

The minus signs are shown as `-` and the plus signs, which were not
punched, as blanks. The second card is 3.38350 × 10^5; the last is
1.00000 × 10^-6. [Chapter 6](06-reading-and-punching.md) reads cards in
this form.

Next: [Variables and names](04-variables.md)
