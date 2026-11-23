#!/usr/bin/env python3
"""Check that every program in the tutorial runs, and prints what the
tutorial says it prints.

Three checks are made.

1. Every program in programs/ is run with the release binary:

       george run programs/NN-name.george [--data NN-name.in] [ARGS]

   ARGS are read from NN-name.args, if there is one. What it punches
   (standard output) must be NN-name.out. What it says on the error stream
   (stops, the output lights, the trace) must be NN-name.err, or nothing
   if there is no .err file. A program may end with a stop, or fail to be
   read, only if it has a .err file.

2. A ```george block in a chapter is a program, and must be the whole
   text of one of the files in programs/. Every file in programs/ must be
   shown in a chapter, and run in a ```console block.

3. A ```console block holds one or more commands, each on a line that
   starts `$ george`, followed by what the command prints. The command is
   run in this folder, and what it prints, on both streams, is compared.
   A line `$ cat FILE` is followed by the text of the file.

    python3 check_tutorial.py            check
    python3 check_tutorial.py --write    rewrite .out, .err and the console
                                         blocks from real runs

The program is taken from $GEORGE, or from ../target/release/george.
"""
import os
import pathlib
import re
import shlex
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent
PROGRAMS = HERE / "programs"
BLOCK = re.compile(r"^```(\w+)\n(.*?)^```", re.S | re.M)


def binary():
    if os.environ.get("GEORGE"):
        return os.environ["GEORGE"]
    path = ROOT / "target" / "release" / "george"
    if path.exists():
        return str(path)
    sys.exit("no program found: run `cargo build --release` or set $GEORGE")


def program_args(path):
    args = ["run", str(path.relative_to(HERE))]
    data = path.with_suffix(".in")
    if data.exists():
        args += ["--data", str(data.relative_to(HERE))]
    extra = path.with_suffix(".args")
    if extra.exists():
        args += shlex.split(extra.read_text())
    return args


def run(george, args, merged=False):
    result = subprocess.run(
        [george] + args, capture_output=not merged, text=True, timeout=60, cwd=HERE,
        **({"stdout": subprocess.PIPE, "stderr": subprocess.STDOUT} if merged else {}))
    return result.stdout, (result.stderr or ""), result.returncode


def tidy(text):
    """Lines without trailing blanks, and no blank lines at the end."""
    return "\n".join(line.rstrip() for line in text.rstrip("\n").split("\n")).rstrip("\n")


def main():
    write = "--write" in sys.argv
    george = binary()
    failures = []
    programs = sorted(PROGRAMS.glob("*.george"))

    # 1. Each program against its .out and .err.
    for path in programs:
        out, err, code = run(george, program_args(path))
        name = path.name
        for suffix, actual in ((".out", out), (".err", err)):
            expected_path = path.with_suffix(suffix)
            if write:
                if actual or suffix == ".out":
                    expected_path.write_text(actual)
                elif expected_path.exists():
                    expected_path.unlink()
                continue
            expected = expected_path.read_text() if expected_path.exists() else ""
            if suffix == ".out" and not expected_path.exists():
                failures.append(f"{name}: no {path.stem}.out")
            elif actual != expected:
                failures.append(f"{name}: {suffix[1:]} differs\n--- expected\n{expected}--- got\n{actual}")
        if code != 0 and not path.with_suffix(".err").exists() and not write:
            failures.append(f"{name}: exit status {code} with no .err file")

    # 2 and 3. The chapters.
    texts = {p.read_text().rstrip("\n") + "\n": p for p in programs}
    shown, ran = set(), set()
    console_checked = 0
    for chapter in sorted(HERE.glob("[0-9][0-9]-*.md")):
        text = chapter.read_text()
        edits = []
        for block in BLOCK.finditer(text):
            kind, body = block.group(1), block.group(2)
            if kind == "george":
                match = texts.get(body.rstrip("\n") + "\n")
                if match is None:
                    failures.append(f"{chapter.name}: a george block is not a file in programs/:\n{body}")
                else:
                    shown.add(match)
            elif kind == "console":
                lines = body.rstrip("\n").split("\n")
                new_lines = []
                k = 0
                while k < len(lines):
                    line = lines[k]
                    k += 1
                    if not line.startswith(("$ george", "$ cat ")):
                        new_lines.append(line)
                        continue
                    expected = []
                    while k < len(lines) and not lines[k].startswith("$ "):
                        expected.append(lines[k])
                        k += 1
                    if line.startswith("$ cat "):
                        out = (HERE / line[len("$ cat "):].strip()).read_text()
                    else:
                        args = shlex.split(line[len("$ george"):])
                        for a in args:
                            if a.startswith("programs/") and a.endswith(".george"):
                                ran.add(HERE / a)
                        out, _, _ = run(george, args, merged=True)
                    console_checked += 1
                    actual = tidy(out)
                    new_lines.append(line)
                    new_lines.extend(actual.split("\n") if actual else [])
                    if actual != tidy("\n".join(expected)):
                        failures.append(f"{chapter.name}: {line}\n--- the chapter says\n"
                                        + "\n".join(expected) + f"\n--- george prints\n{actual}")
                new_body = "\n".join(new_lines) + "\n"
                if write and new_body != body:
                    edits.append((block.start(2), block.end(2), new_body))
        if write and edits:
            for start, end, new in reversed(edits):
                text = text[:start] + new + text[end:]
            chapter.write_text(text)
            print(f"{chapter.name}: rewrote {len(edits)} block(s)")

    for path in programs:
        if path not in shown:
            failures.append(f"{path.name}: not shown in any chapter")
        if path not in ran:
            failures.append(f"{path.name}: not run in any console block")

    if write:
        print(f"wrote the expected output of {len(programs)} programs")
        failures = [f for f in failures if "not shown" in f or "not run" in f or "george block" in f]
    for f in failures:
        print(f, end="\n\n")
    print(f"{len(programs)} programs and {console_checked} commands checked, {len(failures)} failed")
    sys.exit(1 if failures else 0)


if __name__ == "__main__":
    main()
