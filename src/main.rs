use std::process::ExitCode;

use george::machine::{self, Event, Machine, Options, Stop};
use george::number::Card;
use george::symbols::{Sym, ALL};
use george::text::{self, Program};

const USAGE: &str = "\
george run FILE [--data FILE] [--columns] [--input N,N,...] [--keys N,N,...]
               [--cards] [--trace] [--single-shot] [--limit N]
george deck FILE        the program as punched: symbol codes, sixteen to a card
george format FILE      the program in the manual's printed form
george table            the coding table (Appendix I)
george stops            the stops (section 2.4)
george deuce ...        the DEUCE simulator (george deuce help)

run obeys a G-program and prints what it punches, one number to a line.
  --data FILE    data cards, one number to a line in ordinary notation
  --columns      ... or, with this, as the twenty card columns of section 2.3
  --input        data cards given on the command line
  --keys         numbers set on the input keys, one for each I obeyed
  --cards        print punched cards as their twenty columns
  --trace        print each symbol obeyed and the accumulator after it
  --single-shot  carry on through resumable stops, as the operator could
  --limit N      give up after N symbols obeyed (default 10000000)";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("george: {message}");
            ExitCode::from(2)
        }
    }
}

fn load(path: &str) -> Result<Program, String> {
    let src = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    text::parse(&src).map_err(|e| format!("{path}: {e}"))
}

fn numbers(list: &str) -> Result<Vec<f64>, String> {
    list.split(',')
        .map(|s| s.trim().parse::<f64>().map_err(|_| format!("{s} is not a number")))
        .collect()
}

fn data_cards(src: &str, columns: bool) -> Result<Vec<Card>, String> {
    let mut cards = Vec::new();
    for (n, line) in src.lines().enumerate() {
        let line = line.split('#').next().unwrap_or("");
        if line.trim().is_empty() {
            continue;
        }
        let card = if columns {
            Card::parse_columns(line.trim_end())
        } else {
            Card::from_decimal(line)
        };
        cards.push(card.ok_or_else(|| format!("data line {}: cannot read {:?}", n + 1, line.trim()))?);
    }
    Ok(cards)
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let Some(command) = args.first() else {
        println!("{USAGE}");
        return Ok(ExitCode::SUCCESS);
    };
    match command.as_str() {
        "table" => {
            print_table();
            Ok(ExitCode::SUCCESS)
        }
        "stops" => {
            for s in machine::ALL_STOPS {
                println!(
                    "{:<28} {}{}",
                    s.indication,
                    s.cause,
                    s.resume.map(|r| format!("; S.S. {r}")).unwrap_or_default()
                );
            }
            Ok(ExitCode::SUCCESS)
        }
        "deck" | "format" => {
            let file = args.get(1).ok_or(USAGE)?;
            let program = load(file)?;
            if command == "format" {
                println!("{}", text::format(&program.symbols));
            } else {
                print_deck(&program.symbols);
            }
            Ok(ExitCode::SUCCESS)
        }
        "run" => run_program(&args[1..]),
        "deuce" => deuce_command(&args[1..]),
        "help" | "--help" | "-h" => {
            println!("{USAGE}");
            Ok(ExitCode::SUCCESS)
        }
        other => Err(format!("unknown command {other}\n\n{USAGE}")),
    }
}

