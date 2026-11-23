# Notes on the transcription of coding sheets 1-11

Source: `GEORGEProgramCode.pdf`, PDF pages 1-11 (the handwritten sheet
numbers 1-11 agree with the PDF page numbers). Each page was rendered at
300 dpi, grey, and read block by block from crops, with 5-6x zooms on
every cell that looked doubtful. A second pass compared every line of each
`sheet-NN.txt` against fresh crops of the image. The checking pass made
no changes to values. It added notes to sheet 06 and sharpened two doubt
notes on sheets 01 and 03.

Pages 12-22 were transcribed separately, in the same format.

## Conventions

- One line per written row, `mc NIS S D C W T go`, with `-` for an empty
  field. Empty rows are not listed, so a gap in the mc numbers means the
  row is blank on the sheet.
- Rows above the mc grid (the card rows Y, X, 0, 1) are under
  `# card rows`, labelled with the card-row label at the right edge of
  the form. On every sheet from 1 block 3 onwards the only such row is
  Y, holding the track constant (`n P1 + 12P5` or `n P1 + 13P5`). Sheet 1
  blocks 1 and 2 (the initial card and the loader) have instructions in
  the card rows. There, two rows labelled `1` appear in the file: the
  card row, under `# card rows`, and mc 1, under `# mc rows`.
- Constants are written `= expression`, copying the subscript notation:
  `P1-16` means digits P1 to P16, `P18,22-32` means P18 and P22 to P32,
  and `31P17` means 31 × P17.
- Extra marks are given in the note, not in the fields:
  - **A bracketed number between W and T**, e.g. `0 (6) 29`, `1 (15) 14`
    or `0 (11) 7`. Its meaning is not established. It occurs mostly on
    rows with S or D 0, 1, 2 or 3, or on stop rows.
  - **Ticks or slashes after T.** These are concentrated on sheet 8
    block 1. They look like checking marks, not "go" marks.
  - **Dots between fields**, which look like pen rests.
  - **`+P1`** written beside T on sheet 11 (block 2 mc 5, block 3 mc 25),
    shown in the go column.
- `x` in the go column is the stop cross as written. `go+P34` on sheet 1
  is the written "go + P34" (see the doubt list).
- `?` at the end of a line marks a doubtful reading. The note says which
  field is in doubt. Struck entries are given as `# struck: ...`.

## Doubtful and struck entries

| Sheet | Doubtful (`?`) | Struck notes | What |
|---|---|---|---|
| 01 | 3 | 0 | b1 mc 7 and mc 31: the subscript of "go + P34" runs into the cell edge. b1 mc 20 NIS: a blot. b2 card row 1 T: 29 or 27. |
| 02 | 1 | 0 | b3 mc 6 D: 30, written over something (2?). |
| 03 | 2 | 3 | b3 mc 3 T: 5 struck, 6 written. b3 mc 31 S: 18, heavy 8. b1 mc 31: S 24, D 22 and W 0 struck, replaced by 27 23 and W 1. b3 mc 10: whole row "7 30 16 - 0 4" struck, nothing put in its place. |
| 04 | 0 | 0 | |
| 05 | 0 | 0 | |
| 06 | 0 | 0 | Notes only: b2 mc 13 `31P17` (could be read P7), b3 Y `13P5`. |
| 07 | 0 | 0 | Note: b3 mc 10 and mc 11 both have T 22, as written. |
| 08 | 1 | 0 | b1 mc 0 C: an inked-out blot. b2 heading: track "13/5" with the 3 overwritten. |
| 09 | 1 | 1 | b2 mc 6 T: a figure blacked out, 7 written above. b2 mc 13 T: 10, with the 1 written over something. |
| 10 | 1 | 1 | b3 heading: DL "8" written above a struck "13/12". b3 mc 27 S: 25, written over something. |
| 11 | 1 | 0 | b2 mc 24 T: 1 with an extra stroke. |
| **Total** | **11** | **5** | Out of about 950 written rows. |

## What each sheet holds

The identifications below come from matching the instructions on the
sheets against the instructions quoted in `../../flowcharts-1958/READING.md`,
written `DL_mc S-D` (for example `3_30 21-28`). A sheet row
`mc NIS S D ...` in block DL matches `DL_mc S-D`. About 140 instruction
references quoted in READING.md were tested against sheets 1-11, and over
80 matched exactly. Most misses are routines that are not on these sheets (rem, log,
exp, sqrt, sin and cos, read and punch), which are presumably on sheets 12-22.

