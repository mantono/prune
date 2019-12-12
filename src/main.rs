#[macro_use]
extern crate clap;

use std::fs;
use std::fs::ReadDir;
use std::path::PathBuf;
use clap::{ArgMatches, App, Arg};
use crate::Size::{Byte, Kilobyte, Megabyte, Gigabyte, Terabyte};

enum Size {
    Byte(u64),
    Kilobyte(u64),
    Megabyte(u64),
    Gigabyte(u64),
    Terabyte(u64)
}

impl Size {
    fn from_arg(arg: &str) -> Size {
        let char: String = arg.chars()
            .filter(|c| c.is_alphabetic())
            .next()
            .unwrap_or('b')
            .to_lowercase()
            .to_string();

        let size: u64 = arg[0..arg.len()-1].parse().expect("Unable to parse size");
        match char.as_ref() {
            "b" => Byte(size),
            "k" => Kilobyte(size),
            "m" => Megabyte(size),
            "g" => Gigabyte(size),
            "t" => Terabyte(size),
            _ => panic!("Invalid size type '{}'", char)
        }
    }

    fn as_bytes(&self) -> u64 {
        match self {
            &Byte(n) => n,
            &Kilobyte(n) => 1024 * n,
            &Megabyte(n) => 1024 * 1024 * n,
            &Gigabyte(n) => 1024 * 1024 * 1024 * n,
            &Terabyte(n) => 1024 * 1024 * 1024 * 1024 * n,
        }
    }
}

fn main() {
    let args = args();
    let min_size: Size = Size::from_arg(args.value_of("size").unwrap());
    let min_size: u64 = min_size.as_bytes();
    let max_depth: u32 = args.value_of("depth").unwrap().parse().unwrap();
    let limit: usize = args.value_of("limit").unwrap_or(&std::u64::MAX.to_string()).parse().unwrap();
    let limit: u64 = limit as u64;

    explore(fs::read_dir("./").unwrap(), max_depth, limit, min_size);
}

fn explore(path: ReadDir, rem_depth: u32, find: u64, min_size: u64) -> u64 {
    let (files, dirs) = path.filter_map(|p| p.ok())
        .map(|p| p.path())
        .partition(|p| p.is_file());

    let files: Vec<PathBuf> = files;

    let found: usize = files.iter()
        .filter(|f: &&PathBuf| {
            let meta = f.metadata().expect("Unable to read metadata");
            meta.len() > min_size
        })
        .take(find as usize)
        .map(|f| f.canonicalize().expect("Unable to get canonical path"))
        .inspect(|f| println!("{:?}", f))
        .count();

    let mut remaining: u64 = find - found as u64;

    if rem_depth > 0 && remaining > 0 {
        dirs.iter()
            .filter_map(|p| read_dirs(p).ok())
            .for_each(|p| remaining -= explore(p, rem_depth - 1, remaining, min_size));
    }

    find - remaining
}

fn read_dirs(path: &PathBuf) -> Result<ReadDir, std::io::Error> {
    let full_path: PathBuf = path.canonicalize()?;
    Ok(fs::read_dir(full_path)?)
}

pub fn args<'a>() -> ArgMatches<'a> {
    let path = Arg::with_name("path")
        .default_value(".")
        .takes_value(true)
        .multiple(true)
        .required(true)
        .help("Paths to look for files in")
        .long_help("Select zero, one or several directories for which to look for files in. If no value is give, the application will default to current directory");

    let depth = Arg::with_name("depth")
        .takes_value(true)
        .default_value("128")
        .max_values(1024)
        .short("d")
        .long("depth")
        .required(false)
        .help("Depth in folder hierarchy")
        .long_help("Descend and search for files in directories with a max depth of this value. A depth of 0 will only look for files at the first level.");

    let size = Arg::with_name("size")
        .default_value("100m")
        .takes_value(true)
        .short("s")
        .long("size")
        .multiple(false)
        .required(true)
        .help("Minimum file size")
        .long_help("Only show files which exceeds this file size. For example 400 is equivalent of 400 bytes, 20m is equivalent of 20 megabytes and 5g is equivalent of 5 gigabytes.");

    let limit = Arg::with_name("limit")
        .takes_value(true)
        .short("l")
        .long("limit")
        .help("Limit how many files to list")
        .long_help("Only list the first N files found given by this limit");

    let args: ArgMatches = App::new(crate_name!())
        .about("Command line tool for finding large files")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(path)
        .arg(depth)
        .arg(size)
        .arg(limit)
        .get_matches();

    return args
}