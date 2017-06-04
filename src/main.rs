extern crate glob;
extern crate getopts;
extern crate fwatcher;

use fwatcher::Fwatcher;
use getopts::{HasArg, Occur, Options};
use glob::Pattern;
use std::{env, process};
use std::path::PathBuf;
use std::time::Duration;

fn main() {
    let mut opts = Options::new();
    opts.optflag("h", "help", "Display this message");
    opts.optflag("v", "version", "Print version info");
    opts.optflag("r", "restart", "Auto restart command");
    opts.opt("d",
             "directory",
             "Watch directory, default to current directory",
             "<dir>",
             HasArg::Yes,
             Occur::Multi);
    opts.opt("p",
             "pattern",
             r#"Watch file glob pattern, default to "*""#,
             "<pattern>",
             HasArg::Yes,
             Occur::Multi);
    opts.opt("i",
             "interval",
             "Interval in seconds to scan filesystem, default to 1",
             "<second>",
             HasArg::Yes,
             Occur::Optional);

    let args: Vec<_> = env::args().collect();
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(why) => {
            println!("{}", why);
            println!("run `fwatcher -h` to get the usage.");
            process::exit(1);
        },
    };

    if matches.opt_present("h") {
        print_usage(opts);
        process::exit(0);
    } else if matches.opt_present("v") {
        println!("fwatcher {}", env!("CARGO_PKG_VERSION"));
        process::exit(0);
    } else if matches.free.len() != 1 {
        print_usage(opts);
        process::exit(1);
    }

    let mut dirs: Vec<_> = matches.opt_strs("directory")
                                  .iter()
                                  .map(|dir| PathBuf::from(dir))
                                  .collect();
    if dirs.len() == 0 {
        dirs.push(env::current_dir().unwrap());
    }
    let mut patterns: Vec<_> =
        matches.opt_strs("pattern")
               .iter()
               .map(|dir| Pattern::new(dir).expect("create pattern error"))
               .collect();
    if patterns.len() == 0 {
        patterns.push(Pattern::new("*").unwrap());
    }
    let interval = matches.opt_str("interval")
                          .map(|i| i.parse().expect("parse interval option error"))
                          .or(Some(1))
                          .map(|i| Duration::new(i, 0))
                          .unwrap();
    let restart = matches.opt_present("restart");
    let cmd: Vec<String> = matches.free[0]
        .split(' ')
        .map(|s| s.to_string())
        .collect();

    let mut fwatcher = Fwatcher::new(dirs, patterns, interval, restart, cmd);
    fwatcher.run();
}

fn print_usage(opts: Options) {
    let brief = "\
Usage:
    fwatcher [options] CMD";
    print!("{}", opts.usage(brief));
}
