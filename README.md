# prune
Find large files on your disk

![Build & Test](https://github.com/mantono/prune/workflows/Build%20&%20Test/badge.svg)

## Usage
```
USAGE:
    prn [FLAGS] [OPTIONS] [path]...

FLAGS:
    -D, --debug
            Print debug information about current build for binary, useful for when an issue is encountered and reported

    -R, --dirs
            Search for directories instead of files

    -x, --filesystem
            Only search for files in the same filesystem for the given path(s), or the current file system if no path is
            given.
    -h, --help
            Prints help information

    -P, --plumbing
            Use plumbing mode (as opposed to 'porcelain' mode) with an output that is more consistent and machine
            readable
    -V, --version
            Prints version information


OPTIONS:
    -d, --depth <depth>
            Descend and search for files or directories in directories with a max depth of this value. A depth of 0 will
            only look for files at the first level. By default the depth is unlimited.
    -l, --limit <limit>
            Only list the first N files found given by this limit. If no value is set for this option, the application
            will not stop until it has gone through all files in the directory and subdirectories.
    -M, --max-mod-time <max-age>
            Filter based on max mod time

            Only include files which modification time is equal to or less than this. Such as `180s` for 180 seconds,
            `45d` for 45 days and `3y` for 3 years.
    -m, --min-mod-time <min-age>
            Filter based on min mod time

            Only include files which modification time is equal to or more than this. Such as `180s` for 180 seconds,
            `45d` for 45 days and `3y` for 3 years.
    -p, --pattern <pattern>
            Only include and count files matching the regular expression.

    -s, --size <size>
            Only show files or directories which exceeds this size. For example 400 is equivalent of 400 bytes, 20m is
            equivalent of 20 megabytes and 5g is equivalent of 5 gigabytes. [default: 100m]
    -v, --verbosity <verbosity>
            Set the verbosity level, from 0 (least amount of output) to 5 (most verbose). Note that logging level
            configured via RUST_LOG overrides this setting. [default: 1]

ARGS:
    <path>...
            Select zero, one or several directories for which to look for files in. If no value is give, the application
            will default to current directory. [default: .]
```

#### Example
The following command will look for all files being 300 megabytes or larger (`-s 300m`), in the current directory and up to five directory levels
below (`-d 5`) stopping when ten files (`-l 10`) have been found and only search for files on the local filesystem (`-x`).

`prn -s 300m -d 5 -l 10 -x`

This could also be written as

`prn --size 300m --depth 5 --limit 10 --filesystem`

Symlinks will never be followed, as this could potentially result in infinite loops when traversing through directories.

## Building
The application is built with [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html). Simply run the following command in the project directory.
```bash
cargo build --release
```
A binary will be created and put in directory `target/release`. 

## Install
Run `cargo install --path .`
