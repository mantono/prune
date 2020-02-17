
use std::path::PathBuf;
use std::fs::{ReadDir, Metadata};
use std::fs;
use std::collections::VecDeque;

pub struct FileExplorer {
    files: VecDeque<PathBuf>,
    dirs: VecDeque<PathBuf>,
    origin: PathBuf,
    max_depth: u32,
    fs_filter: Option<Vec<PathBuf>>
}

impl FileExplorer {
    pub fn for_path(path: &PathBuf, max_depth: u32, fs_filter: Option<Vec<PathBuf>>) -> FileExplorer {
        let (files, dirs) = FileExplorer::load(path).expect("Unable to load path");
        let dirs = if max_depth > 0 {
            VecDeque::from(dirs)
        } else {
            VecDeque::with_capacity(0)
        };
        let files = VecDeque::from(files);
        FileExplorer {
            files,
            dirs,
            origin: path.clone(),
            max_depth,
            fs_filter
        }
    }

    fn load(path: &PathBuf) -> Result<(Vec<PathBuf>, Vec<PathBuf>), std::io::Error> {
        let path: ReadDir = read_dirs(&path)?;
        let (files, dirs) = path.filter_map(|p| p.ok())
            .map(|p| p.path())
            .filter(|p: &PathBuf| is_valid_target(p))
            .partition(|p| p.is_file());
        Ok((files, dirs))
    }

    fn push(&mut self, path: &PathBuf) {
        match FileExplorer::load(path) {
            Ok((files, dirs)) => {
                self.files.extend(files);
                let current_depth: u32 = self.depth(path) as u32;
                if current_depth < self.max_depth {
                    self.dirs.extend(dirs);
                }
            },
            Err(e) => log::warn!("{}: {:?}", e, path)
        }
    }

    fn depth(&self, dir: &PathBuf) -> usize {
        let comps0 = self.origin.canonicalize().unwrap().components().count();
        let comps1 = dir.canonicalize().unwrap().components().count();
        comps1 - comps0
    }
}

impl Iterator for FileExplorer {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        match self.files.pop_front() {
            Some(f) => Some(f),
            None => match self.dirs.pop_front() {
                Some(d) => {
                    self.push(&d);
                    self.next()
                },
                None => None
            }
        }
    }
}

fn read_dirs(path: &PathBuf) -> Result<ReadDir, std::io::Error> {
    let full_path: PathBuf = path.canonicalize()?;
    Ok(fs::read_dir(full_path)?)
}

fn is_valid_target(path: &PathBuf) -> bool {
    if !is_symlink(path) {
        let metadata: Metadata = path.metadata().expect("Unable to retrieve metadata:");
        metadata.is_file() || metadata.is_dir()
    } else {
        false
    }
}

fn is_symlink(path: &PathBuf) -> bool {
    match path.symlink_metadata() {
        Ok(sym) => sym.file_type().is_symlink(),
        Err(err) => {
            log::warn!("{}: {:?}", err, path);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::expl::FileExplorer;

    const TEST_DIR: &str = "test_dirs";

    #[test]
    fn test_depth_only_root_dir() {
        let dir = PathBuf::from(TEST_DIR);
        let found = FileExplorer::for_path(&dir, 0, None).count();
        assert_eq!(1, found);
    }

    #[test]
    fn test_depth_one() {
        let dir = PathBuf::from(TEST_DIR);
        let found = FileExplorer::for_path(&dir, 1, None).count();
        assert_eq!(3, found);
    }
}
