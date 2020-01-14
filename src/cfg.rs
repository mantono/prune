use crate::args::Size;
use clap::ArgMatches;
use regex::Regex;
use std::str::FromStr;

pub struct Config {
    pub paths: Vec<String>,
    pub min_size: u64,
    pub max_depth: u32,
    pub limit: usize,
    pub pattern: Option<Regex>,
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

        Config {
            paths,
            min_size,
            max_depth,
            limit,
            pattern
        }
    }
}
