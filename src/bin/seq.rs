use std::io::stdout;
use std::{env, process};
fn main() {
    if let Err(err) = seq::run(&mut stdout(), env::args().skip(1).collect()) {
        eprintln!("{}", err);
        process::exit(2);
    }
}

mod seq {
    use std::io::Write;
    use std::{cmp, error, str};
    extern crate getopts;
    use getopts::Options;

    type Result<T> = ::std::result::Result<T, Box<dyn error::Error>>;
    type Sequence = (f64, f64, f64, usize);

    pub fn run(out: &mut dyn Write, argv: Vec<String>) -> Result<()> {
        let opts = options();
        let matches = match opts.parse(argv) {
            Ok(m) => m,
            Err(f) => return Err(f.to_string().into()),
        };
        let seq =
            getseq(&matches.free).or(Err(opts.short_usage("seq") + " [first [incr]] last"))?;
        let sep = matches.opt_str("s").unwrap_or("\n".to_string());
        let width = if matches.opt_present("w") {
            cmp::max(
                format!("{0:.1$}", seq.0, seq.3).len(),
                format!("{0:.1$}", seq.2, seq.3).len(),
            )
        } else {
            1
        };

        emitseq(out, seq, &sep, width, matches.opt_str("t"))?;
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

    fn emitseq(
        out: &mut dyn Write,
        seq: Sequence,
        sep: &str,
        width: usize,
        term: Option<String>,
    ) -> Result<()> {
        let mut cur = seq.0;
        let mut iter = 0isize;
        if seq.0 <= seq.2 {
            while cur <= seq.2 {
                write!(out, "{:0>1$.2$}{3}", cur, width, seq.3, sep)?;
                iter += 1;
                cur = seq.0 + seq.1 * iter as f64;
            }
            if cur < seq.2 {
                write!(out, "{:0>1$.2$}{3}", seq.2, width, seq.3, sep)?;
            }
        } else {
            while cur >= seq.2 {
                write!(out, "{:0>1$.2$}{3}", cur, width, seq.3, sep)?;
                iter += 1;
                cur = seq.0 - seq.1 * iter as f64;
            }
            if cur > seq.2 {
                write!(out, "{:0>1$.2$}{3}", seq.2, width, seq.3, sep)?;
            }
        }
        if let Some(term) = term {
            write!(out, "{}", term)?;
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn test_options() -> Result<()> {
            let opts = options();
            // Start with no args.
            let matches = opts.parse(vec![""; 0])?;
            for opt in vec!["w", "s", "t"] {
                assert!(matches.opt_defined(opt), "Option -{} not defined", opt);
                assert!(
                    !matches.opt_present(opt),
                    "Option -{} should not be present",
                    opt
                );
            }
            assert_eq!(matches.free, vec![""; 0], "Should have no free strings");

            // Try one arg.
            let matches = opts.parse(vec!["10"])?;
            for opt in vec!["w", "s", "t"] {
                assert!(
                    !matches.opt_present(opt),
                    "Option -{} should not be present",
                    opt
                );
            }
            assert_eq!(matches.free, vec!["10"], "Should have one free string");

            // Add the -w flag.
            let matches = opts.parse(vec!["-w", "8"])?;
            assert!(matches.opt_present("w"), "Option -w not found");
            assert!(!matches.opt_present("s"), "Option -s should not be found");
            assert!(!matches.opt_present("t"), "Option -t should not be found");
            assert_eq!(matches.free, vec!["8"], "Should have one free string");

            // Add additional args.
            let matches = opts.parse(vec!["-w", "8", "10"])?;
            assert!(matches.opt_present("w"), "Option -w not found");
            assert!(!matches.opt_present("s"), "Option -s should not be found");
            assert!(!matches.opt_present("t"), "Option -t should not be found");
            assert_eq!(
                matches.free,
                vec!["8", "10"],
                "Should have two free strings"
            );

            // Add negated args.
            let matches = opts.parse(vec!["-w", "--", "-8", "10"])?;
            assert!(matches.opt_present("w"), "Option -w not found");
            assert!(!matches.opt_present("s"), "Option -s should not be found");
            assert!(!matches.opt_present("t"), "Option -t should not be found");
            assert_eq!(
                matches.free,
                vec!["-8", "10"],
                "Should have negated and non-negated free strings"
            );

            // Add other args.
            let matches = opts.parse(vec!["-w", "-t", "foo", "-s", "^", "10"])?;
            assert!(matches.opt_present("w"), "Option -w not found");
            assert!(matches.opt_present("s"), "Option -s not found");
            assert!(matches.opt_present("t"), "Option -t not found");
            assert_eq!(
                matches.opt_str("s"),
                Some("^".to_string()),
                "Missing -s string"
            );
            assert_eq!(
                matches.opt_str("t"),
                Some("foo".to_string()),
                "Missing -t string"
            );
            assert_eq!(matches.free, vec!["10"], "Should have one free string");

            Ok(())
        }
    }
}
