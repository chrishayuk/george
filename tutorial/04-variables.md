# 4. Variables and names

## The alphabet

> Variables represent storage locations. An "alphabet" of 32 letters is
> provided, comprising all letters of the Roman alphabet except "o", and
> certain Greek letters (see coding table in Appendix I). (Section 1.3)

The 32 letters, in the order of the coding table, are:

```text
a b c d e f g h i j k l m n θ p q r s t u v w x y z α β γ λ μ ν
```

θ stands where o would be. There is no o, presumably because it would be
too easily taken for a nought; the manual does not say.

A letter alone is a *variable*. Writing it puts its value into the first
vacant cell of the accumulator, as a numeral does. The 32 letters have 32
locations, the "unsuffixed variable" store. Every location starts at
zero:

```george
q (P) ;
```
```console
$ george run programs/04-zero.george
0
```

## Names

To put a number into the store, you *name* it:

> When a number has been calculated in the accumulator it may be put away
> in the number store by using a "name" symbol. This is equivalent to
> "naming" (or renaming) the number with a certain letter or suffixed
> letter. A name-symbol is written as the corresponding variable in
> brackets, e.g. "(a)" (treated as a single symbol). (Section 1.4)

`(a)` is one symbol, not three. It copies the number in the last occupied
cell into the location for a.

```george
3 (a) ; 4 (b) ;            # put a = 3 and b = 4
a b + (c) ;                # put c = a + b
c (P) ;
```
```console
$ george run programs/04-names.george
7
```

The program reads like the manual's own examples: "Put c = a + b" is
`a b + (c) ;`. There is no assignment statement in G-code; there is a
formula, and then a name for its result.

## A name does not cancel

> A name does not, however, "cancel" the last occupied cell of the
> accumulator. For this purpose a special symbol ";" is used if required.
> (Section 1.4)

That is why every name so far has had `;` after it. Leaving the `;` off
is sometimes exactly what is wanted. Here x is set to 5 and the 5 is
squared without fetching it again:

```george
5 (x) dup × (P) ;          # (x) leaves the 5 where it was
x (P) ;
```
```console
$ george run programs/04-name-keeps.george
25
5
```

"Add 1 to n" is `n 1 + (n) ;`:

```george
6 (n) ;                    # "Put n = 6"
n 1 + (n) ;                # "Add 1 to n"
n (P) ;
```
```console
$ george run programs/04-add-one.george
7
```

## Letters written together

The manual writes `ab+` as freely as `a b +`, and so may you. A run of
letters is read as one variable for each letter:

```george
3 (a) ; 4 (b) ;
ab× (P) ;                  # a b ×
abab+++ (P) ;              # a b a b + + +
2 (α) ; α α × (P) ;        # alpha squared
2 (alpha) ; alpha (P) ;    # the same letter, spelt in ASCII
```
```console
$ george run programs/04-letters.george
12
14
4
2
```

`abab+++` is a + (b + (a + b)), which is 14. The Greek letters can be
typed as they are or spelt out; `alpha` is α.

There is one catch. A run of letters that makes one of the words of
G-code is read as that word. `max` is the maximum operator, not m, a and
x:

```george
2 (m) ; 3 (a) ; 4 (x) ;
max + + (P) ;              # meant as m a x + +
```
```console
$ george run programs/04-max.george
stop 1 30-21x (IA), 1 1-1x (II): program uses too many accumulator cells or has instructed taking number from empty accumulator (accumulator empty)
  at symbol 9 "max", line 2, column 1
```

The stop is on `max`, which found an empty accumulator. Write `m a x` with
spaces when that is what you mean. The words to watch are `dup`, `rev`,
`rep`, `neg`, `mod`, `max`, `log`, `exp`, `pow`, `rem`, `sin`, `cos` and
`wait`, and the ASCII spellings `mul`, `div`, `up`, `down`, `and`, `not`,
`sqrt` and the Greek names. Since there is no o, `or` and `not` could never
be letters anyway.

And there is no o:

```george
2 (o) ;
```
```console
$ george run programs/04-no-o.george
george: programs/04-no-o.george: line 1, column 3: (o) is not a name: a name is one letter of the alphabet in brackets
```

## Two formulae from the manual

Section 1.2's two formulae, 3a + 4b and .14(a + 33.8), with a = 1 and
b = 2:

```george
# 3a + 4b and .14(a + 33.8), section 1.2, with a = 1 and b = 2.
1 (a) ; 2 (b) ;
a 3 × b 4 × + (P) ;
a 33.8 + .14 × (P) ;
```
```console
$ george run programs/04-manual-formulae.george
11
4.872
```

## Other stores

On the machine, the unsuffixed store was cleared when a program was put
in with the "initial input" key, and the other stores by a special card
in the basic pack. Programs stacked one behind another kept the store,
unless that card was put between them: "The accumulator is cleared
between programs, but the number store is not: hence a program too large
for the instruction store may be sectionalised, later sections using the
results of earlier" (section 2.1). `george` runs one program at a time,
so every run starts with a clear store.

The unsuffixed store is one of three. The other two hold vectors and
matrices, and are the subject of the next chapter.

Next: [Suffixes](05-suffixes.md)