fn run_program(args: &[String]) -> Result<ExitCode, String> {
    let mut file = None;
    let mut options = Options {
        limit: 10_000_000,
        ..Options::default()
    };
    let (mut data, mut columns, mut input, mut cards) = (None, false, None, false);
    let mut k = 0;
    while k < args.len() {
        let value = || args.get(k + 1).cloned().ok_or(format!("{} needs a value", args[k]));
        match args[k].as_str() {
            "--data" => {
                data = Some(value()?);
                k += 1;
            }
            "--input" => {
                input = Some(value()?);
                k += 1;
            }
            "--keys" => {
                options.keys = numbers(&value()?)?;
                k += 1;
            }
            "--limit" => {
                options.limit = value()?.parse().map_err(|_| "--limit needs a whole number")?;
                k += 1;
            }
            "--columns" => columns = true,
            "--cards" => cards = true,
            "--trace" => options.trace = true,
            "--single-shot" => options.single_shot = true,
            a if a.starts_with("--") => return Err(format!("unknown option {a}")),
            a => file = Some(a.to_string()),
        }
        k += 1;
    }
    let file = file.ok_or(USAGE)?;
    let program = load(&file)?;
    let mut machine = Machine::new();
    if let Some(path) = data {
        let src = std::fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))?;
        machine.cards.extend(data_cards(&src, columns)?);
    }
    if let Some(list) = input {
        for x in list.split(',') {
            machine
                .cards
                .push_back(Card::from_decimal(x).ok_or(format!("{x} is not a number"))?);
        }
    }
    let outcome = machine.run(&program, &options);
    // Punched numbers and trace lines are printed in the order they happened:
    // each trace line records how many events had come before it.
    let mut trace = outcome.trace.iter().peekable();
    let print_trace_upto = |n: usize, trace: &mut std::iter::Peekable<std::slice::Iter<machine::TraceLine>>| {
        while let Some(line) = trace.next_if(|l| l.events <= n) {
            let sym = program.symbols[line.at];
            let shown = if sym.is_numeral() {
                numeral_at(&program, line.at)
            } else {
                sym.text()
            };
            let acc: Vec<String> = line.accumulator.iter().map(|n| Card::of(*n).decimal()).collect();
            eprintln!("{:>4}  {:<8} {}", line.at, shown, acc.join("  "));
        }
    };
    for (k, event) in outcome.events.iter().enumerate() {
        print_trace_upto(k, &mut trace);
        match event {
            Event::Punched(n) => {
                let c = Card::of(*n);
                println!("{}", if cards { c.columns() } else { c.decimal() });
            }
            Event::Lights(n) => eprintln!(
                "wait: the output lights show {}",
                n.map(|n| Card::of(n).decimal())
                    .unwrap_or("the empty-accumulator indication".into())
            ),
            Event::Resumed(stop) => eprintln!(
                "{} (single shot: {})",
                describe(stop, &program),
                stop.kind.resume.unwrap()
            ),
        }
    }
    print_trace_upto(usize::MAX, &mut trace);
    if outcome.limit_reached {
        eprintln!("george: gave up after {} symbols obeyed (--limit)", outcome.obeyed);
        return Ok(ExitCode::from(1));
    }
    match &outcome.stop {
        None => Ok(ExitCode::SUCCESS),
        Some(stop) => {
            eprintln!("{}", describe(stop, &program));
            Ok(ExitCode::from(1))
        }
    }
}

fn numeral_at(program: &Program, at: usize) -> String {
    let mut k = at;
    let mut s = String::new();
    while k < program.symbols.len() && program.symbols[k].is_numeral() {
        s.push_str(&text::format(&program.symbols[k..k + 1]));
        k += 1;
    }
    s
}

fn describe(stop: &Stop, program: &Program) -> String {
    let mut s = format!("stop {}: {}", stop.kind.indication, stop.kind.cause);
    if let Some(d) = &stop.detail {
        s += &format!(" ({d})");
    }
    if let Some(at) = stop.at {
        s += &format!(
            "\n  at symbol {} \"{}\", {}",
            at,
            program.symbols[at].text(),
            program.positions[at]
        );
    }
    s
}

fn print_table() {
    let cols = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 15];
    print!("    ");
    for c in cols {
        print!("{c:>6}");
    }
    println!();
    for row in 0..16u8 {
        print!("{row:>4}");
        for c in cols {
            let cell = match Sym::from_code(row, c) {
                Some(Sym::Digit(n)) if n >= 10 => format!("{n}̄"),
                Some(s) => s.text(),
                None if (row, c) == (0, 0) => "-".into(),
                None => "".into(),
            };
            print!("{:>6}", cell);
        }
        println!();
    }
    println!("\nThe code of each symbol is its row number followed by its column number.");
    println!("Following an asterisk, symbols of column 3 represent numbers 16-31:");
    println!("otherwise any symbol of column 3 represents decimal point.");
    let _ = ALL;
}

