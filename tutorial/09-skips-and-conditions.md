# 9. Marks, skips and conditions

## Marks and skips

> A place marker, consisting of the symbol "*" followed by a single
> numeral symbol as described in the previous section, may be inserted
> anywhere in a program: it will be omitted by the translator, which
> however notes in a special list called the "mark list" the address of
> the next instruction following. A "skip" instruction, or instruction to
> resume the program from a point so marked, may be given by writing the
> number of the mark followed by the symbol "↑". This symbol operates in
> the same way as the "skip to subroutine" symbol "↓", except that it
> makes no provision for later returning to the point of interruption.
> (Section 1.8)

So a subroutine number and a place mark are the same thing. There is one
mark list, of 32 entries. `n ↓` skips to mark n and leaves a link to come
back by; `n ↑` skips to it and does not. The manual's example reads pairs
of numbers and punches their sums for ever:

```george
* 0 R R + (P) ; 0 ↑
```
```console
$ george run programs/09-pairs.george --data programs/09-pairs.in
3
7
7.5
stop 1 11-6x, 1 7-16x: data card wanted
  at symbol 2 "R", line 1, column 5
```

It ends when the cards run out. On the machine that was an ordinary way
to finish: the operator saw "data card wanted", and if there were no more
cards, the job was done.

## Relations

> Skips may however be conditional on the results of a calculation. For
> this purpose two "relational" operators "=" and ">", and three "logical"
> operators "~", "&" and "∨" are provided. A skip ("↑" or "↓") is treated
> as conditional whenever one of the two relational operators has
> preceded it (since the previous skip).
>
> The relational operators are diadic operators which give as result a
> digit (the sign digit) to indicate whether the relation is "true" or
> "false". (Section 1.8)

`a b =` gives truth if a = b, and `a b >` gives truth if a > b. There is
no `<`; write `b a >`. "Note that as constants '0' may be written to
represent 'falsehood' and '-1' (i.e. '1 neg') for 'truth'." So a truth
value punched is -1 or 0:

```george
3, 4 = (P) ;               # falsehood
3, 3 = (P) ;               # truth
4, 3 > (P) ;               # truth
3, 4 > (P) ;               # falsehood
```
```console
$ george run programs/09-relations.george
0
-1
-1
0
```

> When a skip is "conditional", the last occupied cell of the accumulator
> is examined to see if it indicates "truth"; and the skip takes place
> only if this is so. Thus: For "If x = 0 skip to mark no. 8 (otherwise
> continue)" write x 0 = 8 ↑. Both the "truth" indication and the mark
> number are cancelled from the accumulator, whether the skip takes place
> or not. (Section 1.8)

The mark number is on top and the truth value under it. Here is a
repetition that punches 1/x for each x read, or 0 where x is zero:

```george
# Punch 1/x for each x read, or 0 where x is zero.
1, 4 rep (i)
R (x) 0 = 8 ↑              # if x = 0 skip to mark 8
1 x ÷ (P) ; 9 ↑            # otherwise 1/x, and skip over
* 8 0 (P) ;
* 9 ]
```
```console
$ george run programs/09-reciprocal.george --data programs/09-reciprocal.in
0.25
0
-0.125
2
```

`R (x)` names the number read and leaves it in the accumulator, so `0 =`
compares it with zero at once. `9 ↑` has no relation before it since the
last skip, so it is not conditional. The mark `* 9` stands just before
the `]`, so that both ways round reach the end of the repetition. There
is no "else" in G-code; there are marks.

## The condition stays set

Note "since the previous skip". A relation makes the next skip
conditional, whether or not the relation's truth value is still where the
skip will look for it. The flow diagrams show a flag, set by `=` and `>`
and cleared only by a skip. Here a relation's value is punched and
cancelled, and the skip after it is still conditional:

```george
2, 2 = (P) ;               # a relation, punched and cancelled
5 ↑                        # this skip is conditional
1 (P) ; ]
* 5 2 (P) ; ]
```
```console
$ george run programs/09-leftover.george
-1
stop 1 30-21x (IA), 1 1-1x (II): program uses too many accumulator cells or has instructed taking number from empty accumulator (accumulator empty)
  at symbol 7 "↑", line 2, column 3
```

The skip took its mark number, 5, and then found no truth value to test.
In a longer program it would have found some other number and tested
that.

## Logic

> "~" (or "not") is monadic and gives logical negation: that is, it
> changes "truth" to "falsehood" or vice versa. [...] "&" (or "and") is
> diadic and gives logical conjunction; i.e. "truth" if both operands
> have "truth", "falsehood" otherwise. [...] "∨" (or "or") is diadic and
> gives logical disjunction; i.e. "truth" if either or both operands have
> "truth", "falsehood" only if both have "falsehood". (Section 1.8)

The manual's examples: a ≠ b is `a b = ~`; a ≤ b is `a b > ~`; "a = b =
0" is `a 0 = b 0 = &`; a < x < b is `x a > b x > &`; "x = 0 or 1" is
`x 0 = x 1 = ∨`; "a or b or both exceed n" is `a n > b n > ∨`.

```george
# a < x < b, written x a > b x > &; and x = 0 or 1, written x 0 = x 1 = ∨.
2 (a) ; 5 (b) ;
1, 6 rep (x)
x a > b x > & (P) ;
x 0 = x 1 = ∨ (P) ;
]
```
```console
$ george run programs/09-between.george
0
-1
0
0
-1
0
-1
0
0
0
0
0
```

For each x from 1 to 6, the first number is whether 2 < x < 5 and the
second whether x is 0 or 1.

## Truth is the sign digit

The manual says a relation gives "a digit (the sign digit)". The flow
diagrams (pp. 22 and 25) show what that means in the machine. A GEORGE
number is a 32-digit word: the exponent in positions 1-10 and the
mantissa in positions 11-32, with the sign digit in position 32 (section
1.9 (vii)). `=` and `>` give a word with only the sign digit set, which as
a number is exactly -1, or a word of all zeros, which is 0. A conditional
skip looks at the sign digit and nothing else: any negative number is
truth.

The logical operators work on the whole word, digit by digit. `&` and `∨`
of -1 and 0 give -1 or 0, as expected. But `~` changes every digit, not
just the sign. Not-truth is the word with every digit set except the
sign, and not-falsehood is the word with every digit set. Neither is a
standard GEORGE number, and they punch as something odd:

```george
3, 3 = ~ (P) ;             # not truth
3, 4 = ~ (P) ;             # not falsehood
```
```console
$ george run programs/09-not.george
0.5
0
```

Not-truth is just under a half. Not-falsehood, all ones, is -2⁻²², a tiny
negative number, but Hamblin's punch routine gives that one word as 0, so
`george` does too. Negating it first shows what it is:

```george
3, 3 = ~ (P) ;             # not truth
3, 4 = ~ neg (P) ;         # not falsehood, negated
1 neg ~ ~ (P) ;            # not not truth
```
```console
$ george run programs/09-not-cards.george --cards
 500000000-000000001
 238419000-000000007
-100000000 000000000
```

The sign digits are right: not-truth is positive, so false, and
not-falsehood is negative, so true. The third card is `1 neg ~ ~`:
complemented twice, truth is -1 again. In a skip, `~` works exactly as
the manual says. This reads pairs and punches 1 where they differ:

```george
# a ≠ b written a b = ~, used in a skip.
1, 3 rep (i)
R R = ~ 1 ↑                # skip to mark 1 if the two are not equal
0 (P) ; 2 ↑
* 1 1 (P) ;
* 2 ]
```
```console
$ george run programs/09-not-skip.george --data programs/09-not-skip.in
0
1
0
```

The odd values matter only if you punch a negated truth value, or do
arithmetic with it. The manual's "-1 for truth" holds for `=`, `>`, `&`
and `∨`, and not for `~`. `DECISIONS.md` (decision 6) gives the reading
of the flow diagrams behind this.

