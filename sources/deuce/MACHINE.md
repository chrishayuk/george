# The DEUCE as the simulator models it

This is a reading of the English Electric DEUCE (Mark I), the machine
GEORGE ran on at UTECOM, written for `src/deuce/`. Each fact gives its
source. **PM** is the *DEUCE Programming Manual* NS-y-16/5-56, cited by
section and printed page. **BP** is *Basic DEUCE Programming* NS y 88
(1958), cited by sheet number. **Course** is the *DEUCE Programming
Course*, cited by PDF page. Anything marked **Inferred** is not stated in
these documents. See `README.md` for the files.

## 1. Words

- A word is 32 binary digits, P1 to P32. P1 is the least significant digit
  and comes first in time. P32 is the most significant, the sign digit in
  the signed (two's complement) convention (PM §2.2, p. 4-5; §2.7, p. 8;
  BP sheet 17). In the code, P*n* is bit *n*-1 of a `u32`.
- A minor cycle (m.c.) is 32 µs, one word time. A major cycle is 32 m.c.
  (1.024 ms) (PM §3.1, p. 9; §3.2, p. 11).
- DEUCE wrote binary least significant digit first ("backward binary"). The
  I.D., the O.S. and cards all do this (PM §2.7, p. 8).
- Notation: *n*P*k* is *n* times P*k* (BP sheet 19). The UTECOM sheets also
  write digit lists such as `P17,22-32`, meaning P17 and P22 to P32 (from
  the transcription notes). They also write six base-32 digits, least
  significant first, followed by `.h` for P31-P32, e.g. `26,12,30,4,8,13.1`
  (**Inferred**, from `flowcharts-1958/READING.md`: this gives √2·2³⁰).

## 2. The store

| Store | Words | Source / destination | Addressed by | Reference |
|---|---|---|---|---|
| DL1-DL12, the long delay lines | 32 each | 1-12 | m.c. mod 32 | PM §3.1.4, p. 10 |
| TS13-TS16, the temporary stores | 1 | 13-16 | (every m.c.) | PM §3.1.1, p. 10 |
| QS17, QS18, the quadruple stores | 4 | 17, 18 | m.c. mod 4: 17₀...17₃ | PM §3.1.3; §3.2, p. 11 |
| DS19-DS21, the double stores | 2 | 19-21 | m.c. mod 2: even "2", odd "3" | PM §3.1.2; §3.2, p. 11 |
| TS COUNT, the instruction register | 1 | D0 only | | PM §5.1, p. 24 |
| Drum | 256 tracks of 32 | via DL11 | | PM §3.3, pp. 11-12 |

- A word in DL*n* at m.c. *k* is written *n*ₖ. The minor cycle of a QS word
  is written as the lowest number in its residue class (17₀ to 17₃). DS
  words are written 2 (even) and 3 (odd) (PM §3.2, p. 11).
- In a double-length number the less significant half is in the even
  m.c., 21₂ (PM §6.6, p. 44; BP sheet 32).
- A transfer between two DLs keeps the m.c. number. A word cannot move from
  *n*ₐ to *m*_b with b ≠ a in one instruction, and the same holds for a DS
  or QS by parity or residue (PM §4.15, p. 20). This falls out of
  addressing every store by the current m.c.
- Minor-cycle numbers are relative. They are fixed by where the first word
  went when the program was read in (PM §3.2, p. 11; §8.1, p. 57).
  **Inferred**: the simulator numbers m.c. as absolute time mod 32, so an
  image loaded through the initial-input triads may sit rotated against the
  coding sheet. Programs do not notice, because all timing is relative.
  The initial card makes the rotation even (§9).

## 3. The instruction word

Layout (PM §5.3, p. 25; the bit diagram is on BP sheet 19):

