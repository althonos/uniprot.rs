use std::str::FromStr;

use crate::error::InvalidValue;

/// A naive date in `YYYY-MM-DD` format.
#[derive(Debug, Default, Clone)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl FromStr for Date {
    type Err = InvalidValue;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let b = s.as_bytes();
        if b.len() == 10 && b[4] == b'-' && b[7] == b'-' {
            let year = atoi::atoi(&b[..4]).ok_or_else(|| InvalidValue(String::from(s)))?;
            let month = atoi::atoi(&b[5..7]).ok_or_else(|| InvalidValue(String::from(s)))?;
            let day = atoi::atoi(&b[8..]).ok_or_else(|| InvalidValue(String::from(s)))?;
            return Ok(Self { year, month, day });
        }

        Err(InvalidValue(String::from(s)))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_from_str() {
        let date = Date::from_str("2012-12-25").unwrap();
        assert_eq!(date.year, 2012);
        assert_eq!(date.month, 12);
        assert_eq!(date.day, 25);
    }
}
