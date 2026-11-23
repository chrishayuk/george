# 7. Repetition

## rep

> It is very frequently required that a calculation be performed
> repetitively on each of a sequence of numbers: e.g. on each element of
> a vector. For this purpose a special facility is provided in G-code;
> thus: To perform an operation for each value of x, in unit steps, from a
> to b, write
>
>     a b rep (x) ...... ]
>
> The program for the operation to be performed is inserted as shown by the
> row of dots, "x" being used in it as a variable. (Section 1.6)

`rep` takes a and b out of the accumulator. `(x)` after it is part of the
same instruction, and names the variable that counts. `]` ends the part
that is repeated. The manual's first example punches the squares of 1 to
20:

```george
1, 20 rep (j) j j × (P) ; ]
```
```console
$ george run programs/07-squares.george
1
4
9
16
25
36
49
64
81
100
121
144
169
196
225
256
289
324
361
400
```

Each time round, `j j ×` puts one number in the accumulator, `(P)`
punches it and `;` cancels it, so the accumulator is empty again when `]`
sends the program back. The count is in steps of one, upwards. There is
no other step.

The manual's next example puts a(j) = b(j)² for j from 1 to n:

```george
# Put a(j) = b(j) squared, for j from 1 to n: section 1.6.
4 (n) ;
1 n R|(b)                  # this line is ours: b1 ... bn from cards
1 n rep (j) j|b dup × j|(a) ; ]
1 n P|(a)                  # and this one
```
```console
$ george run programs/07-squares-of-b.george --data programs/07-squares-of-b.in
2.25
4
9
100
```

## Summation

The accumulator does not have to be empty at the `]`. A number left in it
on purpose carries over from one time round to the next:

> A frequent use of this facility will be for repeated summation. For this
> purpose an accumulator cell can first be marked out with zero, and then
> terms repeatedly added to it. (Section 1.6)

The sum of the squares from 1 to 100:

```george
0, 1, 100 rep (j) j j × + ] (P) ;
```
```console
$ george run programs/07-sum-of-squares.george
338350
```

Read it as: zero; then for j from 1 to 100, j squared, added on. The
first `0,` is the cell "marked out"; each `+` adds a square to it. The
manual leaves the sum in the accumulator. The `(P) ;` is ours.

"Repeated products can be handled similarly, first marking out a cell
with the number 1." This puts a(n) = n! for n from 1 to 15:

```george
1, 1, 15 rep (x) x × x|(a) ] ;
1, 15 P|(a)
```
```console
$ george run programs/07-factorials.george
1
2
6
24
120
720
5040
40320
362880
3628800
3.99168e7
4.79002e8
6.22702e9
8.71783e10
1.30767e12
```

Each time round, `x ×` multiplies the running product by x, and `x|(a)`
names the result a(x) without cancelling it, so it is still there to be
multiplied next time. `;` after the `]` clears it at the end. The second
line, ours, punches the vector. Six figures is not enough for 13!, which
is 6227020800.

## A dummy variable

> Simple repetition a fixed number of times can be effected using a
> "dummy" variable, i.e. one not required in the calculation. (Section 1.6)

Five times round x = (x + y/x)/2, from 1, is Newton's method for the
square root of y:

```george
2 (y) ;
1, 1, 5 rep (a) dup y rev ÷ + 2 ÷ ]
(P) ;
```
```console
$ george run programs/07-newton.george
1.41421
```

a counts and is not used. With a `(P)` inside, the five values come out
one by one:

```george
2 (y) ;
1, 1, 5 rep (a) dup y rev ÷ + 2 ÷ (P) ]
;
```
```console
$ george run programs/07-newton-each.george
1.5
1.41667
1.41422
1.41421
1.41421
```

## Nesting

"Repetitive operations may be 'nested' one within another." The sum of
the squares from 1 to n, for each n from 1 to 20, stored as a(1) to a(20):

