use regex::Regex;
use std::{
    fs::Metadata,
    time::{Duration, SystemTime},
};
use walkdir::DirEntry;

use crate::{
    cfg::{Config, Mode},
    size::Size,
};

pub struct Filter {
    only_local_fs: bool,
    max_age: Option<Duration>,
    pattern: Option<Regex>,
    min_size: u64,
    mode: Mode,
}

const PROC: &str = "/proc";

impl Filter {
    pub fn new(
        only_local_fs: bool,
        max_age: Option<Duration>,
        pattern: Option<Regex>,
        min_size: Size,
        mode: Mode,
    ) -> Filter {
        Filter {
            only_local_fs,
            max_age,
            pattern,
            min_size: min_size.as_bytes(),
            mode,
        }
    }
    pub fn accept(&self, e: &DirEntry) -> bool {
        let metadata: Metadata = match e.metadata() {
            Ok(metadata) => metadata,
            Err(err) => {
                log::warn!("Unable to obtain metadata for {:?}: {:?}", e.path(), err);
                return false;
            }
        };

        if let Mode::File = self.mode {
            if metadata.len() < self.min_size {
                return false;
            }
        }

        if !metadata.is_file() {
            return false;
        }

        let accept_age: bool = match self.max_age {
            Some(max_age) => Filter::filter_mod_time(&metadata, &max_age),
            None => true,
        };

        if !accept_age {
            return false;
        }

        let file_name: String = match Filter::file_name(e) {
            Some(name) => name,
            None => return false,
        };

        if e.path().starts_with(PROC) {
            return false;
        }

        match &self.pattern {
            Some(pattern) => pattern.is_match(&file_name),
            None => true,
        }
    }

    fn file_name(entry: &DirEntry) -> Option<String> {
        match entry.path().file_name() {
            Some(name) => name.to_str().map(|n| n.to_string()),
            None => None,
        }
    }

    fn filter_mod_time(metadata: &Metadata, max_age: &Duration) -> bool {
        let mod_time: SystemTime = match metadata.modified() {
            Ok(m) => m,
            Err(_) => return false,
        };

        let now = SystemTime::now();
        if mod_time > now {
            log::warn!(
                "Found modification timestamp set in the future: {:?}",
                mod_time
            );
            return false;
        }
        let elapsed_time: Duration = match now.duration_since(mod_time) {
            Ok(duration) => duration,
            Err(e) => {
                log::error!("Cannot get duration since {:?}: {}", mod_time, e);
                return false;
            }
        };
        elapsed_time > *max_age
    }
}

impl From<&Config> for Filter {
    fn from(cfg: &Config) -> Self {
        Filter {
            only_local_fs: cfg.only_local_fs,
            max_age: cfg.max_age,
            pattern: cfg.pattern.clone(),
            min_size: cfg.min_size_bytes(),
            mode: cfg.mode(),
        }
    }
}

pub fn summarize(files: Vec<DirEntry>) -> (u64, u64) {
    let found: u64 = files.len() as u64;
    let size: u64 = files.iter().map(|f| f.metadata().unwrap().len()).sum();

    (found, size)
}

#[cfg(test)]
mod tests {
    use crate::{cfg::Config, create_walker, find::summarize, walk_files};
    use regex::Regex;
    use std::path::PathBuf;
    use std::str::FromStr;
    use walkdir::DirEntry;

    const TEST_DIR: &str = "test_dirs";
    const PROC: &str = "/proc";

    #[test]
    fn test_stop_at_one_found_file() {
        let dir = PathBuf::from(TEST_DIR);
        let files: Vec<DirEntry> = create_walker(&Config::default(), &dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .take(1)
            .collect();
        let result: (u64, u64) = summarize(files);
        assert_eq!(1, result.0);
    }

    #[cfg(unix)]
    #[test]
    fn test_filter_by_file_size() {
        let dir = PathBuf::from(TEST_DIR);
        let files: Vec<DirEntry> = create_walker(&Config::default(), &dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|f: &DirEntry| f.metadata().unwrap().is_file())
            .filter(|f| filter_size(f, 100))
            .collect();

        let result: (u64, u64) = summarize(files);
        assert_eq!(1, result.0);
        assert_eq!(100, result.1);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_filter_out_proc() {
        let cfg = Config::default().with_path(PROC);
        let (found, _) = walk_files(&cfg);
        assert_eq!(0, found);
    }

    #[test]
    fn test_filter_by_file_pattern() {
        let dir = PathBuf::from(TEST_DIR);
        let pattern: Option<Regex> = Some(Regex::from_str("file[01]$").unwrap());
        let files: Vec<DirEntry> = create_walker(&Config::default(), &dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|f| filter_name(f, &pattern))
            .collect();

        assert_eq!(2, files.len());
        assert_ne!(
            "file2",
            files.first().unwrap().file_name().to_str().unwrap()
        );
        assert_ne!("file2", files.last().unwrap().file_name().to_str().unwrap());
    }
}
