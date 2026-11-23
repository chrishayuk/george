# 1. Introduction

GEORGE is the General Order Generator. Charles Hamblin wrote it in 1957
and 1958 for the English Electric DEUCE at the New South Wales University
of Technology in Sydney, which became the University of New South Wales in
1958. The university's DEUCE was called UTECOM.

It let a mathematician give the machine a formula, more or less as a
formula, and have it worked out. The manual begins:

> GEORGE, or the General Order Generator, is a program for DEUCE permitting
> mathematical problems to be presented to the machine in a simple
> "addressless" instruction language, here called "G-code". To use this
> code the programmer must learn a special method of writing mathematical
> formulae, known as "reverse Polish" notation. Once this is mastered,
> however, programming is considerably easier and quicker than by other
> methods. (Manual, title page)

To see what "addressless" saved, consider what a DEUCE programmer
otherwise had to write. DEUCE kept its store in mercury delay lines and on
a magnetic drum. Each instruction named a source and a destination, and a
programmer had to know which minor cycle of which delay line a number
would be passing through when the instruction came to need it. GEORGE
hid all of that. In G-code there are no addresses at all, for numbers or
for instructions. A number is named by a letter; a place in the program
is named by a mark; and every calculation happens in one place, the
"running accumulator", which the next chapter is about.

This tutorial teaches G-code. It starts with a single sum and ends with
the worked examples of the manual, including one that has been wrong since
1958.

## The manual

Everything here comes from one document: C. L. Hamblin, *GEORGE IA and
II: A semi-translation programming scheme for DEUCE. Programming and
Operation Manual*, University of New South Wales, probably mid 1958. It is
23 typed pages. Section 1 is "Programming" and section 2 is "Operation".
This tutorial follows section 1 closely and quotes it often, because it is
short and clear. References are to its sections and to its own page
numbers, which are printed at the top of each page in brackets.

There were two versions of the basic program:

> GEORGE IA and GEORGE II accept the same programs and differ only in
> details of internal operation. GEORGE IA is a "semi-translation" scheme,
> and the program goes through two phases: (i) translation into
> "keywords", which are DEUCE instructions leading to the stored routines
> required by the program, and (ii) calculation, in which the keywords are
> taken one at a time and the appropriate routines performed. [...] GEORGE
> II is a "pure interpretive" scheme and is slower in operation but has
> considerably more facilities for manual operation: it will be used
> principally for the testing of programs for subsequent use on GEORGE IA.
> (Manual, p. 1)

The `george` program used in this tutorial obeys a program a symbol at a
time, as GEORGE II did. Where the two versions stop on different
indications, it gives both.

Two other documents survive, a set of flow diagrams and a set of coding
sheets, both handwritten. They describe the penultimate version of GEORGE
IA. Where the manual leaves a question open, the reconstruction settles it
from the flow diagrams if it can, and says so in `DECISIONS.md`. This
tutorial points out those places as it comes to them.

## Running george

You need [Rust](https://rustup.rs). In the top folder of the repository:

```
cargo build --release
```

That puts `george` in `target/release`. The commands in this tutorial are
written as if `george` were on your path and you were in the `tutorial`
folder, where the programs are.

A program is a text file of symbols. This one multiplies six by seven and
punches the product on a card:

```george
6, 7 × (P) ;
```
```console
$ george run programs/01-answer.george
42
```

Each number punched comes out on a line of its own, to the six figures
that DEUCE's punch gave it. GEORGE had no printer of its own: results
went out on cards.

`george help` says what else it can do:

```console
$ george help
george run FILE [--data FILE] [--columns] [--input N,N,...] [--keys N,N,...]
               [--cards] [--trace] [--single-shot] [--limit N]
george deck FILE        the program as punched: symbol codes, sixteen to a card
george format FILE      the program in the manual's printed form
george table            the coding table (Appendix I)
george stops            the stops (section 2.4)
george deuce ...        the DEUCE simulator (george deuce help)

run obeys a G-program and prints what it punches, one number to a line.
  --data FILE    data cards, one number to a line in ordinary notation
  --columns      ... or, with this, as the twenty card columns of section 2.3
  --input        data cards given on the command line
  --keys         numbers set on the input keys, one for each I obeyed
  --cards        print punched cards as their twenty columns
  --trace        print each symbol obeyed and the accumulator after it
  --single-shot  carry on through resumable stops, as the operator could
  --limit N      give up after N symbols obeyed (default 10000000)
```

## How to read this tutorial

A block headed `george` is a whole program. Every one of them is a file
in `tutorial/programs`, with the same name as in the command under it.
A block headed `console` shows a command and what it printed, both the
numbers punched and anything said on the error stream, such as a stop.

All of these are checked. `check_tutorial.py` runs every program and every
command with the release build and compares what it prints with the
tutorial:

```
python3 tutorial/check_tutorial.py
```

So what you see under each program is what it does.

## The symbols

The manual prints programs in mathematical symbols, some of them not on a
keyboard. `george` reads them as printed, and also has ASCII spellings:

| printed | ASCII | | printed | ASCII |
|---|---|---|---|---|
| `×` | `mul` | | `↓` | `down` |
| `÷` | `/` | | `↑` | `up` |
| `√` | `sqrt` | | `~` | `not` |
| `∨` | `or` | | `&` | `and` |
| `α β γ λ μ ν θ` | `alpha beta gamma lambda mu nu theta` | | | |

This tutorial uses the printed forms. `#` starts a comment, which runs to
the end of the line. Comments are ours; there were none on the cards.

Next: [Reverse Polish and the running accumulator](02-reverse-polish.md)