| Digits | Field | Range | Meaning |
|---|---|---|---|
| P1 | not used | | ignored by Control |
| P2-P4 | NIS, next instruction source | 0-7 | DL1-7; 0 means DL8 (PM App. 2) |
| P5-P9 | S, source | 0-31 | |
| P10-P14 | D, destination | 0-31 | |
| P15-P16 | C, characteristic | 0-3 | 0 single, 1 long, 2 double, 3 (see §4) |
| P17-P21 | W, wait number | 0-31 | |
| P22-P25 | not used by Control: the "Joe digits" | 0-15 | see below |
| P26-P30 | T, timing number | 0-31 | |
| P31 | not used | | |
| P32 | G, go digit | 0/1 | 0 = stopper ("X"), 1 = obey at once |

- "Digits 1, 22 to 25 and 31 are not used in the instruction word; their
  value is ignored by Control" (PM §5.3, p. 25).
- **The bracketed number on the UTECOM sheets** (`9 (1) 8`, `30 (15) 9`)
  is the value of P22-P25. Course lecture 14 (pp. 144-146), "Automatic
  coding by Joe digits", names these digits and writes instructions in this
  same way, e.g. "A, 10-29 1 (15) T X". Adding P17 to W 31 times with
  P22-25 = 15 makes the carry "spill" through P22-P25 into T, which changes
  the next instruction and ends a counting loop without a counter. The
  simulator keeps J in the word. It confirms the reading: with the brackets
  loaded, the GEORGE basic-pack loader (sheet 1 block 2, mc 21,
  `1, 0-11, 0, 1 (15) 14 X`) counts 32 rows and leaves through the spill
  (`31(15), 14` becomes `0, 15`). Without the (15) it would loop forever.
- Written form: "N, S-D, [C,] W, T [X]". C is omitted when it is 0, and an
  X marks a stopper (PM §5.7, p. 27). A sheet's NIS of 8 means DL8 and is
  coded 0 (sheet 22 writes 8).

## 4. Timing: when the transfer happens, and where the next instruction comes from

Let an instruction enter TS COUNT in m.c. *m* (PM §5.5-5.11, pp. 25-29).

- m.c. *m*+1 is the **set-up** m.c. Nothing is transferred (PM §5.5, p. 25).
- **C = 0, single transfer.** The transfer is in m.c. *m*+W+2. The next
  instruction enters in *m*+T+2. If W > T, T counts as T+32 (PM §5.7, §5.7.1,
  pp. 26-27). Worked example: "5, 13-3, 6, 10" in 1₁₇ transfers TS13 to 3₂₅
  and takes the next instruction from 5₂₉ (p. 27). Test
  `single_transfer_pm_p27`.
- **C = 2, double transfer.** The transfer is in *m*+W+2 and *m*+W+3. T counts
  as T+32 when W ≥ T (PM §5.8, p. 28). Example: "4, 20-17, 2, 1, 5" in 1₁₃
  transfers in m.c. 16 and 17 and goes on to 4₂₀. Test `double_transfer_pm_p28`.
- **C = 1, long transfer.** The transfer runs from *m*+W+2 to *m*+T+2
  inclusive, T−W+1 m.c. If W > T it is T−W+33 m.c., ending at *m*+T+34, and
  the next instruction enters in the last m.c. of the transfer (PM §5.9,
  p. 28). Example: "7, 15-25, 1, 29, 1" in 6₅ adds TS15 into TS13 five
  times. Test `long_transfer_pm_p28`.
- **C = 3.** The same as C = 1 if W ≠ T. If W = T it is a 33 m.c. transfer
  from *m*+W+2 to *m*+W+34, and the next instruction enters at *m*+W+34
  (PM §5.10, pp. 28-29). The UTECOM sheets use C = 3, e.g. sheet 3
  `3, 3-22, 3, 30 (15) 9 X`.
- **Time taken** is T+2 if T > W, or if T = W and C is 0 or 1. Otherwise it
  is T+34 (PM §5.11, p. 29). Test `time_of_instruction_pm_p29`.
- **Next instruction.** The word that DL NIS presents in the m.c. the next
  instruction enters (PM §5.4, §5.7). Only DL1-8 can supply it (PM §5.2,
  p. 24).
