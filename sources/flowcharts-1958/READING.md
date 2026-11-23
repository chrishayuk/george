# Reading notes: GEORGE flow diagrams (Hamblin, 1957-58), 52 pp.

Source: `GEORGEFlowCharts.pdf` (page numbers below are the handwritten page numbers, which equal the PDF page numbers).
Images were read at 100 dpi (`pages/f-NN.png`). Key pages were re-rendered at 170-200 dpi: 12, 18-23, 25-27 and 47.

## How the pages are written

The diagrams are not boxes and arrows. They are DEUCE instruction listings in the usual DEUCE flow-diagram notation:

- `7_9 10_28-21_2` means an instruction stored in DL7, minor cycle 9, which transfers source 10 (m.c. 28) to destination 21 (word 2).
- `(n)` is a long transfer of n minor cycles.
- `x` marks a stop.
- `Q_n (...)` is a quasi-instruction that has been planted in m.c. n.
- A forked line under `-27` branches on sign (`+` / `-`), and one under `-28` branches on zero (`0` / `φ`).
- Bracketed text such as `[31P17]` gives the constant held in the planted word.
- Boxed names are subroutine calls, with a page reference.

All quotations below are transcriptions of this notation.

**DEUCE source/destination meanings used in the inferences below.** I worked these out from the flowcharts themselves. I did not take them from a DEUCE manual.

- On p.22, the routines for `~`, `&` and `∨` are only consistent if source 25 = TS14 AND TS15, source 26 = TS14 XOR TS15, destination 25 = add into TS13, destination 26 = subtract from TS13, source 31 = all ones (-1) and source 30 = zero.
- On p.27, source 27 decrements the repeat count, and the manual says that count is held in P1-16. So source 27 is the unit P1.
- On p.26, source 29 is added to the repeat link entry. On p.23, `]` tests that entry with a sign test (`13-27`). So source 29 is P32, the sign digit.
- By elimination, source 28 is P17.

These identifications are **inferred**. Each answer below that depends on them says so.

---

## 1. Page index

| p. | Heading as written (and subject) |
|---|---|
| 1 | "GEORGE flow diagrams." Basic pack card 0 (standard initial card). "Clear suffixed variable store" card (CSVS), "a modified Deuce 'clear drum' card". |
| 2 | Basic pack cards 1-3. A loader that reads the rest of the pack (in triads) onto the drum. |
| 3 | G-program initial card (GPIC). Calls the clock track 13/14 to DL8. |
| 4 | "Read in" clock track 13/14. Acts as an initial card and calls 13/13 to DL1. "Loop with buzz if out of synchronism." |
| 5 | Read in G-program. Symbols go into consecutive m.c. of the position-11 tracks, in P17-24. Codes 0/0 and 15/15 are ignored, and 14/15 stops the read. |
| 6 | Commence translation. Tracks 13/0-13/7 go to DL1-8. DL12 is saved to track 12/0 and then becomes the mark list. Keywords are written to position 10. |
| 7 | Translator subroutine "call in next" (CIN), and decode symbol. |
| 8 | Translator subroutines SN / SNT / SNWT (store next keyword, unmodified or with its timing / wait numbers modified). "SNT 1st of two". "Store track". |
| 9 | Translator, general operations. A table giving each symbol's code, decode instruction and keyword. |
| 10 | Translator: `\|` and `\|\|` (suffix bars), `rep`, unsuffixed variables, unsuffixed names. |
| 11 | Translator, marks `*`. Mark list in DL12. Stop if a mark is used twice. |
| 12 | Translator, numerals. "Numeral is converted to floating binary and stored as second keyword." |
| 13 | Translator, long operations and read and punch. |
| 14 | Translator, finish and bridge to calculation. The blank symbol 0/0 ends translation. Stop 30-28x if the I.D. is set. |
| 15 | Punch keywords and mark list (test facility). |
| 16 | **Calculation.** Control routines NK (next keyword), NKD (next keyword, double) and "call new track". Accumulator fetch and store subroutines. "Accumulator is D.L.10 m.c. 1-12." |
| 17 | Keyword routines for unsuffixed variables, unsuffixed names, numerals, `dup` and `rev`. |
| 18 | Floating arithmetic subroutines: unpack one, unpack two, standardise. |
| 19 | Floating add and subtract. Also `mod` and `neg`. |
| 20 | Floating multiply and divide. |
| 21 | Fetch integer (subroutine), and `max`. |
| 22 | Relations and logical operations: `~` `&` `∨` `=` `>`. |
| 23 | End bracket `]`, and `wait`. |
| 24 | "Store link list" subroutine, used by `↓` and `rep`. "Link list is D.L.10 m.c. 18-23." |
| 25 | Skips `↓` and `↑`: the conditional test and the mark-list lookup. |
| 26 | `rep`: first entry (the count-down set-up). |
| 27 | "rep (continued) - on recall after ']' from 'end bracket' program." Count decrement and x increment. |
| 28 | Suffixed variables (`\|` and `\|\|`): computing the track and m.c. |
| 29 | Suffixed variables (continued). |
| 30 | End of calculation (from the space keyword and from `]`). "Put away track" subroutine. |
| 31 | Long operations, and read and punch: dispatching on the second keyword. |
| 32 | `log`, and the "log" subroutine. Uses the DEUCE library routines M12 and S06. |
| 33 | `exp`, and the "exp" subroutine. |
| 34 | `pow`. |
| 35 | `pow` (continued). |
| 36 | `rem`. |
| 37 | `√` (square root). |
| 38 | `cos` / `sin`. |
| 39 | `R` and `(P)`. |
| 40 | `R\|` and `R\|\|` (block read). |
| 41 | `P\|` and `P\|\|` (block punch). |
| 42 | `R\|` and `P\|`: assemble and modify. |
| 43 | `R\|\|` and `P\|\|`: assemble and modify. |
| 44 | Call read loop and read: card rows to bits. |
| 45 | Read translate: decimal card to binary floating point. |
| 46 | Read translate (continued). Constants log₂10 and 2²⁰·10⁻⁵, and a coefficients table. |
| 47 | Punch translate: binary to decimal. Constants log₁₀2 and log₂10, and a "r.o." constant. |
| 48 | Punch translate (continued), and Punch (link Q12 in 16). |
| 49 | Reset after read and punch. |
| 50 | GEORGE diagnostic card (DIAG). |
| 51 | Clock track, diagnostic punch routine. |
| 52 | Diagnostic punch routine (from the clock track). |

