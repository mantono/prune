use clap::{ArgMatches, App, Arg};
use crate::args::Size::{Byte, Kilobyte, Megabyte, Gigabyte, Terabyte};

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
        .validator(validate_size)
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

use regex::Regex;

fn validate_size(size: String) -> Result<(), String> {
    let regex = Regex::new(r"^\d+[bkmgtBKMGT]?$").unwrap();
    if regex.is_match(size.as_str()) {
        Ok(())
    } else {
        let error: String = format!("Input is not a valid size: '{}'", size);
        Err(error)
    }
}

pub enum Size {
    Byte(u64),
    Kilobyte(u64),
    Megabyte(u64),
    Gigabyte(u64),
    Terabyte(u64)
}

impl Size {

    pub fn from_arg(arg: &str) -> Size {
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

    pub fn as_bytes(&self) -> u64 {
        match self {
            &Byte(n) => n,
            &Kilobyte(n) => 1024 * n,
            &Megabyte(n) => 1024 * 1024 * n,
            &Gigabyte(n) => 1024 * 1024 * 1024 * n,
            &Terabyte(n) => 1024 * 1024 * 1024 * 1024 * n,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::args::validate_size;

    #[test]
    fn validate_size_bytes() {
        assert!(validate_size(String::from("5")).is_ok());
        assert!(validate_size(String::from("5b")).is_ok());
        assert!(validate_size(String::from("5B")).is_ok());
    }

    #[test]
    fn validate_size_kilobytes() {
        assert!(validate_size(String::from("5k")).is_ok());
        assert!(validate_size(String::from("5K")).is_ok());
    }

    #[test]
    fn validate_size_megabytes() {
        assert!(validate_size(String::from("5m")).is_ok());
        assert!(validate_size(String::from("5M")).is_ok());
    }

    #[test]
    fn validate_size_gigabytes() {
        assert!(validate_size(String::from("5g")).is_ok());
        assert!(validate_size(String::from("5G")).is_ok());
    }

    #[test]
    fn validate_size_terabytes() {
        assert!(validate_size(String::from("5t")).is_ok());
        assert!(validate_size(String::from("5T")).is_ok());
    }

    #[test]
    fn validate_size_fail_negative() {
        assert!(validate_size(String::from("-5b")).is_err());
        assert!(validate_size(String::from("-5")).is_err());
    }

    #[test]
    fn validate_size_fail_invalid_unit() {
        assert!(validate_size(String::from("5j")).is_err());
    }
}