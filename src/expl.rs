
use std::path::{PathBuf, Path};
use std::fs::{ReadDir, Metadata};
use std::{fs, io};
use std::collections::VecDeque;
use std::io::Error;

pub struct FileExplorer<T: PathEntry> {
    files: VecDeque<T>,
    dirs: VecDeque<T>,
    origin: T,
    max_depth: u32
}

impl<T: PathEntry> FileExplorer<T> {
    pub fn for_path(path: &T, max_depth: u32) -> FileExplorer<T> {
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
            max_depth
        }
    }

    fn load(path: &T) -> Result<(Vec<T>, Vec<T>), std::io::Error> {
        let (files, dirs) = path.children()?
            .filter(|p: &T| is_valid_target(p))
            .partition(|p| p.is_file());
        Ok((files, dirs))
    }

    fn push(&mut self, path: &T) {
        match FileExplorer::load(path) {
            Ok((files, dirs)) => {
                self.files.extend(files);
                let current_depth: u32 = self.depth(path) as u32;
                if current_depth < self.max_depth {
                    self.dirs.extend(dirs);
                }
            },
            Err(e) => eprintln!("{}: {:?}", e, path)
        }
    }

    fn depth(&self, dir: &PathBuf) -> usize {
        let comps0 = self.origin.canonicalize().unwrap().components().count();
        let comps1 = dir.canonicalize().unwrap().components().count();
        comps1 - comps0
    }
}

impl<T: PathEntry> Iterator for FileExplorer<T> {
    type Item = T;

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

fn read_dirs<T: PathEntry>(path: &T) -> Result<ReadDir, std::io::Error> {
    let full_path: T = path.canonical()?;
    Ok(fs::read_dir(full_path)?)
}

fn is_valid_target<T: PathEntry>(path: &T) -> bool {
    !path.is_symlink() && (path.is_dir() || path.is_file())
}

pub trait PathEntry: AsRef<Path> + Clone + Sized {
    fn is_symlink(&self) -> bool;
    fn is_file(&self) -> bool;
    fn is_dir(&self) -> bool;
    fn children(&self) -> Result<&dyn Iterator<Item=Self>, std::io::Error>;
    fn canonical(&self) -> io::Result<Self>;
}

impl PathEntry for PathBuf {
    fn is_symlink(&self) -> bool {
        match self.symlink_metadata() {
            Ok(sym) => sym.file_type().is_symlink(),
            Err(err) => {
                eprintln!("{}: {:?}", err, self);
                false
            }
        }
    }

    fn is_file(&self) -> bool {
        self.metadata().expect("Unable to retrieve metadata:").is_file()
    }

    fn is_dir(&self) -> bool {
        self.metadata().expect("Unable to retrieve metadata:").is_dir()
    }

    fn children(&self) ->Result<&dyn Iterator<Item=Self>, std::io::Error> {
        let path: ReadDir = read_dirs(&self).unwrap();
        path.filter_map(|p| p.ok())
            .map(|p| p.path())
            .it
    }

    fn canonical(&self) -> Result<Self, Error> {
        self.canonicalize()
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
        let found = FileExplorer::for_path(&dir, 0).count();
        assert_eq!(1, found);
    }

    #[test]
    fn test_depth_one() {
        let dir = PathBuf::from(TEST_DIR);
        let found = FileExplorer::for_path(&dir, 1).count();
        assert_eq!(3, found);
    }
}