---

## 2. Answers to the open questions

### a. `a b rep (x) ... ]`

**What is written.**

p.10 (translator): `rep` becomes a pair of keywords. p.26: the second keyword is "variable no. × P17".

p.26, rep entry, in order:

- `fetch integer` → `Q30 21_3-20_3`. This pops the top cell (b), as an integer.
- `fetch integer` → `Q30 21_3-19_3`. This pops a.
- `7_2 7_11-13 [16-12_0]`, `7_15 20_2-25`, `3_28 13-0`, `Q30 (16-12)`. This builds the quasi-instruction 16-12_v, which stores TS16 into the unsuffixed-variable store (DL12) at m.c. v, where v is the variable number.
- `7_13 20_3-21_3`, `7_16 19_3-23_3`. This forms b − a.
- `7_19 21_3-14`, `7_21 7_23-15 [P1-16]`, `7_24 25-13`. This masks b − a to P1-16.
- `7_26 29-25`, `7_28 28-25(2)`, then the variable number is shifted in (`7_31 20_2-14`, `7_3 24-14(9)`, `7_17 24-25`).
- `8_10 store link list` (p.24), then `1_21 (control NKD)`.

**There is no discrimination anywhere on p.26.** No `-27` or `-28` branch appears on the entry path.

p.23, `]`: the routine fetches and pops the top link-list entry and tests its sign (`7_7 13-27`). On `-` it goes to "re-enter 'rep' see p.27".

p.27 ("on recall after ']'"):

- `7_29 7_4-15 [P1-16]`, `7_14 25-28`. This tests the count field.
- **0** → `7_18 1-1` → `1_24 (control NK)`. The loop exits and the program continues after `]`.
- **φ** → `7_27 27-26`, which decrements the count. The entry is then written back onto the link list (`Q30 (13-10)`, "convert back, add P17,22").
- Next, `Q30 (12-16)` fetches x from DL12 into TS16.
- It is unpacked (`22-21(20)`), and a unit shifted by a variable amount is added (`8_23 27-14`, `Q30 (24-14)`, `7_20 14-22_3`).
- `standardise` (p.18), then `Q30 [16-12_v]` stores x back.
- `1_29 control`, which jumps back to the start of the body.

**Answers. All of these are inferred from the instruction sequence above.**

