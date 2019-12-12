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
        let char = arg.chars().take_while(|c| c.is_alphabetic()).last().expect("No chars found").to_lowercase();
        let size = arg.chars().take_while(|c| c.is_ascii_digit())
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

const ONE_HUNDRED_KILOBYTE: u64 = 100 * 1024;

fn main() {
    let args = args();
    let paths: ReadDir = fs::read_dir("./").unwrap();
    paths.filter_map(|p| p.ok())
        .map(|p| p.path())
        .filter(|p| p.is_file())
        //.filter_map(|f| f.metadata().ok())
        .filter(|f: &PathBuf| {
            let meta = f.metadata().expect("Unable to read metadata");
            meta.len() > ONE_HUNDRED_KILOBYTE
        })
        .map(|f| f.canonicalize().expect("Unable to canonicalize path"))
        .for_each(|f| println!("Name: {:?}", f));
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
        .short("-d")
        .long("--depth")
        .required(false)
        .help("Depth in folder hierarchy")
        .long_help("Descend and search for files in directories with a max depth of this value. A depth of 0 will only look for files at the first level.");

    let limit = Arg::with_name("limit")
        .default_value("100m")
        .takes_value(true)
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
        .arg(limit)
        .get_matches();

    return args
}