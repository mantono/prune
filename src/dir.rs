use core::cmp::Ordering;
use core::iter::Sum;
use fwalker::Walker;
use humansize::{file_size_opts as options, FileSize};
use itertools::{Group, Itertools};
use std::cmp::min;
use std::collections::{HashMap, VecDeque};
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::path::{Component, PathBuf};

trait Relation<T: Relation<T>> {
    fn is_child_of(&self, other: &T) -> bool;
}

impl Relation<PathBuf> for PathBuf {
    fn is_child_of(&self, other: &PathBuf) -> bool {
        let comp_self: Vec<&OsStr> = self.components().map(|c| c.as_os_str()).collect();
        let comp_other: Vec<&OsStr> = other.components().map(|c| c.as_os_str()).collect();
        let threshold: usize = comp_other.len();
        let common: usize = comp_self
            .iter()
            .zip(comp_other)
            .take_while(|(left, right)| left.to_os_string() == right.to_os_string())
            .count();

        common == threshold
    }
}

#[derive(Debug)]
pub struct Dir {
    path: String,
    size: u64,
    dirs: HashMap<String, Vec<Box<Dir>>>,
}

impl Dir {
    pub fn from(root: PathBuf, size: u64) -> Result<Dir, String> {
        if root.is_dir() {
            let dir = Dir {
                path: root,
                size,
                dirs: Vec::with_capacity(4),
            };
            Ok(dir)
        } else {
            Err(format!("Path is not a directory: {:?}", root))
        }
    }

    pub fn add_vec(mut self, mut path: VecDeque<OsString>, size: u64) -> Dir {
        path.pop_front()
    }

    pub fn add(mut self, path: PathBuf, size: u64) -> Dir {
        if path == self.path || !path.is_dir() {
            log::info!(
                "{:?} == {:?} || {:?} is not a dir",
                &path,
                self.path,
                self.path
            );
            self
        } else if path.is_child_of(&self.path) {
            log::info!("{:?} is child of {:?}", path, self.path);
            let dir = Dir::from(path, size).unwrap();
            self.dirs.push(Box::new(dir));
            self
        } else if self.path.is_child_of(&path) {
            log::info!("{:?} is child of {:?}", self.path, path);
            let mut dir = Dir::from(path, size).unwrap();
            dir.dirs.push(Box::new(self));
            dir
        } else {
            panic!("Illegal relation")
        }
    }

    pub fn size(&self) -> u64 {
        let children_size: u64 = self.dirs.iter().map(|c| c.size()).sum::<u64>();
        self.size + children_size
    }

    fn write_tree(dir: &Dir, parent: Option<&PathBuf>, fmt: &mut Formatter<'_>, level: usize) {
        let prefix: String = match level {
            0 => format!("{:?}", string_of(parent, &dir.path)),
            1 => format!(" ├── {:?}", string_of(parent, &dir.path)),
            _ => format!(
                " │{}└── {:?}",
                " ".repeat(3 * (level - 1)),
                string_of(parent, &dir.path)
            ),
        };
        let size = dir.size().file_size(options::CONVENTIONAL).unwrap();
        write!(fmt, "{}, {}\n", prefix, size).unwrap();
        for d in &dir.dirs {
            Dir::write_tree(d, Some(&dir.path), fmt, level + 1)
        }
    }
}

impl Display for Dir {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Dir::write_tree(self, None, f, 0);
        Ok(())
    }
}

fn string_of(parent: Option<&PathBuf>, path: &PathBuf) -> String {
    let fallback = PathBuf::from("/");
    let parent: &PathBuf = parent.unwrap_or(&fallback);
    parent
        .iter()
        .zip(path.iter())
        .skip_while(|(left, right)| left == right)
        .map(|(left, _)| left.to_str().unwrap().to_string())
        .join("/")
}

// fn name_of_diff(parent: &PathBuf, child: &PathBuf) -> String {
//     let drop = parent.components().collect_vec().len();
//     let remaining: Vec<Component> = child.components().skip(drop).collect_vec();
//     remaining
//         .iter()
//         .map(|c| c.as_os_str().to_str().unwrap().to_string())
//         .join("/")
// }

#[cfg(test)]
mod tests {
    use crate::dir::Dir;
    use std::path::PathBuf;

    #[test]
    fn test_build_in_order() {
        let test_dirs = PathBuf::from("test_dirs").canonicalize().unwrap();
        let sub_dir = PathBuf::from("test_dirs/sub_dir").canonicalize().unwrap();
        let mut dir = Dir::from(test_dirs, 1024).unwrap();
        let dir = dir.add(sub_dir, 1024);
        assert_eq!(2048, dir.size());
    }

    #[test]
    fn test_build_out_of_order() {
        let test_dirs = PathBuf::from("test_dirs").canonicalize().unwrap();
        let sub_dir = PathBuf::from("test_dirs/sub_dir").canonicalize().unwrap();
        let mut dir = Dir::from(sub_dir, 1024).unwrap();
        let dir = dir.add(test_dirs, 1024);
        assert_eq!(2048, dir.size());
    }
}
