use regex::Regex;
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
            log::warn!("{}: {:?}", e, file);
            false
        }
    }
}

pub fn filter_name(path: &PathBuf, pattern: &Option<Regex>) -> bool {
    match pattern {
        None => true,
        Some(regex) =>  {
            let file_name: &str = path.file_name().unwrap().to_str().unwrap();
            regex.is_match(file_name)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::find::{summarize, filter_size, filter_name};
    use crate::expl::FileExplorer;
    use regex::Regex;
    use std::str::FromStr;

    const TEST_DIR: &str = "test_dirs";

    #[test]
    fn test_stop_at_one_found_file() {
        let dir = PathBuf::from(TEST_DIR);
        let files: Vec<PathBuf> = FileExplorer::for_path(&dir, 10, vec![]).take(1).collect();
        let result: (u64, u64) = summarize(files);
        assert_eq!(1, result.0);
    }

    #[test]
    fn test_filter_by_file_size() {
        let dir = PathBuf::from(TEST_DIR);
        let files: Vec<PathBuf> = FileExplorer::for_path(&dir, 10, vec![])
            .filter(|f| filter_size(f, 100))
            .collect();

        let result: (u64, u64) = summarize(files);
        assert_eq!(1, result.0);
        assert_eq!(100, result.1);
    }

    #[test]
    fn test_filter_by_file_pattern() {
        let dir = PathBuf::from(TEST_DIR);
        let pattern: Option<Regex> = Some(Regex::from_str("file[01]$").unwrap());
        let files: Vec<PathBuf> = FileExplorer::for_path(&dir, 10, vec![])
            .filter(|f| filter_name(f, &pattern))
            .collect();

        assert_eq!(2, files.len());
        assert_eq!("file0", files.first().unwrap().file_name().unwrap().to_str().unwrap());
        assert_eq!("file1", files.last().unwrap().file_name().unwrap().to_str().unwrap());
    }
}
