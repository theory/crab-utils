use std::env;

pub fn run(args: Vec<String>) -> i32 {
    let (first, incr, last) = match args.len() {
        1 => (1, 1, s2i(&args[0])),
        2 => (s2i(&args[0]), 1, s2i(&args[1])),
        3 => (s2i(&args[0]), s2u(&args[1]), s2i(&args[2])),
        _ => {
            return usage();
        }
    };

    for num in (first..=last).step_by(incr) {
        println!("{}", num);
    }
    0
}

fn s2i(arg: &str) -> isize {
    arg.trim().parse().unwrap_or_else(|_| {
        usage();
        1
    })
}

fn s2u(arg: &str) -> usize {
    arg.trim().parse().unwrap_or_else(|_| {
        usage();
        1
    })
}

fn usage() -> i32 {
    println!(
        "Usage: {:?} [-w] [-f format] [-s string] [-t string] [first [incr]] last",
        env::current_exe().unwrap()
    );
    1
}
