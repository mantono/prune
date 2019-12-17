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

    let (found, sizes) = cfg.paths.iter()
        .map(|p| {
            let current_dir = PathBuf::from(p);
            let result: (u64, u64) = explore(current_dir, cfg.max_depth, limit, cfg.min_size, &mut PrintFile);
            limit -= result.0;
            result
        })
        .unzip::<u64, u64, Vec<u64>, Vec<u64>>();

    let sum_found: u64 = found.iter().sum();
    let sum_size: u64 = sizes.iter().sum();
    let human_size = sum_size.file_size(options::CONVENTIONAL).unwrap();
    println!("Found {} files with a total size of {}", sum_found, human_size);
}

struct PrintFile;

impl SideEffect for PrintFile {
    fn submit(&mut self, file: &PathBuf) {
        let canon: PathBuf = file.canonicalize().expect("Unable to get canonical path");
        let size = file.metadata().unwrap().len().file_size(options::CONVENTIONAL).unwrap();
        println!("{}, {:?}", size, canon)
    }
}