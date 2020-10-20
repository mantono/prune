use crate::args::Size;
use clap::ArgMatches;
use regex::Regex;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug)]
pub struct Config {
    pub paths: Vec<PathBuf>,
    pub min_size: u64,
    pub max_depth: u32,
    pub limit: usize,
    pub pattern: Option<Regex>,
    pub max_age: Option<Duration>,
    pub verbosity_level: u8,
    pub only_local_fs: bool,
    pub print_dbg: bool
}

impl Config {
    pub fn from_args(args: ArgMatches) -> Config {
        let min_size: Size = Size::from_arg(args.value_of("size").unwrap());
        let min_size: u64 = min_size.as_bytes();
        let max_depth: u32 = args
            .value_of("depth")
            .unwrap_or(&std::u32::MAX.to_string())
            .parse()
            .unwrap();
        let limit: usize = args
            .value_of("limit")
            .unwrap_or(&std::u64::MAX.to_string())
            .parse()
            .unwrap();
        let paths: Vec<PathBuf> = args
            .values_of("path")
            .unwrap()
            .map(|v| PathBuf::from(v).canonicalize().unwrap())
            .collect();
        let pattern: Option<Regex> = args
            .value_of("pattern")
            .map(|p| Regex::from_str(p).expect("Unable to parse regex"));
        let max_age: Option<Duration> = parse_duration(args.value_of("mod_time"));
        let verbosity_level: u8 = args.value_of("verbosity").unwrap().parse::<u8>().unwrap();
        let only_local_fs: bool = args.is_present("filesystem");
        let print_dbg: bool = args.is_present("debug");

        Config {
            paths,
            min_size,
            max_depth,
            limit,
            pattern,
            max_age,
            verbosity_level,
            only_local_fs,
            print_dbg
        }
    }
}

const MINUTE: u64 = 60;
const HOUR: u64 = 60 * MINUTE;
const DAY: u64 = 24 * HOUR;
const WEEK: u64 = 7 * DAY;
const YEAR: f64 = 365.25 * (DAY as f64);
const MONTH: f64 = YEAR / 12.0;

fn parse_duration(input: Option<&str>) -> Option<Duration> {
    let last_char: char = input?.chars().last()?;
    if last_char.is_ascii_digit() {
        let amount: u64 = input.unwrap().parse().unwrap();
        return Some(Duration::from_secs(amount))
    }
    let input: &str = input.unwrap();
    let last_index: usize = input.len() - 1;
    let amount: u64 = (&input[0..last_index]).parse().unwrap();
    let seconds: u64 = match last_char {
        's' => amount,
        'm' => amount * MINUTE,
        'h' => amount * HOUR,
        'd' => amount * DAY,
        'w' => amount * WEEK,
        'M' => ((amount as f64) * MONTH) as u64,
        'y' => ((amount as f64) * YEAR) as u64,
        _ => panic!(format!("Invalid unit: {}", last_char))
    };
    Some(Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests {
    use crate::cfg::parse_duration;
    use std::time::Duration;

    #[test]
    fn test_seconds_no_suffix() {
        let duration: Option<Duration> = parse_duration(Some("42"));
        assert_eq!(Some(Duration::from_secs(42)), duration);
    }

    #[test]
    fn test_seconds() {
        let duration: Option<Duration> = parse_duration(Some("42s"));
        assert_eq!(Some(Duration::from_secs(42)), duration);
    }

    #[test]
    fn test_minutes() {
        let duration: Option<Duration> = parse_duration(Some("3m"));
        assert_eq!(Some(Duration::from_secs(180)), duration);
    }

    #[test]
    fn test_hours() {
        let duration: Option<Duration> = parse_duration(Some("2h"));
        assert_eq!(Some(Duration::from_secs(7200)), duration);
    }

    #[test]
    fn test_days() {
        let duration: Option<Duration> = parse_duration(Some("3d"));
        assert_eq!(Some(Duration::from_secs(259200)), duration);
    }

    #[test]
    fn test_weeks() {
        let duration: Option<Duration> = parse_duration(Some("2w"));
        assert_eq!(Some(Duration::from_secs(1209600)), duration);
    }

    #[test]
    fn test_months() {
        let duration: Option<Duration> = parse_duration(Some("5M"));
        assert_eq!(Some(Duration::from_secs(13149000)), duration);
    }

    #[test]
    fn test_years() {
        let duration: Option<Duration> = parse_duration(Some("3y"));
        assert_eq!(Some(Duration::from_secs(94672800)), duration);
    }

    #[test]
    fn test_none() {
        assert_eq!(None, parse_duration(None));
    }
}