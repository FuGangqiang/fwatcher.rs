//! Auto run command when some files changed.
//!
//! # Usage
//!
//! Dependencies in your project's `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! glob = "0.2"
//! notify = "4.0"
//! fwatcher = "*"
//! ```
//!
//! # Example
//!
//! ```rust
//! extern crate glob;
//! extern crate fwatcher;
//!
//! use fwatcher::Fwatcher;
//! use glob::Pattern;
//! use std::path::PathBuf;
//! use std::time::Duration;
//!
//! fn main() {
//!     let dirs =vec![PathBuf::from("src")];
//!     let patterns = vec![Pattern::new("**/*.py").unwrap()];
//!     let interval =  Duration::new(1, 0);
//!     let restart = false;
//!     let cmd = vec!["pytest".to_string()];
//!
//!     let mut fwatcher = Fwatcher::new(dirs, patterns, interval, restart, cmd);
//!     fwatcher.run();
//! }
//! ```

#![doc(html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
       html_favicon_url = "https://www.rust-lang.org/favicon.ico",
       html_root_url = "https://fugangqiang.github.io/doc/fwatcher")]

extern crate getopts;
extern crate glob;
extern crate notify;

use std::path::PathBuf;
use std::process::{Command, Child};
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use glob::Pattern;
use notify::{Watcher, RecursiveMode, DebouncedEvent, watcher};

/// a struct save `Fwatcher` state
pub struct Fwatcher {
    dirs: Vec<PathBuf>,
    patterns: Vec<Pattern>,
    interval: Duration,
    restart: bool,
    cmd: Vec<String>,
    last_run: Option<Instant>,
    child: Option<Child>,
}


impl Fwatcher {
    /// Constructs a new `Fwatcher`
    pub fn new(dirs: Vec<PathBuf>,
               patterns: Vec<Pattern>,
               interval: Duration,
               restart: bool,
               cmd: Vec<String>)
               -> Self {
        Fwatcher {
            dirs: dirs,
            patterns: patterns,
            interval: interval,
            restart: restart,
            cmd: cmd,
            last_run: None,
            child: None,
        }
    }

    /// run `Fwatcher`
    pub fn run(&mut self) {
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_millis(500))
            .expect("can not create a watcher");

        for d in self.dirs.iter() {
            watcher.watch(d, RecursiveMode::Recursive).expect("can not watch dir");
        }
        self.restart_child();

        loop {
            match rx.recv() {
                Ok(event) => {
                    if self.interval_consumed() {
                        self.process_event(event)
                    }
                }
                Err(why) => println!("watch error: {:?}", why),
            }
        }
    }

    fn interval_consumed(&mut self) -> bool {
        let now = Instant::now();

        if let Some(last_run) = self.last_run {
            if now.duration_since(last_run) < self.interval {
                return false;
            }
        }

        return true;
    }

    fn process_event(&mut self, event: DebouncedEvent) {
        match event {
            DebouncedEvent::Write(ref fpath) |
            DebouncedEvent::Create(ref fpath) => {
                println!("Modified: {:?}", fpath);
                if self.patterns.iter().any(|ref pat| pat.matches_path(fpath)) {
                    self.restart_child();
                }
            }
            _ => {}
        }
    }

    fn restart_child(&mut self) {
        if let Some(ref mut child) = self.child {
            if self.restart {
                let _ = child.kill();
            }
        }
        self.child = Command::new(&self.cmd[0])
            .args(&self.cmd[1..])
            .spawn()
            .ok();
        self.last_run = Some(Instant::now());
    }
}
