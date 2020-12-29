use std::{cmp, env, error, process, str};
extern crate getopts;
use getopts::Options;

type Result<T> = ::std::result::Result<T, Box<dyn error::Error>>;
type Sequence = (f64, f64, f64, usize);

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
        cmp::max(
            format!("{0:.1$}", seq.0, seq.3).len(),
            format!("{0:.1$}", seq.2, seq.3).len(),
        )
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
    let mut seq: Sequence = match args.len() {
        1 => (1.0, 1.0, args[0].trim().parse()?, 0),
        2 => (args[0].trim().parse()?, 1.0, args[1].trim().parse()?, 0),
        3 => (
            args[0].trim().parse()?,
            (args[1].trim().parse::<f64>()?).abs(),
            args[2].trim().parse()?,
            0,
        ),
        _ => return Err("Not enough arguments".into()),
    };

    // Determine precision. Necessary because format!() has no equivalent to
    // the sprintf %g format found in other languages.
    for num in args {
        if let Some(idx) = num.chars().position(|x| x == '.') {
            // Keep the greater precision.
            seq.3 = seq.3.max(num.len() - (idx + 1));
        }
    }

    Ok(seq)
}

fn emitseq(seq: Sequence, sep: &str, width: usize) {
    let mut cur = seq.0;
    let mut iter = 0.0;
    if seq.0 <= seq.2 {
        while cur <= seq.2 {
            print!("{:0>1$.2$}{3}", cur, width, seq.3, sep);
            iter += 1.0;
            cur = seq.0 + seq.1 * iter;
        }
        if cur < seq.2 {
            print!("{:0>1$.2$}{3}", seq.2, width, seq.3, sep);
        }
    } else {
        while cur >= seq.2 {
            print!("{:0>1$.2$}{3}", cur, width, seq.3, sep);
            iter += 1.0;
            cur = cur - seq.1 * iter;
        }
        if cur > seq.2 {
            print!("{:0>1$.2$}{3}", seq.2, width, seq.3, sep);
        }
    }
}
