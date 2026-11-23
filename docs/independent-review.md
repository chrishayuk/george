# Independent review

## Who reviewed, and what was checked

The repository was reviewed by OpenAI Codex CLI (codex-cli 0.166.0) on 23
November 2026. It is an independent second model, not a person. It checked
the code against the sources, ran everything, and looked for claims that go
beyond the evidence. It changed no files outside `target/`. Other agents
were still editing the DEUCE simulator during the review, so it froze a
snapshot of the sources and reproduced its findings against that.

It ran the build, `cargo test --workspace` (84 passed, 1 ignored at the
time), clippy, `cargo fmt --check`, the tutorial checker (100 programs, 113
commands), the WebAssembly build and tests (16 examples, 89 checks) and the
VS Code extension's tests (12), all of which passed. It ran every example
through both `george run` and Hamblin's own code (`george deuce george`) and
got the same 14 identical and 2 differing in the sixth figure. It worked the
Simpson erratum out independently and agrees with it, and checked all 134
entries of the coding table against `src/symbols.rs`.

It checked 31 places in the coding-sheet transcription and 7 claims about
the flow diagrams against 200 dpi renders of the scans, and the transcription
of the DEUCE library routines S06 and M12 against the library sheets.

Each finding below was checked again here, by running it on both the
interpreter and Hamblin's code, before anything was changed.

## Spot-check counts

Of the 31 coding-sheet places:

- 22 agree with the scan, including all five sin/cos coefficient rows on
  sheet 21, the sheet-1 T = 27 resolution, and the three places where a
  sheet contradicts its flow diagram (each "clearly" as transcribed);
- 8 could not be told either way (a cramped subscript, a blotted or
  overwritten digit); most of these were already marked doubtful;
- 1 is a reconstruction presented as a reading: sheet 21, DL8 track 15/13,
  m.c. 21, which had been resolved from flow diagram p. 38. The reviewer
  could not read it securely on the scan.

Of the 7 flow-diagram claims, 6 agree; the seventh, that p. 38 gives the
sin/cos coefficient addresses as m.c. 14, 20, 4 and 3, is wrong. Those
numbers mark intermediate transfers. (The simulator work had reached the
same conclusion.) The S06 and M12 transcriptions match the library sheets.

## Findings, and what was done

| # | Finding | Checked here | Done |
| --- | --- | --- | --- |
| 1 | `rem` loses the remainder when the quotient passes double precision: `2, 100 pow 3 rem` gave 0 | Hamblin's code gives 1 | Worked out exactly from the mantissa and exponent; now 1 |
| 2 | On Hamblin's code `max`, sin, cos and `pow` of minus a power of two give wrong results | Reproduced | `max` and `pow` traced to the code as written (ERRATA.md); sin and cos fixed by simulating the multiplier digit by digit from the DEUCE Logical Design Manual, and now agree to six figures |
| 3 | `george format` changed programs: `R |a` became `R|a`, `* 1 2` became `* 12` | Reproduced | Fixed; the printed form reads back as the same program, tested |
| 4 | `R||` and `P||` were not held to the letters a to d | Reproduced (`P||(e)` aliased matrix a) | Refused when read, as `||` is |
| 5a | "Only the sixth figure is affected" by rounding is false after later calculation | `1, 2097152 ÷ 1 + 1 - 2097152 ×`: 2 here, 0 on Hamblin's code | DECISIONS.md 2 corrected; the + and - rule is still being pinned down |
| 5b | Underflow gave zero; Hamblin's code stops | 3 14-21x on his code for 1e-154 ÷ 4; 7 30-21x for `1000 neg exp` | Adopted: both stop |
| 5c | `=` compared values; the flow diagram compares words | `0 ~ 0 + 0 ~ =`: true here, false on his code | Adopted: words |
| 6 | Sheet 21 m.c. 21 and the blanket P31 rule promote reconstruction to transcription | Agreed | The literal reading is restored (struck, doubtful, both layers noted); the reconstruction is a separate run-time overlay (`george::RECONSTRUCTIONS`). Each class of P31 word is justified separately in RUNNING-GEORGE.md, and the one that rests only on "it makes it run" says so |
| 7 | End-to-end tests check output but not how the run ended, and skip `matrix` | Agreed | Every example now asserts how it ends, matrix included; Hamblin's Simpson values are pinned exactly |
| 8 | The divider accepts equal magnitudes the Programming Manual forbids | Agreed | Checked, with a test |
| 9 | Wording: ERRATA.md credits 0.374597 to "GEORGE" (Hamblin's code gives 0.374596); "exactly 0.9" (it is 0.89998); "computes sin and cos exactly"; chapter 14 says the reconstruction rests on the manual and flow diagrams alone, and "there is no evidence" | Agreed | All reworded |

Found while checking finding 5: Hamblin's punch rounds a tie at the sixth
figure up (2^-10 punches 0.000976563); `george run` gave 0.000976562. The
punch now rounds the exact value, halves up. (Later superseded: the punch
was then traced instruction by instruction and is now Hamblin's own
fixed-point conversion, word for word; see RUNNING-GEORGE.md finding 25.)

The reviewer also noted that the agreement between the two routes is only
partly independent: both share the program reader, the symbol codes, the
preparation of data cards and the decoding of punched output. Agreement
cannot check those shared parts.

## What the reviewer could not check

Every transcription row and every flow-diagram path; whether the remaining
failures on Hamblin's code are his or the reconstruction's; the P31
assignment against an original deck, since none survives; the multiplier
and divider's behaviour part-way through an operation; and chapter 14's
history against its primary publications.
