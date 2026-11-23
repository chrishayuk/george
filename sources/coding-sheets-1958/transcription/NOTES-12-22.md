# Notes on the transcription of coding sheets 12-22

Source: `GEORGEProgramCode.pdf`, PDF pages 12-22 (the sheet numbers written on the pages agree with the PDF page numbers). Each block was read at 300 dpi, and doubtful cells at 500-800 dpi. Each sheet then had a second pass, comparing the text line by line against the 300 dpi crops. Pages 1-11 were done by another agent, and their notes are in `NOTES-1-11.md`.

## Conventions

- One line per written row: `mc NIS S D C W T go`. `-` means an empty field. Empty rows are left out.
- **Card rows.** Every block on sheets 12-22 except one has a "Y" row above the mc grid. It holds an expression like `nP1 + 14P5` or `nP1 + 15P5`, written as `Y  = ...`.
  - n is the track's position within the track group: for example, `3P1 + 15P5` heads track 15/3. P5 times 14 or 15 is the track group.
  - Sheet 17's DL6 track 15/0 has `15P5` alone. Sheet 22's DL8 track 15/15 adds `+ P32`.
  - The meaning of the Y row is inferred, not checked: it is presumably the card's identification or track-address word.
  - Only the G-program initial card (sheet 22, block 3) has instructions in the card rows. Those rows are labelled `X` and `1`, after the row labels at the right edge.
- **Constants.** These are written `mc  = <as written>`.
  - Bit lists are written as on the sheet, for example `P17-26`, `P1-7,17-21`, `4P5 + 12P17`.
  - Some constants are written as six or seven numbers separated by commas, with a raised dot or short dash before the last one, for example `26,12,30,4,8,13.1`. These are transcribed with a `.` before the last number.
    - Following READING.md (p.32), these are base-32 digits, least significant first; the last one is the top 2 bits.
    - I checked the decoding: `26,12,30,4,8,13.1` = 1518500250 = √2·2³⁰.
- **W and T in brackets.** Some instructions have a bracketed number between W and T, for example `2 (15) 23`. These are transcribed as `W(n)`, e.g. `2(15)`, and a note gives the form as written.
- **Stops.** `x` in the go column is a stop. On sheet 22 the go column also holds `go` and `go + P34`, written small as "go" over "+P34", and transcribed `go+P34`.
- **Doubt.** A doubtful line has ` ?` after the go field, and the note says what is doubtful. Struck entries are described in notes.

## Doubtful entries

There are 9 doubtful lines in total:

| Sheet | Doubtful | Lines |
|---|---|---|
| 12 | 1 | DL7 14/1 mc 12: S 25 is written over another figure |
| 13 | 1 | DL7 14/3 mc 24: a slanting stroke in the go column, not read as a stop |
| 14 | 1 | DL5 14/7 mc 15: the same kind of stroke |
| 15 | 1 | DL7 14/10 mc 1: W is 6 or 0 |
| 16 | 0 | |
| 17 | 0 | |
| 18 | 1 | DL8 15/4 mc 8: T 22 replaces a struck value, 29 or 24 |
| 19 | 0 | |
| 20 | 0 | |
| 21 | 3 | DL8 15/13 mc 20 and mc 21 are heavily amended or struck; mc 30 has a dot in the C column |
| 22 | 1 | Block 3 mc 19: faint figures under "Diagnostic card" |

Struck items that are not otherwise doubtful:

- Sheet 18, DL8 15/4: the whole of mc 24 (`26,12,30,4,8,13.0`) is struck and written again in mc 25.
- Sheet 22, block 3: two diagonal lines run across rows X to mc 7. They may strike those rows, or the firm one may be a flow arrow. The note in the file says so.

## What each sheet holds

All the sheets have the printed heading "UTECOM Programme GEORGE" and a sheet number. The only other title is on sheet 22, block 3, "G-program initial card."

I identified routines by matching the coding-sheet instructions against the flow diagrams. The sheet instruction `NIS S D C W T` at DL n, mc m corresponds to the flowchart's `n_m S-D`. Where the flowchart gives a source with a word number, it equals S at m + 2 + W.

Many matches were exact. In particular, p.42 has "14/0" written in its margin, and every instruction on it is on sheet 12 block 1 or block 3.

