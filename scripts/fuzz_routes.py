#!/usr/bin/env python3
"""Run random G-programs through both routes and compare what they do.

    python3 scripts/fuzz_routes.py [--count N] [--seed S] [--length L]
                                   [--ops LIST] [--single-shot] [--keep DIR]

Each program is run with `george run` (the interpreter written from the
manual) and `george deuce george` (Hamblin's own GEORGE IA from the
transcribed coding sheets, on the DEUCE simulator). What each does is taken
as a list of events in the order they happened (punched numbers, `wait`
lights, stops resumed) and how it ended: at the end of the program, or on a
stop identified by its indication. The interpreter's stop and the DEUCE's
indication are matched as `george deuce george` matches an indication to the
manual's stop table (an indication with two meanings matches either). The
outcome is classed as:

  same       the same events in the same order, and the same ending
  figure     the same except in the sixth significant figure of numbers
  astray     the interpreter stopped on a repetition variable below 1/2
             (Hamblin's code goes astray there: RUNNING-GEORGE.md, finding
             30) and Hamblin's code, after the same events, ended on
             anything other than the end of the program or a stop in the
             table
  different  anything else

The generator keeps the accumulator valid: it tracks the cells each piece
of program takes, leaves and needs on the way (`mod 1 + log` needs one
more), and the conditional state left by = and > (a later ↓ then takes a
cell as its condition), so that no program should stop on the accumulator,
the link list or a mark. The stops that are left come from the arithmetic
(exp or pow out of range, overflow); they are counted by kind.

Programs that are not "same" are written to --keep (default target/fuzz/)
with both outputs, for a closer look.
"""

import argparse
import collections
import os
import random
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
GEORGE = os.environ.get("GEORGE", os.path.join(ROOT, "target", "release", "george"))

# name: (cells taken, cells left, extra cells needed on the way, text)
OPS = {
    "+": (2, 1, 0, "+"),
    "-": (2, 1, 0, "-"),
    "×": (2, 1, 0, "×"),
    "÷": (2, 1, 0, "÷"),
    "neg": (1, 1, 0, "neg"),
    "mod": (1, 1, 0, "mod"),
    "dup": (1, 2, 0, "dup"),
    "rev": (2, 2, 0, "rev"),
    "√": (1, 1, 0, "mod √"),  # of a modulus, so never negative
    "exp": (1, 1, 1, "10 ÷ exp"),  # kept small
    "log": (1, 1, 1, "mod 1 + log"),  # of 1 + |x|
    "sin": (1, 1, 0, "sin"),
    "cos": (1, 1, 0, "cos"),
    "rem": (1, 1, 1, None),  # x b rem with b a numeral
    "pow": (1, 1, 1, None),  # |x| + 1 to a small power
    "max": (2, 1, 0, "max"),
    "store": (1, 1, 0, None),  # (v), leaving it
    "fetch": (0, 1, 0, None),  # v
    "suffix": (1, 1, 1, None),  # store in a vector and fetch back
    "rep": (0, 1, 3, None),  # a sum over a small repetition
    "cond": (1, 1, 2, None),  # if x > c skip over a neg: a conditional skip
    "logic": (2, 1, 1, None),  # a truth value from = > ~ & ∨
    "call": (1, 1, 2, None),  # a subroutine, attached after the main program
}
# each takes one cell and leaves one, needing at most two more on the way
SUBROUTINES = ["dup × 1 +", "2 × 1 -", "dup 0, > 31 ↑ neg * 31", "mod √"]
FIRST_SUB = 20
DEFAULT_OPS = [k for k in OPS if k != "max"]  # max is a known difference
LETTERS = "abcdefghk"
CELLS = 12
# marks for cond: not the subroutines' (20-23) nor the one inside them (31)
COND_MARKS = [k for k in range(31) if not FIRST_SUB <= k < FIRST_SUB + len(SUBROUTINES)]


def numeral(rng):
    kind = rng.random()
    if kind < 0.4:
        return str(rng.randint(0, 99))
    if kind < 0.8:
        whole = rng.randint(0, 999)
        frac = rng.randint(1, 9999)
        return f"{whole}.{frac}"
    return f".{rng.randint(1, 99999):05d}"


