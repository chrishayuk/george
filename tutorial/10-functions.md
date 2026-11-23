# 10. The functions

## The table of operations

Section 1.1 ends with a "Table of operations provided in basic program".
All of them take their operands from the accumulator and put one result
back:

| symbol | | operation | for | write |
|---|---|---|---|---|
| `+` | diadic | addition | a + b | `a b +` |
| `-` | diadic | subtraction | a - b | `a b -` |
| `×` | diadic | multiplication | ab | `a b ×` |
| `÷` | diadic | division | a/b | `a b ÷` |
| `neg` | monadic | minus sign | -a | `a neg` |
| `mod` | monadic | modulus | \|a\| | `a mod` |
| `max` | diadic | maximum | max(a, b) | `a b max` |
| `log` | monadic | natural logarithm | logₑ a | `a log` |
| `exp` | monadic | exponential | eᵃ | `a exp` |
| `pow` | diadic | power | aᵇ | `a b pow` |
| `rem` | diadic | remainder | rem. of a on div. by b | `a b rem` |
| `√` | monadic | square root | √a | `a √` |
| `sin` | monadic | sine (angle in radians) | sin a | `a sin` |
| `cos` | monadic | cosine (angle in radians) | cos a | `a cos` |

There is no tangent, arc tangent or arc sine; there is no constant π.
Anything else is a subroutine.

```george
2 log (P) ;                # natural logarithm
1 exp (P) ;                # e
2, 10 pow (P) ;            # 2 to the power 10
2, .5 pow (P) ;            # the square root of 2, by pow
2 √ (P) ;                  # and by √
3, 7 max (P) ;             # the greater
3 neg mod (P) ;            # modulus
3 neg neg (P) ;            # minus minus three
```
```console
$ george run programs/10-functions.george
0.693147
2.71828
1024
1.41421
1.41421
7
3
3
```

`2, .5 pow` and `2 √` agree to six figures. They were different routines
on the machine: `pow` went by way of the logarithm and the exponential,
and `√` by an iteration of its own, Newton's, with the divider (pp. 34-37;
findings 18-22 of `sources/deuce/RUNNING-GEORGE.md`). `george` does each
as the machine did, digit for digit.

## Remainder

> Note: the "remainder of a on division by b" is defined as a number x
> such that nb + x = a, where n and b are both integers and 0 ≤ x < b.
> Note that "a 1 rem" gives the positive fractional part of a. (Section
> 1.1)

So the remainder is never negative, even when a is:

```george
7, 2 rem (P) ;             # 7 = 3 × 2 + 1
7 neg, 2 rem (P) ;         # -7 = -4 × 2 + 1
2.75, 1 rem (P) ;          # "a 1 rem" gives the positive fractional part
2.75 neg, 1 rem (P) ;      # -2.75 = -3 × 1 + 0.25
```
```console
$ george run programs/10-rem-good.george
1
1
0.75
0.25
```

b must be a positive whole number. A negative or zero divisor stops the
machine:

```george
7, 2 rem (P) ;             # 7 = 3 × 2 + 1
7, 2 neg rem (P) ;         # a negative divisor: a stop
```
```console
$ george run programs/10-rem.george
1
stop 1 30-16x: divisor negative or zero (rem)
  at symbol 10 "rem", line 2, column 10
```

A fractional b is not allowed by the manual, but the machine did something
definite with one, and `george` does the same. Hamblin's code, run on the
DEUCE simulator, fetches b with the routine that fetches suffixes, which
rounds it to the nearest whole number, halves going up, and then checks
that it is positive. So `7, 3.6 rem` is `7, 4 rem`, which is 3. The
routine shifts the number by its exponent taken modulo 16, which is right
only from one half up to 2¹⁵: `7, .4 rem` is `7, 26 rem`, which is 7.
Better not to rely on it.

## Powers

`pow` raises a to the power b. For a negative a, b must be a whole number;
the sign of the result is then put right. A zero a gives zero, whatever
b is, even a negative b: the flow diagrams (p. 34) test for a zero base
before anything else.

