use ripseq;
use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    process::exit(ripseq::run(args));
}