def program(rng, ops, length):
    out, depth, stored = [], 0, []
    marks = iter(COND_MARKS)
    called = set()
    # an = or > has left the conditional state on: the next ↓ or ↑ takes
    # the top cell as its condition
    pending = False
    for _ in range(length):
        choices = []
        for o in ops:
            take, leave, extra, _ = OPS[o]
            if o == "call" and pending:
                take += 1  # the condition as well
            if take <= depth and depth - take + leave <= CELLS - 1 and depth + extra + (leave > take) <= CELLS:
                choices.append(o)
        if depth < CELLS - 1 and (depth < 2 or rng.random() < 0.3 or not choices):
            choices.append("num")
        if "fetch" in choices and not stored:
            choices.remove("fetch")
        op = rng.choice(choices)
        if op == "num":
            # a comma after every numeral: GEORGE ignores it, and it keeps the
            # next numeral from running into this one
            out.append(numeral(rng) + ",")
            depth += 1
            continue
        take, leave = OPS[op][0], OPS[op][1]
        if op == "rem":
            out.append(f"mod {rng.randint(1, 9)} rem")
        elif op == "pow":
            out.append(f"mod 1 + {rng.choice(['2', '3', '.5', '1.5'])} pow")
        elif op == "store":
            v = rng.choice(LETTERS)
            stored.append(v)
            out.append(f"({v})")
        elif op == "fetch":
            out.append(rng.choice(stored))
        elif op == "suffix":
            k = rng.randint(0, 31)
            out.append(f"{k}|(x) ; {k}|x")
        elif op == "cond":
            k = next(marks, None)
            if k is None:
                out.append("neg")
            else:
                out.append(f"dup {numeral(rng)} > {k} ↑ neg * {k}")
            pending = False
        elif op == "logic":
            out.append(rng.choice(["=", ">", "> ~", "= ~"]) + " " + rng.choice(["", "dup &", "dup ∨", "0, ∨", "1 neg &"]))
            pending = True
        elif op == "call":
            j = rng.randrange(len(SUBROUTINES))
            called.add(j)
            out.append(f"{j + FIRST_SUB} ↓")
            if pending:
                take += 1
            pending = False
        elif op == "rep":
            a = rng.randint(1, 3)
            b = a + rng.randint(0, 4)
            out.append(f"0, {a}, {b} rep (i) i {rng.choice(['', 'dup ×', '2 ÷', 'sin'])} + ]")
        else:
            out.append(OPS[op][3])
        depth += leave - take
    tail = " ".join(["(P) ;"] * depth)
    subs = ""
    if called:
        # the program ends at its first ], before the subroutines; one ] each
        subs = " ]" + "".join(f"\n* {j + FIRST_SUB} {SUBROUTINES[j]} ]" for j in sorted(called))
    return f"{' '.join(out)} {tail}{subs}\n"


IND = re.compile(r"^\d+ \d+-\d+x?$")


def table_indications(indication):
    """The DEUCE indications a stop-table entry stands for ("1 30-21x (IA),
    1 1-1x (II)" is 1 30-21x on IA)."""
    return {" ".join(i.split(" ")[:2]) for i in indication.split(", ")}


def run_interpreter(path, extra=()):
    p = subprocess.run([GEORGE, "run", *extra, path], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, timeout=120)
    events, end = [], ("end",)
    lines = p.stdout.splitlines()
    k = 0
    while k < len(lines):
        line = lines[k]
        nxt = lines[k + 1] if k + 1 < len(lines) else ""
        if line.startswith("stop "):
            ind = line[5:].split(":")[0]
            resumed = "(single shot" in line or "(single shot" in nxt
            if nxt.startswith("  at symbol"):
                k += 1
            if resumed:
                events.append(("resumed", frozenset(table_indications(ind))))
            else:
                end = ("stop", frozenset(table_indications(ind)) if ind != "none" else "astray")
        elif line.startswith("wait:"):
            events.append(("wait", line))
        elif line.startswith("george: gave up"):
            end = ("limit",)
        elif line.startswith("george:"):
            end = ("error", line)
        else:
            events.append(("P", line))
        k += 1
    return events, end, p.stdout


