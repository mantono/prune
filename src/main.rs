#[macro_use]
extern crate clap;
extern crate humansize;
mod args;
mod cfg;
mod dbg;
mod find;
mod logger;

use crate::cfg::Config;
use crate::find::{filter_name, filter_size, summarize};
use crate::logger::setup_logging;
use fwalker::Walker;
use humansize::{file_size_opts as options, FileSize};
use std::path::PathBuf;
use std::process;
use crate::dbg::dbg_info;

fn main() {
    let cfg: Config = Config::from_args(args::args());
    setup_logging(cfg.verbosity_level);

    if cfg.print_dbg {
        println!("{}", dbg_info());
        process::exit(0);
    }

    let files: Vec<PathBuf> = cfg
        .paths
        .iter()
        .map(PathBuf::from)
        .inspect(check_path)
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

    let walker: Walker = if cfg.only_local_fs {
        walker.only_local_fs()
    } else {
        walker
    };
    log::debug!("walker: {:?}", walker);
    walker
}

fn check_path(path: &PathBuf) {
    if !path.exists() {
        log::error!("Path does not exist: {:?}", path);
        process::exit(1);
    }
    if !path.is_dir() {
        log::error!("Path is not a directory: {:?}", path);
        process::exit(2);
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
