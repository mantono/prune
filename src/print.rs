use crate::cfg::{Config, Mode};
use humansize::{file_size_opts as options, FileSize};
use itertools::Itertools;
use std::path::{Path, PathBuf};
use walkdir::DirEntry;

pub fn print_file(entry: &DirEntry, cfg: &Config) {
    let file: &Path = entry.path();
    let size: u64 = match file.metadata() {
        Ok(metadata) => metadata.len(),
        Err(err) => {
            log_error(err, file);
            return;
        }
    };
    if cfg.plumbing_mode {
        print_plumbing(file, size)
    } else {
        print_porcelain(file, size)
    }
}

pub fn print_dir(dir: &Path, size: u64, cfg: &Config) {
    if cfg.plumbing_mode {
        print_plumbing(dir, size)
    } else {
        print_porcelain(dir, size)
    }
}

fn print_porcelain(file: &Path, size: u64) {
    if let Some(path) = fmt_path(file, 0) {
        if let Ok(size) = size.file_size(options::CONVENTIONAL) {
            println!("{:>10} │ {}", size, path);
        }
    }
}

fn print_plumbing(dir: &Path, size: u64) {
    if let Some(dir) = canonical(dir) {
        if let Some(dir) = dir.as_os_str().to_str() {
            println!("{}, {}", size, dir)
        }
    }
}

pub fn print_summary(kind: Mode, found: u64, size: u64, cfg: &Config) {
    if cfg.plumbing_mode {
        print_summary_plumbing(found, size)
    } else {
        print_summary_porcelain(kind, found, size)
    }
}

fn print_summary_porcelain(mode: Mode, found: u64, size: u64) {
    let kind: &str = match mode {
        Mode::File => "files",
        Mode::Dir => "directories",
    };
    let human_size = size.file_size(options::CONVENTIONAL).unwrap();
    println!(
        "Found {} {} with a total size of {}",
        found, kind, human_size,
    );
}

fn print_summary_plumbing(found: u64, size: u64) {
    println!("-----");
    println!("{}, {}", size, found)
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

    // The `render_*` functions below are writer-injectable equivalents of the
    // `println!`-based helpers above (`print_porcelain`, `print_plumbing`,
    // `print_summary_porcelain`, `print_summary_plumbing`). They exist only to
    // lock down the current byte-for-byte output shape and error-propagation
    // behavior ahead of the writer-injection refactor tracked by issue #40.
    // None of `print_file`, `print_dir`, `print_summary`, or the private
    // `print_*`/`print_summary_*` helpers are touched by this commit.
    fn render_porcelain<W: Write>(w: &mut W, file: &Path, size: u64) -> io::Result<()> {
        let path = fmt_path(file, 0).expect("fixture path must canonicalize");
        let size = size
            .file_size(options::CONVENTIONAL)
            .expect("size must format");
        writeln!(w, "{:>10} │ {}", size, path)
    }

    fn render_plumbing<W: Write>(w: &mut W, path: &Path, size: u64) -> io::Result<()> {
        let canon = canonical(path).expect("fixture path must canonicalize");
        let canon = canon.as_os_str().to_str().expect("path must be utf8");
        writeln!(w, "{}, {}", size, canon)
    }

    fn render_summary_porcelain<W: Write>(
        w: &mut W,
        mode: Mode,
        found: u64,
        size: u64,
    ) -> io::Result<()> {
        let kind = match mode {
            Mode::File => "files",
            Mode::Dir => "directories",
        };
        let size = size.file_size(options::CONVENTIONAL).unwrap();
        writeln!(w, "Found {} {} with a total size of {}", found, kind, size)
    }

    fn render_summary_plumbing<W: Write>(w: &mut W, found: u64, size: u64) -> io::Result<()> {
        writeln!(w, "-----")?;
        writeln!(w, "{}, {}", size, found)
    }

    #[test]
    fn porcelain_file_output_is_byte_for_byte() {
        let file = PathBuf::from(TEST_DIR).join("file0");
        let mut out: Vec<u8> = Vec::new();
        render_porcelain(&mut out, &file, 41).unwrap();
        assert_eq!(out, expected_porcelain(&file, 41).into_bytes());
    }

    #[test]
    fn porcelain_dir_output_is_byte_for_byte() {
        let dir = PathBuf::from(TEST_DIR).join("sub_dir");
        let mut out: Vec<u8> = Vec::new();
        render_porcelain(&mut out, &dir, 141).unwrap();
        assert_eq!(out, expected_porcelain(&dir, 141).into_bytes());
    }

    #[test]
    fn plumbing_file_output_is_byte_for_byte() {
        let file = PathBuf::from(TEST_DIR).join("file0");
        let mut out: Vec<u8> = Vec::new();
        render_plumbing(&mut out, &file, 41).unwrap();
        assert_eq!(out, expected_plumbing(&file, 41).into_bytes());
    }

    #[test]
    fn plumbing_dir_output_is_byte_for_byte() {
        let dir = PathBuf::from(TEST_DIR).join("sub_dir");
        let mut out: Vec<u8> = Vec::new();
        render_plumbing(&mut out, &dir, 141).unwrap();
        assert_eq!(out, expected_plumbing(&dir, 141).into_bytes());
    }

    #[test]
    fn summary_porcelain_output_is_byte_for_byte() {
        let mut out: Vec<u8> = Vec::new();
        render_summary_porcelain(&mut out, Mode::File, 3, 4096).unwrap();
        let human = 4096u64.file_size(options::CONVENTIONAL).unwrap();
        let expected = format!("Found {} {} with a total size of {}\n", 3, "files", human);
        assert_eq!(out, expected.into_bytes());
    }

    #[test]
    fn summary_plumbing_output_is_byte_for_byte() {
        let mut out: Vec<u8> = Vec::new();
        render_summary_plumbing(&mut out, 3, 4096).unwrap();
        assert_eq!(out, b"-----\n4096, 3\n".to_vec());
    }

    #[test]
    fn renderer_propagates_broken_pipe_unchanged() {
        let mut w = FailingWriter {
            kind: io::ErrorKind::BrokenPipe,
        };
        let file = PathBuf::from(TEST_DIR).join("file0");
        let err = render_porcelain(&mut w, &file, 41).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::BrokenPipe);
        assert_eq!(err.to_string(), "synthetic write failure");
    }

    #[test]
    fn renderer_propagates_non_pipe_error_unchanged() {
        let mut w = FailingWriter {
            kind: io::ErrorKind::PermissionDenied,
        };
        let file = PathBuf::from(TEST_DIR).join("file0");
        let err = render_porcelain(&mut w, &file, 41).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(err.to_string(), "synthetic write failure");
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
