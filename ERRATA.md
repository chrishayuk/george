# Errata in the manual

What running the manual's programs found.

## `simpson`: Example 3's coefficients are a quarter of what they should be

Example 3 (pages 7-8) integrates exp(-x²/2) / (1 + exp(-ax)) from 1 to 5 by
Simpson's rule with 16 intervals. The argument is worked out as
`j 4 ÷ 1 +`, so the step is h = 1/4. Simpson's rule weights the ordinates
by h/3 = 1/12, 4/12, 2/12, ... 1/12. The manual says the "Simpson vector"
holds "1/48, 4/48, 2/48, 4/48, ... 1/48", and its program for preparing the
vector begins `1, 48 ÷`. Run as printed, every result comes out a quarter
of what Simpson's rule gives:

| a | as printed | with 1/12 | Simpson in double precision |
| --- | --- | --- | --- |
| 0 | 0.0497078 | 0.198831 | 0.198831 |
| 0.5 | 0.0675882 | 0.270353 | 0.270352 |
| 1 | 0.0807849 | 0.323139 | 0.323139 |
| 2 | 0.093649 | 0.374596 | 0.374596 |
| 4 | 0.0988802 | 0.395521 | 0.395521 |

The last column is Simpson's rule with the same 17 ordinates in double
precision. The difference in the sixth figure at a = 0.5 comes from
GEORGE's arithmetic: + and - do not round to nearest. These are the values
Hamblin's own code punches on the simulator, and the interpreter now does
its + and - in the same way
([RUNNING-GEORGE.md](sources/deuce/RUNNING-GEORGE.md), finding 15). Simpson's rule with this step is itself out from the exact
integral in the fifth figure (0.198845 at a = 0).

Everything else in the program is right; only the constant is wrong. 1/48
would be the weight for a step of 1/16, which suggests the example was
changed from another range at some point. That is a guess.

`examples/simpson.george` is as printed; `examples/simpson-corrected.george`
has 12 for 48.

# Places where the coding sheets contradict the flow diagrams

Found by running Hamblin's own code, the transcribed coding sheets, on the
DEUCE simulator (`george deuce george`). Each value was checked on a 300 dpi
render of the scan and is clearly as written; each contradicts the flow
diagram, and with the flow diagram's version the programs run. They are
applied at run time only (`george::CORRECTIONS`; `GEORGE_AS_WRITTEN=1` turns
them off); the transcription keeps what the sheet says. The sheets are the
penultimate version, so these may well have been put right in the last one.
The fourth is plainly so: the flow diagram carries Hamblin's own amendment,
and the sheet the value from before it.
Details and evidence: [RUNNING-GEORGE.md](sources/deuce/RUNNING-GEORGE.md),
findings 5, 8, 9 and 16.

| Where | As written | Flow diagram | Effect as written |
| --- | --- | --- | --- |
| Sheet 3, DL5 track 12/5, m.c. 29 | T = 1, on into multiply | p.20: T = 0, on to 4_31 | `3, 4 ÷` punches zeros without end |
| Sheet 12, DL8 track 14/2, m.c. 18 | T = 1, to 1_21 | p.42: T = 11, to the link planted at 1_31 | `R|` and `P|` do nothing |
| Sheet 17, track 15/0, m.c. 28 (Hamblin's addition to S06/M12) | NIS 1, T 27, to 1_25 | p.47: on to 6_10 | every number below about 0.25 punches zeros without end |
| Sheet 8, DL5 track 13/4, m.c. 6 (the keyword template for `wait`) | `15-30 → 3_8`, clear at 600 dpi | p.9: amended by hand to `14-30 → 1_3` (14 over 15, 1_3 over 3_8), which reaches the wait routine of p.23 on track 12/14 | `wait` reads the skips' track and falls into an empty keyword |

Two other readings were settled by running, both on cells the transcriber
had already marked as doubtful, and are recorded in the transcription with a
note: sheet 1, block 2, card row 1 (T read as 27, not 29, or the basic pack
does not load) and sheet 21, DL8 track 15/13, m.c. 21 (struck and blank;
resolved from flow diagram p.38).

# Behaviour of this version that the manual does not describe

These are as the code is written, not slips of transcription as far as can
be told. `george run` follows the manual for the first and third, and now
copies the second and fourth, which the manual does not rule out.

- **`max`** returns the larger-exponent operand with the sign of a - b, so
  `2, 5 max` punches -5. The flow diagram for `max` (p.21) carries a hand
  amendment; trying it fixes this case and breaks `7, 3 max`, so it is not
  applied.
- **`pow` of minus a power of two** (-1, -2, -4, -0.5) stops on 1 30-21x
  ("argument zero or negative (log)"): negated, the standardised mantissa -1
  becomes +1, which the log routine's normalising shift carries into the sign
  digit. Other negative bases work: (-3)² = 9, (-3)³ = -27. `george run`
  now does the same, since pow is done word for word (RUNNING-GEORGE.md,
  finding 22); resumed with a single shot, it gives 1.
- **`√` of a negative number, resumed with a single shot,** gives 0.5, not
  the zero the manual's stop table promises ("S.S. resumes giving result
  zero"): the stop (flow diagram p.37, `7_11 4-24X`) goes on to store, and
  TS16 still holds the routine's first approximation, P31, which is 0.5.
  `george run --single-shot` gives 0, as the manual says.
- **Punching `~` of falsehood** (the all-ones word) gives 0, where the value
  itself is -2⁻²². `george run` now punches it the same way.
