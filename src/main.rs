#[macro_use]
extern crate clap;
extern crate humansize;
mod args;
mod cfg;
mod dbg;
mod dir;
mod find;
mod logger;

use crate::cfg::Config;
use crate::cfg::Mode;
use crate::dbg::dbg_info;
use crate::find::{filter_mod_time, filter_name, filter_size, summarize};
use crate::logger::setup_logging;
use core::cmp::Ordering;
use core::iter::Sum;
use fwalker::Walker;
use humansize::{file_size_opts as options, FileSize};
use itertools::{Group, Itertools};
use std::cmp::min;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::path::PathBuf;
use std::process;
use std::time::SystemTime;

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
    use itertools::GroupBy;

    //let dirs: HashMap<PathBuf, u64> =
    cfg.paths
        .iter()
        .map(PathBuf::from)
        .inspect(check_path)
        .flat_map(|path: PathBuf| create_walker(&cfg, &path))
        .filter(|f: &PathBuf| filter_mod_time(f, &cfg.max_age))
        .filter(|f: &PathBuf| filter_name(f, &cfg.pattern))
        .map(|f: PathBuf| size_of(&f))
        .for_each(|(dir, size)| {
            let cur_size: u64 = *acc_size.get(&dir).unwrap_or(&0u64);
            let new_size = cur_size + size;
            acc_size.insert(dir.to_path_buf(), new_size);
        });
    // .group_by(|(dir, _)| dir)
    // .into_iter()
    // .map(|(dir, sizes)| (dir, sizes.collect()))
    // .map(|(dir, sizes)| sum(dir.clone(), sizes))
    //.collect();

    // .filter(|f: &PathBuf| filter_name(f, &cfg.pattern))
    // .filter(|f: &PathBuf| filter_mod_time(f, &cfg.max_age))
    // .take(cfg.limit)
    // .inspect(|f| print(f))
    // .collect();

    //let (found, size) = summarize(files);

    acc_size
        .iter()
        .filter(|(_, size)| **size >= cfg.min_size)
        .sorted_by(|(dir0, _), (dir1, _)| cmp_path(&dir0, &dir1))
        .for_each(|(dir, size)| print_dir(dir, *size));

    let size: u64 = acc_size.values().sum();
    let found: usize = acc_size.len();
    let human_size = size.file_size(options::CONVENTIONAL).unwrap();
    println!("Found {} files with a total size of {}", found, human_size);
}

// struct DirTree {
//     level: usize,
//     dir: PathBuf,
//     children: Vec<Box<DirTree>>,
// }

// impl DirTree {
//     pub fn size(&self) -> u64 {
//         let own_size: u64 = self.dir.metadata().unwrap().len();
//         let children_size: u64 = self.children.iter().map(|c| c.size()).sum::<u64>();
//         own_size + children_size
//     }
//
//     pub fn from(root: PathBuf) -> DirTree {
//         DirTree {
//             level: 0,
//             dir: root,
//             children: Vec::with_capacity(8),
//         }
//     }
//
//     pub fn add(&mut self, dir: PathBuf) -> &mut DirTree {
//         if dir.is_child_of(&self.dir) {
//             let child = DirTree {
//                 level: self.level + 1,
//                 dir
//             }
//             self.children
//         }
//     }
//
//     fn create(dir: PathBuf, others: Vec<PathBuf>, level: usize) -> DirTree {
//         let children: Vec<Box<DirTree>> = Vec::with_capacity(others.len());
//         let this = DirTree {
//             level,
//             dir: dir.clone(),
//             children,
//         };
//
//         others
//             .iter()
//             .filter(|c| c.starts_with(&dir))
//             .filter(|c| **c != dir)
//             .collect();
//     }
//
//     fn from_level(map: HashMap<PathBuf, u64>, level: usize) -> DirTree {
//         map.iter().sorted().ma
//     }
// }

// impl Display for DirTree {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         let prefix: String = match self.level {
//             0 => String::from(""),
//             1 => String::from("├── "),
//             _ => format!("│{}└── ", " ".repeat((3 * self.level) as usize)),
//         };
//         write!(f, "{}{:?}", prefix, self.dir)
//     }
// }

fn cmp_path(left: &PathBuf, right: &PathBuf) -> core::cmp::Ordering {
    let level_left: usize = left.components().count();
    let level_right: usize = right.components().count();
    match level_left.cmp(&level_right) {
        Ordering::Less => Ordering::Less,
        Ordering::Greater => Ordering::Greater,
        Ordering::Equal => left.cmp(right),
    }
}

fn size_of(file: &PathBuf) -> (PathBuf, u64) {
    let size: u64 = file.metadata().unwrap().len();
    let parent: PathBuf = file.parent().unwrap().to_path_buf();
    (parent, size)
}

fn sum(dir: PathBuf, sizes: Vec<(PathBuf, u64)>) -> (PathBuf, u64) {
    (dir, sizes.iter().map(|(_, v)| v).sum())
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
    let canon: PathBuf = file
        .canonicalize()
        .expect("Unable to get canonical path for file");
    let size = file
        .metadata()
        .unwrap()
        .len()
        .file_size(options::CONVENTIONAL)
        .unwrap();
    println!("{}, {:?}", size, canon);
}

fn print_dir(dir: &PathBuf, size: u64) {
    let canon: PathBuf = dir
        .canonicalize()
        .expect("Unable to get canonical path for dir");
    let size = size.file_size(options::CONVENTIONAL).unwrap();
    println!("{}, {:?}", size, canon);
}
