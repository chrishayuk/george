# 12. Worked examples

The manual has three "program examples" (pp. 7-8), and says of them:

> Many complete programs will need no other facilities than those already
> introduced. It will be noticed that these include the majority of
> problems handled by "tabular" or "matrix" interpretive schemes. Although
> GEORGE is not intended as a competitor of these, it may in the case of
> problems which are not too large compare in speed with them; since a
> sequence of elementary operations may be performed in a single
> "repetition", without intermediate access to the number-store.

This chapter runs them, and two examples of matrix arithmetic from
section 1.6.

## Example 1: mean and root mean square

> A set of 20 numbers is to be read in, and their mean and root mean
> square computed and punched out.

The program is as printed, with the manual's comments:

```george
1, 20 R|(a)                                        # read in data
0, 1, 20 rep (j) j|a + ] 20 ÷ (P) ;                # calculate and punch mean
0, 1, 20 rep (j) j|a dup × + ] 20 ÷ .5 pow (P) ;   # root mean square
```

The data here are the numbers 1 to 20. Their mean is 10.5, and their
root mean square is √(2870/20) = √143.5 = 11.9791.

```console
$ george run programs/12-mean.george --data programs/12-mean.in
10.5
11.9791
```

Each sum starts from a cell marked out with zero. `.5 pow` is the square
root; `√` would do as well.

> Alternatively the mean and root mean square could be calculated in a
> single repetition as follows:

```george
1, 20 R|(a)
0, 0, 1, 20 rep (j) j|a + rev j|a dup × + rev ]
20 ÷ (P) ; 20 ÷ .5 pow (P) ;
```
```console
$ george run programs/12-mean-one-pass.george --data programs/12-mean-one-pass.in
10.5
11.9791
```

Here two cells are marked out with zero, one for the sum of squares and,
above it, one for the sum. Each time round, `j|a +` adds a(j) to the
upper cell; `rev` brings the lower one up; `j|a dup × +` adds the square
to it; and `rev` puts the two back. After the `]` the sum is on top.
`20 ÷ (P) ;` punches the mean and cancels it, which leaves the sum of
squares on top for `20 ÷ .5 pow (P) ;`. One repetition does the work of
two.

## Example 2: more than one set of data

> To repeat the above program n times for different sets of data, where n
> is given on a first card, prefix with "R (n) ; 1 n rep (x)" or
> "1 R rep (x)" and add "]" at end.

With two sets of twenty, 1 to 20 and then 2, 4, ... 40:

```george
R (n) ; 1 n rep (x)                                # Example 2's prefix
1, 20 R|(a)
0, 1, 20 rep (j) j|a + ] 20 ÷ (P) ;
0, 1, 20 rep (j) j|a dup × + ] 20 ÷ .5 pow (P) ;
]                                                  # and its bracket
```
```console
$ george run programs/12-mean-n.george --data programs/12-mean-n.in
10.5
11.9791
21
23.9583
```

The repetition variable x is a dummy. The two inner repetitions use j,
and do not disturb it.

## Example 3: Simpson's rule

> To calculate the definite integral
>
>     ∫ from 1 to 5 of exp(-x²/2) / (1 + exp(-ax)) dx
>
> by Simpson's rule with 16 intervals of argument, for each of five values
> of a, as given on cards.

The program first fills a "Simpson vector" s0 ... s16 with the weights of
the rule, then evaluates the integrand at seventeen points for each value
of a and adds up the weighted values. Here it is as printed:

```george
# The Simpson vector s0 ... s16 = 1/48, 4/48, 2/48, 4/48, ... 1/48:
1, 48 ÷ 0|(s) 16|(s) 2 × 1, 7 rep (j) j 2 ×|(s) ]
2 × 0, 7 rep (j) j 2 × 1 +|(s) ] ;
# The main part:
1, 5 R|(a)                       # read in values a1 ... a5
1, 5 rep (i) i|a (a) ;           # repeat with a = a1 ... a5 in turn
0, 0, 16 rep (j)                 # mark acc. space with zero, j from 0 to 16
j 4 ÷ 1 + (x)                    # calculate argument
dup × 2 ÷ neg exp x a × neg exp 1 + ÷   # calculate integrand
j|s × + ]                        # multiply by Simpson coefficient and add
i|(r) ; ]                        # store result; end outer repeat
1, 5 P|(r)                       # punch results
```

The first two lines build the vector. `1, 48 ÷` is 1/48; `0|(s) 16|(s)`
puts it in s0 and s16. `2 ×` makes 2/48, and the first repetition puts it
in the even places s2 ... s14. `2 ×` again makes 4/48, and the second puts
it in the odd places s1 ... s15. The `;` clears the 4/48 at the end.
Punched, the vector is:

```george
1, 48 ÷ 0|(s) 16|(s) 2 × 1, 7 rep (j) j 2 ×|(s) ]
2 × 0, 7 rep (j) j 2 × 1 +|(s) ] ;
0, 16 P|(s)
```
```console
$ george run programs/12-simpson-vector.george
0.0208333
0.0833333
0.0416667
0.0833333
0.0416667
0.0833333
0.0416667
0.0833333
0.0416667
0.0833333
0.0416667
0.0833333
0.0416667
0.0833333
0.0416667
0.0833333
0.0208333
```

