#[macro_use]
extern crate clap;
extern crate humansize;
mod args;
mod cfg;
mod expl;
mod find;
mod logger;
mod trie;

use crate::cfg::Config;
use crate::expl::FileExplorer;
use crate::find::{filter_name, filter_size, summarize};
use crate::logger::setup_logging;
use humansize::{file_size_opts as options, FileSize};
use std::path::PathBuf;

fn main() {
    let cfg: Config = Config::from_args(args::args());
    setup_logging(cfg.verbosity_level);

    let fs_filters: Option<Vec<PathBuf>> = resolve_filesystems(cfg.only_local_fs, &cfg.paths);
    let files: Vec<PathBuf> = cfg
        .paths
        .iter()
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
            return None;
        }
    };

    let mut mounts: Vec<&str> = mounts
        .lines()
        .map(|line: &str| line.split_ascii_whitespace().skip(1).next().unwrap())
        .collect();

    mounts.sort_by_key(|m| m.matches("/").collect::<Vec<&str>>().len());

    for m in &mounts {
        log::debug!("{}", m);
    }

    for p in paths {
        dbg!(resolve_fs_for_path(&mounts, p));
    }

    match only_local_fs {
        true => Some(vec![]),
        false => None,
    }
}

fn resolve_fs_for_path(filesystems: &Vec<&str>, path: &String) -> String {
    filesystems.iter().fold(String::from("/"), {
        |current, next| {
            if next.len() > current.len() && path.contains(next) {
                String::from(*next)
            } else {
                String::from(current)
            }
        }
    })
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
