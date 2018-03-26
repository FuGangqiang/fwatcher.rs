extern crate glob;
extern crate getopts;
extern crate fwatcher;

use fwatcher::Fwatcher;
use getopts::{HasArg, Occur, Options, ParsingStyle};
use glob::Pattern;
use std::{env, process};
use std::path::PathBuf;
use std::time::Duration;

fn main() {
    let mut opts = Options::new();
    opts.parsing_style(ParsingStyle::StopAtFirstFree);
    opts.optflag("h", "help", "Display this message");
    opts.optflag("v", "version", "Print version info");
    opts.optflag("r", "restart", "Auto restart command, default to false");
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
    opts.opt("P",
             "exclude_pattern",
             "Watch file glob pattern exclusively, default null",
             "<exclude_pattern>",
             HasArg::Yes,
             Occur::Multi);
    opts.opt("",
             "delay",
             "Delay in seconds for watcher, default to 2",
             "<second>",
             HasArg::Yes,
             Occur::Optional);
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
            eprintln!("{}", why);
            eprintln!("run `fwatcher -h` to get the usage.");
            process::exit(1);
        },
    };

    if matches.opt_present("h") {
        print_usage_and_exit(opts, 0);
    } else if matches.opt_present("v") {
        println!("fwatcher {}", env!("CARGO_PKG_VERSION"));
        process::exit(0);
    } else if matches.free.len() == 0 {
        print_usage_and_exit(opts, 1);
    }

    let dirs: Vec<_> = matches.opt_strs("directory")
                              .iter()
                              .map(|dir| PathBuf::from(dir))
                              .collect();
    let mut patterns: Vec<_> =
        matches.opt_strs("pattern")
               .iter()
               .map(|dir| Pattern::new(dir).expect("create pattern error"))
               .collect();
    if patterns.is_empty() {
        patterns.push(Pattern::new("*").unwrap());
    }
    let exclude_patterns: Vec<_> =
        matches.opt_strs("exclude_pattern")
               .iter()
               .map(|dir| Pattern::new(dir).expect("create pattern error"))
               .collect();
    let delay = matches.opt_str("delay")
                       .map(|i| i.parse().expect("parse delay option error"))
                       .or(Some(2))
                       .map(|i| Duration::new(i, 0))
                       .unwrap();
    let interval = matches.opt_str("interval")
                          .map(|i| i.parse().expect("parse interval option error"))
                          .or(Some(1))
                          .map(|i| Duration::new(i, 0))
                          .unwrap();
    let restart = matches.opt_present("restart");
    let mut fwatcher = Fwatcher::new(dirs, matches.free);
    fwatcher.patterns(&patterns)
            .exclude_patterns(&exclude_patterns)
            .delay(delay)
            .interval(interval)
            .restart(restart)
            .run();
}

fn print_usage_and_exit(opts: Options, exit_code: i32) -> ! {
    let brief = "\
Usage:
    fwatcher [options] CMD";
    if exit_code == 0 {
        print!("{}", opts.usage(brief));
    } else {
        eprint!("{}", opts.usage(brief));
    }
    process::exit(exit_code);
}