/// "Symbol-codes of a program are punched consecutively up to sixteen per
/// card, the first number of each code in an odd column and the second in
/// the following even column. Code numbers 10, 11, ... 15 are punched as
/// 0--5 with an overpunching in X-row." The last card has two adjacent
/// Y-row punches.
fn print_deck(symbols: &[Sym]) {
    let chunks: Vec<&[Sym]> = symbols.chunks(16).collect();
    for (n, card) in chunks.iter().enumerate() {
        let codes: Vec<String> = card
            .iter()
            .map(|s| {
                let (r, c) = s.code();
                format!("{r}/{c}")
            })
            .collect();
        let last = if n + 1 == chunks.len() {
            "   (Y-row: last card)"
        } else {
            ""
        };
        println!("card {:>2}: {}{}", n + 1, codes.join(" "), last);
    }
}

const DEUCE_USAGE: &str = "\
george deuce run IMAGE [--start DL,MC] [--cards DECK] [--id N] [--trace]
                       [--single-shot] [--stop-key] [--limit N]
george deuce boot FILE... [--cards DECK] [--id N] [--trace] [--single-shot] [--limit N]
george deuce cards IMAGE    the image's blocks as punched cards (deck text)
george deuce decode WORD    a word (any constant form) as an instruction
george deuce george FILE [--data FILE] [--columns] [--input N,N,...] [--full]
                         [--keys N,N,...] [--single-shot] [--trace] [--limit N]
                            run a G-program through Hamblin's GEORGE IA (the
                            transcribed coding sheets) and print what it punches.
                            The wait stop (1 16-29x) is reported and given a
                            single shot. --keys sets the I.D. to each number
                            (as a GEORGE word) when its I is obeyed;
                            --single-shot also resumes the resumable stops

run loads a store image (the loader text form, or a transcribed coding sheet)
and obeys it from --start or the image's start line. boot puts the files' cards
in the reader (images become cards, rows Y X 0 1 then m.c. 0-31, in the order
of the card numbers in their block headers; .deck files follow as they are)
and presses INITIAL INPUT.
  --cards DECK   more cards for the reader after these (deck text form)
  --id N         set the input dynamiciser keys
  --trace        print each instruction obeyed (to stderr)
  --single-shot  press the single-shot key whenever a stopper waits for it
  --stop-key     every instruction is a stopper
  --limit N      give up after N instructions (default 10000000)";

