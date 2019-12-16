#[macro_use]
extern crate clap;
extern crate humansize;
mod cfg;
mod find;
mod args;

use humansize::{FileSize, file_size_opts as options};
use std::path::PathBuf;
use crate::find::{explore, SideEffect};
use crate::cfg::Config;

fn main() {
    let cfg: Config = Config::from_args(args::args());
    let mut limit: u64 = cfg.limit as u64;

    cfg.paths.iter().for_each(|p| {
        let current_dir = PathBuf::from(p);
        limit -= explore(current_dir, cfg.max_depth, limit, cfg.min_size, &mut PrintFile);
    });
}

struct PrintFile;

impl SideEffect for PrintFile {
    fn submit(&mut self, file: &PathBuf) {
        let canon: PathBuf = file.canonicalize().expect("Unable to get canonical path");
        let size = file.metadata().unwrap().len().file_size(options::CONVENTIONAL).unwrap();
        println!("{}, {:?}", size, canon)
    }
}