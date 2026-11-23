# 14. GEORGE and what came after

This chapter is about where GEORGE stands. It is careful to say only what
the sources support, and it says where they stop.

## Hamblin's papers

Hamblin described the ideas behind GEORGE in two papers in 1957, before
the manual:

- C. L. Hamblin, "An addressless coding scheme based on mathematical
  notation", read at the first Australian conference on computing and
  data processing, Salisbury, South Australia, June 1957.
- C. L. Hamblin, "Computer languages", *The Australian Journal of
  Science*, 20 (1957).

Neither paper is among the sources of this reconstruction, which rests
on the manual, the flow diagrams, the coding sheets, and the DEUCE manuals
and library routines used to run them. The manual itself does not
cite them, or anything else; it has an acknowledgement, "to Mr G. Karoly,
and to Messrs R. Brigham and G. Bell, and to Mr R. Smart, for stimulation
and suggestions", and nothing more.

## What was new

"Polish" notation is the logician Jan Łukasiewicz's, of the 1920s: a
notation without brackets, with each operator written before its
operands. Writing the operators after their operands instead is the
"reverse" of it. The manual takes the notation for granted, as something
the programmer "must learn", and does not say where it came from.

What GEORGE added was a machine to go with the notation: the running
accumulator, a last-in-first-out store that holds the operands as they
wait for their operators. In 1957-58 that was a new idea, but not
Hamblin's alone. In Munich, Friedrich L. Bauer and Klaus Samelson worked
out the same principle, which they called the *Kellerprinzip* (the
cellar principle), as a way of translating formulae; they applied for a
patent on it in 1957. The two pieces of work were contemporary, and this
tutorial makes no claim about whether either side knew of the other.

The running accumulator is a stack, though the manual never uses the
word. The link list is a second stack, for return addresses, kept apart
from the first. `dup` and `rev` are operations on the stack itself rather
than on numbers. Those three things are the parts of GEORGE that look most
familiar now.

## The KDF9

English Electric's KDF9, first delivered in 1964, built a stack into the
hardware. Its arithmetic was done in a "nesting store" of sixteen cells:
an instruction such as add took its operands from the top of the nesting
store and left its result there, so that formulae were coded in reverse
Polish order. A separate "subroutine jump nesting store" held return
addresses, as GEORGE's link list did.

It is often said that the KDF9's designers knew of Hamblin's work. This
tutorial has not found a primary source that says so, and does not claim
it. The resemblance is plain; the line of descent, if there is one, is
not documented here.

The Burroughs B5000, announced in 1961, was a stack machine of another
kind, designed to run programs in Algol. It is mentioned only so that the
KDF9 is not taken to be the only one.

## Forth

Forth, which Charles H. Moore developed in the late 1960s and early 1970s,
has a data stack for operands and a separate return stack, writes its
arithmetic in reverse Polish, and has `DUP` and `SWAP`, which do what
GEORGE's `dup` and `rev` do. A Forth programmer would read `a b + dup ×`
at sight.

No evidence was found, in the sources examined, that Moore knew of GEORGE,
and this tutorial does not suggest that he did. The likeness comes from the same idea arrived at
again: once operands wait on a stack, an operator to copy the top and an
operator to exchange the top two are the first things anyone wants. Some
of the differences are as telling:

| | GEORGE | Forth |
|---|---|---|
| operand stack | 12 cells; overflow is a stop | usually larger |
| return stack | 6 entries, shared by subroutines and repetitions | separate, larger |
| subroutines | numbered 0 to 31, called by number | named words, defined by the programmer |
| loops | `a b rep (x) ... ]`, counting up by one | `DO ... LOOP` and others |
| conditions | a skip to a mark, conditional after a relation | `IF ... THEN` |
| numbers | floating point only | integers, usually |
| variables | 32 letters, plus vectors and matrices | named, defined by the programmer |

## Calculators

The other place reverse Polish is still met is on calculators. Hewlett-
Packard's scientific calculators, from the HP-35 of 1972, have a stack of
four registers and an ENTER key that separates two numbers typed one after
the other. GEORGE's comma did the same job fourteen years earlier, for the
same reason: two numerals in a row would otherwise be one number. Again,
this is a likeness, not a descent.

## Hamblin

Charles Leonard Hamblin (1922-1985) went on to be professor of philosophy
at the University of New South Wales. He is best known now for *Fallacies*
(1970), a book on the logic of argument. GEORGE was the work of a few
years. The manual, the flow diagrams and the coding sheets are what is
left of it.

Next: [Reference](15-reference.md)
