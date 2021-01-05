use std::{env, io::stdout, process};

#[path = "seq/seq.rs"]
mod seq;

fn main() {
    if let Err(err) = seq::run(&mut stdout(), &env::args().skip(1).collect::<Vec<_>>()) {
        eprintln!("{}", err);
        process::exit(2);
    }
}