- **Count-down.** The count is (b − a) computed as integers, masked to the 16-bit field P1-16, and stored in the link-list entry. It means "repetitions still to be made after this one", which agrees with the manual's diagnostic description (§2.5). The sign digit of the entry (source 29, P32) marks the entry as a repeat rather than a subroutine link. *High confidence on the structure. The P1 integer scaling comes from the manual plus the decrement by source 27.*
- **Is the body executed at least once when a > b?** **Yes.** The entry path has no test, and the count is first examined at the first `]`. With a > b, b − a is negative. Masked to P1-16 it becomes 2¹⁶ − (a − b), so the body would run about 65 536 − (a − b) + 1 times rather than zero. *The "at least once" part is high confidence. The wrap-around count is medium confidence, because it depends on the integer form returned by "fetch integer", which I did not fully decode.*
- **Is x stored at entry and incremented at each `]`?** **Yes to both.** At entry x is set from TS16, which still holds the last number fetched: a, as the original floating value, not truncated. Medium-high confidence, based on the reading that "fetch integer" (p.21) leaves the floating word in TS16. At each `]` where the count is non-zero, x is **re-read from the store**, increased by 1 in floating point, standardised and stored back. There is no hidden loop variable, so if the body assigns to x, that change affects later values of x. The count is kept separately and is not affected.
- **Value of x after the loop.** At the final `]` (count = 0) control goes straight to NK without touching x. So x keeps the value used in the last pass through the body: a + (b − a) = **b**, if the body did not change x. *Inferred, high confidence.*
- Other details:
  - After unpacking, the "+1" is added to the mantissa at a position that depends on x's exponent.
  - `7_28 28-25(2)` adds 2·P17 (source 28 = P17 by elimination) to the link entry. I think this adjusts the return keyword address past the two-keyword `rep` pair. *This is a guess.*
  - p.27 is annotated "8_13 10-31 NOT IN 1B; 1C" and "1_16 in 1B".

### b. Truth values; `=` `>` `~` `&` `∨`; conditional skips

**What is written (p.22).**

- `=`: `fetch`, then `Q21 16-14`. There was originally a second `fetch` and `Q21 16-15`, but the lines `7_23 7_25-15 / fetch / Q21 16-15` are **struck through**. Then `7_26 26-28`, which tests TS14 XOR TS15 for zero:
  - **0** (equal) → `7_11 29-16`
  - **φ** → `7_12 30-16`
  - then `7_16 31-18_0`, then `store and NK`.
- `>`: `5_11 subtract (p.19)`. Then `Q31`: "29-22_3" is struck and replaced by "27-23_2". Then `7_3 21_3-27`:
  - `+` → `7_14`, where "30-16" is corrected to **29-16**
  - `-` → `7_12`, where "29-16" is corrected to **30-16**
  - then `7_16 31-18_0`, then `store and NK`.
- `~`: `fetch`; `Q21 31-14` (all ones into TS14); `7_26 16-15` (operand into TS15); `7_0 26-16`; `store and NK`.
- `&`: fetch into TS14, fetch into TS15, `7_15 25-16`, store.
- `∨`: fetch `16-14`; fetch `31-13`; `16-26`; `13-15`; `25-14`; `31-13`; `26-26`; `13-16`; store.

**Answers.**

- **The truth word is source 29. The false word is source 30, which is all zeros.** If source 29 is P32 (see the derivation at the top), then truth is the word with **only the sign digit set**. Read as a GEORGE floating number (exponent in P1-10, mantissa P11-32 with the binary point between P31 and P32), that word is mantissa −1 with exponent 0, which is **exactly −1.0**. That matches the manual's "−1 for truth". *Source 29 is written. Its identification as P32 is inferred, medium-high confidence.*
- `=` (original, un-struck form) compares the two **whole 32-bit words bit for bit**, using XOR and a zero test. The struck second fetch means I **cannot determine** the final revised form of `=` from this page.
- `>` computes a − b with the floating subtract routine (taking the result before standardising). It then subtracts a unit (source 27 = P1) from the low word, so that a − b = 0 becomes negative, and tests the sign. Truth is given when a − b > 0. The corrections on the page are what make this strict: `+` gives truth. *Inferred, medium confidence.*
- **`~` is a full bitwise complement** (XOR with all ones). It does not just flip the sign digit, although it does flip it. So `~` applied to truth gives 0111…1, which has sign digit 0 and therefore counts as false. `~` applied to 0 gives all ones, which counts as true. The results are not the canonical 0 / −1. *Written, with the AND/XOR source meanings inferred, high confidence.*
- **`&` is a bitwise AND of the two whole words. `∨` is a bitwise OR**, done by De Morgan through TS13. *Same basis.*
- `=` and `>` both execute `31-18_0`. This sets a "relation pending" flag (QS18 word 0) to −1. `~`, `&` and `∨` leave the flag alone.
- **Conditional skip (p.25).** Both `↓` and `↑` run:
  1. `fetch integer`, which pops the mark number.
  2. `7_1 18_0-27`, which tests the flag.
     - Flag `+` (not set): the skip is unconditional.
     - Flag `-` (set): `fetch`, which pops the truth value, then `Q21 16-27` tests its **sign digit**.
       - `+` (false) → `7_14 30-18_0`, which clears the flag, then NK. No skip.
       - `-` (true) → go on to the skip. The flag is cleared at `7_30 30-18_0`.

  So "truth" means **the sign digit of the cell is 1**, any negative word. Both the truth value and the mark number are popped either way. That is written, and agrees with the manual.

  *Inference:* the flag is cleared only by a skip. So a relation that is not followed by a skip leaves every later skip conditional, and that later skip will pop a cell it treats as a truth value. This fits the manual's wording "whenever one of the two relational operators has preceded it (since the previous skip)".

