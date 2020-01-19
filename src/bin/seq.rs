use std::{env, error, process};
extern crate getopts;
use getopts::Options;

type Result<T> = ::std::result::Result<T, Box<dyn error::Error>>;
type Sequence = (isize, usize, isize);

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if let Err(err) = run(args) {
        eprintln!("{}", err);
        process::exit(2);
    }
}

fn run(argv: Vec<String>) -> Result<()> {
    let opts = options();
    let matches = match opts.parse(argv) {
        Ok(m) => m,
        Err(f) => return Err(f.to_string().into()),
    };
    let seq = getseq(&matches.free).or(Err(opts.short_usage("seq")))?;
    emitseq(seq);
    if let Some(term) = matches.opt_str("t") {
        print!("{}", term);
    }
    Ok(())
}

fn options() -> Options {
    let mut opts = Options::new();
    opts.optflag("w", "", " Equalize the widths of all numbers");
    opts.optopt("f", "", "Use printf-style FORMAT for each number", "FORMAT");
    opts.optopt("s", "", "Use STRING to separate numbers", "STRING");
    opts.optopt(
        "t",
        "",
        "Use STRING to terminate sequence of numbers",
        "STRING",
    );
    opts
}

fn getseq(args: &Vec<String>) -> Result<Sequence> {
    let seq: Sequence = match args.len() {
        1 => (1, 1, args[0].trim().parse()?),
        2 => (args[0].trim().parse()?, 1, args[1].trim().parse()?),
        3 => (
            args[0].trim().parse()?,
            args[1].trim().parse()?,
            args[2].trim().parse()?,
        ),
        _ => return Err("Not enough arguments".into()),
    };
    Ok(seq)
}

fn emitseq(seq: Sequence) {
    for num in (seq.0..=seq.2).step_by(seq.1) {
        println!("{}", num);
    }
}