- **Discrimination.** After D27 (sign) or D28 (zero), or the 2-24 TIL
  trigger, the next instruction enters one m.c. later, *m*+T+3, if any word
  sent was negative (D27) or non-zero (D28), or if TIL was present. "There is
  no other effect." The two successors are in adjacent m.c. (PM §5.14,
  pp. 32-33; §4.11, p. 17; §6.3, p. 39). The console note calls *m*+T+2 the
  "natural side". Test `discrimination_pm_p33`.
- **Destination 0.** The source replaces the word coming from NIS while the
  transfer lasts. TS COUNT takes it only if the transfer is still going on
  in the m.c. the next instruction enters. So D0 needs C = 1, or C = 0 with
  W = T, or C = 2 with T = W+1 (PM §5.13, p. 31; §8.1, p. 56). The word
  then obeyed is a "quasi" instruction Qₙ, coded as if stored in m.c. *n*
  (PM §5.13, p. 32). Test `destination_zero_pm_p31`.

### What the coding sheets show

The transcribers of the UTECOM sheets inferred these rules from the sheets
and the flow diagrams:

- next at mc+2+T, the other branch one m.c. later;
- transfer at mc+2+W;
- C = 1 long, C = 2 double, C = 3 unexplained;
- an unexplained bracketed number between W and T.

All four agree with PM. In detail: the next instruction enters at *m*+T+2
(+32 under the rules above) and the second branch is at +3 (§5.14). The
transfer is at *m*+W+2. C = 1 is long from *m*+W+2 to *m*+T+2. C = 2 is a
double transfer. C = 3 is §5.10 above. The bracket is P22-P25 (§3).

### Points the documents leave to inference

- **A stopper and the single shot.** **Inferred**: a stopper that entered
  TS COUNT in m.c. *m* is obeyed, once the single shot arrives, as if it had
  entered at the first later time with the same m.c. number. W and T then
  keep their meaning. PM §8.1 (p. 57) relies on this: "2, 0-2, 30, 31 X
  entering TS COUNT in m.c. *m* is to send the word on the next row to 2ₘ
  and take the next instruction from 2ₘ₊₁", and the m.c. table on p. 57
  works out exactly under this rule (test
  `initial_input_with_initial_card_and_triad_pm_p56`).
- **Writing to the NIS delay line in the m.c. the next instruction
  enters.** PM p. 57 has "2, 0-2, 1, 26, 25 X" filling DL2 with 32 copies,
  and "one of these enters TS COUNT" in the same m.c. as the last write.
  **Inferred**: TS COUNT sees the word being written. The simulator performs
  the transfer of an m.c. before the fetch.
- **Sources read the word before any write in the same m.c.** This matters
  only for TCA. PM §6.5 (p. 43) says "16-10 (32 m.c.)" with TCA on moves
  every word of DL10 on one place, which needs the delayed word to be the
  old one (test `tca_shifts_dl10_pm_p43`).

## 5. Sources

PM Appendix 2 (PDF p. 77) gives the full list. §6.1-6.2 (pp. 34-35), §6.5
(p. 42) and §6.7 (p. 45) give the details.

