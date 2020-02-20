#[macro_use]
extern crate clap;
extern crate humansize;
mod cfg;
mod find;
mod args;
mod expl;
mod logger;
mod prefix_tree;

use humansize::{FileSize, file_size_opts as options};
use std::path::PathBuf;
use crate::find::{summarize, filter_size, filter_name};
use crate::cfg::Config;
use crate::expl::FileExplorer;
use crate::logger::setup_logging;

fn main() {
    let cfg: Config = Config::from_args(args::args());
    setup_logging(cfg.verbosity_level);

    let fs_filters: Option<Vec<PathBuf>> = resolve_filesystems(cfg.only_local_fs, &cfg.paths);
    let files: Vec<PathBuf> = cfg.paths.iter()
        .map(|p| PathBuf::from(p))
        .flat_map(|path: PathBuf| FileExplorer::for_path(&path, cfg.max_depth, fs_filters.clone()))
        .filter(|f: &PathBuf| filter_size(f, cfg.min_size))
        .filter(|f: &PathBuf| filter_name(f, &cfg.pattern))
        .take(cfg.limit)
        .inspect(|f| print(f))
        .collect();

    let (found, size) = summarize(files);

    let human_size = size.file_size(options::CONVENTIONAL).unwrap();
    println!("Found {} files with a total size of {}", found, human_size);
}

const LINUX_MOUNTS_FILE: &str = "/proc/mounts";

/// On Linux, read mounted file systems for /proc/mounts and cross reference
/// them with paths to search with, and filter out any overlaps.
///
/// Mac OS X should be similar. Have no idea how to solve Windows, yet.
fn resolve_filesystems(only_local_fs: bool, paths: &Vec<String>) -> Option<Vec<PathBuf>> {
    let mounts: String = match std::fs::read_to_string(LINUX_MOUNTS_FILE) {
        Ok(content) => content,
        Err(_) => {
            log::warn!("Could not find {}", LINUX_MOUNTS_FILE);
            return None
        }
    };

    let mut mounts: Vec<&str> = mounts
        .lines()
        .map(|line: &str| line.split_ascii_whitespace().skip(1).next().unwrap())
        .collect();

    mounts.sort();

    for m in mounts {
        log::debug!("{}", m);
    }

    match only_local_fs {
        true => Some(vec![]),
        false => None
    }
}

fn print(file: &PathBuf) {
    let canon: PathBuf = file.canonicalize().expect("Unable to get canonical path");
    let size = file.metadata().unwrap().len().file_size(options::CONVENTIONAL).unwrap();
    println!("{}, {:?}", size, canon);
}