## Two examples from the manual

"If any a(j) is zero (j = 1, ... n) obey subroutine no. 3":

```text
0, 1 n rep (j) j|a 0 = ∨ ] 3 ↓
```

The `0,` marks out a cell with falsehood, and each time round `∨` takes
in whether a(j) is zero. After the repetition the cell holds truth if any
was, and `3 ↓` is conditional because `=` came before it. Around it, ours,
a reading of n and the vector, and a subroutine 3 that punches 99:

```george
# "If any a(j) is zero (j = 1, ... n) obey subroutine no. 3."
R (n) ; 1 n R|(a)
0, 1 n rep (j) j|a 0 = ∨ ] 3 ↓
1 (P) ; ]
* 3 99 (P) ; ]
```
```console
$ george run programs/09-any-zero.george --data programs/09-any-zero.in
99
1
```

Then the program goes on to punch 1, as it would have if no a(j) had been
zero.

Iterating x = (x + y/x)/2 "until successive values differ by not more
than e":

```george
# Iterate x = (x + y/x)/2 from unity until successive values differ by
# not more than e. Setting y and e and punching the result are ours.
2 (y) ; .00001 (e) ;
1 * 0 (x) y x ÷ + 2 ÷ dup x - mod e > 0 ↑
(P) ;
```
```console
$ george run programs/09-converge.george
1.41421
```

The second line is the manual's. It begins `1 * 0`: the 1 is put in the
accumulator once, and mark 0 is after it. `(x)` names whatever is in the
last cell, the first time 1 and later the newest value. `dup x - mod e >`
compares the change with e, and `0 ↑` goes round again while it is
greater. "The result is left in the accumulator."

## Loops without rep

A skip back to a mark makes a loop that runs until a condition fails,
which `rep` cannot do. This counts the steps of the "3n + 1" sequence from
27 to 1: halve n if it is even, otherwise put n = 3n + 1.

```george
# Halve n if it is even, otherwise put n = 3n + 1, until n is 1;
# s counts the steps.
27 (n) ; 0 (s) ;
* 1 n 2 rem 0 = 2 ↑                # n even: to mark 2
n 3 × 1 + (n) ; 3 ↑
* 2 n 2 ÷ (n) ;
* 3 s 1 + (s) ; n 1 = ~ 1 ↑        # n not 1: round again
s (P) ;
```
```console
$ george run programs/09-collatz.george
111
```

`n 2 rem 0 =` is "n is even". `n 1 = ~ 1 ↑` goes round again while n is
not 1. The program is ours.

## A subroutine that calls itself

A subroutine can call itself, since `↓` only needs a mark. Each call
takes an entry in the link list until it returns. This is n!, as
n × (n - 1)!, stopping at n ≤ 1:

```george
# n! by a subroutine that calls itself.
R 1 ↓ (P) ; ]
* 1 dup 1 > ~ 2 ↑                  # n not more than 1: n! is n
dup 1 - 1 ↓ ×                      # otherwise n (n - 1)!
* 2 ]
```
```console
$ george run programs/09-factorial.george --data programs/09-factorial.in
720
```

`dup 1 > ~ 2 ↑` skips to the `]` at mark 2 when n is not more than 1,
leaving n as its own factorial. Otherwise `dup 1 - 1 ↓` calls the
subroutine on n - 1, and `×` multiplies. 6! needs six calls in progress
at once, for 6, 5, 4, 3, 2 and 1, which is the whole link list. 7! needs
seven:

```george
# The same, for 7!.
R 1 ↓ (P) ; ]
* 1 dup 1 > ~ 2 ↑
dup 1 - 1 ↓ ×
* 2 ]
```
```console
$ george run programs/09-too-deep.george --data programs/09-too-deep.in
stop 1 8-21x (IA), 1 3-3x (II): too many entries in link list (nested subroutines or repeats to higher than sixth order)
  at symbol 18 "↓", line 4, column 11
```

Next: [The functions](10-functions.md)
