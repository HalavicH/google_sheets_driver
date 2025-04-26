use derive_more::Deref;
use google_sheets4::chrono::{Duration, NaiveDate};

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct SpreadSheetDateTime {
    date: NaiveDate,
}

impl SpreadSheetDateTime {
    /// Base date: December 30, 1899
    const BASE_DATE: NaiveDate =
        NaiveDate::from_ymd_opt(1899, 12, 30).expect("Expected valid base SpreadSheetDateTime");

    /// Create from f64
    pub fn from_raw(value: f64) -> Option<Self> {
        let days = value.floor() as i64;
        let date = Self::BASE_DATE.checked_add_signed(Duration::days(days))?;
        Some(Self { date })
    }

    /// Convert back to f64
    pub fn to_raw(&self) -> f64 {
        (self.date - Self::BASE_DATE).num_days() as f64
    }

    pub fn date(&self) -> &NaiveDate {
        &self.date
    }
}