### c. Floating-point format and arithmetic

**What is written.**

p.18 *Unpack one*: `fetch`; `Q21 16-21_3`; `3_23 30-21_2`; `3_0 22-21(20)`; `3_21 21_2-14`; `3_26 23-21_2`.

p.18 *Standardise*:

- `3_27 14-1_30`; `3_30 21-28(2)`.
  - **0** → `3_1 21_3-16` → link.
  - **φ** → `3_2 27-22_3`, then a loop: `3_5 21_3-14`; `3_7 24-14(9)`; `3_18 24-15`; `3_20 26-27`.
    - `+` → `3_24 10_27-26 [P22]`; `3_29 21-22(2)`; `3_3 27-23_3`; back to 3_5.
    - `-` → `3_25 10_27-25 [P22]`; `3_31 22-21(2)`; `1_4 13-14`; `3_6 24-15`; `3_8 26-27`.
      - `+` `3_10 15-21_2`
      - `-` `3_11 14-21_2 x` (stop)
      - then `3_12 21-22(20)`; `3_1 21_3-16`; link.

p.19 *Add*:

- `unpack two`. The top operand goes to DS20 and the next to DS19.
- `4_4 20_2-26` (difference of exponents); `1_6 13-28`.
  - 0 → add the mantissas directly (`4_9 20_3-22_3`).
  - Otherwise, a sign test. On `+` the operands are swapped (`4_12 20-21(2)`, `4_16 19-20(2)`, `4_19 21-19(2)` …, "to 4_2").
  - On the other path, `5_16 4_18-25 [23P22]` (a comparison of the exponent difference against 23), a variable-length shift `Q30 (22-21)` built from `[22-21(32) J=1]`, then the add.
- Exit through "normal link 4_27-14 → 3_27 to 'standardise' and store and NK".

p.19 *Subtract*: negates b's mantissa (`1_5 13-20_3`) and joins add. *neg* is computed as 0 − x through the subtract routine (`6_2`…`5_28 10_31-1_31`). *mod*: `21_3-27`. On `+` the operand is re-stored unchanged; on `-` it goes to the neg path.

p.20 *Multiply*: `unpack two`; `0-24` (the hardware multiply); `19_2-13`, `20_2-25` (adds the exponents); sign corrections `5_9`…`5_20`; `5_23 21-22(22)`; then standardise.

p.20 *Divide*: `21_3-28`. A zero dividend goes straight to standardise. Otherwise `1-24` (the hardware divide); `3_9 14-28`, where **0 → `3_14 4-24X` stop** ("30-16x" is struck out); exponent subtract `20_2-26`; then standardise.

**Answers.**

- **Format.** The page confirms the manual: exponent in P1-10 and mantissa in P11-32. Unpack shifts the double register right by 10 places (`22-21(20)` = 20 minor cycles). Pack shifts it left by 10 (`3_12 21-22(20)`). The exponent is carried separately during the arithmetic. *Inferred from the shift counts, medium-high confidence.*
- **Normalisation.** Standardise tests whether two adjacent bits of the mantissa are equal: it takes the 9-place and 10-place left shifts, XORs them and tests the sign. While they are equal it shifts left (`21-22(2)`, doubling) and adjusts the exponent by P22 each time. When they differ it backs off one right shift (`22-21(2)`) and packs. This is **two's-complement normalisation**: the bit after the sign differs from the sign, so the mantissa lies in [½, 1) or [−1, −½). The back-off also handles carry-out from addition. As a result, −½ is held as mantissa −1 × 2^(e−1). That fits truth (P32 alone) being a well-formed −1.0. *Medium confidence on the exact bit positions, high confidence that it is shift-until-sign-and-next-bit-differ.*
- **Exponent overflow.** `3_11 14-21_2 x` stops. On resumption it stores TS14 rather than TS15 (TS15 = TS14 shifted once), which matches the manual's "S.S. resumes with exponent halved" (stop 14-21x). *Written, and the match with the manual is strong.*
- **Zero.** Standardise tests the whole double register for zero first (`3_30 21-28(2)`). If it is zero, it stores 21_3 (= 0) directly. So **zero is the all-zero word** (mantissa 0 and exponent field 0), and it is never normalised. Divide also sends a zero dividend straight to standardise. In numerals (p.12), a zero value skips the normalising loop. *Medium-high confidence.*
- **Rounding or truncation.** **Not determinable with confidence.**
  - Standardise always executes `3_2 27-22_3` on the non-zero path, adding source 27 (P1, inferred) to the high word before normalising. This may be a rounding step, but I cannot establish where the guard bits sit relative to P1.
  - Add, multiply and divide contain no other rounding constant. The alignment shift in add moves bits into the low word, and they are dropped at the final pack. So, apart from that one `27-22_3`, I would expect truncation. *This is a guess.*