| Sheet | Block | DL / track / cards | Contents (flowchart page) | Basis |
|---|---|---|---|---|
| 1 | 1 | -/-, card 0 | "Basic pack initial card", with "Clear suffixed variable store" from mc 19 (p.1). | Heading written on the sheet. |
| 1 | 2 | DL1, cards 1-3 | Basic pack loader cards 1-3 (p.2). | Card numbers, and the pack order. |
| 1 | 3 | DL1, 12/1, cards 4-6 | Calculation control: NK/NKD, accumulator fetch and store, unsuffixed names (pp.16-17). | `1_4 13-14`, `1_5 13-20`, `1_6 13-28`, `1_9 17-21` (W0, so word 3), `1_11 10-22` (W12, so 10_25), `1_14 30-21 x` (accumulator spill stop). |
| 2 | 1 | DL7, 12/2, cards 7-9 | "Long operations": dispatch on the second keyword (p.31). | Heading written on the sheet. |
| 2 | 2 | DL3, 12/3, cards 10-12 | Unpack, standardise, the exponent-overflow stop, divide (pp.18, 20). | All 23 standardise and unpack references match, plus `3_9 14-28`, `3_14 4-24 x` and `3_28 13-0`. |
| 2 | 3 | DL4, 12/4, cards 13-15 | Add and subtract (p.19). | `4_4 20-26`, `4_16 19-20`, `4_19 21-19`. `4_9` and `4_12` do not match, but READING.md calls those labels uncertain. |
| 3 | 1 | DL5, 12/5, cards 16-18 | Add, continued, and multiply (pp.19-20). | `5_16 4-25` (the `[23P22]` constant is at 4_18 on sheet 2), `5_23 21-22`, `5_28 10-1`. |
| 3 | 2 | DL6, 12/6, cards 19-21 | Not identified. Possibly mod, neg or max (pp.19, 21). | No direct match. |
| 3 | 3 | DL7, 12/7, cards 22-24 | Relations `=` and `>`, and `max` (pp.21-22). | `7_2 21-27`, `7_3 21-27`, `7_11 29-16`, `7_12 30-16`, `7_16 31-18`, `7_23 7-15`, `7_26 26-28`. |
| 4 | 1 | DL7, 12/8, cards 25-27 | Not identified. | |
| 4 | 2 | DL8, 12/9, cards 28-30 | Not identified. | |
| 4 | 3 | DL7, 12/10, cards 31-33 | Logical `~`, `&`, `∨` (p.22). | `7_0 26-16`, `7_15 25-16`, `7_5 13-16`. |
| 5 | 1 | DL8, 12/11, cards 34-36 | "Store link list" (p.24). | `8_8 28-13`, `8_30 8-21`. |
| 5 | 2 | DL7, 12/12, cards 37-39 | End bracket `]` (p.23), and rep on recall (p.27). | `7_7 13-27`, `7_16 10-31`, `7_17 13-30`, `7_21 13-31`, `7_26 21-10`, `7_29 7-15`, `7_14 25-28`, `7_18 1-1`, `7_27 27-26`, `7_20 14-22`. |
| 5 | 3 | DL8, 12/13, cards 40-42 | rep on recall, continued (p.27). | `8_23 27-14`, `8_13 10-31`. |
| 6 | 1 | DL7, 12/14, cards 43-45 | rep, first entry (p.26), and `wait` (p.23). | 11 of 11 p.26 references match, and `7_18 16-29 x` matches wait. |
| 6 | 2 | DL7, 12/15, cards 46-48 | Skips `↓` and `↑` (p.25). | `7_1 18-27`, `7_14 30-18`, `7_15 18-27`, `7_30 30-18`, `7_2 25-29` (the "skip to non-existent mark" stop loop). |
| 6 | 3 | DL1, 13/0, cards 49-51 | Start of the translator tracks (13/x). Probably commence translation and CIN (pp.6-7). | Inference from the track numbering and the masks P18,22-32 and P1-4,22. |
| 7 | 1-2 | DL2 13/1 and DL3 13/2, cards 52-57 | Translator (pp.5-11), not pinned down. | |
| 7 | 3 | DL4, 13/3, cards 58-60 | The decode ladder of flow diagrams p.9: m.c. k is "5_(k+2)-25", on to SNT/SNWT. Identified later (RUNNING-GEORGE.md, finding 16). | m.c. 2 `5_4-25 -> 1_2` (`;`), m.c. 4 `5_6-25 -> 1_3` (wait), as p.9. |
| 8 | 1 | DL5 13/4, cards 61-63 | The keyword templates of p.9 ("keyword D.L.5 as Q_0"). Identified later (RUNNING-GEORGE.md, finding 16). | m.c. 4 `4_23-15 -> 1_25` (`;`), 13 `7-30 -> 1_3` (max), 14-17 (dup, rev, =, >), 23-25 (`]`, `↓`, `↑`) match p.9. m.c. 6 (wait) is `15-30 -> 3_8`, the reading p.9 amends to `14-30 -> 1_3`. |
| 8 | 2 | DL6 13/5, cards 64-66 | Translator, not pinned down. | |
| 8 | 3 | DL7, 13/6, cards 67-69 | Translator, numerals (p.12). | `7_6 24-19`, `7_7 16-25`, `7_8 28-26`, `7_9 1-24`, `7_17 23-14`, `7_18 29-19`, `7_20 4-24`, `7_22 24-27`. The bracketed constants of p.12 are where the timing rule puts them: `7_0 7-16` (W 1) reads mc 3, which holds `10P28`, and `7_13 7-15` (W 0) reads mc 15, which holds `P21`. |
| 9 | 1 | DL8, 13/7, cards 70-72 | Numerals, continued (p.12). | `8_0 21-22`, `8_9 19-21`, `8_11 22-19`, `8_14 27-22`, `8_15 25-28`, `8_16 22-14`, `8_17 25-25`, `8_26 28-26`. |
| 9 | 2 | DL7, 13/8, cards 73-75 | Translator, not identified. | |
| 9 | 3 | DL10, 13/9, cards 76-78 | Mostly constants and masks, loaded into DL10 (P1-10, P5, P17,22, P23,28, P22,27, P22, 31P17), with a few instructions, two of them stops. | |
| 10 | 1 | DL7, 13/10, cards 79-81 | Translator finish and the bridge to calculation (p.14). | `7_12 30-28 x` is the manual's stop 30-28x ("I.D. set"). The rest is a regular sequence of `11-n` / `n-30` pairs for n = 1, 3, 4, 5, 6: tracks being written to or read from DLs. |
| 10 | 2-3 | DL8 13/11 and DL8 13/12, cards 82-87 | Not identified. Possibly the test punch of keywords and the mark list (p.15), or the diagnostic routines (pp.50-52). | Inference only. |
| 11 | 1 | DL1, 13/13, cards 88-90 | Not identified. | |
| 11 | 2-3 | DL8 13/14 and DL8 13/15, cards 91-96 | Long runs of `0 0 0 - 0 1` and `0 1 1 - 0 n` with `+P1` marks. These look like timing loops for the card reader or punch. Not identified. | Inference only. |

