use clap::ArgMatches;
use crate::args::Size;

pub struct Config {
    pub paths: Vec<String>,
    pub min_size: u64,
    pub max_depth: u32,
    pub limit: usize
}

impl Config {
    pub fn from_args(args: ArgMatches) -> Config {
        let min_size: Size = Size::from_arg(args.value_of("size").unwrap());
        let min_size: u64 = min_size.as_bytes();
        let max_depth: u32 = args.value_of("depth").unwrap().parse().unwrap();
        let limit: usize = args.value_of("limit").unwrap_or(&std::u64::MAX.to_string()).parse().unwrap();
        let paths: Vec<String> = args.values_of("path").unwrap().map(|v| v.to_string()).collect();

        Config {
            paths,
            min_size,
            max_depth,
            limit
        }
    }
}