| Sheet | Block | Track | Contents | Flowchart | Basis |
|---|---|---|---|---|---|
| 12 | 1 | DL7 14/0 | `R\|`, `P\|` assemble | p.42 | Matched instruction by instruction, e.g. 7_12 7_14-15 with 7_14 = 31P22, and 7_24 11-8(32) |
| 12 | 2 | DL7 14/1 | Probably `R\|\|`, `P\|\|` assemble | p.43 | Not checked line by line. Inferred from the track order and the same pattern of fetches |
| 12 | 3 | DL8 14/2 | `R\|`, `P\|` modify | p.42 | Matched exactly: 8_28 16-1_31 through 8_18 21-8_20,21(2), with the constants P17-26 at 8_8, 13-11 at 8_24, 31P17 at 8_30 via 10_30, 15P5 at 8_0, and "8-31; 0-30" at 8_26,27 |
| 13 | 1 | DL7 14/3 | `P\|` and `P\|\|` | p.41 | Matched: 7_4 4-30, 7_7 5-30, 7_8 6-30, 7_9 15-31, 7_10 0-30, 7_11 11-6(32), 7_21 11-1(32), 7_0 10-24, 7_12 7_14-16, 7_13 7_15-16, 7_18 8_20-13, 7_20 7_24-25, 7_23 8_3-13 |
| 13 | 2 | DL1 14/4 | Parts of the punch loop and punch | pp.41, 47-48 | 1_6 11-3(32), 1_30 11-4(32) and 1_0 11-5(32) are in p.41's "call punch loop". The rest was not matched |
| 13 | 3 | DL3 14/5 | Punch translate | p.47 | Has log₁₀2 at 32 b.p., `2,10,19,0,17,6.1` (mc 2), and log₂10 at 30 b.p., `12,2,30,20,9,10.3` (mc 19). There is a stop at mc 28 |
| 14 | 1 | DL4 14/6 | Punch translate (continued) and punch | p.48 | 4_26 4_28-23 with `9/10 × r.o.` = `31,27,18` at mc 28. Punch is 4_31 and 4_11 28-26(31), 4_10 16-0. Rows 0-7 hold the coefficient table described below |
| 14 | 2 | DL5 14/7 | Punch translate | pp.47-48 | `r.o.` = `3,31,20` at mc 6, added by 5_4 5_6-22. 10⁻² at 31 b.p., `20,16,11,15,20,0.0`, at mc 31 |
| 14 | 3 | DL7 14/8 | Reset after read and punch | p.49 | Matches 7_2 12-31, 7_16 27-18_2,3 (C 2), 7_20 7_31-13, and 7_21-7_30 (n-30, 11-n(32)). Two differences: the sheet has 7_17 30-18 where the flowchart has 8-30, and 7_19 11-8(32) where the flowchart has 11-7(32) |
| 15 | 1 | DL8 14/9 | Probably `R\|\|`, `P\|\|` modify | p.43 | Laid out like sheet 12 block 3 (same 13-11 constant at mc 24, 16-1 at mc 28). Not checked line by line |
| 15 | 2 | DL7 14/10 | `R\|`/`R\|\|`, and the start of "call read loop and read" | pp.40, 44 | p.40: 7_4 7_7-16, 7_11 9-24, 7_12 8_3-13, 7_5 8_21-13, 7_21 7_24-25, 7_3 18_3-28, 7_13 14-31, 7_0 8-30, 7_15 11-7(32), 7_14 7_20-16 [Y row] with stop. p.44: 7_8 11-30, 7_9 11-1(32), 7_16 12-30, 7_17 11-3(32), 7_1 12-24, 7_18 13-30, 7_29 11-4(32) |
| 15 | 3 | DL1 14/11 | Read loop and read (DL1 part), plus a read-translate constant | p.44 | 1_0 11-6(32)x [Y row], 1_16 0-1_22x [signs], 1_22 14-30, 1_23 11-5(32), 1_30 15-31, 1_31 0-30, 1_28 13-0. The constant 2²⁰·10⁻⁵ at 26 b.p., `18,24,26,2,31,20.0`, is at mc 10 (p.45) |
| 16 | 1 | DL3 14/12 | Read translate | p.45 | P11-20 and P1-10 at mc 26-27, which separate mantissa and exponent digits |
| 16 | 2 | DL4 14/13 | Read (card rows to bits), and the coefficient table at 4_12-4_19 | pp.44, 46 | 4_7 0-25x, 4_10 16-1_30x, 4_31 0-21x, 4_11 0-21x, 4_1 0-22(2)x, 4_3, 4_4 and 4_5 x, 4_6 0-25 all match p.44. Rows 12-19 are the "Coefficients" of p.46 |
| 16 | 3 | DL5 14/14 | Read (right-hand column of p.44) and read translate | pp.44, 46 | 5_9 0-22(2), 5_11 21-19(2), 5_12 0-22(2)x, 5_15 2-24, 5_21 21-20(2). log₂10 is at mc 3 |
| 17 | 1 | DL8 14/15 | Not identified | | Track 14 code ending at mc 29 with a stop `31-30 C3 x` |
| 17 | 2 | DL6 15/0 | The DEUCE library routines | pp.32-33 | A note: "[Deuce subroutines S06 and M12, version 6, with additions as shown]". Only two additions are written: mc 10 `1 21 14 1 13` and mc 28 `1 27 22 1 27`. The library code itself is not on the sheet |
| 17 | 3 | DL7 15/1 | `rem` | p.36 | 7_26 19_3-26, 7_29 13-27, 7_31 30-16x (the "divisor negative or zero" stop), 7_13 19_3-23_3, 7_11 21_3-27, 7_15 30-16, 7_9 = Q30 16-18_1 |
| 18 | 1 | DL8 15/2 | `rem` (continued) | p.36 | 8_1 21_3-28, 8_17 19_3-22_3, 8_23 19_3-22_3 |
| 18 | 2 | DL7 15/3 | `log` | p.32 | ln 2 at 31 b.p., `28,31,2,18,11,12.1` (mc 20), and 7_24 0-24 |
| 18 | 3 | DL8 15/4 | The "log" subroutine | p.32 | 8_8 30-21(2)x stop. √2 (30 b.p., mc 9), 1/√2 (30 b.p., mc 19) and √2−1 = `26,12,30,4,8,13.0` (30 b.p., mc 25) |
| 19 | 1 | DL11 15/5 | Tables for S06/M12 | | Two eight-word tables of constants, described below |
| 19 | 2 | DL7 15/6 | `exp` | p.33 | [9P18] at mc 15, [16P18] at mc 24, and 7_20 30-21(2)x, the overflow stop |
| 19 | 3 | DL8 15/7 | The "exp" subroutine | p.33 | [11P5+21P17] at mc 14, [7P17] at mc 17, and log₂e at 31 b.p. (unsigned) at mc 12, `10,25,14,20,10,28.2` |
| 20 | 1 | DL7 15/8 | `pow` | p.34 | 7_5 30-21(2) |
| 20 | 2 | DL5 15/9 | `pow` | pp.34-35 | 5_7 30-21(2)x, the stop for a fractional power of a negative number. 5_12 30-1_1 and 5_13 29-1_1 set the "index odd" flag |
| 20 | 3 | DL8 15/10 | `pow` (continued) | p.35 | 8_1 0-24 |
| 21 | 1 | DL7 15/11 | `√` | p.37 | 7_1 20_3-27, 7_7 1-24, 7_9 25-28, 7_11 4-24x, 7_14 16-23_2, 7_16 27-22_2, 7_18 22_2-28. P23 and P31 at mc 5 and 8 |
| 21 | 2 | DL7 15/12 | `sin` / `cos` entry | p.38 | 7_4 29-19 (sin) and 7_6 30-19 (cos). The constant 2/π is at mc 29 |
| 21 | 3 | DL8 15/13 | `sin` / `cos` polynomial | p.38 | 8_15 0-24 (multiply). The coefficients are at mc 1, 3, 5, 7 and 13, and P31 at mc 31 |
| 22 | 1 | DL7 15/14 | Not identified | possibly pp.51-52 | Many stops sending to destination 29 (`17-29 x`, `10-29 x`, `2-29 x`). The diagnostic punch routine is a guess |
| 22 | 2 | DL8 15/15 | Not identified | | Y row `15P1 + 15P5 + P32`, the last track of group 15 |
| 22 | 3 | none | "G-program initial card" (mc X to 7, ending `go + P34`), then a second card from mc 9 to 17, then "Diagnostic card" (mc 19 to 31, ending `go + P34`) | pp.3, 50 | Mc 8 reads "Present Initial Card for" |

