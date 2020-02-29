use crate::args::Size;
use clap::ArgMatches;
use regex::Regex;
use std::str::FromStr;
use crate::fs::{filesystems, fs_boundaries};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct Config {
    pub paths: Vec<String>,
    pub min_size: u64,
    pub max_depth: u32,
    pub limit: usize,
    pub pattern: Option<Regex>,
    pub verbosity_level: u8,
    pub fs_boundaries: HashMap<PathBuf, Vec<PathBuf>>
}

impl Config {
    pub fn from_args(args: ArgMatches) -> Config {
        let min_size: Size = Size::from_arg(args.value_of("size").unwrap());
        let min_size: u64 = min_size.as_bytes();
        let max_depth: u32 = args
            .value_of("depth")
            .unwrap_or(&std::u32::MAX.to_string())
            .parse()
            .unwrap();
        let limit: usize = args
            .value_of("limit")
            .unwrap_or(&std::u64::MAX.to_string())
            .parse()
            .unwrap();
        let paths: Vec<String> = args
            .values_of("path")
            .unwrap()
            .map(|v| v.to_string())
            .collect();
        let pattern: Option<Regex> = args
            .value_of("pattern")
            .map(|p| Regex::from_str(p).expect("Unable to parse regex"));
        let verbosity_level: u8 = args.value_of("verbosity").unwrap().parse::<u8>().unwrap();

        let only_local_fs: bool = args.is_present("filesystem");
        let mut boundaries: HashMap<PathBuf, Vec<PathBuf>> = HashMap::with_capacity(paths.len());

        if only_local_fs {
            let fs: Vec<PathBuf> = filesystems().expect("Unable to locate mounted filesystems");
            paths.iter()
                .map(|p| PathBuf::from(p).canonicalize().unwrap())
                .map(|p: PathBuf| {
                    (p.clone(), fs_boundaries(&fs, &p))
                })
                .for_each(|p| {
                    boundaries.insert(p.0, p.1);
                });
        }

        Config {
            paths,
            min_size,
            max_depth,
            limit,
            pattern,
            verbosity_level,
            fs_boundaries: boundaries
        }
    }
}
