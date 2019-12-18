#[macro_use]
extern crate clap;
extern crate humansize;
mod cfg;
mod find;
mod args;

use humansize::{FileSize, file_size_opts as options};
use std::path::PathBuf;
use crate::find::{FileExplorer, summarize, filter_size};
use crate::cfg::Config;

type FileIterator = dyn Iterator<Item=PathBuf>;

fn main() {
    let cfg: Config = Config::from_args(args::args());

    let files: Vec<&PathBuf> = cfg.paths.iter()
        .map(|p| PathBuf::from(p))
        .flat_map(|path: PathBuf| FileExplorer::for_path(&path, cfg.max_depth))
        .filter(|f: &PathBuf| filter_size(f, cfg.min_size))
        .take(cfg.limit)
        .inspect(|f| print(f))
        .collect();

    let (found, size) = summarize(files);

    let human_size = size.file_size(options::CONVENTIONAL).unwrap();
    println!("Found {} files with a total size of {}", found, human_size);
}

fn print(file: &PathBuf) {
    let canon: PathBuf = file.canonicalize().expect("Unable to get canonical path");
    let size = file.metadata().unwrap().len().file_size(options::CONVENTIONAL).unwrap();
    println!("{}, {:?}", size, canon)
}