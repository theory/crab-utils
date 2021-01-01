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
    use std::{cmp, error, result, str};
    extern crate getopts;
    use getopts::Options;

    type Result<T> = result::Result<T, Box<dyn error::Error>>;
    type Sequence = (f64, f64, f64, usize);

    pub fn run(out: &mut dyn Write, argv: Vec<String>) -> Result<()> {
        let opts = options();
        let matches = match opts.parse(argv) {
            Ok(m) => m,
            Err(f) => return Err(f.to_string().into()),
        };
        let seq = match getseq(&matches.free) {
            Ok(s) => s,
            Err(e) => return Err(usage(opts, Err(e)).into()),
        };
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

    fn usage(opts: Options, err: Result<()>) -> String {
        let usage = opts.short_usage("seq") + " [first [incr]] last";
        if let Err(e) = err {
            return format!("{}\n{}", e, usage);
        }
        usage
    }

    fn options() -> Options {
        let mut opts = Options::new();
        opts.optflag("w", "equal-width", " Equalize the widths of all numbers");
        opts.optopt("s", "separator", "Use string to separate numbers", "string");
        opts.optopt(
            "t",
            "terminator",
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
        } else {
            while cur >= seq.2 {
                write!(out, "{:0>1$.2$}{3}", cur, width, seq.3, sep)?;
                iter += 1;
                cur = seq.0 + seq.1 * iter as f64;
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
            struct TestCase<'a> {
                desc: String,
                args: Vec<&'a str>,
                unset: Vec<&'a str>,
                free: Vec<&'a str>,
                set: Vec<&'a str>,
                vals: Vec<Vec<&'a str>>,
            };

            let defined_opts = vec!["w", "s", "t"];
            let opts = options();
            for item in vec![
                TestCase {
                    desc: "no args".into(),
                    args: Vec::new(),
                    unset: defined_opts.clone(),
                    free: Vec::new(),
                    set: Vec::new(),
                    vals: Vec::new(),
                },
                TestCase {
                    desc: "one free".into(),
                    args: vec!["10"],
                    unset: defined_opts.clone(),
                    free: vec!["10"],
                    set: Vec::new(),
                    vals: Vec::new(),
                },
                TestCase {
                    desc: "w and one free".into(),
                    args: vec!["-w", "8"],
                    unset: vec!["s", "t"],
                    set: vec!["w"],
                    free: vec!["8"],
                    vals: Vec::new(),
                },
                TestCase {
                    desc: "w and two free".into(),
                    args: vec!["-w", "8", "10"],
                    unset: vec!["s", "t"],
                    set: vec!["w"],
                    free: vec!["8", "10"],
                    vals: Vec::new(),
                },
                TestCase {
                    desc: "negated arg".into(),
                    args: vec!["-w", "--", "-8", "10"],
                    unset: vec!["s", "t"],
                    set: vec!["w"],
                    free: vec!["-8", "10"],
                    vals: Vec::new(),
                },
                TestCase {
                    desc: "all options".into(),
                    args: vec!["-w", "-t", "foo", "-s", "^", "10"],
                    unset: Vec::new(),
                    set: vec!["w", "t", "s"],
                    free: vec!["10"],
                    vals: vec![vec!["t", "foo"], vec!["s", "^"]],
                },
                TestCase {
                    desc: "long options".into(),
                    args: vec![
                        "--equal-width",
                        "--terminator",
                        "🤖",
                        "--separator",
                        ":",
                        "--",
                        "10",
                        "-5",
                    ],
                    unset: Vec::new(),
                    set: vec!["equal-width", "terminator", "separator"],
                    free: vec!["10", "-5"],
                    vals: vec![vec!["t", "🤖"], vec!["s", ":"]],
                },
            ] {
                let matches = opts.parse(item.args)?;
                // Make sure all options are defined.
                for opt in &defined_opts {
                    assert!(matches.opt_defined(opt), "Option -{} not defined", opt);
                }

                // Check for unset args.
                for opt in &item.unset {
                    assert!(
                        !matches.opt_present(opt),
                        "Option -{} should not be present with {}",
                        opt,
                        item.desc,
                    );
                }

                // Check for presence of set args.
                for opt in &item.set {
                    assert!(
                        matches.opt_present(opt),
                        "Option -{} should not be present with {}",
                        opt,
                        item.desc,
                    );
                }

                // Check for option values.
                for kv in &item.vals {
                    assert_eq!(
                        matches.opt_str(kv[0]),
                        Some(kv[1].into()),
                        "Missing -{} string with {}",
                        kv[0],
                        item.desc,
                    );
                }

                // Check for free args.
                assert_eq!(
                    matches.free, item.free,
                    "Invalid free strings with {}",
                    item.desc
                );
            }

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
                    desc: "arg 10".into(),
                    args: vec!["10".into()],
                    seq: (1f64, 1f64, 10f64, 0usize),
                },
                TestCase {
                    desc: "args 10, 20".into(),
                    args: vec!["10".into(), "20".into()],
                    seq: (10f64, 1f64, 20f64, 0usize),
                },
                TestCase {
                    desc: "args 10, 2, 20".into(),
                    args: vec!["10".into(), "2".into(), "20".into()],
                    seq: (10f64, 2f64, 20f64, 0usize),
                },
                TestCase {
                    desc: "args -10, 5, 10".into(),
                    args: vec!["-10".into(), "5".into(), "10".into()],
                    seq: (-10f64, 5f64, 10f64, 0usize),
                },
                TestCase {
                    desc: "args -10, 5, 0".into(),
                    args: vec!["-10".into(), "5".into(), "0".into()],
                    seq: (-10f64, 5f64, 0f64, 0usize),
                },
                TestCase {
                    desc: "args -10, 2, -6".into(),
                    args: vec!["-10".into(), "2".into(), "-6".into()],
                    seq: (-10f64, 2f64, -6f64, 0usize),
                },
                TestCase {
                    desc: "args 10, -2, -6".into(),
                    args: vec!["10".into(), "-2".into(), "-6".into()],
                    seq: (10f64, -2f64, -6f64, 0usize),
                },
                TestCase {
                    desc: "args 10, -6".into(),
                    args: vec!["10".into(), "-6".into()],
                    seq: (10f64, -1f64, -6f64, 0usize),
                },
                TestCase {
                    desc: "args 10.0, 2.25".into(),
                    args: vec!["10.0".into(), "2.25".into()],
                    seq: (10f64, -1f64, 2.25f64, 2usize),
                },
                TestCase {
                    desc: "args 10.4".into(),
                    args: vec!["10.4".into()],
                    seq: (1f64, 1f64, 10.4f64, 1usize),
                },
                TestCase {
                    desc: "args 10.225".into(),
                    args: vec!["10.225".into()],
                    seq: (1f64, 1f64, 10.225f64, 3usize),
                },
                TestCase {
                    desc: "args 1, 0.5, 10.500".into(),
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
                    args: vec!["x".into(); 4],
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
                    Err(e) => assert_eq!(
                        e.to_string(),
                        item.err,
                        "Should get error for {}",
                        item.desc
                    ),
                    Ok(_) => assert!(false, "Should get error for {}", item.desc),
                }
            }
        }

        #[test]
        fn test_emitseq() -> Result<()> {
            struct TestCase<'a> {
                desc: &'a str,
                seq: Sequence,
                sep: &'a str,
                width: usize,
                term: Option<String>,
                exp: String,
            };

            for item in vec![
                TestCase {
                    desc: "1-3",
                    seq: (1.0, 1.0, 3.0, 0),
                    sep: "\n",
                    width: 1,
                    term: None,
                    exp: "1\n2\n3\n".into(),
                },
                TestCase {
                    desc: "neg 1-3",
                    seq: (-1.0, -1.0, -3.0, 0),
                    sep: "\n",
                    width: 1,
                    term: None,
                    exp: "-1\n-2\n-3\n".into(),
                },
                TestCase {
                    desc: "neg 3-1",
                    seq: (-3.0, 1.0, -1.0, 0),
                    sep: "\n",
                    width: 1,
                    term: None,
                    exp: "-3\n-2\n-1\n".into(),
                },
                TestCase {
                    desc: "1, 3, 5",
                    seq: (1.0, 2.0, 5.0, 0),
                    sep: "\n",
                    width: 1,
                    term: None,
                    exp: "1\n3\n5\n".into(),
                },
                TestCase {
                    desc: "neg 1, 3, 5",
                    seq: (-1.0, -2.0, -5.0, 0),
                    sep: "\n",
                    width: 1,
                    term: None,
                    exp: "-1\n-3\n-5\n".into(),
                },
                TestCase {
                    desc: "1.0-3.0",
                    seq: (1.0, 1.0, 3.0, 1),
                    sep: "\n",
                    width: 1,
                    term: None,
                    exp: "1.0\n2.0\n3.0\n".into(),
                },
                TestCase {
                    desc: "1-3 x 0.5",
                    seq: (1.0, 0.5, 3.0, 1),
                    sep: "\n",
                    width: 1,
                    term: None,
                    exp: "1.0\n1.5\n2.0\n2.5\n3.0\n".into(),
                },
                TestCase {
                    desc: "1-2.1 x 0.3",
                    seq: (1.0, 0.3, 2.1, 1),
                    sep: ",",
                    width: 1,
                    term: None,
                    exp: "1.0,1.3,1.6,1.9,".into(),
                },
                TestCase {
                    desc: "neg 1-2.1 x 0.3",
                    seq: (-1.0, -0.3, -2.1, 1),
                    sep: ",",
                    width: 1,
                    term: None,
                    exp: "-1.0,-1.3,-1.6,-1.9,".into(),
                },
                TestCase {
                    desc: "1-3 precision 3",
                    seq: (1.0, 1.0, 3.0, 3),
                    sep: ",",
                    width: 1,
                    term: None,
                    exp: "1.000,2.000,3.000,".into(),
                },
                TestCase {
                    desc: "1-3 width 6 precision 3",
                    seq: (1.0, 1.0, 3.0, 3),
                    sep: ",",
                    width: 6,
                    term: None,
                    exp: "01.000,02.000,03.000,".into(),
                },
                TestCase {
                    desc: "8-10 width 6 precision 3",
                    seq: (8.0, 1.0, 10.0, 3),
                    sep: ",",
                    width: 6,
                    term: None,
                    exp: "08.000,09.000,10.000,".into(),
                },
                TestCase {
                    desc: "8-10 x 0.25 width 5 precision",
                    seq: (8.0, 0.25, 10.0, 2),
                    sep: ",",
                    width: 5,
                    term: None,
                    exp: "08.00,08.25,08.50,08.75,09.00,09.25,09.50,09.75,10.00,".into(),
                },
                TestCase {
                    desc: "simple 1-100",
                    seq: (1.0, 1.0, 100.0, 0),
                    sep: "\n",
                    width: 1,
                    term: None,
                    exp: (1..=100)
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join("\n")
                        + "\n",
                },
                TestCase {
                    desc: "1-100 with alt sep",
                    seq: (1.0, 1.0, 100.0, 0),
                    sep: ":",
                    width: 1,
                    term: None,
                    exp: (1..=100)
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(":")
                        + ":",
                },
                TestCase {
                    desc: "1-100 with width",
                    seq: (1.0, 1.0, 100.0, 0),
                    sep: "\n",
                    width: 4,
                    term: None,
                    exp: (1..=100)
                        .map(|x| format!("{:0>4}", x))
                        .collect::<Vec<String>>()
                        .join("\n")
                        + "\n",
                },
                TestCase {
                    desc: "1-100 with term",
                    seq: (1.0, 1.0, 100.0, 0),
                    sep: "\n",
                    width: 1,
                    term: Some("foo".into()),
                    exp: (1..=100)
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join("\n")
                        + "\n"
                        + "foo",
                },
                TestCase {
                    desc: "-5 0.25 2",
                    seq: (-5.0, 0.25, 2.0, 2),
                    sep: ",",
                    width: 1,
                    term: None,
                    exp: "-5.00,-4.75,-4.50,-4.25,-4.00,-3.75,-3.50,-3.25,-3.00,-2.75,-2.50,-2.25,-2.00,-1.75,-1.50,-1.25,-1.00,-0.75,-0.50,-0.25,0.00,0.25,0.50,0.75,1.00,1.25,1.50,1.75,2.00,".into(),
                },
            ] {
                let mut buf: Vec<u8> = vec![];
                emitseq(&mut buf, item.seq, item.sep, item.width, item.term)?;
                assert_eq!(
                    item.exp,
                    String::from_utf8(buf).unwrap(),
                    "Invalid output for {}",
                    item.desc,
                );
            }

            Ok(())
        }
    }
}
