use crate::{duration::parse_duration, size::Size};
use itertools::Itertools;
use regex::Regex;
use std::path::Path;
use std::time::Duration;
use std::{path::PathBuf, str::FromStr};
use structopt::StructOpt;

#[cfg(not(target_os = "windows"))]
static APP_NAME: &str = "prn";
#[cfg(target_os = "windows")]
static APP_NAME: &str = "prune";

#[derive(StructOpt, Debug)]
#[structopt(name = APP_NAME, author, about)]
pub struct Config {
    /// Paths to look for files in
    ///
    /// Select zero, one or several directories for which to look for files in. If no value is
    /// given, the application will default to current directory
    #[structopt(parse(from_os_str), default_value = ".")]
    paths: Vec<PathBuf>,

    /// Print debug information
    ///
    /// Print debug information about current build for binary, useful for when an issue is
    /// encountered and reported
    #[structopt(short = "D", long = "debug")]
    pub print_dbg: bool,

    ///Search for directories
    ///
    /// Search for directories instead of files
    #[structopt(short = "R", long)]
    dirs: bool,

    /// Current filesystem only
    ///
    /// Only search for files in the same filesystem for the given path(s), or the current file
    /// system if no path is given.
    #[structopt(short = "x", long = "filesystem")]
    pub only_local_fs: bool,

    /// Use plumbing mode
    ///
    /// Use plumbing mode (as opposed to 'porcelain' mode) with an output that is more consistent
    /// and machine readable
    #[structopt(short = "P", long = "plumbing")]
    pub plumbing_mode: bool,

    /// Filter files by regex pattern
    ///
    /// Descend and search for files or directories in directories with a max depth of this value.
    /// A depth of 0 will only look for files at the first level. By default the depth is unlimited.
    #[structopt(short = "d", long = "max-depth")]
    depth: Option<usize>,

    /// Limit how many files to list
    ///
    /// Only list the first N files found given by this limit. If no value is set for this option,
    /// the application will not stop until it has gone through all files in the directory and
    /// subdirectories.
    #[structopt(short, long)]
    pub limit: Option<usize>,

    /// Filter based on min mod time
    ///
    /// Only include files which modification time is equal to or more than this.
    /// Such as `180s` for 180 seconds, `45d` for 45 days and `3y` for 3 years.
    #[structopt(short = "m", long = "min-mod-time", parse(try_from_str = parse_duration))]
    pub min_age: Option<Duration>,

    /// Filter based on max mod time
    ///
    /// Only include files which modification time is equal to or less than this.
    /// Such as `180s` for 180 seconds, `45d` for 45 days and `3y` for 3 years.
    #[structopt(short = "M", long = "max-mod-time", parse(try_from_str = parse_duration))]
    pub max_age: Option<Duration>,

    /// Filter files by regex pattern
    ///
    /// Only include and count files matching the regular expression
    #[structopt(short, long)]
    pub pattern: Option<Regex>,

    /// Set verbosity level, 0 - 5
    ///
    /// Set the verbosity level, from 0 (least amount of output) to 5 (most verbose). Note that
    /// logging level configured via RUST_LOG overrides this setting.
    #[structopt(short, long = "verbosity", default_value = "1")]
    pub verbosity_level: u8,

    /// Minimum file size
    ///
    /// Only show files or directories which exceeds this size. For example 400 is equivalent of 400
    /// bytes, 20m is equivalent of 20 megabytes and 5g is equivalent of 5 gigabytes.
    #[structopt(
        short = "s",
        long = "size",
        default_value = "100m",
        parse(try_from_str)
    )]
    min_size: Size,
}

struct Verbosity(u8);

impl FromStr for Verbosity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let n: u8 = s
            .parse()
            .map_err(|_| format!("Unable to parse {} as u8", s))?;
        match (0u8..=5u8).contains(&n) {
            true => Ok(Verbosity(n)),
            false => Err(format!("Value out of range (0 - 5): {}", n)),
        }
    }
}

impl Config {
    pub fn with_path<T: Into<PathBuf>>(mut self, path: T) -> Self {
        self.paths.push(path.into());
        self
    }

    pub fn with_limit(mut self, limit: Option<usize>) -> Self {
        self.limit = limit;
        self
    }

    pub fn min_size_bytes(&self) -> u64 {
        self.min_size.as_bytes()
    }

    pub fn max_depth(&self) -> usize {
        self.depth.unwrap_or(usize::MAX)
    }

    pub fn mode(&self) -> Mode {
        if self.dirs {
            Mode::Dir
        } else {
            Mode::File
        }
    }

    pub fn paths(&self) -> Vec<PathBuf> {
        self.paths
            .clone()
            .into_iter()
            .sorted()
            .filter(|p| Config::filter(&p))
            .collect_vec()
    }

    fn filter(path: &Path) -> bool {
        if !path.exists() {
            log::error!("Path does not exist: {:?}", path);
        }
        path.exists()
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            paths: Vec::with_capacity(1),
            print_dbg: false,
            dirs: false,
            only_local_fs: true,
            plumbing_mode: true,
            depth: None,
            limit: None,
            min_age: None,
            max_age: None,
            pattern: None,
            verbosity_level: 0,
            min_size: Size::Megabyte(100),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    File,
    Dir,
}
