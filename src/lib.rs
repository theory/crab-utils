use std::error;
type Result<T> = ::std::result::Result<T, Box<dyn error::Error>>;

pub fn run(args: Vec<String>) -> Result<()> {
    let (first, incr, last) = match args.len() {
        1 => (1, 1, s2i(&args[0])?),
        2 => (s2i(&args[0])?, 1, s2i(&args[1])?),
        3 => (s2i(&args[0])?, s2u(&args[1])?, s2i(&args[2])?),
        _ => return Err(usage().into()),
    };

    for num in (first..=last).step_by(incr) {
        println!("{}", num);
    }
    Ok(())
}

fn s2i(arg: &str) -> Result<isize> {
    arg.trim().parse().or(Err(usage().into()))
}

fn s2u(arg: &str) -> Result<usize> {
    arg.trim().parse().or(Err(usage().into()))
}

fn usage() -> &'static str {
    "Usage: seq [-w] [-f format] [-s string] [-t string] [first [incr]] last"
}
