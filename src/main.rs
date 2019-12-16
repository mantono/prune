#[macro_use]
extern crate clap;
extern crate humansize;
mod cfg;
mod find;
mod args;

use humansize::{FileSize, file_size_opts as options};
use std::path::PathBuf;
use crate::find::{explore, ConsumeFile};
use crate::cfg::Config;

fn main() {
    let args = args::args();
    let cfg: Config = Config::from_args(args);
    let mut limit: u64 = cfg.limit as u64;
    let mut printer = FilePrinter;

    cfg.paths.iter().for_each(|p| {
        let current_dir = PathBuf::from(p);
        limit -= explore(current_dir, cfg.max_depth, limit, cfg.min_size, &mut printer);
    });
}

struct FilePrinter;

impl ConsumeFile for FilePrinter {
    fn submit(&mut self, file: &PathBuf) {
        let canon: PathBuf = file.canonicalize().expect("Unable to get canonical path");
        let size = file.metadata().unwrap().len().file_size(options::CONVENTIONAL).unwrap();
        println!("{}, {:?}", size, canon)
    }
}