In the main part, `0, 0, 16 rep (j)` marks out a cell with zero and runs
j over the seventeen points. `j 4 ÷ 1 + (x)` is the argument, x = 1 + j/4,
named and left in the accumulator for `dup ×`. The integrand is computed
on top of the running sum, multiplied by s(j) and added in. Run as
printed, with a = 0, 0.5, 1, 2 and 4:

```console
$ george run programs/12-simpson.george --data programs/12-simpson.in
0.0497078
0.0675882
0.0807849
0.093649
0.0988802
```

## The erratum

Those numbers are wrong. For a = 0 the integrand is exp(-x²/2)/2, and its
integral from 1 to 5 is 0.19884. The program gives a quarter of that.

The step is h = 1/4, since x = 1 + j/4. Simpson's rule weights the
ordinates by h/3, 4h/3, 2h/3, ... h/3, which is 1/12, 4/12, 2/12, ...
1/12. The manual has 1/48, 4/48, 2/48, ... 1/48, both in its description
of the vector and in the program that builds it. Every result comes out a
quarter of what it should. Everything else in the program is right. 1/48
would be correct for a step of 1/16, which suggests the example was
changed from another range at some point; but that is a guess.

With 12 for 48:

```george
# The Simpson vector with h/3 = 1/12 for 1/48:
1, 12 ÷ 0|(s) 16|(s) 2 × 1, 7 rep (j) j 2 ×|(s) ]
2 × 0, 7 rep (j) j 2 × 1 +|(s) ] ;
# The main part:
1, 5 R|(a)                       # read in values a1 ... a5
1, 5 rep (i) i|a (a) ;           # repeat with a = a1 ... a5 in turn
0, 0, 16 rep (j)                 # mark acc. space with zero, j from 0 to 16
j 4 ÷ 1 + (x)                    # calculate argument
dup × 2 ÷ neg exp x a × neg exp 1 + ÷   # calculate integrand
j|s × + ]                        # multiply by Simpson coefficient and add
i|(r) ; ]                        # store result; end outer repeat
1, 5 P|(r)                       # punch results
```
```console
$ george run programs/12-simpson-corrected.george --data programs/12-simpson-corrected.in
0.198831
0.270353
0.323139
0.374596
0.395521
```

| a | as printed | with 1/12 | Simpson's rule, in double precision |
|---|---|---|---|
| 0 | 0.0497078 | 0.198831 | 0.198831 |
| 0.5 | 0.0675882 | 0.270353 | 0.270352 |
| 1 | 0.0807849 | 0.323139 | 0.323139 |
| 2 | 0.093649 | 0.374596 | 0.374596 |
| 4 | 0.0988802 | 0.395521 | 0.395521 |

The last column is the same rule with the same seventeen ordinates,
worked in double precision; the difference at a = 0.5 is in GEORGE's
twenty-second binary digit: its + and - do not round to nearest (see
`DECISIONS.md`, 2). Simpson's rule itself, with sixteen intervals,
is good here to about four figures: the integral for a = 0 is
0.1988445.
The erratum is in the repository's `ERRATA.md`.

## Matrix products

Section 1.6 gives three programs on an n × n matrix a and a vector x. The
first puts b(i) = Σ a(i,j) x(j):

```text
1 n rep (i) 0, 1 n rep (j) i j||a j|x × + ] i|(b) ; ]
```

The second and third compute the quadratic form Σ a(i,j) x(i) x(j), "or
for somewhat higher speed" the same sum with x(i) taken outside the inner
repetition:

```text
0, 1 n rep (i) 1 n rep (j) i j||a i|x j|x × × + ] ]
0, 1 n rep (i) 0, 1 n rep (j) i j||a j|x × + ] i|x × + ]
```

The faster one does one multiplication fewer for each a(i,j), and has two
running sums, one inside the other. With n, the matrix and the vector read
from cards, and the results punched, which is ours:

```console
$ cat programs/12-matrix.in
3
2
1
0
1
3
1
0
1
4
1
2
3
```

```george
R (n) ; 1 n 1 n R||(a) 1 n R|(x)             # ours: n, the matrix, the vector
1 n rep (i) 0, 1 n rep (j) i j||a j|x × + ] i|(b) ; ]
1 n P|(b)                                    # ours
0, 1 n rep (i) 1 n rep (j) i j||a i|x j|x × × + ] ] (P) ;
0, 1 n rep (i) 0, 1 n rep (j) i j||a j|x × + ] i|x × + ] (P) ;
```
```console
$ george run programs/12-matrix.george --data programs/12-matrix.in
4
10
14
66
66
```

For the matrix with rows (2, 1, 0), (1, 3, 1), (0, 1, 4) and x = (1, 2,
3), b = (4, 10, 14), and the quadratic form is x · b = 4 + 20 + 42 = 66,
both ways.

Next: [The cards and the codes](13-cards-and-codes.md)
