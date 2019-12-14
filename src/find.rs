
use std::path::PathBuf;
use std::fs::{ReadDir, Metadata};
use std::fs;

pub trait ConsumeFile {
    fn submit(&mut self, file: &PathBuf);
}

pub fn explore(current_dir: PathBuf, rem_depth: u32, find: u64, min_size: u64, fc: &mut dyn ConsumeFile) -> u64 {
    let path: ReadDir = match read_dirs(&current_dir) {
        Err(e) => {
            eprintln!("{}: {:?}", e, current_dir);
            return 0
        },
        Ok(p) => p
    };
    let (files, dirs) = path.filter_map(|p| p.ok())
        .map(|p| p.path())
        .filter(|p: &PathBuf| is_valid_target(p))
        .partition(|p| p.is_file());

    let files: Vec<PathBuf> = files;

    let found: usize = files.iter()
        .filter(|f: &&PathBuf| {
            let meta = f.metadata().expect("Unable to read metadata");
            meta.len() > min_size
        })
        .take(find as usize)
        .map(|f| f.canonicalize().expect("Unable to get canonical path"))
        .inspect(|f| fc.submit(f))
        .count();

    let mut remaining: u64 = find - found as u64;

    if rem_depth > 0 && remaining > 0 {
        dirs.iter()
            .for_each(|p| remaining -= explore(p.clone(), rem_depth - 1, remaining, min_size, fc));
    }

    find - remaining
}

fn read_dirs(path: &PathBuf) -> Result<ReadDir, std::io::Error> {
    let full_path: PathBuf = path.canonicalize()?;
    Ok(fs::read_dir(full_path)?)
}

fn is_valid_target(path: &PathBuf) -> bool {
    let symlink: bool = path.symlink_metadata().unwrap().file_type().is_symlink();
    if !symlink {
        let metadata: Metadata = path.metadata().expect("Unable to retrieve metadata:");
        metadata.is_file() || metadata.is_dir()
    } else {
        false
    }
}