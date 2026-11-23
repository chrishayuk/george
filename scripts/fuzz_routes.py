#!/usr/bin/env python3
"""Run random G-programs through both routes and compare what they punch.

    python3 scripts/fuzz_routes.py [--count N] [--seed S] [--ops LIST] [--keep DIR]

Each program is generated so that the accumulator never underflows or holds
more than twelve cells. It is run with `george run` (the interpreter written
from the manual) and `george deuce george` (Hamblin's own GEORGE IA from the
transcribed coding sheets, on the DEUCE simulator), and the outcome is
classed as:

  same       the same numbers punched, the same way of ending
  figure     the same except in the sixth significant figure
  different  anything else

Programs that are not "same" are written to --keep (default
target/fuzz/) with both outputs, for a closer look.
"""

import argparse
import os
import random
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
GEORGE = os.environ.get("GEORGE", os.path.join(ROOT, "target", "release", "george"))

# name: (cells taken, cells left, text)
OPS = {
    "+": (2, 1, "+"),
    "-": (2, 1, "-"),
    "×": (2, 1, "×"),
    "÷": (2, 1, "÷"),
    "neg": (1, 1, "neg"),
    "mod": (1, 1, "mod"),
    "dup": (1, 2, "dup"),
    "rev": (2, 2, "rev"),
    "√": (1, 1, "mod √"),          # of a modulus, so never negative
    "exp": (1, 1, "10 ÷ exp"),      # kept small
    "log": (1, 1, "mod 1 + log"),   # of 1 + |x|
    "sin": (1, 1, "sin"),
    "cos": (1, 1, "cos"),
    "rem": (1, 1, None),            # x b rem with b a numeral
    "pow": (1, 1, None),            # |x| + 1 to a small power
    "max": (2, 1, "max"),
    "store": (1, 1, None),          # (v), leaving it
    "fetch": (0, 1, None),          # v
    "suffix": (1, 1, None),         # store in a vector and fetch back
    "rep": (0, 1, None),            # a sum over a small repetition
    "cond": (1, 1, None),           # if x > c skip over a neg: a conditional skip
    "logic": (2, 1, None),          # a truth value from = > ~ & ∨
    "call": (1, 1, None),           # a subroutine, attached after the main program
}
SUBROUTINES = ["dup × 1 +", "2 × 1 -", "dup 0, > 31 ↑ neg * 31", "mod √"]
DEFAULT_OPS = [k for k in OPS if k != "max"]  # max is a known difference
LETTERS = "abcdefghk"


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
    marks = iter(range(30))
    called = set()
    for _ in range(length):
        choices = [o for o in ops if OPS[o][0] <= depth and depth - OPS[o][0] + OPS[o][1] <= 11]
        if depth < 2 or rng.random() < 0.3:
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
        elif op == "logic":
            out.append(rng.choice(["=", ">", "> ~", "= ~"]) + " " + rng.choice(["", "dup &", "dup ∨", "0, ∨", "1 neg &"]))
        elif op == "call":
            j = rng.randrange(len(SUBROUTINES))
            called.add(j)
            out.append(f"{j + 20} ↓")
        elif op == "rep":
            a = rng.randint(1, 3)
            b = a + rng.randint(0, 4)
            out.append(f"0, {a}, {b} rep (i) i {rng.choice(['', 'dup ×', '2 ÷', 'sin'])} + ]")
        else:
            out.append(OPS[op][2])
        depth += OPS[op][1] - OPS[op][0]
    tail = " ".join(["(P) ;"] * depth)
    subs = "".join(f" ]\n* {j + 20} {SUBROUTINES[j]} ]" for j in sorted(called))
    if subs:
        # the program ends at its first ], before the subroutines; one ] each
        subs = " ]" + "".join(f"\n* {j + 20} {SUBROUTINES[j]} ]" for j in sorted(called))
    return f"{' '.join(out)} {tail}{subs}\n"


def run(args, path):
    p = subprocess.run([GEORGE, *args, path], capture_output=True, text=True, timeout=120)
    lines = [l for l in p.stdout.splitlines() if not l.startswith("end:")]
    ended = "end" if p.returncode == 0 else "stop"
    stop = ""
    for l in (p.stderr + p.stdout).splitlines():
        if l.startswith("stop") or "stop table" in l:
            stop = l.split(":")[0].replace("stop ", "").strip()
            break
    return lines, ended, stop, p.stdout + p.stderr


def figures_close(a, b):
    try:
        x, y = float(a), float(b)
    except ValueError:
        return False
    if x == y:
        return True
    scale = max(abs(x), abs(y))
    return abs(x - y) <= 1.5e-5 * scale


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--count", type=int, default=200)
    ap.add_argument("--seed", type=int, default=1958)
    ap.add_argument("--length", type=int, default=8)
    ap.add_argument("--ops", default=",".join(DEFAULT_OPS))
    ap.add_argument("--keep", default=os.path.join(ROOT, "target", "fuzz"))
    a = ap.parse_args()
    ops = [o for o in a.ops.split(",") if o]
    rng = random.Random(a.seed)
    os.makedirs(a.keep, exist_ok=True)
    tally = {"same": 0, "figure": 0, "different": 0}
    for n in range(a.count):
        src = program(rng, ops, rng.randint(2, a.length))
        path = os.path.join(a.keep, f"p{n:05d}.george")
        with open(path, "w") as f:
            f.write(src)
        i_lines, i_end, _, i_all = run(["run"], path)
        h_lines, h_end, _, h_all = run(["deuce", "george"], path)
        if i_lines == h_lines and i_end == h_end:
            kind = "same"
        elif i_end == h_end and len(i_lines) == len(h_lines) and all(
            figures_close(x, y) for x, y in zip(i_lines, h_lines)
        ):
            kind = "figure"
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
    print(f"kept for a closer look: {a.keep}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
