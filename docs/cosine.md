# Where did Hamblin's cosine come from? (GEORGE-NUM-1, open)

GEORGE's `sin` and `cos` evaluate one polynomial, cos(πy/2) for y in
quarter-periods, with five coefficients read from coding sheet 21 (DL8, track
15/13, minor cycles 1, 3, 5, 7 and 13) at 30 binary places, and P31 (1.0) for
the constant term (`src/functions.rs`, `SINCOS`;
`sources/coding-sheets-1958/transcription/NOTES-12-22.md`). In units of
2⁻³⁰ they are

    −1324675860   272375348   −22401135   985355   −25564

and the polynomial is within 3.0·10⁻⁸ of cos(πx/2) on [0, 1]. How were they
found? `scripts/cosine-fit.py` compares them with the methods a programmer of
1957 could have used. Everything below is its output, at 40 digits.

## What they are not

| method | coefficients minus Hamblin's, units of 2⁻³⁰ | largest error on [0, 1] |
|---|---|---|
| Hamblin's | | 2.98·10⁻⁸ |
| Taylor series, five terms | up to ±1,700 | 4.6·10⁻⁷ |
| minimax, absolute error, constant term 1 | −7, 43, −74, 113, −43 | 2.4·10⁻¹⁰ |
| minimax, relative error | larger | 9.6·10⁻¹⁰ |
| least squares | −10, 71, −172, 242, −101 | 7.2·10⁻¹⁰ |
| minimax on [0, b], b from 0.5 to 0.95 | hundreds | |
| **Chebyshev economisation of the Taylor series to x¹²** | **−1, 5, 28, −14, 21** | 6.3·10⁻⁹ |
| the same, to x¹⁴, or interpolation at Chebyshev nodes | −2, 13, −6, 45, −18 | 4.4·10⁻¹⁰ |

Hamblin's error is not the even ripple of a minimax fit. It is under
5·10⁻¹⁰ up to x = 0.5 and then falls steadily, close to −3·10⁻⁸·x¹², to
−2.98·10⁻⁸ at x = 1: a polynomial accurate in its lower terms and a little
wrong in its higher ones.

The nearest family is economisation: take the Taylor series to x¹², replace
x¹² by x¹² − T₁₂(x)/2¹¹, and keep the constant term at 1. Its first two
coefficients are within 1 and 5 units of his; the last three are 14 to 28
units away. Working in decimals first and then converting does not close the
gap: rounded to 8 places, the differences are 3, 0, 24, −14, 20.

## What was tried and could not be read

- Hastings, *Approximations for Digital Computers* (1955), the standard table
  of the time: the Internet Archive copy is lending-only, and its text could
  not be read. It is the next thing to check.
- The DEUCE library routines on David Green's archive (F01, F02, F04/1,
  S01/1, S03D, S05, S06; F03D would not download): none is a sine or cosine.
  F01, F02 and F04 are square roots; S03D, S05 and S06 sum power series.
- Abramowitz and Stegun 4.3.99, a Hastings approximation to cos x on
  [0, π/2], rescaled by (π/2)²ᵏ: 9 to 74 units away. The coefficients were
  typed from memory and have not been checked against the page, so this row
  is not evidence either way.

## So far

The coefficients are close to, but not, a Chebyshev economisation of the
Taylor series, and are not a minimax fit. The pattern of the error (exact
low terms, high terms off by a few parts in 10⁸) fits a hand or machine
calculation with an error in the last three coefficients, or a published
table not yet read. Which, is open.
