extern crate humansize;
extern crate lazy_static;
extern crate structopt;

mod cfg;
mod dbg;
mod duration;
mod find;
mod logger;
mod parse;
mod print;
mod size;

use crate::cfg::Config;
use crate::dbg::dbg_info;
use crate::find::summarize;
use crate::logger::setup_logging;
use crate::print::{print_dir, print_file, print_summary};
use crate::structopt::StructOpt;
use cfg::Mode;
use find::Filter;
use itertools::Itertools;
use std::collections::HashMap;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Eq, PartialEq)]
enum OutputErrorDisposition {
    SilentSuccess,
    DiagnosticFailure,
}

fn classify_output_error(error: &std::io::Error) -> OutputErrorDisposition {
    match error.kind() {
        std::io::ErrorKind::BrokenPipe => OutputErrorDisposition::SilentSuccess,
        _ => OutputErrorDisposition::DiagnosticFailure,
    }
}

fn run(cfg: &Config, writer: &mut impl Write) -> io::Result<()> {
    if cfg.print_dbg {
        return writeln!(writer, "{}", dbg_info());
    }

    let (found, size): (u64, u64) = match cfg.mode() {
        Mode::File => walk_files(cfg, writer),
        Mode::Dir => walk_dirs(cfg, writer),
    }?;

    print_summary(writer, cfg.mode(), found, size, cfg)
}

fn main() {
    let cfg: Config = Config::from_args();
    setup_logging(cfg.verbosity_level);
    log::debug!("Config: {:?}", cfg);

    let stdout: io::Stdout = io::stdout();
    let mut writer: BufWriter<io::StdoutLock<'_>> = BufWriter::new(stdout.lock());
    let _ = run(&cfg, &mut writer);
}

fn walk_files(cfg: &Config, writer: &mut impl Write) -> io::Result<(u64, u64)> {
    let limit: usize = cfg.limit.unwrap_or(usize::MAX);
    let filter: Filter = cfg.into();
    let files: Vec<DirEntry> = cfg
        .paths()
        .iter()
        .map(|path: &PathBuf| create_walker(cfg, path))
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter(|entry: &DirEntry| filter.accept(entry))
        .take(limit)
        .try_fold(
            Vec::new(),
            |mut files: Vec<DirEntry>, file: DirEntry| -> io::Result<Vec<DirEntry>> {
                print_file(writer, &file, cfg)?;
                files.push(file);
                Ok(files)
            },
        )?;

    Ok(summarize(files))
}

fn walk_dirs(cfg: &Config, writer: &mut impl Write) -> io::Result<(u64, u64)> {
    let mut acc_size: HashMap<PathBuf, u64> = HashMap::new();
    let paths: Vec<PathBuf> = cfg.paths();
    let root: &Path = paths.first().unwrap();
    let filter: Filter = cfg.into();

    cfg.paths()
        .iter()
        .map(|path: &PathBuf| create_walker(cfg, path))
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e: &DirEntry| filter.accept(e))
        .map(|f: DirEntry| size_of(&f))
        .for_each(|(dir, size)| update_size(&mut acc_size, dir, root, size));

    let limit: usize = cfg.limit.unwrap_or(usize::MAX);
    let sizes: Vec<u64> = acc_size
        .iter()
        .filter(|(_, size)| **size >= cfg.min_size_bytes())
        .take(limit)
        .sorted_by(|(path0, _), (path1, _)| path0.cmp(path1))
        .try_fold(
            Vec::new(),
            |mut sizes: Vec<u64>, (path, size): (&PathBuf, &u64)| -> io::Result<Vec<u64>> {
                print_dir(writer, path, *size, cfg)?;
                sizes.push(*size);
                Ok(sizes)
            },
        )?;

    let size: u64 = *sizes.iter().max().unwrap_or(&0);
    let found: u64 = sizes.len() as u64;

    Ok((found, size))
}

fn update_size(acc_size: &mut HashMap<PathBuf, u64>, path: PathBuf, root: &Path, size: u64) {
    let cur_size: u64 = *acc_size.get(&path).unwrap_or(&0u64);
    let new_size = cur_size + size;
    acc_size.insert(path.clone(), new_size);
    if path == *root {
        return;
    }
    if let Some(parent) = path.parent() {
        update_size(acc_size, PathBuf::from(parent), root, size)
    }
}

fn size_of(entry: &DirEntry) -> (PathBuf, u64) {
    let size: u64 = match entry.metadata() {
        Ok(metadata) => metadata.len(),
        Err(_) => 0,
    };
    let parent: PathBuf = entry.path().parent().unwrap().to_path_buf();
    (parent, size)
}

fn create_walker(cfg: &Config, path: &Path) -> WalkDir {
    let walker = WalkDir::new(path)
        .follow_links(false)
        .max_depth(cfg.max_depth())
        .same_file_system(cfg.only_local_fs);

    log::debug!("walkdir: {:?}", walker);
    walker
}

#[cfg(test)]
mod tests {
    use super::{classify_output_error, run, OutputErrorDisposition};
    use crate::cfg::Config;
    use crate::dbg::dbg_info;
    use std::io::{self, Write};
    use structopt::StructOpt;

    struct FailingWriter {
        writes: usize,
    }

    impl Write for FailingWriter {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            self.writes += 1;
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "synthetic output failure",
            ))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn cli_execution_writes_debug_output_through_injected_writer() {
        let cfg: Config = Config::from_iter_safe(&["prn", "--debug"]).unwrap();
        let mut output: Vec<u8> = Vec::new();

        run(&cfg, &mut output).unwrap();

        assert_eq!(output, format!("{}\n", dbg_info()).into_bytes());
    }

    #[test]
    fn cli_execution_stops_at_and_propagates_first_output_failure() {
        let cfg: Config = Config::from_iter_safe(&["prn", "-P", "-s", "1", "test_dirs"]).unwrap();
        let mut writer: FailingWriter = FailingWriter { writes: 0 };

        let error: io::Error = run(&cfg, &mut writer).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(error.to_string(), "synthetic output failure");
        assert_eq!(writer.writes, 1);
    }

    #[test]
    fn classify_output_error_treats_broken_pipe_as_silent_success() {
        let error: io::Error = io::Error::from(io::ErrorKind::BrokenPipe);

        assert_eq!(
            classify_output_error(&error),
            OutputErrorDisposition::SilentSuccess
        );
    }

    #[test]
    fn classify_output_error_treats_non_pipe_as_diagnostic_failure() {
        let error: io::Error = io::Error::from(io::ErrorKind::PermissionDenied);

        assert_eq!(
            classify_output_error(&error),
            OutputErrorDisposition::DiagnosticFailure
        );
    }
}
