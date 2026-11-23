# DEUCE documents

These are the documents the DEUCE simulator (`src/deuce/`) was built from.
They are on David Green's DEUCE archive,
<https://www.uraone.com/computers/deuce/deucdocs.html>, and were downloaded
on 23 November 2026. The PDFs are not kept in the repository (`.gitignore`
has `sources/*/*.pdf`). Page renders and Tesseract OCR are in `pages/`, which
is not kept either, and nor are the saved copies of web pages, `deucdocs.html`
and `ldm/`: fetch them from the addresses given.

```
cd sources/deuce
for f in DEUCE_Prgmg_Manual_May56.pdf DEUCEProgrammingCourse.pdf \
         NSy88_BasicProgramming_2ndIssue.pdf MSB200_Guide_to_Programming.pdf \
         DEUCEConsole.pdf subs/m12.pdf subs/s06.pdf zp13-1.pdf zp13-16.pdf; do
  curl -sLO http://uraone.com/computers/deuce/$f
done
pdftoppm -r 200 -gray -png DEUCE_Prgmg_Manual_May56.pdf pages/pm/p   # and so on
tesseract pages/pm/p-33.png pages/pm/p-33
```

| File | Document | Pages | What it covers | How it was read |
|---|---|---|---|---|
| `DEUCE_Prgmg_Manual_May56.pdf` | English Electric, *DEUCE Programming Manual*, NS-y-16/5-56 (May 1956). Cited here as **PM**. | 85 | The whole machine: number representation (§2), the store (§3), the instruction word and its timing (§5), every source and destination (§6, Appendix 2), multiplier and divider (§6.8-6.10), drum (§3.3, §6.4), reader and punch (§4.8, §6.3, Appendix 1 timings), initial input, triads and the initial card (§8). Appended (pp. 82-85): the Automatic Instruction Modifier. | OCR of every page at 200 dpi. Tables, the coding examples and Appendix 2 were checked on the page images. **The main source.** |
| `NSy88_BasicProgramming_2ndIssue.pdf` | Price and Davis, *Basic DEUCE Programming* (2nd issue), English Electric report NS y 88, 29 April 1958. Cited as **BP** with its sheet numbers. | 54 | A later course text that repeats PM with some corrections. The instruction-word layout diagram (sheet 19), the card columns (sheet 20: P1 in column 21 ... P32 in column 52), triads (sheet 20), reader and punch (sheets 28-29), DS21 and S22 (sheets 32-33), multiplier (sheets 33-35), divider (sheets 36-38), drum (sheets 40-41), TCA (sheet 42). | OCR of every page. Used to cross-check PM. |
| `DEUCEProgrammingCourse.pdf` | English Electric (Kidsgrove), *DEUCE Programming Course* lecture notes, c. 1960 ("DPCS2"). Cited as **Course**, by PDF page. | 356 | 31 lectures. Lecture 14 (PDF pp. 144-146), "Automatic coding by Joe digits": P22-P25 of an instruction are the "Joe digits", which link W and T so that adding P17 to W repeatedly "spills" into T after a set count. PDF p. 329: the full list of destination triggers as at 1 June 1960 (Mark II/IIA additions such as 64-column working, paper tape, magnetic tape). | OCR of every page at 150 dpi. Only the lectures named here were read closely. |
| `MSB200_Guide_to_Programming.pdf` | *DEUCE Guide to Programming*, MSB 200. | 12 | A short summary. | OCR; skimmed. |
| `DEUCEConsole.pdf` | D. G. Burnett-Hall, *The DEUCE Console*, Computing Note M.S./B./201. | 12 | Console keys: CONT TCI, CONT TT, CONT TIL, DISCRIM, PROG DISPLAY, RELEASE, the STOP key (NORMAL / STOP / AUG. STOP), the SINGLE SHOT key, the O.S. and I.D. lamp rows, the "external tree", RUN IN / RUN OUT on the reader. | OCR; pp. 1-4 read. |
| `m12.pdf` | DEUCE library subroutine M12, "fast multiplication with fixed shift" (N. Dowell, NS t 9). GEORGE's `log` uses it (flow diagrams p. 32). | 3 | | OCR of the front sheet only. |
| `s06.pdf` | DEUCE library subroutine S06, a second-order subroutine "for summing a power series of up to 32 terms" (N. Dowell, NS t 10). Used by GEORGE's `log`, `exp` and numeral conversion. | 3 | | OCR of the front sheet only. |
| `zp13-1.pdf`, `zp13-16.pdf` | ZP13/1 "Clear drum (one card programme)"; ZP13-ZP16 clear drum, read to drum, write clock track, synchronise. The GEORGE basic pack's first card is "a modified DEUCE 'clear drum' card". | 3, 6 | | Not yet read. |
| `deucdocs.html` | The archive's index page. | | | |

`MACHINE.md` is the reading of the machine that the simulator follows.

## Other DEUCE simulators

A search on 23 November 2026 found one: a prototype "DEUCE emulator" for
Windows, macOS and Linux from the University of Glasgow's 60th-anniversary
work, distributed as binaries on itch.io
(<https://dasha1362.itch.io/deuce-emulator>), whose author says it "isn't
complete" and does not yet match the machine. The Glasgow School of
Computing Science history page also mentions a Java DEUCE simulator written
by students. No source code was found, and none was used here. The Computer
Conservation Society's emulator list
(<https://www.computerconservationsociety.org/emu/index.htm>) has no DEUCE
entry.

## Also here

- `library/m12-s06-dl6.txt`: the DEUCE library routines M12 and S06,
  coded for DL6 and transcribed from `m12.pdf` and `s06.pdf`. They fill
  GEORGE track 15/0, which sheet 17 only names.
- `RUNNING-GEORGE.md`: running Hamblin's GEORGE IA from the coding sheets
  on the simulator, with findings and results.
- `ldm/`: English Electric, *DEUCE Logical Design Manual*, Part I,
  NS-y-37/11-57. It is the web transcription on the Barrett pages,
  <https://www.uraone.com/computers/Barrett/ldmpart1.html> (`ldmpart1.html`,
  with a plain-text copy `ldmpart1.txt`), downloaded 23 November 2026, with
  Figure 10.9 (`ldmpics/fig10.9.jpg`; Figure 10.8 is missing on the
  server).
  - Chapter 10, "The Multiplier-Divider Unit", pp. 55-75, gives the
    multiplier digit by digit (pp. 60-65), the long accumulator's carry and
    sign extension (pp. 58-60), S22, and the divider's method.
  - Part II (the drawings) is linked from
    <https://www.uraone.com/computers/Barrett/Manuals.html> but was not
    used.
