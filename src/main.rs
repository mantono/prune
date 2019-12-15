#[macro_use]
extern crate clap;
extern crate humansize;
mod find;
mod args;

use humansize::{FileSize, file_size_opts as options};
use std::path::PathBuf;
use crate::find::{explore, ConsumeFile};
use crate::args::Size;

fn main() {
    let args = args::args();
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