```george
3 neg, 3 pow (P) ;         # a negative number to a whole power
3 neg, 2 pow (P) ;
4, .5 neg pow (P) ;        # a negative power
0, 3 pow (P) ;
0, 1 neg pow (P) ;         # zero to any power is zero
```
```console
$ george run programs/10-pow.george
-27
9
0.5
0
0
```

The base must not be minus a power of two: `2 neg, 3 pow` stops on
`1 30-21x`, the stop for the logarithm of a number that is not positive.
`pow` works with the logarithm of |a|, and it makes |a| by negating the
mantissa; a negative power of two has the mantissa -1, and negated that is
+1, one more than a mantissa can hold, which the logarithm routine sees
as negative. That is how Hamblin's code behaves, and `george` does the
same.

## Sines and cosines

The angle is in radians. There is no π, so a program that wants degrees
writes it out. This makes a table at fifteen-degree steps; nine figures is
the most a numeral may have:

```george
# The sine and cosine of 0, 15, 30, ... 90 degrees.
3.14159265, 12 ÷ (θ) ;     # 15 degrees in radians
0, 6 rep (k) k θ × sin (P) ; ]
0, 6 rep (k) k θ × cos (P) ; ]
```
```console
$ george run programs/10-sines.george
-2.98023e-8
0.258819
0.5
0.707107
0.866025
0.965926
1
1
0.965926
0.866025
0.707107
0.5
0.258819
2.85916e-7
```

The first value is sin 0, which should be 0, and the last is cos 90°.
Neither is quite 0. cos 90° is the cosine of an angle a little away from
π/2, since π/2 was held to 22 binary digits. sin 0 is -2⁻²⁵ because of
how the machine worked it out, which `george` follows digit for digit.
The routine (`sources/coding-sheets-1958/transcription/NOTES-12-22.md`,
and finding 19 of `sources/deuce/RUNNING-GEORGE.md`) multiplies the
argument by 2/π to measure it in quarter periods, with 31 binary places
after the point, and works out cos(πx/2) for x within one quarter period
by a polynomial in x², with sin as cos shifted back by a quarter period.
The five coefficients, on track 15/13 of DL8, are

    -1.2337005  +0.2536693  -0.0208627  +0.00091768  -0.0000238

They are close to the Taylor coefficients, -(π/2)²/2! and +(π/2)⁴/4! and
so on, but not equal to them, in the way of a polynomial "economised" to
fit the whole interval; with them the polynomial is within about 3 × 10⁻⁸
of cos(πx/2). Every product is rounded to 30 binary places as the machine
rounded it. For sin 0 the argument is a whole quarter period back, -1,
where the polynomial comes to -2⁻²⁵ instead of 0.

## Logarithms

`log` is the natural logarithm. Common logarithms are log x / log 10:

```george
# Common logarithms of 1 ... 10, as log x / log 10.
10 log (λ) ;
1, 10 rep (x) x log λ ÷ (P) ; ]
```
```console
$ george run programs/10-log-table.george
0
0.30103
0.477121
0.60206
0.69897
0.778151
0.845098
0.90309
0.954242
1
```

`10 log (λ) ;` works the divisor out once, outside the repetition.

## When a function fails

Each function that can fail has a stop of its own in the table of section
2.4:

| indication | cause |
|---|---|
| `3 14-21x` | result of arithmetic operation exceeds permissible magnitude |
| `1 4-24x` | divisor zero (÷) or argument negative (√) |
| `1 30-16x` | divisor negative or zero (rem) |
| `1 30-21x` | argument zero or negative (log) |
| `7 30-21x` | result exceeds permissible magnitude (exp or pow) |
| `5 30-21x` | fractional power of negative number (pow) |

All of them can be got past by the operator, as the next chapter shows,
"though of course incorrect answers will be obtained".

Next: [When things go wrong](11-when-things-go-wrong.md)