```george
1, 20 rep (n) 0, 1 n rep (x) x x × + ] n|(a) ; ]
1, 20 P|(a)
```
```console
$ george run programs/07-nested.george
1
5
14
30
55
91
140
204
285
385
506
650
819
1015
1240
1496
1785
2109
2470
2870
```

The inner `rep (x)` runs from 1 to n, and n is the variable of the outer
one. Each `]` belongs to the innermost repetition still open.

## How rep counts

The manual gives no more detail than the above. The flow diagrams settle
three points (`DECISIONS.md`, decision 5):

- The variable is set to a when the repetition starts, and 1 is added to
  it at each `]` that goes back.
- The body is always obeyed at least once. The count is made at the `]`,
  not at the start.
- At the end the variable is left at b.

```george
1, 5 rep (i) ]             # an empty body, obeyed five times
i (P) ;                    # i is left at 5
3.5, 5 rep (i) i (P) ; ]   # the first value need not be whole
```
```console
$ george run programs/07-after.george
5
3.5
4.5
```

The second repetition shows what "may give incorrect results" means for a
fractional first value. The count is worked out from whole numbers, each
rounded to the nearest with halves going up, so 3.5 counts as 4 and the
count is 5 - 4 = 1: two passes. The variable itself goes up from 3.5. This
is what Hamblin's own code does, run on the DEUCE simulator.

When a is greater than b, the body is obeyed once, and then the count,
which the machine kept in sixteen binary digits, has wrapped round to a
very large number. `--limit` stops `george` after a given number of
symbols:

```george
5, 3 rep (i) i (P) ; ]
```
```console
$ george run programs/07-backwards.george --limit 40
5
6
7
8
9
10
11
12
13
george: gave up after 40 symbols obeyed (--limit)
```

As the flow diagrams are read, the program above would go round 65535
times, 5, 6, 7 and on; `DECISIONS.md` gives that reading medium
confidence. There is no way in G-code to repeat zero times with `rep`; a
program that might have to must skip round it (chapter 9).

## The link list

Repetitions, and subroutines (next chapter), use a set of locations called
the "link list":

> An entry is inserted in this list whenever a subroutine or repetitive
> operation is entered, and cancelled (by the bracket-symbol "]") when it
> is completed. The link list is limited to six entries, and a program
> must not at any one time have more than this number of subroutines
> and/or repetitive operations nested one within another. (Section 1.9
> (ii))

Six repetitions inside each other are allowed. This counts 2^6 times
round the innermost:

```george
0, 1, 2 rep (a) 1, 2 rep (b) 1, 2 rep (c)
1, 2 rep (d) 1, 2 rep (e) 1, 2 rep (f)
1 + ] ] ] ] ] ] (P) ;
```
```console
$ george run programs/07-count.george
64
```

A seventh is not:

```george
1, 2 rep (a) 1, 2 rep (b) 1, 2 rep (c)
1, 2 rep (d) 1, 2 rep (e) 1, 2 rep (f)
1, 2 rep (g) ] ] ] ] ] ] ]
```
```console
$ george run programs/07-too-deep.george
stop 1 8-21x (IA), 1 3-3x (II): too many entries in link list (nested subroutines or repeats to higher than sixth order)
  at symbol 33 "rep", line 3, column 6
```

## Forgetting the cancel

The manual warns that a program may demand more than twelve cells "by
omitting a 'cancel' sign (';') in a repetitive operation". Here is the
squares program without its `;`:

```george
1, 20 rep (j) j j × (P) ]
```
```console
$ george run programs/07-forgot-cancel.george
1
4
9
16
25
36
49
64
81
100
121
stop 1 30-21x (IA), 1 1-1x (II): program uses too many accumulator cells or has instructed taking number from empty accumulator (all 12 cells occupied)
  at symbol 7 "j", line 1, column 17
```

Each square stayed in the accumulator. Eleven went out, and the twelfth
time round the second `j` found all twelve cells full.

Next: [Subroutines](08-subroutines.md)
