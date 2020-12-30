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
            1 => (1.0, 0.0, args[0].trim().parse()?, 0),
            2 => (args[0].trim().parse()?, 0.0, args[1].trim().parse()?, 0),
            3 => {
                let s: Sequence = (
                    args[0].trim().parse()?,
                    args[1].trim().parse()?,
                    args[2].trim().parse()?,
                    0,
                );

                // Make sure the increment is valid.
                if s.1 == 0.0 {
                    return Err(
                        format!("zero {}crement", if s.0 < s.2 { "in" } else { "de" }).into(),
                    );
                }
                if s.1 <= 0.0 && s.0 < s.2 {
                    return Err("needs positive increment".into());
                }
                if s.1 >= 0.0 && s.0 > s.2 {
                    return Err("needs negative decrement".into());
                }
                s
            }
            0 => return Err("Not enough arguments".into()),
            _ => return Err("Too many arguments".into()),
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
                cur = seq.0 + seq.1 * iter as f64;
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

        #[test]
        fn test_bad_options() {
            let opts = options();
            match opts.parse(vec!["-x"]) {
                Err(e) => assert_eq!(e.to_string(), "Unrecognized option: \'x\'"),
                Ok(_) => assert!(false, "Option -x should be invalid"),
            }
            match opts.parse(vec!["--foo"]) {
                Err(e) => assert_eq!(e.to_string(), "Unrecognized option: \'foo\'"),
                Ok(_) => assert!(false, "Option -foo should be invalid"),
            }
        }

        #[test]
        fn test_getseq() {
            struct TestCase {
                desc: String,
                args: Vec<String>,
                seq: Sequence,
            }

            for item in vec![
                TestCase {
                    desc: "arg 10".to_string(),
                    args: vec!["10".to_string()],
                    seq: (1f64, 1f64, 10f64, 0usize),
                },
                TestCase {
                    desc: "args 10, 20".to_string(),
                    args: vec!["10".to_string(), "20".to_string()],
                    seq: (10f64, 1f64, 20f64, 0usize),
                },
                TestCase {
                    desc: "args 10, 2, 20".to_string(),
                    args: vec!["10".to_string(), "2".to_string(), "20".to_string()],
                    seq: (10f64, 2f64, 20f64, 0usize),
                },
                TestCase {
                    desc: "args -10, 5, 10".to_string(),
                    args: vec!["-10".to_string(), "5".to_string(), "10".to_string()],
                    seq: (-10f64, 5f64, 10f64, 0usize),
                },
                TestCase {
                    desc: "args -10, 5, 0".to_string(),
                    args: vec!["-10".to_string(), "5".to_string(), "0".to_string()],
                    seq: (-10f64, 5f64, 0f64, 0usize),
                },
                TestCase {
                    desc: "args -10, 2, -6".to_string(),
                    args: vec!["-10".to_string(), "2".to_string(), "-6".to_string()],
                    seq: (-10f64, 2f64, -6f64, 0usize),
                },
                TestCase {
                    desc: "args 10, -2, -6".to_string(),
                    args: vec!["10".to_string(), "-2".to_string(), "-6".to_string()],
                    seq: (10f64, -2f64, -6f64, 0usize),
                },
                TestCase {
                    desc: "args 10, -6".to_string(),
                    args: vec!["10".to_string(), "-6".to_string()],
                    seq: (10f64, -1f64, -6f64, 0usize),
                },
                TestCase {
                    desc: "args 10.0, 2.25".to_string(),
                    args: vec!["10.0".to_string(), "2.25".to_string()],
                    seq: (10f64, -1f64, 2.25f64, 2usize),
                },
                TestCase {
                    desc: "args 10.4".to_string(),
                    args: vec!["10.4".to_string()],
                    seq: (1f64, 1f64, 10.4f64, 1usize),
                },
                TestCase {
                    desc: "args 10.225".to_string(),
                    args: vec!["10.225".to_string()],
                    seq: (1f64, 1f64, 10.225f64, 3usize),
                },
                TestCase {
                    desc: "args 1, 0.5, 10.500".to_string(),
                    args: vec!["1".into(), ".5".into(), "10.5004".into()],
                    seq: (1f64, 0.5f64, 10.5004f64, 4usize),
                },
            ] {
                assert_eq!(
                    getseq(&item.args).unwrap(),
                    item.seq,
                    "Should get expected sequence for {}",
                    item.desc,
                );
            }
        }

        #[test]
        fn test_bad_getseq() {
            struct TestCase {
                desc: String,
                args: Vec<String>,
                err: String,
            }

            for item in vec![
                TestCase {
                    desc: "no args".into(),
                    args: vec![],
                    err: "Not enough arguments".into(),
                },
                TestCase {
                    desc: "too many args".into(),
                    args: vec!["x".to_string(); 4],
                    err: "Too many arguments".into(),
                },
                TestCase {
                    desc: "zero increment".into(),
                    args: vec!["1".into(), "0".into(), "5".into()],
                    err: "zero increment".into(),
                },
                TestCase {
                    desc: "zero decrement".into(),
                    args: vec!["1".into(), "0".into(), "-5".into()],
                    err: "zero decrement".into(),
                },
                TestCase {
                    desc: "positive increment".into(),
                    args: vec!["1".into(), "-1".into(), "5".into()],
                    err: "needs positive increment".into(),
                },
                TestCase {
                    desc: "negative decrement".into(),
                    args: vec!["1".into(), "1".into(), "-5".into()],
                    err: "needs negative decrement".into(),
                },
            ] {
                match getseq(&item.args) {
                    Err(e) => assert_eq!(e.to_string(), item.err),
                    Ok(_) => assert!(false, "Should get error for {}", item.desc),
                }
            }
        }
    }
}