## Findings on the open questions in DECISIONS.md

### sin and cos coefficients: found (sheet 21, DL8 track 15/13)

Mc 1, 3, 5, 7 and 13 hold five base-32 constants. Read as signed 32-bit words at 30 binary places, they are:

| m.c. | As written | Value (30 b.p.) | Taylor term of cos(πx/2) |
|---|---|---|---|
| 1 | `12,7,3,22,16,24.2` | −1.2337005325 | −(π/2)²/2! = −1.2337005501 |
| 3 | `20,17,7,24,3,8.0` | +0.2536693104 | +(π/2)⁴/4! = +0.2536695079 |
| 5 | `17,28,11,20,10,31.3` | −0.0208626827 | −(π/2)⁶/6! = −0.0208634808 |
| 7 | `11,8,2,30,0,0.0` | +0.0009176834 | +(π/2)⁸/8! = +0.0009192603 |
| 13 | `4,1,7,31,31,31.3` | −0.0000238083 | −(π/2)¹⁰/10! = −0.0000252020 |

DL8 mc 31 holds P31, which is 1.0 at 30 b.p. The polynomial 1 + Σ cₖ x²ᵏ over these five coefficients matches cos(πx/2) on [0, 1] to within **3.0·10⁻⁸** (I evaluated it at 1001 points). The coefficients differ from Taylor's in the way an economised or minimax fit does, so I am confident the reading is right.