fn deuce_command(args: &[String]) -> Result<ExitCode, String> {
    use george::deuce::{self, cards, loader, word};
    let read = |path: &str| std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"));
    let image = |path: &str| loader::parse_image(&read(path)?).map_err(|e| format!("{path}: {e}"));
    let Some(sub) = args.first() else {
        println!("{DEUCE_USAGE}");
        return Ok(ExitCode::SUCCESS);
    };
    match sub.as_str() {
        "cards" => {
            let img = image(args.get(1).ok_or(DEUCE_USAGE)?)?;
            let deck: Vec<cards::Card> = img.blocks.iter().flat_map(|b| b.cards()).collect();
            print!("{}", cards::format_deck(&deck));
            Ok(ExitCode::SUCCESS)
        }
        "decode" => {
            let w = word::parse_constant(&args[1..].join(" "))?;
            println!("{w:#010x}  {}  {}", word::Instr::decode(w), word::p_notation(w));
            Ok(ExitCode::SUCCESS)
        }
        "george" => deuce_george(&args[1..]),
        "run" | "boot" => {
            let mut files = Vec::new();
            let (mut start, mut extra, mut id, mut trace, mut shots) = (None, None, None, false, false);
            let mut config = deuce::Config::default();
            let mut limit: u64 = 10_000_000;
            let mut k = 1;
            while k < args.len() {
                let value = || args.get(k + 1).cloned().ok_or(format!("{} needs a value", args[k]));
                match args[k].as_str() {
                    "--start" => {
                        let v = value()?;
                        let (d, m) = v.split_once(',').ok_or("--start needs DL,MC")?;
                        start = Some((
                            d.parse::<u8>().map_err(|_| "bad DL")?,
                            m.parse::<u8>().map_err(|_| "bad m.c.")?,
                        ));
                        k += 1;
                    }
                    "--cards" => {
                        extra = Some(value()?);
                        k += 1;
                    }
                    "--id" => {
                        id = Some(word::parse_constant(&value()?)?);
                        k += 1;
                    }
                    "--limit" => {
                        limit = value()?.parse().map_err(|_| "--limit needs a whole number")?;
                        k += 1;
                    }
                    "--trace" => trace = true,
                    "--single-shot" => shots = true,
                    "--stop-key" => config.stop_key = true,
                    a if a.starts_with("--") => return Err(format!("unknown option {a}")),
                    a => files.push(a.to_string()),
                }
                k += 1;
            }
            if files.is_empty() {
                return Err(DEUCE_USAGE.into());
            }
            let mut m = deuce::Machine::new(config);
            if trace {
                m.trace_on();
            }
            if sub == "run" {
                let img = image(&files[0])?;
                m.load_image(&img);
                if let Some((d, mc)) = start {
                    m.start(d, mc);
                } else if img.start.is_none() {
                    return Err("no start: give --start DL,MC or a start line".into());
                }
            } else {
                // Blocks from images go in card-number order (as the sheet
                // headers number them); .deck files follow as they are.
                let mut blocks = Vec::new();
                for f in files.iter().filter(|f| !f.ends_with(".deck")) {
                    blocks.extend(image(f)?.blocks);
                }
                blocks.sort_by_key(|b| b.card_numbers().0.unwrap_or(u32::MAX));
                m.load_cards(blocks.iter().flat_map(|b| b.cards()));
                for f in files.iter().filter(|f| f.ends_with(".deck")) {
                    m.load_cards(cards::parse_deck(&read(f)?).map_err(|e| format!("{f}: {e}"))?);
                }
            }
            if let Some(f) = extra {
                m.load_cards(cards::parse_deck(&read(&f)?).map_err(|e| format!("{f}: {e}"))?);
            }
            if let Some(id) = id {
                m.id = id;
            }
            if sub == "boot" {
                m.initial_input();
            }
            let mut shown = 0;
            let outcome = loop {
                let left = limit.saturating_sub(m.obeyed);
                let o = m.run(left.min(10_000));
                if let Some(tr) = &m.trace {
                    for s in &tr[shown..] {
                        eprintln!("{s}");
                    }
                    shown = tr.len();
                    if shown > 100_000 {
                        m.trace = Some(Vec::new());
                        shown = 0;
                    }
                }
                match o {
                    deuce::Outcome::Limit if m.obeyed < limit => continue,
                    deuce::Outcome::AwaitSingleShot if shots && m.obeyed < limit => m.single_shot(),
                    o => break o,
                }
            };
            for (n, c) in m.punched().iter().enumerate() {
                println!("card {}", n + 1);
                for (r, w) in c.rows.iter().enumerate() {
                    println!(
                        "{:>2}: {}  {:>11}  {}",
                        cards::ROW_NAMES[r],
                        word::backward_binary(*w),
                        *w as i32,
                        word::p_notation(*w)
                    );
                }
            }
            for w in &m.warnings {
                eprintln!("warning: {w}");
            }
            eprintln!(
                "{:?} after {} instructions, {} m.c.; TS COUNT {} from {:?}; O.S. {}; alarm {}; cards read {}, left {}",
                outcome,
                m.obeyed,
                m.now,
                word::Instr::decode(m.count),
                m.count_from,
                word::backward_binary(m.os),
                if m.alarm { "on" } else { "off" },
                m.cards_read(),
                m.cards_left()
            );
            Ok(ExitCode::SUCCESS)
        }
        _ => {
            println!("{DEUCE_USAGE}");
            Ok(ExitCode::SUCCESS)
        }
    }
}