| S | Gives | Notes |
|---|---|---|
| 0 | the row under the reading brushes while the reader is running, otherwise the I.D. keys | PM §4.8, p. 15; §6.3, p. 39. Between rows, and from 6 major cycles after a row's single shot, it gives zeros. It does not revert to the I.D. between cards while cards run continuously (PM App. 1 (a), (d), p. 73). |
| 1-12 | DL1-12 at the current m.c. | |
| 13-15 | TS13-15 | |
| 16 | TS16; with **TCA** on, DL10 delayed by one m.c. | PM §6.5, p. 42; App. 2 note (a) |
| 17, 18 | QS17, QS18 at m.c. mod 4 | |
| 19-21 | DS19-21 at m.c. mod 2 | |
| 22 | DS21 ÷ 2 (signed). Even m.c.: A2...A32, X, with X = B1 if TCB is off and A32 if TCB is on. Odd m.c.: B2...B32, B32. (A = 21₂, B = 21₃.) | PM §6.7, p. 45; BP sheet 33 |
| 23 | TS14 ÷ 2: P1-31 ← P2-32, P32 repeated ("true division, without round-off, in the signed convention") | PM §6.2, p. 34 |
| 24 | TS14 × 2: P1 = 0, P2-32 ← P1-31 | PM §6.2, p. 34 |
| 25 | TS14 & TS15 (AND) | PM §6.2, p. 34 |
| 26 | TS14 ≢ TS15 (exclusive or, "14 not equivalent to 15") | PM §6.2, p. 34 |
| 27 | P1 | PM §6.1, p. 34 |
| 28 | P17, the unit of W | PM §6.1, p. 34; §5.12, p. 30 |
| 29 | P32 | PM §6.1, p. 34 |
| 30 | zeros | PM §6.1, p. 34 |
| 31 | ones (-1) | PM §6.1, p. 34 |

The worked example of S23-26 on PM p. 35 is test `logical_sources_pm_p35`.
A note written by hand on Appendix 2 says "14 ∨ 15 = (14 & 15) + (14 ≢ 15)".

### Check of the inferences in `flowcharts-1958/READING.md`

| READING.md inferred | PM | Verdict |
|---|---|---|
| S25 = TS14 AND TS15 | S25 "TS14 & TS15" (§6.2, App. 2) | **agrees** |
| S26 = TS14 XOR TS15 | S26 "14 ≢ 15", a one wherever the digits differ (§6.2) | **agrees** |
| D25 adds into TS13, D26 subtracts | "Destinations 25 and 26 are respectively the additive and subtractive inputs to TS13" (§4.3, p. 13) | **agrees** |
| S31 = all ones (-1), S30 = zero | §4.5, p. 14; §6.1 | **agrees** |
| S27 = P1 | §6.1 | **agrees** |
| S29 = P32 | §6.1 | **agrees** |
| S28 = P17 (by elimination) | §6.1: "P17, the units digit in the Wait number" | **agrees** |
| `-27` forks on sign, `-28` on zero | D27 "Discrim. (Sign)", D28 "Discrim. (Zero)" (§4.11, App. 2). Zero counts as positive. | **agrees** |
| S22, 23, 24 shift (unverified in READING) | S22 = DS21 ÷ 2 (TCB-dependent, above), S23 = TS14 ÷ 2, S24 = TS14 × 2 | **agrees** |
| Double-register word order | the even m.c. is the less significant half | **agrees** |

So every identification READING.md made by inference is confirmed by the
manual.

## 6. Destinations

| D | Does | Reference |
|---|---|---|
| 0 | TS COUNT: the source supplies the next instruction (§4) | PM §5.13 |
| 1-12 | DL1-12 at the current m.c. | |
| 13-15 | TS13-15 | |
| 16 | TS16. Any transfer to TS16 clears TCA. | PM §6.5, p. 42 |
| 17-21 | QS / DS by m.c. | |
| 22 | add into DS21 | PM §4.3, §6.6 |
| 23 | subtract from DS21 | PM §4.3, §6.6 |
| 24 | triggers, selected by the *source number* (§7) | PM §4.7, p. 14 |
| 25 | add into TS13 | PM §4.3 |
| 26 | subtract from TS13 | PM §4.3 |
| 27 | discriminate on sign (P32) | PM §4.11, §5.14 |
| 28 | discriminate on zero | PM §4.11, §5.14 |
| 29 | the O.S., or the punch while it runs | PM §4.8, p. 15; §6.3, p. 38 |
| 30 | drum read (C even) or write (C odd); S = head | PM §6.4, pp. 41-42 |
| 31 | drum head shift: read heads (C even) or write heads (C odd); S = position | PM §6.4, p. 42 |

**DS21 arithmetic** (PM §6.6, pp. 44-45; BP sheet 32):

