use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref NUM: Regex = Regex::new(r"^\d+").unwrap();
    static ref CHR: Regex = Regex::new(r"[a-zA-Z]$").unwrap();
}

pub fn digest(input: &str) -> Option<(u64, Option<char>)> {
    let num: u64 = NUM.find(input)?.as_str().parse().ok()?;
    let chr: Option<char> = CHR.find(input).map(|i| i.as_str().chars().next()).flatten();
    Some((num, chr))
}
