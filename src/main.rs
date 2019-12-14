#[macro_use]
extern crate clap;
extern crate humansize;
mod find;
mod lib;
mod args;

use find::explore;
use humansize::{FileSize, file_size_opts as options};
use std::fs;
use std::fs::{ReadDir, Metadata};
use std::path::PathBuf;
use clap::{ArgMatches, App, Arg};
use crate::Size::{Byte, Kilobyte, Megabyte, Gigabyte, Terabyte};
use crate::find::find::{explore, ConsumeFile};

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
    let mut limit: u64 = limit as u64;
    let mut printer = FilePrinter;

    args.values_of("path").unwrap().for_each(|p| {
        let current_dir = PathBuf::from(p);
        limit -= explore(current_dir, max_depth, limit, min_size, &mut printer);
    });
}

struct FilePrinter;

impl ConsumeFile for FilePrinter {
    fn submit(&mut self, file: &PathBuf) {
        let canon: PathBuf = file.canonicalize().expect("Unable to get canonical path");
        let size = file.metadata().unwrap().len().file_size(options::CONVENTIONAL).unwrap();
        println!("{}, {:?}", size, canon)
    }
}