- With TCB **on**, 21₂ and 21₃ are two separate 32-digit accumulators.
- With TCB **off**, the carry from P32 of the even word goes into P1 of the
  odd word.
- With TCB off, a word sent to D22/D23 in an even m.c. that is the *last*
  m.c. of the transfer is extended. "32 copies of the sign digit of this
  number are respectively added or subtracted into DS21 in the following odd
  minor cycle." So `29-23₂` adds P32 into 21₂ (PM §6.9 note (l), p. 49).
- The simulator applies the carry and extension in the next (odd) m.c.,
  after that m.c.'s source has been read. That is how S22 in "22-22 (2 m.c.
  e,o) multiplies by 1.5" sees the old odd word.

Tests: `ds21_add_subtract_tcb_pm_p44`, `s22_to_d21_halves_double_length_pm_p46`.

**D29 and the O.S.** **Inferred**: a transfer to D29 *sets* lamps (an OR)
and does not replace the word. Several things clear it: 8-24, the key,
starting the punch, and the punching of each row (PM App. 2; §6.3, p. 38;
BP sheet 28). The evidence is that a separate "Clear O.P.S." trigger
exists, and that PM §6.8 (p. 46) uses "30-29" as a *dummy* instruction.

## 7. Destination triggers (D24)

On the Mark I, from PM Appendix 2:

| S-24 | Action |
|---|---|
| 0 | start multiplication |
| 1 | start division |
| 2 | TIL: the next instruction is one m.c. later if the Twelfth Impulse Line is present |
| 3 | TCA on |
| 4 | TCB off |
| 5 | TCB on |
| 6 | alarm off |
| 7 | alarm on (buzzer and red lamp) |
| 8 | clear the O.S. |
| 9 | clear (stop) both punch and reader |
| 10 | start the punch |
| 12 | start the reader |

11 and 13-31 are not used on the Mark I. Course p. 329 (1960) lists later
additions: 64-column working, paper tape, TCC, magnetic tape. The simulator
records a warning for these.

### Multiplier

PM §6.8, pp. 46-47; BP sheets 33-34.

- `0-24` must be obeyed in an **odd** m.c.
- The multiplier forms the unsigned 64-digit product of TS16 and 21₃. 21₂
  "must be empty".
- It takes 65 m.c. Started in m.c. 7, it is complete at the end of m.c. 7
  two major cycles later, and "21-19 (2 m.c.)" can transfer in m.c. 8 and 9.
- TCB is cleared at the start and left off.
- D16, D21, D22 and D23 must not be used during the multiplication. S21 and
  S22 give partial products meanwhile.
- **Digit by digit.** *DEUCE Logical Design Manual* Part I, NS-y-37/11-57
  (**LDM**), ch. 10, pp. 60-65, describes the multiplier:
  - The partial product and the waning multiplier share DS21 (p.63). "The
    functions of the second and third of these can be combined in one
    double-length storage position."
  - The first (odd) m.c. runs the multiplier through so that its top digit
    can be examined. After that the process goes by pairs of m.c.:
    - in the even m.c., the bottom half is shifted up one place and TS16 is
      added if the digit examined was 1 (Trigger Y, pp.64-65);
    - in the odd m.c., the top half (partial product below, remaining
      multiplier digits above) is shifted up one place, taking the digit
      and carry from below, and its new top digit is examined.
  - So after step *k* (m.c. *s*+2*k*), DS21 = 2·DS21 + y·TS16 (mod 2⁶⁴),
    with *y* the top digit before the shift. The figure on p.64 traces
    13 × 14 in four-digit words.
  - The simulator does exactly this, one m.c. at a time
    (`Machine::mult_until`). DS21 can be read part-way through; a read gives
    the word as it emerges, before that m.c.'s step. Writes part-way
    through act on the word being processed: D22/D23 through the long
    accumulator's adder, D21 by replacing it.
  - GEORGE's sin and cos depend on this. They read 21₂ three m.c. after
    `0-24`, when it holds TS16 if the multiplier's top digit was 1, and
    subtract it from 21₃ after the product is complete (RUNNING-GEORGE.md,
    findings 14 and 19).
  - When nothing touches DS21 during the multiplication, the result is the
    plain product, as before. Test `multiplier_step_by_step_ldm_p64`.
