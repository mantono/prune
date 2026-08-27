use crate::cfg::{Config, Mode};
use humansize::{file_size_opts as options, FileSize};
use itertools::Itertools;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use walkdir::DirEntry;

pub fn print_file<W: Write>(writer: &mut W, entry: &DirEntry, cfg: &Config) -> io::Result<()> {
    let file: &Path = entry.path();
    let size: u64 = match file.metadata() {
        Ok(metadata) => metadata.len(),
        Err(err) => {
            log_error(err, file);
            return Ok(());
        }
    };
    if cfg.plumbing_mode {
        print_plumbing(writer, file, size)
    } else {
        print_porcelain(writer, file, size)
    }
}

pub fn print_dir<W: Write>(writer: &mut W, dir: &Path, size: u64, cfg: &Config) -> io::Result<()> {
    if cfg.plumbing_mode {
        print_plumbing(writer, dir, size)
    } else {
        print_porcelain(writer, dir, size)
    }
}

fn print_porcelain<W: Write>(writer: &mut W, file: &Path, size: u64) -> io::Result<()> {
    if let Some(path) = fmt_path(file, 0) {
        if let Ok(size) = size.file_size(options::CONVENTIONAL) {
            return writeln!(writer, "{:>10} │ {}", size, path);
        }
    }
    Ok(())
}

fn print_plumbing<W: Write>(writer: &mut W, dir: &Path, size: u64) -> io::Result<()> {
    if let Some(dir) = canonical(dir) {
        if let Some(dir) = dir.as_os_str().to_str() {
            return writeln!(writer, "{}, {}", size, dir);
        }
    }
    Ok(())
}

pub fn print_summary<W: Write>(
    writer: &mut W,
    kind: Mode,
    found: u64,
    size: u64,
    cfg: &Config,
) -> io::Result<()> {
    if cfg.plumbing_mode {
        print_summary_plumbing(writer, found, size)
    } else {
        print_summary_porcelain(writer, kind, found, size)
    }
}

fn print_summary_porcelain<W: Write>(
    writer: &mut W,
    mode: Mode,
    found: u64,
    size: u64,
) -> io::Result<()> {
    let kind: &str = match mode {
        Mode::File => "files",
        Mode::Dir => "directories",
    };
    let human_size: String = size.file_size(options::CONVENTIONAL).unwrap();
    writeln!(
        writer,
        "Found {} {} with a total size of {}",
        found, kind, human_size,
    )
}

fn print_summary_plumbing<W: Write>(writer: &mut W, found: u64, size: u64) -> io::Result<()> {
    writeln!(writer, "-----")?;
    writeln!(writer, "{}, {}", size, found)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};

    const TEST_DIR: &str = "test_dirs";

    /// Minimal test-only `Write` double that fails every operation with a
    /// configured `io::ErrorKind`. Used to prove that renderers propagate
    /// writer errors unchanged instead of swallowing or reclassifying them.
    struct FailingWriter {
        kind: io::ErrorKind,
    }

    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(self.kind, "synthetic write failure"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::new(self.kind, "synthetic flush failure"))
        }
    }

    #[test]
    fn porcelain_file_output_is_byte_for_byte() {
        let file: PathBuf = PathBuf::from(TEST_DIR).join("file0");
        let entry: DirEntry = entry_for(&file);
        let mut cfg: Config = Config::default();
        cfg.plumbing_mode = false;
        let mut out: Vec<u8> = Vec::new();
        print_file(&mut out, &entry, &cfg).unwrap();
        assert_eq!(out, expected_porcelain(&file, 41).into_bytes());
    }

    #[test]
    fn porcelain_dir_output_is_byte_for_byte() {
        let dir: PathBuf = PathBuf::from(TEST_DIR).join("sub_dir");
        let mut cfg: Config = Config::default();
        cfg.plumbing_mode = false;
        let mut out: Vec<u8> = Vec::new();
        print_dir(&mut out, &dir, 141, &cfg).unwrap();
        assert_eq!(out, expected_porcelain(&dir, 141).into_bytes());
    }

    #[test]
    fn plumbing_file_output_is_byte_for_byte() {
        let file: PathBuf = PathBuf::from(TEST_DIR).join("file0");
        let entry: DirEntry = entry_for(&file);
        let cfg: Config = Config::default();
        let mut out: Vec<u8> = Vec::new();
        print_file(&mut out, &entry, &cfg).unwrap();
        assert_eq!(out, expected_plumbing(&file, 41).into_bytes());
    }

    #[test]
    fn plumbing_dir_output_is_byte_for_byte() {
        let dir: PathBuf = PathBuf::from(TEST_DIR).join("sub_dir");
        let cfg: Config = Config::default();
        let mut out: Vec<u8> = Vec::new();
        print_dir(&mut out, &dir, 141, &cfg).unwrap();
        assert_eq!(out, expected_plumbing(&dir, 141).into_bytes());
    }

    #[test]
    fn summary_porcelain_output_is_byte_for_byte() {
        let mut cfg: Config = Config::default();
        cfg.plumbing_mode = false;
        let mut out: Vec<u8> = Vec::new();
        print_summary(&mut out, Mode::File, 3, 4096, &cfg).unwrap();
        let human: String = 4096u64.file_size(options::CONVENTIONAL).unwrap();
        let expected: String = format!("Found {} {} with a total size of {}\n", 3, "files", human);
        assert_eq!(out, expected.into_bytes());
    }

    #[test]
    fn summary_plumbing_output_is_byte_for_byte() {
        let cfg: Config = Config::default();
        let mut out: Vec<u8> = Vec::new();
        print_summary(&mut out, Mode::File, 3, 4096, &cfg).unwrap();
        assert_eq!(out, b"-----\n4096, 3\n".to_vec());
    }

    #[test]
    fn renderer_propagates_broken_pipe_unchanged() {
        let mut writer: FailingWriter = FailingWriter {
            kind: io::ErrorKind::BrokenPipe,
        };
        let file: PathBuf = PathBuf::from(TEST_DIR).join("file0");
        let cfg: Config = Config::default();
        let err: io::Error = print_dir(&mut writer, &file, 41, &cfg).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::BrokenPipe);
        assert_eq!(err.to_string(), "synthetic write failure");
    }

    #[test]
    fn renderer_propagates_non_pipe_error_unchanged() {
        let mut writer: FailingWriter = FailingWriter {
            kind: io::ErrorKind::PermissionDenied,
        };
        let file: PathBuf = PathBuf::from(TEST_DIR).join("file0");
        let cfg: Config = Config::default();
        let err: io::Error = print_dir(&mut writer, &file, 41, &cfg).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(err.to_string(), "synthetic write failure");
    }

    fn entry_for(path: &Path) -> DirEntry {
        walkdir::WalkDir::new(TEST_DIR)
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .find(|entry: &DirEntry| entry.path() == path)
            .expect("fixture entry must exist")
    }

    fn expected_porcelain(file: &Path, size: u64) -> String {
        let path = fmt_path(file, 0).expect("fixture path must canonicalize");
        let size = size
            .file_size(options::CONVENTIONAL)
            .expect("size must format");
        format!("{:>10} │ {}\n", size, path)
    }

    fn expected_plumbing(path: &Path, size: u64) -> String {
        let canon = canonical(path).expect("fixture path must canonicalize");
        let canon = canon.as_os_str().to_str().expect("path must be utf8");
        format!("{}, {}\n", size, canon)
    }
}
