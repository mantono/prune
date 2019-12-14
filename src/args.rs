use clap::{ArgMatches, App, Arg};

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
        .min_values(1)
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
        .min_values(1)
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