- The sources are saved in `sources/deuce/ldm/` (`ldmpart1.html`, its text,
  and Figure 10.9).
- For a signed product, subtract the other factor from the top half for
  each negative factor (PM §6.9, BP sheet 34). Test
  `unsigned_product_and_sign_correction_bp_sheet_34`.

### Divider

PM §6.10, pp. 49-51; BP sheets 36-38.

- The divisor goes in TS16 and the dividend A in 21₃, with 21₂ clear.
  `1-24` is obeyed in an odd m.c. Division takes 66 m.c. TCB is set at the
  start.
- With 2³¹A = QB + R, where 0 ≤ R < B if B > 0 and B ≤ R < 0 if B < 0:
  Q goes to 21₂ and the "modified remainder" r = 2(2R − B) to 21₃.
- **Simplification:** computed from this definition, not digit by digit.
  LDM pp.65-75 describes the divider's two-choice non-restoring process,
  which is not modelled step by step. Out-of-range operands give a warning
  and an arbitrary result: |A| > |B|, |A| = |B| other than A negative and B
  positive (PM p.50), or B = 0. The restriction on loading TS16 in the odd m.c. just before the
  start is not modelled.
- Tests: `divider_definition_pm_p50`, `division_pm_p51`.

## 8. The drum

PM §3.3, pp. 11-12; §6.4, pp. 41-42; BP sheets 40-41.

- There are 256 tracks of 32 words. Track number = 16 × head-block
  position + head, written "a/b".
- There are separate read and write head blocks, each with 16 positions.
- `n-31` with C even moves the read heads to position *n*; with C odd, the
  write heads.
- `h-30` with C even reads the track under read head *h* into DL11; with C
  odd it writes DL11 onto the track under write head *h*. Transfers are only
  through DL11, a whole track at a time.
- A read or write takes 13 major cycles. A head shift takes 64 major cycles
  according to PM, and "about 35 ms" according to BP. The simulator uses 35
  major cycles (configurable). "n-31" with the heads already at *n* takes no
  time (BP sheet 41).
- **Control-magnetics interlock**: an instruction using S11, D11, D30 or D31
  is held in TS COUNT, "as though it were a stopper", until the drum
  operation finishes (PM §6.4, p. 42).
- **Inferred**:
  - track word *k* ↔ DL11 m.c. *k*;
  - the transfer of data happens at the end of the operation;
  - S > 15 in a drum instruction is taken mod 16, with a warning.

  The coding sheets' card row Y, `nP1 + 12P5`, holds track 12/*n* in this
  same a/b layout (head in P1-4, position in P5-8). It tells the GEORGE
  loader which track a triad goes to.

## 9. Cards, reader and punch

- **Columns.** A card has 80 columns and 12 rows, which pass the brushes
  and knives in the order Y, X, 0, 1, ..., 9. DEUCE uses only columns 21-52.
  Column 21 is P1 and column 52 is P32 (PM §2.6, p. 7: "binary numbers are
  always punched with their least significant digit on the left"; BP
  sheet 20 states it digit by digit). Column 54 is P34 in the same
  numbering. A hole there stops the reader after that row: "the
  Single-Shot occurs for that row (the one in which P34 is punched), but
  not for any subsequent ones" (PM §6.3, p. 39). The sheets' "go + P34"
  marks this hole.
- **Single shots.** While the reader or punch runs, it sends a single shot
  as each row arrives, and stoppers are obeyed on them. The console key is
  then inoperative (PM §4.18, p. 22). A shot that comes when no stopper is
  waiting is lost (PM §6.3, p. 39).