- **Divide by zero.** `3_14 4-24X`, then on resumption `1_20 store and NK`. The manual says this "resumes giving result zero".

### d. Library functions

- **`max`** (p.21): calls `subtract` (p.19), giving a − b. It tests the sign of the result (`7_2 21_3-27`) and keeps a (`19_3`, `19_2`) or b (`20_3`, `20_2`). The result is re-packed without going through standardise. *Written.* (One correction on the page, `7_14 30-21_3` → `7_21 20_3-23_3`, is starred.)
- **`mod`** (p.19): unpack, then a sign test. Non-negative values are re-stored unchanged; negative values go through the `neg` path, which is 0 − x via subtract.
- **`rem`** (p.36):
  - **The divisor b is fetched with "fetch integer"** (`6_23`, then `Q30 16-18_1`, then `21_3-19_3`). The dividend a is fetched as an ordinary float.
  - `7_26 19_3-26`, `7_29 13-27`: if b ≤ 0 → `7_31 30-16x` stop (the manual's "30-16x divisor negative or zero (rem)"). It resumes with a zero result.
  - Otherwise a loop runs: shift left (`8_15 21-22(2)`), count down by P22 (`7_21 [P22]`), and conditionally subtract or add b (`7_13 19_3-23_3`, `8_23 19_3-22_3`). This is a binary long-division remainder.
  - A final sign correction adds b back if the remainder is negative (`7_11 21_3-27` → `8_17 19_3-22_3`), so the result lies in [0, b).
  - Then standardise. Finally, `Q30 18_1-23_3` and `8_1 21_3-28`: if the result equals b it is replaced by 0 (`7_15 30-16`).

  *Structure inferred, medium confidence.* Using "fetch integer" for b implies **non-integer divisors are not supported**; a fractional b would presumably be truncated. That is inference, though it agrees with the manual's only example, `a 1 rem`.
- **`pow`** (pp.34-35): `unpack two`, with DS19 = base a and DS20 = index b.
  - a = 0 → result zero (`7_5 30-21(2)`).
  - b = 0 → a constant built from `[P22]`, presumably 1.
  - a > 0 → set the flag `1_1` to 0.
  - a < 0 → test whether b is an integer. If it is fractional → `5_7 30-21(2)x` stop (the manual's "fractional power of negative number"). If it is an integer, its parity sets `1_1` (`5_13 29-1_1 [index "odd"]` or `5_12 30-1_1`).
  - Then `log` (p.32) of |a|, multiply by b (`8_1 0-24`), and `exp` (p.33).
  - If `1_1` is negative (odd index), the result is negated (`5_2`, `5_26`, `5_30`).
  - Then standardise.

  So **a^b = exp(b·ln|a|)**, with the sign fixed for a negative base and an integer index. *Written, with the interpretation high confidence.*
- **`log`** (p.32):
  - Unpack; save the exponent (`Q30 21_2-13`).
  - The "log" subroutine tests the argument: 0 or negative → `8_8 30-21(2)x` stop.
  - It compares the mantissa with the constants **√2 (30 b.p., written `26,12,30,4,8,13.1`)**, **1/√2 (30 b.p., `13,6,15,2,20,22.0`)** and **√2−1 (30 b.p.)**.
  - It calls the DEUCE library routines **M12** and **S06** ("Deuce subroutine").
  - Back in the main routine, the exponent is multiplied (`7_24 0-24`) by **logₑ2 (31 b.p.)** and added.

  **Inference:** the mantissa is reduced to about [1/√2, √2), its log is found with a library series (S06), and n·ln2 is added. I checked the constant notation: it is base-32 digits, least significant first, so `26,12,30,4,8,13.1` = 1518500250 = √2·2³⁰. What M12 and S06 do is **not determinable** from these pages.
- **`exp`** (p.33):
  - Unpack, then multiply by **log₂e (31 b.p., unsigned)**.
  - The "exp" subroutine splits the result into integer and fractional parts, using shifts and exponent tests against `[9P18]` and `[16P18]`.
  - Overflow → `7_20 30-21(2)x` stop (the manual's "result exceeds permissible magnitude").
  - It calls **S06** on the fractional part and adds the integer part into the exponent ("correct exponent"; `[11P5+21P17]`, `[7P17]`).

  **Inference:** eˣ = 2^(x·log₂e), with 2^frac from a library routine.
- **`√`** (p.37):
  - Unpack and save the exponent.
  - A parity test on the exponent (`7_9 25-28`) chooses a pre-shift of `21-22(18)` or `21-22(16)`.
  - Then a loop: `7_7 1-24` (hardware divide) …
    - `7_1 20_3-27`: a negative argument → `7_11 4-24 x` stop (the manual's "argument negative (√)", which resumes with zero).
    - `7_14 16-23_2`, `7_16 27-22_2`, then `7_18 22_2-28`.
    - **0** → exit. **φ** → update the estimate and repeat the divide.
  - On exit, the exponent is halved and the result packed.

  **Inference:** a Newton/Heron iteration x ← (x + N/x)/2, run until the correction is exactly zero, on a mantissa adjusted so the exponent is even.
- **`sin` / `cos`** (p.38): both share one routine. `sin` sets `29-19_2` (P32) and `cos` sets `30-19_2`, a flag. Then:
  - unpack;
  - range manipulation using shifts and sign tests (`28-25(10)`, `28-26(32)`, `28-25(16)`);
  - a hardware multiply `0-24` (x², inferred);
  - a loop that multiplies (`8_15 0-24`) and adds coefficients taken from DL8 at m.c. 14, 20, 4 and 3 (`Q30 (8-22_3)` "spill under");
  - a final multiply by x, then standardise.

  **Inference:** a polynomial (Horner) approximation in x² after reducing the argument by quarter-periods. **The coefficients are not shown on the page. They are held in the routine's track, so they might be recoverable from the coding sheets.**

### e. Numerals, and punch / read conversion

**Numerals (p.12).**

- The accumulator (TS13) is cleared, as is a flag/counter word, 19_3.
- For each symbol, `7_13 [P21]`, `8_15 25-28` tests whether the code has the P21 bit, i.e. whether it is in column 3. (The manual says column 3 means decimal point.)
  - **Digit** → `7_17 23-14(16)`, `7_2 [15P1]` (extract the digit), `7_5 13-16`, `7_7 16-25(9)`, `8_17 25-25`. That is **TS13 ← 10·TS13 + digit**, done as nine additions plus the original.
  - **Point** → `7_18 29-19_3`, which sets a flag.
  - The next symbol comes from CIN. If it is still a numeral, the loop repeats. 19_3 is updated on each pass (`8_9 19_3-21_3`, `8_11 22_3-19_3`); I read this as counting digits after the point, but that is inference.
- At the end of the numeral:
  - zero → stored as zero;
  - otherwise normalise with exponent start `[31P17]`, shifting left until the sign changes (`7_22 24-27` loop, `8_26 28-26`).
  - Then a loop: `7_0 [10P28]` (the constant 10); `7_9 1-24` (**divide**); `7_6 24-19_3` (step the counter); `7_8 28-26(2)` (adjust the exponent); `8_14 27-22_2`; `8_16 22_2-14`; `7_20 4-24`; back to normalise.
  - Then pack (`8_0 21-22(20)`) and `SN`, which stores the value as the second keyword.

**Answer.** The digit string is accumulated as an exact binary integer, ignoring the point, and normalised to floating point. **It is then divided by 10 once for each digit after the decimal point**, renormalising each time. *The loop structure is written, and the per-digit divide is inferred at medium confidence.* The `8_14 27-22_2` inside the loop adds a unit to the low word, which may be a rounding of each quotient; the page has two blots beside 8_14 and 8_16, possibly marking an amendment. **Rounding behaviour is not determinable.** The conversion happens at translation time, as the manual says.

**Punch (pp.47-48).**

- The mantissa is made positive ("mod. mantissa 30 b.p."), and the sign goes to the card separately (`1_17 14-27` → `27-11_0` or `27-11_1`).
- The decimal exponent is estimated as the binary exponent × **log₁₀2**. The constant is written "32 b.p. `2,10,19,0,17,6.1`" = 0.30103·2³².
- The power-of-ten scaling uses **log₂10** (30 b.p.), **S06** and **M12**.
- There is a scaling step by **10⁻²** (31 b.p., written `20,16,11,15,20,0.0` = 0.0100000·2³¹).
- Then a constant annotated "**r.o.** `3,31,20`" is added (`5_4 5_6-22_2`). The annotation might also read "+.o."; I read it as "round-off".
- p.48 has a second constant, "**9/10 × r.o.** `31,27,18`". The decoded values (21475 and 19327) are in the ratio 0.89998, about 0.9, which supports reading these as round-off constants.
- 21475 = 1.0·10⁻⁵·2³¹, or 2.0·10⁻⁵·2³⁰.

**Answer.** Punch output **is rounded** by adding a constant, with a 0.9 variant used on one branch (presumably after a ×10 rescale). Its value is about 10⁻⁵ in the routine's scaling. I **could not determine** exactly how that relates to half a unit in the 6th significant figure, or where truncation to six figures happens. The manual's statement (§2.3) that cols 8-10 and 12-17 are always zero is consistent with this, but not proven by it.

**Read (pp.44-46).** Card rows are assembled into digit codes (p.44). The mantissa digits P11-20 and exponent digits P1-10 are separated (p.45). The mantissa integer is scaled by **2²⁰·10⁻⁵** (26 b.p., written `18,24,26,2,31,20.0` = 10.48576·2²⁶), and the decimal exponent is turned into binary using **log₂10** with S06 and M12 (p.46). There is a "Coefficients" table on p.46 (4_12-4_19).

### f. How `]`, `↓`, `↑` and marks work

- **`]`** (p.23):
  - Fetch the top link-list entry through the "link list fetch quasi" (`10_28`).
  - **If the list is empty** (Q30 `(10-14)` "spill under") → `7_21 13-31` → `1_2 (see "finish" p.30)`, which is the **end of the program**. On p.30, "End of calculation" is shared with the space keyword (the program end) and ends with the stop `Q0 0-0x`.
  - Otherwise the entry is popped (`7_22 … [P17,22]`, `7_26 21_2-10_28`). The return address is rebuilt from its fields, and the sign is tested with `7_7 13-27`:
    - **`+` → subroutine return**: `7_16 10-31` → `1_29 control`.
    - **`-` → repeat**: `7_17 13-30` → "re-enter 'rep' p.27". That page either finishes the loop (count 0 → NK, with the entry already popped) or re-pushes the entry with the count decremented and jumps back.

  So the decision order is: empty list → end of program; sign digit set → repeat; sign digit clear → subroutine return. A `]` always closes the innermost open entry. *Written, high confidence.*
- **Marks** (p.11): "The address of the next keyword is stored in m.c. of D.L.12 corresponding with the mark number." If that m.c. already holds a positive entry, the translator stops on `3_30 2-2x` (the manual's "mark number used twice"). Empty slots are negative (P32, per manual App. II). p.14: "track 12/0 receives the mark list". **There is one `*` routine and one list. Subroutine-number marks and place marks are the same thing** and share the 32-slot list. *Written, high confidence.*
- **`↓` and `↑`** (p.25):
  - Both run the same code. `↓` sets `30-18_1` (0) and `↑` sets `31-18_1` (−1).
  - Both pop the mark number with "fetch integer", then run the conditional test described in (b).
  - On a skip, `7_15 18_1-27`:
    - **`↓`** (`+`) → `8_8 store link list` (p.24, entered at "8_8 28-13 ← from ↓").
    - **`↑`** (`-`) does not store a link.
  - Then both look up the mark list: `7_11 [31P17]` masks the number to 5 bits, i.e. **modulo 32** as the manual says, and `Q30 (11-14)` fetches the entry.
    - A negative (empty) entry → `7_2 25-29` stop loop (the manual's "skip to non-existent mark", 7 25-29).
    - Otherwise the flag is cleared and control jumps.
- **Link list**: "D.L.10 m.c. 18-23", so six entries (p.24). Overflow takes the "spill" path at `8_30 8_2-21_2 [zero]`, and I could not fully follow where it goes from there. The `↓` entry is built from source 28 (P17, inferred), with sign +. The `rep` entry has P32 added (`29-25`, p.26).

### g. Accumulator effects of individual symbols

- **The accumulator itself** (p.16): "Accumulator is D.L.10 m.c. 1-12." There is a pointer, the fetch quasi-instruction held in QS17 word 3 (17_3).
  - *Fetch* pops: it executes `(10-16)`, which loads the top cell into TS16, then decrements the pointer.
  - *Store* pushes TS16.
  - Both stop on `1_14 30-21_2 x` if the pointer spills, i.e. the accumulator is empty or more than 12 cells are used.
- **`(name)`** (p.17, Names (unsuffixed)): `1_9 17_3-21_3`, `1_11 10_25-22_3 [convert]`, `Q31 (10-16)`. This loads the top cell into TS16 **without updating the pointer**. Then the second keyword `Q21 16-12` stores it, and control goes to NKD. **The value is not popped.** *Written.*
- **`;`** (p.9): the keyword `4_23-15 → 1_25` is a plain *fetch* (pop) with the return link set to NK. **It pops and discards.** *Written.*
- **Unsuffixed variable** (p.17): `k 12-16` (load from DL12), then `1_20` store, which pushes it.
- **`dup`** (p.17): `k 3-24`, then `1_20 (see "variables" above)`. That is just a *store of whatever is in TS16*. This works because every routine leaves the value it last pushed or fetched in TS16. **Inferred quirk:** after `;`, TS16 holds the value that was just discarded, so `; dup` would push the discarded value back, not copy the new top. The same applies after a conditional skip (TS16 = the truth value) or at the restart of a `rep` body (TS16 = the new x). Medium confidence; worth checking against the coding sheets.
- **`rev`** (p.17): swaps the top two cells by direct DL10 transfers.
- **`R`** (p.39): read, then translate, then `Q0 13-16`, then reset, then `store and NKD`. It **pushes** one number.
- **`(P)`** (p.39): `8_22 17_3-13 [accumulator fetch quasi]` "convert to 10-15", `Q30 (10-15)`. This copies the top cell **without changing the pointer**, then punch-translates, punches, resets and goes to NKD. **No pop.** *Written.*
- **`R|` and `P|`** (pp.40-42): "assemble" calls **fetch integer twice**, popping the two limits. **`R||` and `P||`** (p.43): fetch integer **four times** (i, m, j, n). The name that follows is part of the second keyword (p.13: "G-code symbol × P26 plus … following name-symbol × P17"). Nothing is pushed; the routines end with NKD. *Written for the number of fetches; "nothing pushed" inferred from the absence of a store.*
- **`wait`** (p.23): copies the top cell without a pop (`Q31 (10-16)` from the fetch quasi), puts it on the output staticiser (`16-29`), and stops (`7_18 16-29 X`).
- **`I`** (p.9): keyword `0-16 → 1_20`, which pushes the raw word from source 0 (the input).

### h. Which version, and differences from the manual

- The flowcharts describe **GEORGE IA**, the semi-translation scheme. There is a full translator to "keywords" (pp.5-15), a mark list built at translation time, and keyword-driven calculation (p.16 onwards). Nothing describes the GEORGE II interpreter: there are no symbol-by-symbol interpretation loops and no test-stop or slow-motion facilities.
- Marginal notes refer to **later variants "1B" and "1C"**:
  - p.6: "6-31 in 1B"
  - p.8: an alternative "1B:" sequence
  - p.11: "(1B)" alternative
  - p.19: "1A & B same", "See next page also", "used 1C"
  - p.27: "8_13 10-31 NOT IN 1B;1C", "1_16 in 1B"

  The manual only mentions IA and II.
- Agreements with the manual: 12 cells in DL10 m.c. 1-12; a six-entry link list in DL10 m.c. 18-23; the mark list as one track, mod 32; the stops 2-2x, 25-29, 30-21x (log), 4-24x (÷ and √), 30-16x (rem), 14-21x ("exponent halved"), 30-21x (exp, pow) and 30-28x; the second keyword of numerals, names and long operations; the 512-symbol storage on position 11. The manual's diagnostic note (§2.5) says repeats are "distinguished by P30". The OCR may be garbled there. The flowchart marks repeats with the **sign digit** (source 29, which `]` tests with `13-27`).
- Corrections that are on the pages but not in the manual:
  - the struck second fetch in `=` (p.22);
  - the reversed truth/false branches and the added `27-23_2` in `>` (p.22), which is what makes `>` strict;
  - the divide-by-zero stop changed from "30-16x" to "4-24x" (p.20);
  - the `max` correction (p.21).

---

## 3. Pages or parts I could not read reliably

- **p.22, `=`**: the struck-through block (`7_23 7_25-15 / fetch / Q21 16-15`). It is unclear whether all three lines are cancelled. This leaves the final form of `=` undetermined.
- **p.8 (bottom)**: the "1B:" variant sequence is cramped and partly illegible.
- **p.19**: several struck or overwritten labels in the add routine (`4_9`/`4_10`, `4_11`-`4_13`, "4_15 to 4_2"). I could follow the routine, but some m.c. numbers are uncertain.
- **pp.28-29 (suffixed variables)**: legible, but I did not transcribe them in detail because they were not needed for questions a-h.
- **pp.33, 46-48**: some bracketed constants and annotations are faint. In particular, "r.o." versus "+.o." on pp.47-48, and one struck item on p.46 that may read "subtract r.o.".
- **p.38**: the `sin`/`cos` reduction logic on the left side is dense. The coefficient values are not on the page.
- None of the 52 pages was illegible as a whole.

The bit-level DEUCE semantics, in particular the shifting sources 22, 23 and 24 and the double-register word order, were not verified against a DEUCE manual. Everything labelled "inferred" in section 2 depends on them to some degree.
