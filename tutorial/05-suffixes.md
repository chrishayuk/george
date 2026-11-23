# 5. Suffixes

## Three stores

The number store has three separate parts (section 1.3):

| store | locations | written |
|---|---|---|
| unsuffixed variables | 32, one for each letter | `a` |
| "single suffix" or "vector" store | 32 × 32 = 1024 | `1\|a` for a₁ |
| "double suffix" or "matrix" store | 32 × 32 × 4 = 4096, for a, b, c and d only | `1, 1\|\|a` for a₁,₁ |

They are entirely separate. `a`, `0|a` and `0, 0||a` are three different
numbers.

## The suffix comes first

> A special notation is however used for suffixes: these are written
> first, and are separated from the variable-symbol by a bar sign "|"
> (single suffixes) or double-bar sign "||" (double suffixes). [...] The
> advantage of this notation is that suffix-values are first treated in
> the ordinary way as numbers, and are placed in the accumulator. This
> permits the use of algebraic suffixes and of arithmetic operations in
> suffixes; the bar sign simply takes the number in the last occupied cell
> and treats it as a suffix of the letter which follows. (Section 1.3)

The suffix is worked out like any other number, and the bar takes it out
of the accumulator. So a(n) is `n|a`, and a(j+n) is `j n +|a`. The bar
with its letter puts the value of the suffixed variable in the cell where
the suffix was. A bar with a name, `n|(a)`, stores into it, and like any
name leaves the number in the accumulator.

```george
7, 1|(a) ;                 # a1 = 7
2 (n) ; 9, n|(a) ;         # a2 = 9, by way of n
1|a n|a + (P) ;            # a1 + a2
1, 1 +|a (P) ;             # a(1+1)
5, 1|a 3 ×|(b) ;           # b(3 a1) = 5: the suffix is 21
21|b (P) ;
```
```console
$ george run programs/05-vector.george
16
9
5
```

Look at the fifth line. `5, 1|a 3 ×` leaves two numbers: 5 and 21. The bar
takes the 21 as the suffix and `(b)` names the 5. Everything before the
bar is ordinary arithmetic.

A suffix can itself be suffixed. a(j(k)) is `k|j|a`:

```george
# a(j(k)), written k|j|a: a suffix that is itself suffixed.
4 (k) ;
6, k|(j) ;                 # j4 = 6
11, 6|(a) ;                # a6 = 11
k|j|a (P) ;
```
```console
$ george run programs/05-indirect.george
11
```

## Double suffixes

Only a, b, c and d take two suffixes. The double bar takes the last two
occupied cells, the first suffix from the earlier of the two. The manual's
"Put b(i,j) = a(i,j)²" is `i j||a dup × i j||(b) ;`:

```george
# Put b(i,j) = a(i,j) squared, for i = 2 and j = 3: section 1.4.
2 (i) ; 3 (j) ;
5, i j||(a) ;              # a(2,3) = 5
i j||a dup × i j||(b) ;    # b(2,3) = a(2,3) squared
2, 3||b (P) ;
i j||a j|x × (P) ;         # a(i,j) x(j), with x(3) still zero
```
```console
$ george run programs/05-matrix.george
25
0
```

The last line is a(i,j) x(j), the manual's other example. x(3) was never
set, so it is zero, and so is the product.

Any other letter with a double bar is refused when the program is read.
The manual does not say what the translator did.

```george
1, 1||e (P) ;
```
```console
$ george run programs/05-double-e.george
george: programs/05-double-e.george: line 1, column 5: only a, b, c and d take double suffixes
```

## Larger suffixes: spill

Suffixes normally run from 0 to 31. Outside that range the store is
simply continuous, in the order of the alphabet:

> When a single suffix exceeds 31, the storage location represented is one
> corresponding with a letter alphabetically following: e.g. "a32"
> represents the same number as "b0", "a33" the same as "b1", "a64" the
> same as "c0" and so on. Hence if the larger suffix-values are required
> the appropriate storage-locations must be reserved. [...] In the extreme
> case the entire vector store may be reserved for (say) a0, ... a1023, no
> other singly-suffixed variables being used. Larger values again will
> merely cause reduplication. Negative suffixes are similarly interpreted
> modulo 1024, causing spill to locations of alphabetically preceding
> letters: e.g. "a-1" represents the same number as "ν31". (Section 1.9
> (vi))

```george
5, 32|(a) ;                # a32 is b0
0|b (P) ;
6, 65|(a) ;                # a65 is c1
1|c (P) ;
9, 1 neg|(a) ;             # a(-1) is the last letter's ν31
31|ν (P) ;
```
```console
$ george run programs/05-spill.george
5
6
9
```

ν is the last letter of the alphabet, so a(-1) wraps round to it.

For double suffixes, "the second suffix is interpreted modulo 32 and does
not cause spill; but the first suffix is interpreted modulo 128 and causes
spill through a, b, c and d in the same way":

```george
5, 32, 0||(a) ;            # first suffix 32: spills into b
0, 0||b (P) ;
6, 1, 33||(a) ;            # second suffix 33: taken modulo 32, no spill
1, 1||a (P) ;
```
```console
$ george run programs/05-matrix-spill.george
5
6
```

So a matrix can have up to 128 rows, if b, c and d are given up to it,
but never more than 32 columns.

## Whole numbers only

Suffixes, like the other "indexing" numbers (skip numbers and the ranges
of a repetition), "are assumed to be integers of modulus less than 2^15,
and use of numbers with fractional parts or outside this range may give
incorrect results" (section 1.9 (v)). `george` takes the whole number
part, rounding down, so `2.5|a` is a₂ and `.5 neg|a` is a(-1). Do not rely
on either.

Next: [Reading and punching](06-reading-and-punching.md)