The track constant in row Y (`n P1 + 12P5` for track 12/n,
`n P1 + 13P5` for 13/n) follows the track number exactly on every block
from sheet 1 block 3 to sheet 11 block 3.

## DEUCE field meanings that the sheets themselves justify

These are worked out from consistency between the sheets and
READING.md's flowchart quotations. They were not taken from a DEUCE
manual.

- **NIS is the DL the next instruction is taken from.** Sheet 2 b2 mc 6
  (`3_6 24-15`) has NIS 1 and T 0, and sheet 1 b3 mc 8 (DL1) is `26-27`.
  That is the `26-27` READING.md puts at "3_8". So the flowchart's 3_8 is
  coded as **1_8**, and its branches go back to 3_10 and 3_11, which
  match.
- **Timing: the next instruction is at mc + 2 + T, and the transfer is
  at mc + 2 + W.** On a `-27` or `-28` test the alternative branch goes to
  the mc one later.
  - `7_26 26-28` (T 15) branches to 7_11 and 7_12, which are 29-16 and
    30-16 as p.22 says.
  - `7_11` (T 3) goes on to 7_16.
  - `7_23 7-15` (W 0) transfers at mc 25, which is p.22's "7_23 7_25-15".
  - `1_11` (W 12) transfers at mc 25, which is "10_25".
  - `7_16 31-18` (W 2) transfers at mc 20, which is word 0 of QS18
    (20 mod 4 = 0), and p.22 says "31-18_0".
