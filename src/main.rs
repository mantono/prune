#[macro_use]
extern crate clap;
extern crate humansize;
mod args;
mod cfg;
mod find;
mod logger;

use crate::cfg::Config;
use crate::find::{filter_name, filter_size, summarize};
use crate::logger::setup_logging;
use fwalker::Walker;
use humansize::{file_size_opts as options, FileSize};
use std::path::PathBuf;

fn main() {
    let cfg: Config = Config::from_args(args::args());
    setup_logging(cfg.verbosity_level);

    let files: Vec<PathBuf> = cfg
        .paths
        .iter()
        .map(PathBuf::from)
        .flat_map(|path: PathBuf| create_walker(&cfg, &path))
        .filter(|f: &PathBuf| filter_size(f, cfg.min_size))
        .filter(|f: &PathBuf| filter_name(f, &cfg.pattern))
        .take(cfg.limit)
        .inspect(|f| print(f))
        .collect();

    let (found, size) = summarize(files);

    let human_size = size.file_size(options::CONVENTIONAL).unwrap();
    println!("Found {} files with a total size of {}", found, human_size);
}

fn create_walker(cfg: &Config, path: &PathBuf) -> Walker {
    let walker = Walker::from(path)
        .expect("Unable to crate Walker from Path")
        .max_depth(cfg.max_depth);

    if cfg.only_local_fs {
        walker
            .only_local_fs()
            .expect("Unable to enable only local fs")
    } else {
        walker
    }
}

fn print(file: &PathBuf) {
    let canon: PathBuf = file.canonicalize().expect("Unable to get canonical path");
    let size = file
        .metadata()
        .unwrap()
        .len()
        .file_size(options::CONVENTIONAL)
        .unwrap();
    println!("{}, {:?}", size, canon);
}
