use seq;
use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if let Err(err) = seq::run(args) {
        eprintln!("{}", err);
        process::exit(2);
    }
}
