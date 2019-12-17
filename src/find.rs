
use std::path::PathBuf;
use std::fs::{ReadDir, Metadata};
use std::fs;
use std::collections::VecDeque;

pub trait SideEffect {
    fn submit(&mut self, file: &PathBuf);
}

pub fn explore(current_dir: PathBuf, rem_depth: u32, find: u64, min_size: u64, fc: &mut dyn SideEffect) -> u64 {
    let files = FileExplorer::for_path(&current_dir, rem_depth);
    files
        .filter(|f: &&PathBuf| filter_size(*f, min_size))
        .take(find as usize)
        .map(|f| f.canonicalize().expect("Unable to get canonical path"))
        .inspect(|f| fc.submit(f))
        //.map(|f| f.metadata().unwrap().len())
        .count() as u64
}

struct FileExplorer<'a> {
    files: VecDeque<&'a PathBuf>,
    dirs: VecDeque<&'a PathBuf>,
    rem_depth: u32
}

impl<'a> FileExplorer<'a> {
    pub fn for_path(path: &PathBuf, max_depth: u32) -> &FileExplorer {
        let (files, dirs) = FileExplorer::load(path);
        let dirs = VecDeque::from(dirs);
        let files = VecDeque::from(files);
        &FileExplorer {
            files,
            dirs,
            rem_depth: max_depth
        }
    }

    fn load(path: &PathBuf) -> (Vec<PathBuf>, Vec<PathBuf>) {
        let path: ReadDir = read_dirs(&path).unwrap();
        path.filter_map(|p| p.ok())
            .map(|p| p.path())
            .filter(|p: &PathBuf| is_valid_target(p))
            .partition(|p| p.is_file())
    }

    fn push(&mut self, path: &PathBuf) {
        let (files, dirs) = FileExplorer::load(path);
        self.files.extend(files);
        self.dirs.extend(dirs);
    }
}

impl<'a> Iterator for FileExplorer<'a> {
    type Item = &'a PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        match self.files.pop_front() {
            Some(f) => Some(&f),
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

fn filter_size(file: &PathBuf, min_size: u64) -> bool {
    match file.metadata() {
        Ok(meta) => meta.len() >= min_size,
        Err(e) => {
            eprintln!("{}: {:?}", e, file);
            false
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
            eprintln!("{}: {:?}", err, path);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::find::explore;
    use crate::find::tests::cf::Saver;

    #[test]
    fn test_depth_only_root_dir() {
        let mut save = Saver::new(10);
        let current_dir = PathBuf::from("test_dirs");
        let found: u64 = explore(current_dir, 0, 100, 1, &mut save);
        assert_eq!(1, found);
        assert_eq!("file0", save.files.first().unwrap().file_name().unwrap());
    }

    #[test]
    fn test_depth_one() {
        let mut save = Saver::new(10);
        let current_dir = PathBuf::from("test_dirs");
        let found: u64 = explore(current_dir, 1, 100, 1, &mut save);
        assert_eq!(3, found);
        assert!(save.files.iter().any(|f| f.file_name().unwrap() == "file0"));
        assert!(save.files.iter().any(|f| f.file_name().unwrap() == "file1"));
        assert!(save.files.iter().any(|f| f.file_name().unwrap() == "file2"));
    }

    #[test]
    fn test_stop_at_one_found_file() {
        let mut save = Saver::new(5);
        let current_dir = PathBuf::from("test_dirs");
        let found: u64 = explore(current_dir, 3, 1, 1, &mut save);
        assert_eq!(1, found);
    }

    #[test]
    fn test_filter_by_file_size() {
        let mut save = Saver::new(5);
        let current_dir = PathBuf::from("test_dirs");
        let found: u64 = explore(current_dir, 3, 10, 100, &mut save);
        assert_eq!(1, found);
        assert_eq!("file2", save.files.first().unwrap().file_name().unwrap());
    }

    mod cf {
        use std::path::PathBuf;
        use crate::find::SideEffect;

        pub struct Saver {
            pub files: Vec<PathBuf>
        }

        impl Saver {
            pub fn new(capacity: usize) -> Saver {
                Saver {
                    files: Vec::with_capacity(capacity)
                }
            }
        }

        impl SideEffect for Saver {
            fn submit(&mut self, file: &PathBuf) {
                self.files.push(file.to_path_buf())
            }
        }
    }
}