- **C = 1 is a long transfer from mc + 2 + W to mc + 2 + T.** `3_7 24-14`
  with W 1 and T 9 gives 9 minor cycles, as in "24-14(9)", and `3_0 22-21`
  with W 0 and T 19 gives 20, as in "22-21(20)". **C = 2 is a two-word
  (double) transfer.** `3_30 21-28 2 0 1` is "21-28(2)". **C = 3** occurs
  only on stop rows with S or D 2 and a bracketed number. Its meaning is
  not established.

## Bearing on the open questions in DECISIONS.md

- **`>` is strict. This is confirmed, with high confidence.** Sheet 3
  b3 mc 3 is `7 21 27 - 0 6`, with a 5 struck. Using the timing above:
  - **Original T 5:** `+` went to 7_10 (`30-16`, false) and `-` went to
    7_11 (`29-16`, true). Row 10, `7 30 16 - 0 4`, is struck through on
    the sheet.
  - **Corrected T 6:** `+` goes to 7_11 (true) and `-` goes to 7_12
    (false).

  This is exactly the reversal READING.md describes on p.22, now
  recorded in the code. The correction is on the sheet, and its effect is
  derived from the timing reading above.
- **`=`. Partly answered.** The flowchart struck `7_23 7_25-15 / fetch /
  Q21 16-15`. On the coding sheet, `7_23 7-15` (NIS 1, W 0, T 0) is
  **present and not struck** (sheet 3 b3 mc 23). Row 25 of the same
  block is `7 16 15`. So the coded version may still fetch a second
  operand into TS15 for `=`. *Inference, low to medium confidence:* I have
  not traced the entry point of `=` to show that 7_23 is on its path, and
  the code may predate the flowchart correction.
- **Divide by zero.** `3_14 4-24 x` is coded (sheet 2 b2 mc 14), with no
  `30-16` anywhere near it. This agrees with the corrected flowchart.
- **Rounding against truncation in standardise, add, multiply and
  divide. Not settled.** The "add P1" is present as written,
  `3_2 27-22 - 1 1`. With W 1 it transfers at mc 5, an odd minor cycle,
  so it goes into the word READING.md writes 22_3. There is no other
  rounding constant in the add, multiply and divide blocks (sheet 2 b2-b3,
  sheet 3 b1). Their constants are only `15P5`, `23P22`, `7P22`, `16P22`
  and `15P26`, which look like shift counts and masks, not round-off
  values.
- **The punch rounding constant.** It is not on sheets 1-11. Apart from
  the Y rows, the only constants on these sheets are masks and small
  multiples of single P's. There is no base-32 digit string like p.47's
  "r.o. 3,31,20". (Sheet 13, from the other half of the work, shows
  `= 2,10,19,0,17,6.1`, the log₁₀2 constant of p.47, so the punch
  constants are probably on sheets 12-22.)
- **rem's divisor.** rem is not on sheets 1-11. None of the DL7 blocks
  has the p.36 sequence `7_26 19-26`, `7_29 13-27`, `7_31 30-16 x`.
- **dup and TS16. Not settled.** The keyword's target, `1_20` "store",
  is on sheet 1 b3 mc 20, `1 4 15 - 1 1`. That is source 4 to TS15, not
  an obvious push of TS16. I cannot follow the store routine from that
  one row, so I draw no conclusion.
- **The sin and cos coefficients.** They are not on sheets 1-11. None
  of the DL8 blocks (12/9, 12/11, 12/13, 13/7, 13/11, 13/12, 13/14,
  13/15) holds anything like polynomial coefficients at m.c. 3, 4, 14 or
  20. They are presumably on sheets 12-22, if anywhere.
- **`max`.** Sheet 3 b3 has `7_2 21-27`, which matches p.21, and the
  pair `7_13 19-21` / `7_14 20-21`. *Inference:* these copy a (DS19) or
  b (DS20) into DS21 after the sign test. This is consistent with
  READING.md's description of max, but it shows the flowchart's
  "7_14 30-21_3" as `20-21` on the sheet.

## Things to watch

- The flowchart's mc labels do not always agree with the sheets.
  For example, 3_8 is coded as 1_8. Elsewhere they agree closely, including
  the minor cycles of planted constants. Where they differ, the sheets
  are the coded form.
- None of this has been checked by a person against the pages.
