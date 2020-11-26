#[macro_use]
extern crate clap;
extern crate humansize;
mod args;
mod cfg;
mod dbg;
mod find;
mod fs;
mod logger;
mod print;

use crate::cfg::Config;
use crate::cfg::Mode;
use crate::dbg::dbg_info;
use crate::find::{filter_mod_time, filter_name, filter_size, summarize};
use crate::fs::FsEntity;
use crate::logger::setup_logging;
use crate::print::{print_dir, print_file, print_summary};
use fwalker::Walker;
use itertools::Itertools;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process;

fn main() {
    let start = std::time::Instant::now();
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

    let end = std::time::Instant::now();
    log::debug!(
        "Elapsed execution time: {} ms",
        end.duration_since(start).as_millis()
    )
}

fn walk_files(cfg: &Config) {
    let files: Vec<FsEntity> = cfg
        .paths
        .iter()
        .flat_map(|path: &PathBuf| create_walker(&cfg, path))
        .filter_map(|f: PathBuf| FsEntity::from_path_buf(f).ok())
        .filter(|f: &FsEntity| filter_size(f, cfg.min_size))
        .filter(|f: &FsEntity| filter_name(f, &cfg.pattern))
        .filter(|f: &FsEntity| filter_mod_time(f, &cfg.max_age))
        .take(cfg.limit)
        .inspect(|f| print_file(f, cfg))
        .collect();

    let (found, size) = summarize(files);

    print_summary("files", found, size, cfg);
}

fn walk_dirs(cfg: &Config) {
    let mut acc_size: HashMap<FsEntity, u64> = HashMap::new();
    let root: String = cfg
        .paths
        .iter()
        .sorted()
        .collect_vec()
        .first()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    cfg.paths
        .iter()
        .flat_map(|path: &PathBuf| create_walker(&cfg, path))
        .filter_map(|f: PathBuf| FsEntity::from_path_buf(f).ok())
        .filter(|f: &FsEntity| filter_mod_time(f, &cfg.max_age))
        .filter(|f: &FsEntity| filter_name(f, &cfg.pattern))
        .map(|f: FsEntity| size_of(&f))
        .for_each(|(dir, size)| update_size(&mut acc_size, dir, &root, size));

    let acc_size: Vec<u64> = acc_size
        .iter()
        .filter(|(_, size)| **size >= cfg.min_size)
        .take(cfg.limit)
        .sorted_by(|(path0, _), (path1, _)| path0.cmp(path1))
        .inspect(|(path, size)| print_dir(path, **size, cfg))
        .map(|(_, size)| *size)
        .collect_vec();

    let size: u64 = *acc_size.iter().max().unwrap_or(&0);
    let found: u64 = acc_size.len() as u64;

    print_summary("directories", found, size, cfg);
}

fn update_size(acc_size: &mut HashMap<FsEntity, u64>, path: FsEntity, root: &String, size: u64) {
    let fs_path: String = match path.path() {
        Some(s) => s.to_string(),
        None => return,
    };
    let cur_size: u64 = *acc_size.get(&path).unwrap_or(&0u64);
    let new_size = cur_size + size;
    let parent: Option<FsEntity> = path.parent();
    acc_size.insert(path, new_size);
    if fs_path == *root {
        return;
    }
    if let Some(parent) = parent {
        update_size(acc_size, parent, root, size)
    }
}

fn size_of(file: &FsEntity) -> (FsEntity, u64) {
    let size: u64 = file.len();
    let parent: FsEntity = file.parent().unwrap();
    (parent, size)
}

fn create_walker(cfg: &Config, path: &PathBuf) -> Walker {
    let walker = Walker::from_with_capacity(path, 128)
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
