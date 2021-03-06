use crate::cfg::{Config, Mode};
use humansize::{file_size_opts as options, FileSize};
use itertools::Itertools;
use std::path::PathBuf;

pub fn print_file(file: &PathBuf, cfg: &Config) {
    let size: u64 = file.metadata().unwrap().len();
    if cfg.plumbing_mode {
        print_plumbing(file, size)
    } else {
        print_porcelain(file, size)
    }
}

fn print_porcelain(file: &PathBuf, size: u64) {
    let path: String = fmt_path(file, 0);
    let size = size.file_size(options::CONVENTIONAL).unwrap();
    println!("{:>10} │ {}", size, path);
}

pub fn print_dir(dir: &PathBuf, size: u64, cfg: &Config) {
    if cfg.plumbing_mode {
        print_plumbing(dir, size)
    } else {
        print_porcelain(&dir, size)
    }
}

fn print_plumbing(dir: &PathBuf, size: u64) {
    let dir = dir.canonicalize().unwrap();
    let dir = dir.as_os_str().to_str().unwrap();
    println!("{}, {}", size, dir);
}

pub fn print_summary(kind: Mode, found: u64, size: u64, cfg: &Config) {
    if cfg.plumbing_mode {
        print_summary_plumbing(found, size)
    } else {
        print_summary_porcelain(kind, found, size)
    }
}

fn print_summary_porcelain(mode: Mode, found: u64, size: u64) {
    let kind: &str = match mode {
        Mode::File => "files",
        Mode::Dir => "directories",
    };
    let human_size = size.file_size(options::CONVENTIONAL).unwrap();
    println!(
        "Found {} {} with a total size of {}",
        found, kind, human_size,
    );
}

fn print_summary_plumbing(found: u64, size: u64) {
    println!("-----");
    println!("{}, {}", size, found)
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
