#[macro_use]
extern crate clap;
extern crate humansize;
mod args;
mod cfg;
mod dbg;
mod find;
mod logger;

use crate::cfg::Config;
use crate::cfg::Mode;
use crate::dbg::dbg_info;
use crate::find::{filter_mod_time, filter_name, filter_size, summarize};
use crate::logger::setup_logging;
use fwalker::Walker;
use humansize::{file_size_opts as options, FileSize};
use itertools::Itertools;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process;

fn main() {
    let cfg: Config = Config::from_args(args::args());
    setup_logging(cfg.verbosity_level);
    log::debug!("Config: {:?}", cfg);

    if cfg.print_dbg {
        println!("{}", dbg_info());
        process::exit(0);
    }

    match cfg.mode {
        Mode::File => walk_files(&cfg),
        Mode::Dir => walk_dirs(&cfg),
    }
}

fn walk_files(cfg: &Config) {
    let files: Vec<PathBuf> = cfg
        .paths
        .iter()
        .map(PathBuf::from)
        .inspect(check_path)
        .flat_map(|path: PathBuf| create_walker(&cfg, &path))
        .filter(|f: &PathBuf| filter_size(f, cfg.min_size))
        .filter(|f: &PathBuf| filter_name(f, &cfg.pattern))
        .filter(|f: &PathBuf| filter_mod_time(f, &cfg.max_age))
        .take(cfg.limit)
        .inspect(|f| print(f))
        .collect();

    let (found, size) = summarize(files);

    let human_size = size.file_size(options::CONVENTIONAL).unwrap();
    println!("Found {} files with a total size of {}", found, human_size);
}

fn walk_dirs(cfg: &Config) {
    let mut acc_size: HashMap<PathBuf, u64> = HashMap::new();
    let root: &PathBuf = cfg.paths.iter().sorted().collect_vec().first().unwrap();
    let root_level: usize = root.components().count();

    cfg.paths
        .iter()
        .map(PathBuf::from)
        .inspect(check_path)
        .flat_map(|path: PathBuf| create_walker(&cfg, &path))
        .filter(|f: &PathBuf| filter_mod_time(f, &cfg.max_age))
        .filter(|f: &PathBuf| filter_name(f, &cfg.pattern))
        .map(|f: PathBuf| size_of(&f))
        .for_each(|(dir, size)| update_size(&mut acc_size, dir, root, size));

    let acc_size: Vec<u64> = acc_size
        .iter()
        .filter(|(_, size)| **size >= cfg.min_size)
        .take(cfg.limit)
        .sorted_by(|(path0, _), (path1, _)| path0.cmp(path1))
        .inspect(|(path, size)| print_dir(path, **size, root_level))
        .map(|(_, size)| *size)
        .collect_vec();

    let size: u64 = *acc_size.iter().max().unwrap_or(&0);
    let found: usize = acc_size.len();
    let human_size = size.file_size(options::CONVENTIONAL).unwrap();
    println!(
        "Found {} directories with a total size of {}",
        found, human_size
    );
}

fn update_size(acc_size: &mut HashMap<PathBuf, u64>, path: PathBuf, root: &PathBuf, size: u64) {
    let cur_size: u64 = *acc_size.get(&path).unwrap_or(&0u64);
    let new_size = cur_size + size;
    acc_size.insert(path.clone(), new_size);
    if path == *root {
        return;
    }
    match path.parent() {
        Some(parent) => update_size(acc_size, PathBuf::from(parent), root, size),
        None => return,
    }
}

fn size_of(file: &PathBuf) -> (PathBuf, u64) {
    let size: u64 = file.metadata().unwrap().len();
    let parent: PathBuf = file.parent().unwrap().to_path_buf();
    (parent, size)
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
    let path: String = fmt_path(file, 0);
    let size = file
        .metadata()
        .unwrap()
        .len()
        .file_size(options::CONVENTIONAL)
        .unwrap();
    println!("{:>10} │ {}", size, path);
}

fn print_dir(dir: &PathBuf, size: u64, root_level: usize) {
    let canon: PathBuf = dir
        .canonicalize()
        .expect("Unable to get canonical path for dir");
    let level: usize = canon.components().count();
    let rel_level: usize = level - root_level;
    let size: String = size.file_size(options::CONVENTIONAL).unwrap();
    let size: String = format!("{:>10}", size);
    let path_str: String = fmt_path(dir, root_level);

    let pad_space = ((rel_level) * 2) + 2;
    match rel_level {
        0 => println!("{} {}", size, path_str),
        1 => println!("{} ├── {}", size, path_str),
        _ => println!(
            "{} │{:>width$} {}",
            size,
            "└──",
            path_str,
            width = pad_space
        ),
    }
}

fn fmt_path(path: &PathBuf, root_level: usize) -> String {
    let skip = if root_level == 0 {
        root_level
    } else {
        root_level - 1
    };

    path.canonicalize()
        .unwrap()
        .components()
        .skip(skip)
        .map(|c| c.as_os_str().to_str().unwrap())
        .join("/")
        .replacen("//", "/", 1)
        .replace("\"", "")
}