/// `george deuce george FILE`: boot the basic pack from the transcribed
/// sheets, present the G-program and its data, run, and decode the cards.
fn deuce_george(args: &[String]) -> Result<ExitCode, String> {
    use george::deuce::{self, george as g, word};
    let (mut file, mut data, mut columns, mut input, mut full, mut trace) = (None, None, false, None, false, false);
    let mut limit: u64 = 20_000_000;
    let mut op = g::Operator::default();
    let mut k = 0;
    while k < args.len() {
        let value = || args.get(k + 1).cloned().ok_or(format!("{} needs a value", args[k]));
        match args[k].as_str() {
            "--data" => {
                data = Some(value()?);
                k += 1;
            }
            "--input" => {
                input = Some(value()?);
                k += 1;
            }
            "--limit" => {
                limit = value()?.parse().map_err(|_| "--limit needs a whole number")?;
                k += 1;
            }
            "--keys" => {
                op.keys = numbers(&value()?)?;
                k += 1;
            }
            "--single-shot" => op.single_shot = true,
            "--columns" => columns = true,
            "--full" => full = true,
            "--trace" => trace = true,
            a if a.starts_with("--") => return Err(format!("unknown option {a}")),
            a => file = Some(a.to_string()),
        }
        k += 1;
    }
    let file = file.ok_or(DEUCE_USAGE)?;
    let program = load(&file)?;
    let mut numbers = Vec::new();
    if let Some(path) = data {
        let src = std::fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))?;
        numbers.extend(data_cards(&src, columns)?);
    }
    if let Some(list) = input {
        for x in list.split(',') {
            numbers.push(Card::from_decimal(x).ok_or(format!("{x} is not a number"))?);
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let blocks = g::george_blocks(root)?;
    let bytes: Vec<u8> = program.symbols.iter().map(|s| s.byte()).collect();
    let deck: Vec<deuce::Card> = numbers.iter().map(g::data_card).collect();
    let (m, run) = g::run_operated(g::boot_george(&blocks)?, &blocks, &bytes, deck, limit, trace, &op)?;
    if let Some(tr) = &m.trace {
        for s in tr {
            eprintln!("{s}");
        }
    }
    // Punched numbers and what the operator saw, in the order they happened.
    let mut console = run.console.iter().peekable();
    let show_console_upto = |n: usize, console: &mut std::iter::Peekable<std::slice::Iter<g::Console>>| {
        while let Some(c) = console.next_if(|c| match c {
            g::Console::Lights { punched, .. } | g::Console::Resumed { punched, .. } => *punched <= n,
        }) {
            match c {
                // With the accumulator empty, wait shows the word below the
                // bottom cell, 10_0, which holds the mask P1-10.
                g::Console::Lights { os, .. } => {
                    let shown = if *os == 0x3FF {
                        "the empty-accumulator indication".to_string()
                    } else {
                        Card::of(george::Num::from_word(*os)).decimal()
                    };
                    if full {
                        eprintln!(
                            "wait: the output lights show {shown} (O.S. {})",
                            word::backward_binary(*os)
                        );
                    } else {
                        eprintln!("wait: the output lights show {shown}");
                    }
                }
                g::Console::Resumed { stop, indication, .. } => eprintln!(
                    "stop {indication}: {} (single shot: {})",
                    stop.cause,
                    stop.resume.unwrap()
                ),
            }
        }
    };
    for (k, c) in run.punched.iter().enumerate() {
        show_console_upto(k, &mut console);
        match g::Punched::read(c) {
            Ok(p) if full => println!("{:<12} {}", p.six_figures().decimal(), p.columns()),
            Ok(p) => println!("{}", p.six_figures().decimal()),
            Err(e) => println!("? card not a number: {e}"),
        }
    }
    show_console_upto(usize::MAX, &mut console);
    let ins = word::Instr::decode(run.count);
    let indication = format!("{} {}-{}{}", ins.nis, ins.s, ins.d, if ins.go { "" } else { "x" });
    // Every meaning the manual's stop table (section 2.4) gives this
    // indication; some indications have more than one.
    let extra = [("5 9-24", "S.S. missed (reader fault), data read")];
    let mut meanings: Vec<String> = machine::ALL_STOPS
        .iter()
        .filter(|s| {
            s.indication
                .split(", ")
                .any(|i| i.split(' ').take(2).collect::<Vec<_>>().join(" ") == indication)
        })
        .map(|s| format!("{} ({})", s.cause, s.indication))
        .collect();
    meanings.extend(
        extra
            .iter()
            .filter(|(i, _)| *i == indication)
            .map(|(i, c)| format!("{c} ({i})")),
    );
    let known = if meanings.is_empty() {
        String::new()
    } else {
        format!("; stop table: {}", meanings.join(" OR "))
    };
    match run.outcome {
        deuce::Outcome::ReaderEmpty if ins.s == 0 && ins.d == 0 => {
            eprintln!("end: {indication} with the reader called (end of program)");
            Ok(ExitCode::SUCCESS)
        }
        o => {
            eprintln!(
                "{o:?}: TS COUNT {indication} ({}) from {:?}; O.S. {}{}",
                ins,
                run.count_from,
                word::backward_binary(run.os),
                known
            );
            for w in &m.warnings {
                eprintln!("warning: {w}");
            }
            Ok(ExitCode::from(1))
        }
    }
}
