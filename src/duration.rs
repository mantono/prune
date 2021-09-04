use std::time::Duration;

use crate::parse::digest;

const MINUTE: u64 = 60;
const HOUR: u64 = 60 * MINUTE;
const DAY: u64 = 24 * HOUR;
const WEEK: u64 = 7 * DAY;
const YEAR: f64 = 365.25 * (DAY as f64);
const MONTH: f64 = YEAR / 12.0;

pub fn parse_duration(input: &str) -> Result<Duration, &str> {
    let (amount, unit) = digest(input).ok_or("Unable to parse input")?;
    let unit: char = unit.unwrap_or('s');
    let seconds: u64 = match unit {
        's' => amount,
        'm' => amount * MINUTE,
        'h' => amount * HOUR,
        'd' => amount * DAY,
        'w' => amount * WEEK,
        'M' => ((amount as f64) * MONTH) as u64,
        'y' => ((amount as f64) * YEAR) as u64,
        _ => panic!("Invalid unit: {}", unit),
    };
    Ok(Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::parse_duration;

    #[test]
    fn test_seconds_no_suffix() {
        let duration: Result<Duration, &str> = parse_duration("42");
        assert_eq!(Ok(Duration::from_secs(42)), duration);
    }

    #[test]
    fn test_seconds() {
        let duration: Result<Duration, &str> = parse_duration("42s");
        assert_eq!(Ok(Duration::from_secs(42)), duration);
    }

    #[test]
    fn test_minutes() {
        let duration: Result<Duration, &str> = parse_duration("3m");
        assert_eq!(Ok(Duration::from_secs(180)), duration);
    }

    #[test]
    fn test_hours() {
        let duration: Result<Duration, &str> = parse_duration("2h");
        assert_eq!(Ok(Duration::from_secs(7200)), duration);
    }

    #[test]
    fn test_days() {
        let duration: Result<Duration, &str> = parse_duration("3d");
        assert_eq!(Ok(Duration::from_secs(259200)), duration);
    }

    #[test]
    fn test_weeks() {
        let duration: Result<Duration, &str> = parse_duration("2w");
        assert_eq!(Ok(Duration::from_secs(1209600)), duration);
    }

    #[test]
    fn test_months() {
        let duration: Result<Duration, &str> = parse_duration("5M");
        assert_eq!(Ok(Duration::from_secs(13149000)), duration);
    }

    #[test]
    fn test_years() {
        let duration: Result<Duration, &str> = parse_duration("3y");
        assert_eq!(Ok(Duration::from_secs(94672800)), duration);
    }
}
