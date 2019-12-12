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
            .last()
            .unwrap_or('b')
            .to_lowercase()
            .to_string();

        let size: u64 = arg[0..arg.len()-1].parse().expect("Unable to parse size");
        println!("{}, {}", char, size);
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
    let paths: ReadDir = fs::read_dir("./").unwrap();
    paths.filter_map(|p| p.ok())
        .map(|p| p.path())
        .filter(|p| p.is_file())
        .filter(|f: &PathBuf| {
            let meta = f.metadata().expect("Unable to read metadata");
            meta.len() > min_size
        })
        .map(|f| f.canonicalize().expect("Unable to get canonical path"))
        .for_each(|f| println!("{:?}", f));
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

    let args: ArgMatches = App::new(crate_name!())
        .about("Command line tool for finding large files")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(path)
        .arg(depth)
        .arg(size)
        .get_matches();

    return args
}