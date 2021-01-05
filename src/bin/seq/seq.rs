use getopts::Options;
use std::{cmp, error, io::Write, result, str};

type Result<T> = result::Result<T, Box<dyn error::Error>>;
type Sequence = (f64, f64, f64, usize);

macro_rules! usage {
    () => {
        "Usage: seq [-w] [-s string] [-t string] [first [incr]] last"
    };
}

macro_rules! width {
    ($x:expr) => {
        cmp::max(
            format!("{0:.1$}", $x.0, $x.3).len(),
            format!("{0:.1$}", $x.2, $x.3).len(),
        )
    };
}

pub fn run(out: &mut dyn Write, argv: &[String]) -> Result<()> {
    let opt = options()
        .parse(argv)
        .or_else(|e| Err(e.to_string() + "\n" + usage!()))?;

    let seq = getseq(&opt.free)?;
    let sep = opt.opt_str("s").unwrap_or("\n".to_string());
    let width = if opt.opt_present("w") { width!(seq) } else { 1 };

    emitseq(out, &seq, &sep, width, &opt.opt_str("t"))
}

fn options() -> Options {
    let mut opts = Options::new();
    opts.optflag("w", "equal-width", "");
    opts.optopt("s", "separator", "", "");
    opts.optopt("t", "terminator", "", "");
    opts
}

macro_rules! float {
    ($x:expr) => {
        $x.trim().parse().or(Err(
            "seq: invalid floating point argument: ".to_string() + &$x
        ))?
    };
}

fn getseq(args: &[String]) -> Result<Sequence> {
    let mut seq: Sequence = match args.len() {
        1 => (1.0, 0.0, float!(args[0]), 0),
        2 => (float!(args[0]), 0.0, float!(args[1]), 0),
        3 => {
            let s: Sequence = (float!(args[0]), float!(args[1]), float!(args[2]), 0);

            // Make sure the increment is valid.
            if s.1 == 0.0 {
                return Err(
                    format!("seq: zero {}crement", if s.0 < s.2 { "in" } else { "de" }).into(),
                );
            }
            if s.1 <= 0.0 && s.0 < s.2 {
                return Err("seq: needs positive increment".into());
            }
            if s.1 >= 0.0 && s.0 > s.2 {
                return Err("seq: needs negative decrement".into());
            }
            s
        }
        _ => return Err(usage!().into()),
    };

    // Set the default increment.
    if seq.1 == 0.0 {
        seq.1 = if seq.0 < seq.2 { 1.0 } else { -1.0 };
    }

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

fn emitseq(
    out: &mut dyn Write,
    s: &Sequence,
    sep: &str,
    width: usize,
    term: &Option<String>,
) -> Result<()> {
    let mut cur = s.0;
    let mut iter = 0isize;

    while if s.0 <= s.2 { cur <= s.2 } else { cur >= s.2 } {
        write!(out, "{:0>1$.2$}{3}", cur, width, s.3, sep)?;
        iter += 1;
        cur = s.0 + s.1 * iter as f64;
    }

    if let Some(term) = term {
        write!(out, "{}", term)?;
    }
    Ok(())
}

#[path = "seq_test.rs"]
#[cfg(test)]
mod test;