DL7 15/12 mc 29, `14,27,0,19,15,17.2`, is **2/π** read as unsigned at 32 b.p. (0.63661977). It is presumably the factor that converts the argument to quarter-periods.

*Inference:* the routine computes cos(πx/2) for x reduced into the unit quarter-period, and sin is cos shifted by one quarter-period. The flag word 19_2 is set to P32 for sin and to 0 for cos (p.38). Which coefficient goes with which power is inferred from the descending magnitudes and alternating signs, not traced through the code. READING.md says p.38 takes the coefficients from DL8 "m.c. 14, 20, 4 and 3"; the sheet has them at 1, 3, 5, 7 and 13. I did not reconcile the two.

### Punch rounding constant

The sheets confirm the flowchart digits:

- **r.o.** `3,31,20` (DL5 14/7 mc 6) = 21475, which is 1.0000·10⁻⁵·2³¹. It is added by 5_4 5_6-22.
- **9/10 × r.o.** `31,27,18` (DL4 14/6 mc 28) = 19327, which is 0.9·10⁻⁵·2³¹. p.48 shows it used by `4_26 4_28-23_2`. Destination 23 is the subtract counterpart of 22 in other routines, so on that branch it may be *subtracted*. That reading of 23 is inferred.
- **10⁻²** `20,16,11,15,20,0.0` (DL5 14/7 mc 31) = 21474836, which is 0.01·2³¹ truncated.

21475 is 10⁻⁵·2³¹ (21474.84) rounded up, and 19327 is 0.9·10⁻⁵·2³¹ (19327.35) rounded down. So the constants are values chosen to fit, not computed by formula. I could not establish what scaling the decimal mantissa has at the point where r.o. is added, so I cannot say whether r.o. is exactly half a unit in the sixth figure.

### `rem`'s divisor

`rem` is on sheet 17 (DL7 15/1) and sheet 18 (DL8 15/2), and it agrees with p.36:

- 7_6 (`6 7 13 - 1 15`) loads the quasi at 7_9 (`16-18_1`) into TS13 as a link and goes to DL6.
- On the flowchart, that is where 6_23 "fetch integer" is. At run time DL6 is presumably the arithmetic track, although sheet 17 shows track 15/0 (S06/M12) loaded into DL6, so DL6 is not always that track.
- The stop 30-16 x (7_31) is there.

The coding sheets do **not** add evidence on whether a fractional divisor was truncated; they repeat the flowchart's structure. The open question stays where READING.md left it.

### Rounding in standardise, add, multiply and divide; `dup` and TS16

These routines (flowchart pp.16-21) are not on sheets 12-22. They should be on pages 1-11 (see `NOTES-1-11.md`). Nothing here bears on them.

### Other constants decoded

All decodings are base-32, least significant digit first, as two's-complement 32-bit words. The binary-point positions are the ones that give recognisable values.

- **Coefficient table A.** It appears three times, digit for digit the same: DL4 14/6 mc 0-7 (punch), DL4 14/13 mc 12-19 (read, the p.46 "Coefficients"), and DL11 15/5 mc 21-28.
  - At 31 b.p. the values are 0.70710678, 0.24506454, 0.04246645, 0.00490592, 0.00042506, 2.946·10⁻⁵, 1.709·10⁻⁶, 8.48·10⁻⁸.
  - These are 2^(−1/2)·(ln2/2)ⁿ/n! for n = 0, 1, 2, 3, ..., so the series sums to 2^((y−1)/2).
  - *Inference:* this is S06's exponential (power-of-2) series.
- **Coefficient table B.** DL11 15/5 mc 13-20.
  - At 30 b.p. the values are 1.44269500, −0.72134506, 0.48083524, −0.35985442, 0.28244662, −0.21291324, 0.12848994, −0.04245950.
  - The first two equal log₂e and −log₂e/2. The third is log₂e/3 to about four figures, and later terms depart more and more from log₂e·(−1)ⁿ⁺¹/n; the fifth is 0.2824, against 0.2885.
  - *Inference (medium confidence):* this is an economised log₂(1+x) series, presumably M12's.
- **log₂e** (DL8 15/7 mc 12) at 31 b.p. is 1.4426950412, against the true 1.4426950409.
- **ln 2** (DL7 15/3 mc 20) at 31 b.p. is 0.6931471806.
- **2²⁰·10⁻⁵** (DL1 14/11 mc 10) at 26 b.p. is 10.48576000.

### DEUCE source and destination numbers

Nothing on these sheets settles a source or destination number beyond what READING.md already infers.

I used one structural fact that the evidence supports. The sheet instruction at mc m with wait W takes a DL source at minor cycle m + 2 + W, and that is how the flowchart's `n_m S_k-D` subscripts line up with the sheet rows. It held for every instruction I compared, for example 7_5 8_21-13, which is `7 8 13 - 14 14` at mc 5.
