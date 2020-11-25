use crate::cfg::Config;
use crate::fs::FsEntity;
use humansize::{file_size_opts as options, FileSize};
use itertools::Itertools;

pub fn print_file(file: &FsEntity, cfg: &Config) {
    let size: u64 = file.len();
    if cfg.plumbing_mode {
        print_plumbing(file, size)
    } else {
        print_porcelain(file, size)
    }
}

fn print_porcelain(file: &FsEntity, size: u64) {
    let path: String = fmt_path(file, 0);
    let size = size.file_size(options::CONVENTIONAL).unwrap();
    println!("{:>10} │ {}", size, path);
}

pub fn print_dir(dir: &FsEntity, size: u64, cfg: &Config) {
    if cfg.plumbing_mode {
        print_plumbing(dir, size)
    } else {
        print_porcelain(&dir, size)
    }
}

fn print_plumbing(dir: &FsEntity, size: u64) {
    let dir = dir.path();
    println!("{}, {}", size, dir);
}

pub fn print_summary(kind: &str, found: u64, size: u64, cfg: &Config) {
    if cfg.plumbing_mode {
        print_summary_plumbing(found, size)
    } else {
        print_summary_porcelain(kind, found, size)
    }
}

fn print_summary_porcelain(kind: &str, found: u64, size: u64) {
    let human_size = size.file_size(options::CONVENTIONAL).unwrap();
    println!(
        "Found {} {} with a total size of {}",
        found, kind, human_size
    );
}

fn print_summary_plumbing(found: u64, size: u64) {
    println!("-----");
    println!("{}, {}", size, found)
}

fn fmt_path(path: &FsEntity, root_level: usize) -> String {
    let skip = if root_level == 0 {
        root_level
    } else {
        root_level - 1
    };

    path.to_path_buf()
        .components()
        .skip(skip)
        .map(|c| c.as_os_str().to_str().unwrap())
        .join("/")
        .replacen("//", "/", 1)
        .replace("\"", "")
}
