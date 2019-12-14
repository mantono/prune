#[cfg(test)]
mod tests {
    use crate::cf::Saver;
    use crate::find::explore;
    use crate::lib::cf::Saver;
    use std::path::PathBuf;

    #[test]
    fn test_depth_only_root_dir() {
        let mut save = Saver::new(10);
        let current_dir = PathBuf::from("testing");
        let found: u64 = explore(current_dir, 0, 100, 1, &save);
        assert_eq!(1, found)
    }
}

pub mod cf {
    use std::path::PathBuf;
    use crate::find::ConsumeFile;

    pub struct Saver {
        files: Vec<PathBuf>
    }

    impl Saver {
        pub fn new(capacity: usize) -> Saver {
            Saver {
                files: Vec::with_capacity(capacity)
            }
        }
    }

    impl ConsumeFile for Saver {
        fn submit(&mut self, file: &PathBuf) {
            self.files.push(file.to_path_buf())
        }
    }
}