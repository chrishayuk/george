# Sources

The scans are not kept in the repository. They are on David Green's DEUCE
archive at <https://www.uraone.com/computers/deuce/deucdocs.html> (section
"GEORGE"), scanned from the UTECOM originals and supplied by Robin Vowels.
Downloaded on 22 November 2026 into the directories below:

| Directory | Document | File |
| --- | --- | --- |
| `manual-1958/` | C. L. Hamblin, *GEORGE IA and II: A semi-translation programming scheme for DEUCE. Programming and Operation Manual*, University of New South Wales, probably mid 1958. 23 pp., typed. | `GEORGEProgrammingManual.pdf` |
| `flowcharts-1958/` | *GEORGE flow diagrams*, probably mid 1958. 52 pp., handwritten. | `GEORGEFlowCharts.pdf` |
| `coding-sheets-1958/` | The program on UTECOM coding sheets ("UTECOM Programme GEORGE"), probably mid 1958. 22 pp., handwritten DEUCE instructions. | `GEORGEProgramCode.pdf` |

```
curl -O http://uraone.com/computers/deuce/GEORGEProgrammingManual.pdf
curl -O http://uraone.com/computers/deuce/GEORGEFlowCharts.pdf
curl -O http://uraone.com/computers/deuce/GEORGEProgramCode.pdf
pdftoppm -r 150 -gray -png GEORGEProgrammingManual.pdf pages/p
```

The archive's note on them: "The flowcharts and coding sheets displayed
here (the only extant copy) correspond to the penultimate version of GEORGE
IA. This version accepted the G-codes as 8-bit numbers, punched one, two,
three, or four per row of an 80-column card." The manual describes the
final card form, symbol codes punched in decimal as pairs, sixteen to a
card.

The archive also lists "GEORGE and Alphacode Exercises Set II", 2 pages,
which is not online.

## What the manual says, against what is usually said

Most accounts of GEORGE, including the Wikipedia article and the summary
this project started from, differ from the manual in ways that matter:

- **The versions.** The manual describes GEORGE IA, a "semi-translation"
  scheme that turns symbols into *keywords*, which are DEUCE instructions
  leading to stored routines, and then obeys them; and GEORGE II, a "pure
  interpretive" scheme for testing programs. The archive lists four
  versions, 1A and 1B (compilers) and 2A and 2B (interpreters), differing in
  capacity (512 or 1024 symbols). The manual knows only IA and II and 512
  symbols. "Compiler" overstates IA: its keywords call routines; they do not
  do the arithmetic in line.
- **The symbols.** Reading is `R` and punching is `(P)`, not `read` and
  `punch`. Skips are `↑` and `↓` with marks `* n`, not words. The coding
  table (Appendix I) has 134 symbols in ten columns.
- **The example** `a y dup × × b y × + c + (e)` is not in the manual. Its
  notation is consistent with it.
- **The "rep" example** usually quoted, `1, 10 rep (i) read dup × punch ; ]`,
  is not in the manual either. The manual's nearest is
  `1, 20 rep (j) j j × (P) ; ]`.
- **The debugging.** GEORGE II's facilities (stop at each symbol, stop on a
  given symbol, slow motion, symbols inserted by hand from the input keys,
  inspection of the accumulator in DL 10, minor cycles 1-12) are real and are
  in section 2.7.

## Reading

`manual-1958/pages/p-NN.txt` (not kept) is Tesseract OCR of the 150 dpi
pages, good enough for the prose. Every program, the coding table, the stop
table and the keyword table were read from the page images, not the OCR.
`flowcharts-1958/READING.md` is an index of the flow diagrams and what they
show about points the manual leaves open.

All of this reading was done by a coding agent and its sub-agents. No
person has yet checked it against the pages.
