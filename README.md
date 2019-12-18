# prune
Find large files on your disk

## Usage
```
USAGE:
    prn [OPTIONS] [path]...

FLAGS:
    -h, --help       
            Prints help information

    -V, --version    
            Prints version information


OPTIONS:
    -d, --depth <depth>    
            Descend and search for files in directories with a max depth of this value. A depth of 0 will only look for
            files at the first level. By default the depth is unlimited.
    -l, --limit <limit>    
            Only list the first N files found given by this limit. If no value is set for this option, the application
            will not stop until it has gone through all files in the directory and subdirectories.
    -s, --size <size>      
            Only show files which exceeds this file size. For example 400 is equivalent of 400 bytes, 20m is equivalent
            of 20 megabytes and 5g is equivalent of 5 gigabytes. [default: 100m]

ARGS:
    <path>...    
            Select zero, one or several directories for which to look for files in. If no value is give, the application
            will default to current directory [default: .]
```

#### Example
The following command will look for all files being 300 megabyte or larger (`-s 300m`), in the current directory and up to five directory levels
below (`-d 5`) stopping when ten files (`-l 10`) have been found.

`prn -s 300m -d 5 -l 10`

Symlinks will never be followed, as this could potentially result in infinite loops when traversing through directories.

## Building
The application is built with [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html). Simply run the following command in the project directory.
```bash
cargo build --release
```
A binary will be created and put in directory `target/release`. 

## Install
Run `cargo install --path .`
