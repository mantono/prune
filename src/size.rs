use crate::size::Size::Gigabyte;
use crate::size::Size::Kilobyte;
use crate::size::Size::Megabyte;
use crate::size::Size::Terabyte;
use crate::{parse::digest, size::Size::Byte};
use std::{convert::TryFrom, str::FromStr};

#[derive(Debug)]
pub enum Size {
    Byte(u64),
    Kilobyte(u64),
    Megabyte(u64),
    Gigabyte(u64),
    Terabyte(u64),
}

impl Size {
    pub fn as_bytes(&self) -> u64 {
        match *self {
            Byte(n) => n,
            Kilobyte(n) => 1024 * n,
            Megabyte(n) => 1024 * 1024 * n,
            Gigabyte(n) => 1024 * 1024 * 1024 * n,
            Terabyte(n) => 1024 * 1024 * 1024 * 1024 * n,
        }
    }
}

impl FromStr for Size {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (size, chr) = digest(value).ok_or("Unparsable")?;
        let chr: char = match chr {
            Some(c) => c
                .to_lowercase()
                .next()
                .ok_or("Unable to convert char to lower case")?,
            None => return Ok(Byte(size)),
        };

        let size: Size = match &chr {
            'b' => Byte(size),
            'k' => Kilobyte(size),
            'm' => Megabyte(size),
            'g' => Gigabyte(size),
            't' => Terabyte(size),
            _ => return Err("Invalid size type"),
        };

        Ok(size)
    }
}

impl TryFrom<&str> for Size {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::Size;

    #[test]
    fn validate_size_bytes() {
        assert!(Size::try_from("5").is_ok());
        assert!(Size::try_from("5b").is_ok());
        assert!(Size::try_from("5B").is_ok());
    }

    #[test]
    fn validate_size_kilobytes() {
        assert!(Size::try_from("5k").is_ok());
        assert!(Size::try_from("5K").is_ok());
    }

    #[test]
    fn validate_size_megabytes() {
        assert!(Size::try_from("5m").is_ok());
        assert!(Size::try_from("5M").is_ok());
    }

    #[test]
    fn validate_size_gigabytes() {
        assert!(Size::try_from("5g").is_ok());
        assert!(Size::try_from("5G").is_ok());
    }

    #[test]
    fn validate_size_terabytes() {
        assert!(Size::try_from("5t").is_ok());
        assert!(Size::try_from("5T").is_ok());
    }

    #[test]
    fn validate_size_fail_negative() {
        assert!(Size::try_from("-5b").is_err());
        assert!(Size::try_from("-5").is_err());
    }

    #[test]
    fn validate_size_fail_invalid_unit() {
        assert!(Size::try_from("5j").is_err());
    }

    #[test]
    fn number_from_size_triple_digit_kilobytes() {
        let size: u64 = Size::try_from("100k")
            .expect("Expected a number")
            .as_bytes();
        assert_eq!(102400, size);
    }

    #[test]
    fn number_from_size_triple_digit_implicit_byte() {
        let size: u64 = Size::try_from("100").expect("Expected a number").as_bytes();
        assert_eq!(100, size);
    }

    #[test]
    fn number_from_size_single_digit_implicit_byte() {
        let size: u64 = Size::try_from("5").expect("Expected a number").as_bytes();
        assert_eq!(5, size);
    }
}
