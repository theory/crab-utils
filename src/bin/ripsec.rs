use seq;
use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    process::exit(seq::run(args));
}
