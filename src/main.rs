#[macro_use]
extern crate clap;
extern crate humansize;
mod args;
mod cfg;
mod dbg;
mod find;
mod logger;
mod print;
mod tree;

use crate::cfg::Config;
use crate::cfg::Mode;
use crate::dbg::dbg_info;
use crate::find::{filter_mod_time, filter_name, filter_size, summarize};
use crate::logger::setup_logging;
use crate::print::{print_dir, print_file, print_summary};
use fwalker::Walker;
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
        .flat_map(|path: &PathBuf| create_walker(&cfg, path))
        .filter(|f: &PathBuf| filter_size(f, cfg.min_size))
        .filter(|f: &PathBuf| filter_name(f, &cfg.pattern))
        .filter(|f: &PathBuf| filter_mod_time(f, &cfg.max_age))
        .take(cfg.limit)
        .inspect(|f| print_file(f, cfg))
        .collect();

    let (found, size) = summarize(files);

    print_summary("files", found, size, cfg);
}

fn walk_dirs(cfg: &Config) {
    let mut acc_size: HashMap<PathBuf, u64> = HashMap::new();
    let root: &PathBuf = cfg.paths.iter().sorted().collect_vec().first().unwrap();
    let root_level: usize = root.components().count();

    cfg.paths
        .iter()
        .flat_map(|path: &PathBuf| create_walker(&cfg, path))
        .filter(|f: &PathBuf| filter_mod_time(f, &cfg.max_age))
        .filter(|f: &PathBuf| filter_name(f, &cfg.pattern))
        .map(|f: PathBuf| size_of(&f))
        .for_each(|(dir, size)| update_size(&mut acc_size, dir, root, size));

    let acc_size: Vec<u64> = acc_size
        .iter()
        .filter(|(_, size)| **size >= cfg.min_size)
        .take(cfg.limit)
        .sorted_by(|(path0, _), (path1, _)| path0.cmp(path1))
        .inspect(|(path, size)| print_dir(path, **size, root_level, cfg))
        .map(|(_, size)| *size)
        .collect_vec();

    let size: u64 = *acc_size.iter().max().unwrap_or(&0);
    let found: u64 = acc_size.len() as u64;

    print_summary("directories", found, size, cfg);
}

fn update_size(acc_size: &mut HashMap<PathBuf, u64>, path: PathBuf, root: &PathBuf, size: u64) {
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

fn size_of(file: &PathBuf) -> (PathBuf, u64) {
    let size: u64 = match file.metadata() {
        Ok(metadata) => metadata.len(),
        Err(_) => 0,
    };
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
