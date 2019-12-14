#[macro_use]
extern crate clap;
extern crate humansize;

use humansize::{FileSize, file_size_opts as options};
use std::fs;
use std::fs::{ReadDir, Metadata};
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

    let current_dir = PathBuf::from(".");
    explore(current_dir, max_depth, limit, min_size);
}

fn explore(current_dir: PathBuf, rem_depth: u32, find: u64, min_size: u64) -> u64 {
    let path: ReadDir = match read_dirs(&current_dir) {
        Err(e) => {
            eprintln!("{}: {:?}", e, current_dir);
            return 0
        },
        Ok(p) => p
    };
    let (files, dirs) = path.filter_map(|p| p.ok())
        .map(|p| p.path())
        .filter(|p: &PathBuf| is_valid_target(p))
        .partition(|p| p.is_file());

    let files: Vec<PathBuf> = files;

    let found: usize = files.iter()
        .filter(|f: &&PathBuf| {
            let meta = f.metadata().expect("Unable to read metadata");
            meta.len() > min_size
        })
        .take(find as usize)
        .map(|f| f.canonicalize().expect("Unable to get canonical path"))
        .inspect(|f| print_file(f))
        .count();

    let mut remaining: u64 = find - found as u64;

    if rem_depth > 0 && remaining > 0 {
        dirs.iter()
            .for_each(|p| remaining -= explore(p.clone(), rem_depth - 1, remaining, min_size));
    }

    find - remaining
}

fn read_dirs(path: &PathBuf) -> Result<ReadDir, std::io::Error> {
    let full_path: PathBuf = path.canonicalize()?;
    Ok(fs::read_dir(full_path)?)
}

fn is_valid_target(path: &PathBuf) -> bool {
    let symlink: bool = path.symlink_metadata().unwrap().file_type().is_symlink();
    if !symlink {
        let metadata: Metadata = path.metadata().expect("Unable to retrieve metadata:");
        metadata.is_file() || metadata.is_dir()
    } else {
        false
    }
}

fn print_file(file: &PathBuf) {
    let canon: PathBuf = file.canonicalize().expect("Unable to get canonical path");
    let size = file.metadata().unwrap().len().file_size(options::CONVENTIONAL).unwrap();
    println!("{}, {:?}", size, canon)
}

pub fn args<'a>() -> ArgMatches<'a> {
    let path = Arg::with_name("path")
        .default_value(".")
        .takes_value(true)
        .required(false)
        .multiple(true)
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
        .required(false)
        .help("Minimum file size")
        .long_help("Only show files which exceeds this file size. For example 400 is equivalent of 400 bytes, 20m is equivalent of 20 megabytes and 5g is equivalent of 5 gigabytes.");

    let limit = Arg::with_name("limit")
        .takes_value(true)
        .short("l")
        .long("limit")
        .help("Limit how many files to list")
        .long_help("Only list the first N files found given by this limit. If no value is set for this option, the application will not stop until it has gone through all files in the directory.");

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