def run_deuce(path, extra=()):
    p = subprocess.run(
        [GEORGE, "deuce", "george", *extra, path], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, timeout=120
    )
    events, end = [], None
    for line in p.stdout.splitlines():
        if line.startswith("warning:"):
            continue
        if line.startswith("end:"):
            end = ("end",)
        elif line.startswith("stop "):
            events.append(("resumed", frozenset({line[5:].split(":")[0]})))
        elif line.startswith("wait:"):
            events.append(("wait", line))
        elif ": TS COUNT " in line:
            outcome = line.split(":")[0]
            ind = line.split(": TS COUNT ")[1].split(" (")[0]
            end = ("limit",) if outcome == "Limit" else ("stop", ind, "stop table" in line)
        elif line.startswith("george:"):
            end = ("error", line)
        else:
            events.append(("P", line))
    return events, end or ("error", "no ending"), p.stdout


def figures_close(a, b):
    try:
        x, y = float(a), float(b)
    except ValueError:
        return False
    if x == y:
        return True
    scale = max(abs(x), abs(y))
    return abs(x - y) <= 1.5e-5 * scale


def events_match(i_ev, h_ev, close):
    if len(i_ev) != len(h_ev):
        return False
    for a, b in zip(i_ev, h_ev):
        if a[0] != b[0]:
            return False
        if a[0] == "P":
            if not (a[1] == b[1] or (close and figures_close(a[1], b[1]))):
                return False
        elif a[0] == "resumed":
            if not (a[1] & b[1]):
                return False
        elif a[1] != b[1]:
            return False
    return True


def ends_match(i_end, h_end):
    if i_end[0] == "end" or h_end[0] == "end":
        return i_end == h_end
    if i_end[0] == "stop" and h_end[0] == "stop" and i_end[1] != "astray":
        return h_end[1] in i_end[1]
    return i_end == h_end


def stop_name(end):
    if end[0] != "stop":
        return end[0]
    if end[1] == "astray":
        return "rep astray"
    return " / ".join(sorted(end[1]))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--count", type=int, default=200)
    ap.add_argument("--seed", type=int, default=1958)
    ap.add_argument("--length", type=int, default=8)
    ap.add_argument("--ops", default=",".join(DEFAULT_OPS))
    ap.add_argument("--single-shot", action="store_true", help="resume resumable stops on both routes")
    ap.add_argument("--keep", default=os.path.join(ROOT, "target", "fuzz"))
    a = ap.parse_args()
    ops = [o for o in a.ops.split(",") if o]
    rng = random.Random(a.seed)
    os.makedirs(a.keep, exist_ok=True)
    tally = collections.Counter({"same": 0, "figure": 0, "astray": 0, "different": 0})
    i_stops, h_stops = collections.Counter(), collections.Counter()
    for n in range(a.count):
        src = program(rng, ops, rng.randint(2, a.length))
        path = os.path.join(a.keep, f"p{n:05d}.george")
        with open(path, "w") as f:
            f.write(src)
        extra = ["--single-shot"] if a.single_shot else []
        i_ev, i_end, i_all = run_interpreter(path, extra)
        h_ev, h_end, h_all = run_deuce(path, extra)
        if i_end[0] != "end":
            i_stops[stop_name(i_end)] += 1
        if h_end[0] != "end":
            h_stops[h_end[1] if h_end[0] == "stop" else h_end[0]] += 1
        if events_match(i_ev, h_ev, False) and ends_match(i_end, h_end):
            kind = "same"
        elif events_match(i_ev, h_ev, True) and ends_match(i_end, h_end):
            kind = "figure"
        elif (
            i_end == ("stop", "astray")
            and events_match(i_ev, h_ev, False)
            and (h_end[0] == "limit" or (h_end[0] == "stop" and not h_end[2]))
        ):
            kind = "astray"
        else:
            kind = "different"
        tally[kind] += 1
        if kind == "same":
            os.remove(path)
        else:
            with open(path + f".{kind}.txt", "w") as f:
                f.write(f"{src}\n--- george run\n{i_all}\n--- george deuce george\n{h_all}")
    total = sum(tally.values())
    print(f"{total} programs (seed {a.seed}): " + ", ".join(f"{v} {k}" for k, v in tally.items()))
    print(f"george run stopped: {sum(i_stops.values())}" + "".join(f"\n  {v:5}  {k}" for k, v in i_stops.most_common()))
    print(
        f"george deuce george stopped: {sum(h_stops.values())}"
        + "".join(f"\n  {v:5}  {k}" for k, v in h_stops.most_common())
    )
    print(f"kept for a closer look: {a.keep}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
