use std::{env, error, process};
extern crate getopts;
use getopts::Options;

type Result<T> = ::std::result::Result<T, Box<dyn error::Error>>;
type Sequence = (f64, f64, f64);

fn main() {
    if let Err(err) = run(env::args().skip(1).collect()) {
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
    let seq = getseq(&matches.free).or(Err(opts.short_usage("seq") + " [first [incr]] last"))?;
    let sep = matches.opt_str("s").unwrap_or(String::from("\n"));
    let width = if matches.opt_present("w") {
        format!("{}", if seq.0 <= seq.2 { seq.2 } else { seq.0 }).len()
    } else {
        1
    };

    emitseq(seq, &sep, width);
    if let Some(term) = matches.opt_str("t") {
        print!("{}", term);
    }
    Ok(())
}

fn options() -> Options {
    let mut opts = Options::new();
    opts.optflag("w", "", " Equalize the widths of all numbers");
    opts.optopt("s", "", "Use string to separate numbers", "string");
    opts.optopt(
        "t",
        "",
        "Use string to terminate sequence of numbers",
        "string",
    );
    opts
}

fn getseq(args: &Vec<String>) -> Result<Sequence> {
    let seq: Sequence = match args.len() {
        1 => (1.0, 1.0, args[0].trim().parse()?),
        2 => (args[0].trim().parse()?, 1.0, args[1].trim().parse()?),
        3 => (
            args[0].trim().parse()?,
            args[1].trim().parse()?,
            args[2].trim().parse()?,
        ),
        _ => return Err("Not enough arguments".into()),
    };
    Ok(seq)
}

fn emitseq(seq: Sequence, sep: &str, width: usize) {
    let mut i = seq.0;
    if seq.0 <= seq.2 {
        while i <= seq.2 {
            print!("{:0>2$}{}", i, sep, width);
            i += seq.1;
        }
    } else {
        while i >= seq.2 {
            print!("{:0>2$}{}", i, sep, width);
            i -= seq.1;
        }
    }
}
