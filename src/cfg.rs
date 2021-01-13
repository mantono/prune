use crate::{duration::parse_duration, size::Size};
use regex::Regex;
use std::time::Duration;
use std::{path::PathBuf, str::FromStr};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "prn")]
/// prn test test
pub struct Config {
    /// Select zero, one or several directories for which to look for files in. If no value is give,
    /// the application will default to current directory
    #[structopt(parse(from_os_str))]
    pub paths: Vec<PathBuf>,

    /// Only show files or directories which exceeds this size. For example 400 is equivalent of 400
    /// bytes, 20m is equivalent of 20 megabytes and 5g is equivalent of 5 gigabytes.
    #[structopt(
        short = "s",
        long = "size",
        default_value = "100m",
        parse(try_from_str)
    )]
    min_size: Size,

    /// Descend and search for files or directories in directories with a max depth of this value.
    /// A depth of 0 will only look for files at the first level. By default the depth is unlimited.
    #[structopt(short = "d", long = "depth")]
    pub max_depth: Option<u32>,

    /// Only list the first N files found given by this limit. If no value is set for this option,
    /// the application will not stop until it has gone through all files in the directory and
    /// subdirectories.
    #[structopt(short, long)]
    pub limit: Option<usize>,

    /// Only include and count files matching the regular expression
    #[structopt(short, long)]
    pub pattern: Option<Regex>,

    /// Only include files which modification time is older than this. For example 180s for 180
    /// seconds, 45d for 45 days or 3y for 3 years.
    #[structopt(short = "m", long = "mod-time", parse(try_from_str = parse_duration))]
    pub max_age: Option<Duration>,

    /// Search for directories instead of files
    #[structopt(short = "R", long)]
    dirs: bool,

    /// Set the verbosity level, from 0 (least amount of output) to 5 (most verbose). Note that
    /// logging level configured via RUST_LOG overrides this setting.
    #[structopt(short, long = "verbosity", default_value = "1")]
    pub verbosity_level: u8,

    /// Only search for files in the same filesystem for the given path(s), or the current file
    /// system if no path is given.
    #[structopt(short = "x", long = "filesystem")]
    pub only_local_fs: bool,

    /// Use plumbing mode (as opposed to 'porcelain' mode) with an output that is more consistent
    /// and machine readable
    #[structopt(short = "P", long = "plumbing")]
    pub plumbing_mode: bool,

    /// Print debug information about current build for binary, useful for when an issue is
    /// encountered and reported
    #[structopt(short = "D", long = "debug")]
    pub print_dbg: bool,
}

struct Verbosity(u8);

impl FromStr for Verbosity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let n: u8 = s.parse().or(Err(format!("Unable to parse {} as u8", s)))?;
        match (0u8..=5u8).contains(&n) {
            true => Ok(Verbosity(n)),
            false => Err(format!("Value out of range (0 - 5): {}", n)),
        }
    }
}

impl Config {
    pub fn min_size_bytes(&self) -> u64 {
        self.min_size.as_bytes()
    }

    pub fn mode(&self) -> Mode {
        if self.dirs {
            Mode::Dir
        } else {
            Mode::File
        }
    }
}

#[derive(Debug)]
pub enum Mode {
    File,
    Dir,
}
