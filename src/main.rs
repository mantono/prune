extern crate humansize;
extern crate lazy_static;
extern crate structopt;

mod cfg;
mod dbg;
mod duration;
mod find;
mod fs;
mod logger;
mod parse;
mod print;
mod size;

use crate::cfg::Config;
use crate::dbg::dbg_info;
use crate::find::summarize;
use crate::logger::setup_logging;
use crate::print::{print_dir, print_file, print_summary};
use crate::structopt::StructOpt;
use cfg::Mode;
use find::Filter;
use itertools::Itertools;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process;
use walkdir::{DirEntry, WalkDir};

fn main() {
    let cfg = Config::from_args();
    setup_logging(cfg.verbosity_level);
    log::debug!("Config: {:?}", cfg);

    if cfg.print_dbg {
        println!("{}", dbg_info());
        process::exit(0);
    }

    let (found, size) = match cfg.mode() {
        Mode::File => walk_files(&cfg),
        Mode::Dir => walk_dirs(&cfg),
    };

    print_summary(cfg.mode(), found, size, &cfg);
}

fn walk_files(cfg: &Config) -> (u64, u64) {
    let limit: usize = cfg.limit.unwrap_or(usize::MAX);
    let filter: Filter = cfg.into();
    let files: Vec<DirEntry> = cfg
        .paths()
        .iter()
        .map(|path: &PathBuf| create_walker(cfg, path))
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e: &DirEntry| filter.accept(e))
        .take(limit)
        .inspect(|f| print_file(f, cfg))
        .collect();

    summarize(files)
}

fn walk_dirs(cfg: &Config) -> (u64, u64) {
    let mut acc_size: HashMap<PathBuf, u64> = HashMap::new();
    let paths: Vec<PathBuf> = cfg.paths();
    let root: &Path = paths.first().unwrap();
    let filter: Filter = cfg.into();

    cfg.paths()
        .iter()
        .map(|path: &PathBuf| create_walker(cfg, path))
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e: &DirEntry| filter.accept(e))
        .map(|f: DirEntry| size_of(&f))
        .for_each(|(dir, size)| update_size(&mut acc_size, dir, root, size));

    let limit: usize = cfg.limit.unwrap_or(usize::MAX);
    let acc_size: Vec<u64> = acc_size
        .iter()
        .filter(|(_, size)| **size >= cfg.min_size_bytes())
        .take(limit)
        .sorted_by(|(path0, _), (path1, _)| path0.cmp(path1))
        .inspect(|(path, size)| print_dir(path, **size, cfg))
        .map(|(_, size)| *size)
        .collect_vec();

    let size: u64 = *acc_size.iter().max().unwrap_or(&0);
    let found: u64 = acc_size.len() as u64;

    (found, size)
}

fn update_size(acc_size: &mut HashMap<PathBuf, u64>, path: PathBuf, root: &Path, size: u64) {
    let cur_size: u64 = *acc_size.get(&path).unwrap_or(&0u64);
    let new_size = cur_size + size;
    acc_size.insert(path.clone(), new_size);
    if path == *root {
        return;
    }
    if let Some(parent) = path.parent() {
        update_size(acc_size, PathBuf::from(parent), root, size)
    }
}

fn size_of(entry: &DirEntry) -> (PathBuf, u64) {
    let size: u64 = match entry.metadata() {
        Ok(metadata) => metadata.len(),
        Err(_) => 0,
    };
    let parent: PathBuf = entry.path().parent().unwrap().to_path_buf();
    (parent, size)
}

fn create_walker(cfg: &Config, path: &Path) -> WalkDir {
    let walker = WalkDir::new(path)
        .follow_links(false)
        .max_depth(cfg.max_depth())
        .same_file_system(cfg.only_local_fs);

    log::debug!("walkdir: {:?}", walker);
    walker
}