- **Reader.** `12-24` starts it and `9-24` stops it.
  - Timings are from PM Appendix 1, pp. 73-76. The simulator uses:
    - first shot 130 major cycles after the call (the minimum);
    - rows 18 major cycles apart (the middle of the manual's 15-21);
    - cards 90 major cycles apart (the middle of the manual's 80-100).
      GEORGE's matrix read `R||` does more than 81 major cycles of work
      between cards (RUNNING-GEORGE.md), so it fails at the minimum.
  - S0 gives the row for "2 major cycles" after its shot and zeros "from 6"
    onwards. **The simulator takes 4.** The standard triad's first
    instruction (`N, 0-N, 1, 26, 25 X`) reads the same row for up to about
    2.8 major cycles under the stopper rule of §4, so 2 is too few for the
    manual's own scheme.
- **TIL.** The Twelfth Impulse Line is on from 2 major cycles before the
  row-9 shot until 18 major cycles after it (PM App. 1 (d), minimum
  figures). `2-24` tests it.
- **Punch.**
  - `10-24` starts the punch and clears the O.S. The first shot comes 180
    major cycles later, rows 38 apart, cards 116 apart (PM §6.3, p. 39;
    App. 1).
  - Each row is punched from the O.S., which is then cleared. A transfer to
    D29 within 4 major cycles of the stopper is "successful", so the
    simulator punches each row 4 major cycles after its shot.
  - `9-24` in mid-card lets the rest of the card run out blank (PM §6.3,
    p. 39).
- **Initial input** (PM §8.1, pp. 56-58). The INITIAL INPUT key clears the
  high-speed store, including TS COUNT, and calls the reader. TS COUNT then
  holds the zero word "0, 0-0, 0, 0 X". That is a stopper with W = T and
  C = 0, so it takes each row into TS COUNT through D0 and obeys it on the
  next row's shot.
- **Triad.** A triad fills DL N from three cards:
  - rows 1-4: blank, "N, 0-N, 1, 26, 25 X", "N, 0-N, 30, 31 X", "1, 0-N, 30, 31 X";
  - then m.c. 0-31: m.c. 0 is card 1 row 2, m.c. 8 is card 2 row Y, m.c. 20
    is card 3 row Y (BP sheet 20).
  - The last triad's fourth row is "M, 0-N, 30, m-1 X", which enters the
    program at M_m.
  - DL1 must be filled last because the scheme uses 1₀ as a blank word.
  - `cards::triad`.
- **Initial card** (PM §8.3, pp. 60-61). In row order:
  - blank rows;
  - 4-24 (TCB off);
  - 31-21 (fill DS21 with ones);
  - 27-22 (add P1: empties DS21 if this is an even m.c.);
  - 21-28 (next instruction 3 or 4 m.c. later);
  - 30-21.

  Together these make the first word of the next triad land in an even
  m.c. `cards::initial_card`; test
  `initial_input_with_initial_card_and_triad_pm_p56`, which reads in and
  runs the successive-squares program from cards.

## 10. Console

D. G. Burnett-Hall, *The DEUCE Console* (MS/B/201), pp. 2-4.

- I.D.: 32 keys with lamps.
- O.S.: 32 lamps. During punching it shows the number being punched.
- SINGLE SHOT key.
- STOP key: NORMAL, STOP (every instruction a stopper, PM §4.19) and
  AUG. STOP.
- CONT TIL, DISCRIM, PROG DISPLAY and the "external tree". These are not
  modelled.

## 11. Not modelled

- Partial quotients during division: the divider gives its result from
  the Programming Manual's definition (p.50), not step by step, and its
  results for out-of-range operands are the model's, not a measured
  DEUCE's. (Partial products during multiplication are modelled: §5.)
- Missed-card and other alarms.
- Exact Hollerith timing jitter.
- Mark II/IIA extras: the 64-column reader and punch, paper tape, magnetic
  tape, extra DLs, TCC.
- Request Stop.
- The Automatic Instruction Modifier is implemented (`Config::aim`, from
  PM pp. 82-83) but off by default, as it was not on a Mark I.
