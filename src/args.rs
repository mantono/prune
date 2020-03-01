use crate::args::Size::{Byte, Gigabyte, Kilobyte, Megabyte, Terabyte};
use clap::{App, Arg, ArgMatches};

pub fn args<'a>() -> ArgMatches<'a> {
    let path = Arg::with_name("path")
        .default_value(".")
        .takes_value(true)
        .required(false)
        .multiple(true)
        .help("Paths to look for files in")
        .long_help("Select zero, one or several directories for which to look for files in. If no value is give, the application will default to current directory.");

    let depth = Arg::with_name("depth")
        .takes_value(true)
        .short("d")
        .long("depth")
        .required(false)
        .help("Depth in folder hierarchy")
        .long_help("Descend and search for files in directories with a max depth of this value. A depth of 0 will only look for files at the first level. By default the depth is unlimited.");

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

    let pattern = Arg::with_name("pattern")
        .takes_value(true)
        .short("p")
        .long("pattern")
        .multiple(false)
        .required(false)
        .help("Filter files by regex pattern")
        .long_help("Only include and count files matching the regular expression.");

    let limit = Arg::with_name("limit")
        .takes_value(true)
        .short("l")
        .long("limit")
        .help("Limit how many files to list")
        .long_help("Only list the first N files found given by this limit. If no value is set for this option, the application will not stop until it has gone through all files in the directory and subdirectories.");

    let verbosity = Arg::with_name("verbosity")
        .takes_value(true)
        .default_value("1")
        .validator(|n: String| {
            let range = 0u8..=5u8;
            let n: u8 = n.parse::<u8>().unwrap();
            if range.contains(&n) {
                Ok(())
            } else {
                Err("Invalid value".to_string())
            }
        })
        .short("v")
        .long("verbosity")
        .help("Set verbosity level, 0 - 5")
        .long_help("Set the verbosity level, from 0 (least amount of output) to 5 (most verbose). Note that logging level configured via RUST_LOG overrides this setting.");

    let args: ArgMatches = App::new(crate_name!())
        .about("Command line tool for finding large files")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(path)
        .arg(depth)
        .arg(size)
        .arg(pattern)
        .arg(limit)
        .arg(verbosity)
        .get_matches();

    args
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

fn number_from_size(size: &str) -> Result<u64, String> {
    let regex = Regex::new(r"^\d+").unwrap();
    let match_str: &str = match regex.find(&size) {
        Some(m) => m.as_str(),
        None => return Err(String::from("No match for size"))
    };
    let number: u64 = match match_str.parse() {
        Ok(n) => n,
        Err(_) => return Err(String::from("Unable to parse int"))
    };
    Ok(number)
}

pub enum Size {
    Byte(u64),
    Kilobyte(u64),
    Megabyte(u64),
    Gigabyte(u64),
    Terabyte(u64),
}

impl Size {
    pub fn from_arg(arg: &str) -> Size {
        let char: String = arg
            .chars()
            .filter(|c| c.is_alphabetic())
            .next()
            .unwrap_or('b')
            .to_lowercase()
            .to_string();

        let size: u64 = number_from_size(&arg.to_string()).unwrap();
        match char.as_ref() {
            "b" => Byte(size),
            "k" => Kilobyte(size),
            "m" => Megabyte(size),
            "g" => Gigabyte(size),
            "t" => Terabyte(size),
            _ => panic!("Invalid size type '{}'", char),
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
    use crate::args::{validate_size, number_from_size};

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

    #[test]
    fn number_from_size_triple_digit_kilobytes() {
        let size: u64 = number_from_size(&String::from("100k")).expect("Expected a number");
        assert_eq!(100, size);
    }

    #[test]
    fn number_from_size_triple_digit_implicit_byte() {
        let size: u64 = number_from_size(&String::from("100")).expect("Expected a number");
        assert_eq!(100, size);
    }

    #[test]
    fn number_from_size_single_digit_implicit_byte() {
        let size: u64 = number_from_size(&String::from("5")).expect("Expected a number");
        assert_eq!(5, size);
    }
}
