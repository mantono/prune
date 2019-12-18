
use std::path::PathBuf;

pub fn summarize(files: Vec<PathBuf>) -> (u64, u64) {
    let found: u64 = files.len() as u64;
    let size: u64 = files.iter()
        .map(|f| f.metadata().unwrap().len())
        .sum();

    (found, size)
}

pub fn filter_size(file: &PathBuf, min_size: u64) -> bool {
    match file.metadata() {
        Ok(meta) => meta.len() >= min_size,
        Err(e) => {
            eprintln!("{}: {:?}", e, file);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::find::{summarize, filter_size};
    use crate::expl::FileExplorer;

    const TEST_DIR: &str = "test_dirs";

    #[test]
    fn test_stop_at_one_found_file() {
        let dir = PathBuf::from(TEST_DIR);
        let files: Vec<PathBuf> = FileExplorer::for_path(&dir, 10).take(1).collect();
        let result: (u64, u64) = summarize(files);
        assert_eq!(1, result.0);
    }

    #[test]
    fn test_filter_by_file_size() {
        let dir = PathBuf::from(TEST_DIR);
        let files: Vec<PathBuf> = FileExplorer::for_path(&dir, 10)
            .filter(|f| filter_size(f, 100))
            .collect();

        let result: (u64, u64) = summarize(files);
        assert_eq!(1, result.0);
        assert_eq!(100, result.1);
    }
}
