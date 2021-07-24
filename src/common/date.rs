use std::ops::Deref;
use std::ops::DerefMut;
use std::str::FromStr;

use chrono::Datelike;
use chrono::offset::Local;
use chrono::format::ParseError;
use chrono::naive::NaiveDate;

/// A naive date in `YYYY-MM-DD` format.
#[derive(Debug, Clone)]
pub struct Date {
    date: NaiveDate,
}

impl Date {
    /// Create a new `Date` from a `chrono` object.
    pub fn new(date: NaiveDate) -> Self {
        Self { date }
    }

    /// Get the year component of the date.
    pub fn year(&self) -> i32 {
        self.date.year()
    }

    /// Get the month component of the date.
    pub fn month(&self) -> u32 {
        self.date.month()
    }

    /// Get the day component of the date.
    pub fn day(&self) -> u32 {
        self.date.day()
    }
}

impl AsRef<NaiveDate> for Date {
    fn as_ref(&self) -> &NaiveDate {
        &self.date
    }
}

impl AsMut<NaiveDate> for Date {
    fn as_mut(&mut self) -> &mut NaiveDate {
        &mut self.date
    }
}

impl Deref for Date {
    type Target = NaiveDate;
    fn deref(&self) -> &NaiveDate {
        &self.date
    }
}

impl DerefMut for Date {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.date
    }
}

impl Default for Date {
    fn default() -> Self {
        Local::now().date().naive_local().into()

    }
}

impl From<NaiveDate> for Date {
    fn from(date: NaiveDate) -> Self {
        Self::new(date)
    }
}

impl From<Date> for NaiveDate {
    fn from(date: Date) -> Self {
        date.date
    }
}

impl FromStr for Date {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").map(Date::new)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    // use chrono::DateLike;

    #[test]
    fn test_from_str() {
        let date = Date::from_str("2012-12-25").unwrap();
        assert_eq!(date.year(), 2012);
        assert_eq!(date.month(), 12);
        assert_eq!(date.day(), 25);
    }
}
