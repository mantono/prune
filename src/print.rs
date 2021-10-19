use crate::cfg::{Config, Mode};
use humansize::{file_size_opts as options, FileSize};
use itertools::Itertools;
use std::path::{Path, PathBuf};
use walkdir::DirEntry;

pub fn print_file(entry: &DirEntry, cfg: &Config) {
    let file: &Path = entry.path();
    let size: u64 = file.metadata().unwrap().len();
    if cfg.plumbing_mode {
        print_plumbing(file, size)
    } else {
        print_porcelain(file, size)
    }
}

pub fn print_dir(dir: &Path, size: u64, cfg: &Config) {
    if cfg.plumbing_mode {
        print_plumbing(dir, size)
    } else {
        print_porcelain(dir, size)
    }
}

fn print_porcelain(file: &Path, size: u64) {
    if let Some(path) = fmt_path(file, 0) {
        let size = size.file_size(options::CONVENTIONAL).unwrap();
        println!("{:>10} │ {}", size, path);
    }
}

fn print_plumbing(dir: &Path, size: u64) {
    if let Some(dir) = canonical(dir) {
        if let Some(dir) = dir.as_os_str().to_str() {
            println!("{}, {}", size, dir)
        }
    }
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

fn fmt_path(path: &Path, root_level: usize) -> Option<String> {
    let skip = if root_level == 0 {
        root_level
    } else {
        root_level - 1
    };

    let path: PathBuf = canonical(path)?;

    let formatted: String = path
        .components()
        .skip(skip)
        .map(|c| c.as_os_str().to_str().unwrap())
        .join("/")
        .replacen("//", "/", 1)
        .replace("\"", "");

    Some(formatted)
}

fn canonical(path: &Path) -> Option<PathBuf> {
    match path.canonicalize() {
        Ok(p) => Some(p),
        Err(e) => {
            log_error(e, path);
            None
        }
    }
}

fn log_error(err: std::io::Error, path: &Path) {
    log::error!("Error when accessing file {:?}: {}", path, &err);
}
