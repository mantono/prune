extern crate humansize;
extern crate lazy_static;
extern crate structopt;

mod cfg;
mod dbg;
mod duration;
mod find;
mod logger;
mod parse;
mod print;
mod size;

use crate::cfg::Config;
use crate::dbg::dbg_info;
use crate::find::{filter_mod_time, filter_name, filter_size, summarize};
use crate::logger::setup_logging;
use crate::print::{print_dir, print_file, print_summary};
use crate::structopt::StructOpt;
use cfg::Mode;
use fwalker::Walker;
use itertools::Itertools;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process;

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
    let files: Vec<PathBuf> = cfg
        .abs_paths()
        .iter()
        .flat_map(|path: &PathBuf| create_walker(&cfg, path))
        .filter(|f: &PathBuf| filter_size(f, cfg.min_size_bytes()))
        .filter(|f: &PathBuf| filter_name(f, &cfg.pattern))
        .filter(|f: &PathBuf| !f.starts_with("/proc"))
        .filter(|f: &PathBuf| filter_mod_time(f, &cfg.max_age))
        .take(cfg.limit())
        .inspect(|f| print_file(f, cfg))
        .collect();

    summarize(files)
}

fn walk_dirs(cfg: &Config) -> (u64, u64) {
    let mut acc_size: HashMap<PathBuf, u64> = HashMap::new();

    for root in cfg.abs_paths() {
        create_walker(&cfg, &root)
            .filter(|f: &PathBuf| filter_mod_time(f, &cfg.max_age))
            .filter(|f: &PathBuf| filter_name(f, &cfg.pattern))
            .filter(|f: &PathBuf| !f.starts_with("/proc"))
            .map(|f: PathBuf| size_of(&f))
            .for_each(|(dir, size)| update_size(&mut acc_size, dir, &root, size));
    }

    let acc_size: Vec<u64> = acc_size
        .iter()
        .filter(|(_, size)| **size >= cfg.min_size_bytes())
        .take(cfg.limit())
        .sorted_by(|(path0, _), (path1, _)| path0.cmp(path1))
        .inspect(|(path, size)| print_dir(path, **size, cfg))
        .map(|(_, size)| *size)
        .collect_vec();

    let size: u64 = *acc_size.iter().max().unwrap_or(&0);
    let found: u64 = acc_size.len() as u64;

    (found, size)
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
    let walker = Walker::from_with_capacity(path, 128)
        .expect("Unable to crate Walker from Path")
        .max_depth(cfg.depth)
        .only_local_fs(cfg.only_local_fs);

    log::debug!("walker: {:?}", walker);
    walker
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        cfg::{Config, Mode},
        create_walker,
        size::Size,
        walk_dirs,
    };

    #[test]
    fn test_walk_dirs() {
        let cfg = Config::default()
            .with_mode(&Mode::Dir)
            .with_min_size(Size::Byte(1))
            .with_path("test_dirs");

        let (found, size) = walk_dirs(&cfg);
        assert_eq!(2, found);
        assert_eq!(182, size);
    }
}
