use std::fmt::Display;
use crate::types::A1CellId;
use crate::types::cell::a1_cell_id::A1CellIdError;
use crate::types::letters::Letters;
use std::num::NonZero;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum A1RangeError {
    #[error("Invalid range format: {0}")]
    InvalidRangeFormat(String),
    #[error("Can't parse cell: {0}")]
    CellParsingError(#[from] A1CellIdError),
}

pub type Result<T> = std::result::Result<T, A1RangeError>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct A1Range {
    pub start: A1CellId,
    pub end: A1CellId,
}

impl A1Range {
    pub(crate) fn iter(&self) -> A1RangeIterator {
        A1RangeIterator {
            range: self.clone(),
            current: self.start.clone(),
        }
    }
}

/// Iterator for the range which iterates over the cells
/// from the top left to the bottom right by rows
pub struct A1RangeIterator {
    range: A1Range,
    current: A1CellId,
}

impl Iterator for A1RangeIterator {
    type Item = A1CellId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current > self.range.end {
            return None;
        }

        let result = self.current.clone();
        if self.current.col == self.range.end.col {
            self.current.row =
                NonZero::new(self.current.row.get() + 1).expect("Expected non-zero number");

            self.current.col = self.range.start.col.clone();
        } else {
            self.current = self.current.delta(1, 0);
        }
        Some(result)
    }
}

#[cfg(test)]
mod range_iterator_tests {
    use super::*;

    #[test]
    fn range_iterator__on_single_cell__ok() {
        let range = A1Range::from_str("A1", "A1").unwrap();
        let mut iter = range.iter();
        assert_eq!(iter.next(), Some(A1CellId::from_primitives("A", 1)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn range_iterator__on_single_row__ok() {
        let range = A1Range::from_str("A1", "C1").unwrap();
        let mut iter = range.iter();
        assert_eq!(iter.next(), Some(A1CellId::from_primitives("A", 1)));
        assert_eq!(iter.next(), Some(A1CellId::from_primitives("B", 1)));
        assert_eq!(iter.next(), Some(A1CellId::from_primitives("C", 1)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn range_iterator__on_single_column__ok() {
        let range = A1Range::from_str("A1", "A3").unwrap();
        let mut iter = range.iter();
        assert_eq!(iter.next(), Some(A1CellId::from_primitives("A", 1)));
        assert_eq!(iter.next(), Some(A1CellId::from_primitives("A", 2)));
        assert_eq!(iter.next(), Some(A1CellId::from_primitives("A", 3)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn range_iterator__on_square__ok() {
        let range = A1Range::from_str("A1", "C3").unwrap();
        let mut iter = range.iter();
        assert_eq!(iter.next(), Some(A1CellId::from_primitives("A", 1)));
        assert_eq!(iter.next(), Some(A1CellId::from_primitives("B", 1)));
        assert_eq!(iter.next(), Some(A1CellId::from_primitives("C", 1)));
        assert_eq!(iter.next(), Some(A1CellId::from_primitives("A", 2)));
        assert_eq!(iter.next(), Some(A1CellId::from_primitives("B", 2)));
        assert_eq!(iter.next(), Some(A1CellId::from_primitives("C", 2)));
        assert_eq!(iter.next(), Some(A1CellId::from_primitives("A", 3)));
        assert_eq!(iter.next(), Some(A1CellId::from_primitives("B", 3)));
        assert_eq!(iter.next(), Some(A1CellId::from_primitives("C", 3)));
        assert_eq!(iter.next(), None);
    }
}

impl A1Range {
    /// Offset the range to the A1 as `from`
    pub(crate) fn into_zero_base_range(self) -> A1Range {
        let delta_numbers = 1 - self.start.row.get() as i32;
        let minus_letters = -(&self.start.col - &Letters::new("A".to_string()));

        A1Range {
            start: A1CellId::from_primitives("A", 1),
            end: self.end.delta(minus_letters, delta_numbers),
        }
    }
}

impl A1Range {
    pub fn new(from: A1CellId, to: A1CellId) -> Self {
        Self {
            start: from,
            end: to,
        }
    }

    pub fn from_str(from: &str, to: &str) -> Result<Self> {
        let start = from.try_into().map_err(Into::<A1RangeError>::into)?;
        let end = to.try_into().map_err(Into::<A1RangeError>::into)?;
        Ok(Self::new(start, end))
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.start.to_string(), self.end.to_string())
    }
}

impl TryFrom<&str> for A1Range {
    type Error = A1RangeError;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let mut parts = value.split(':');

        let from = parts
            .next()
            .ok_or_else(|| A1RangeError::InvalidRangeFormat(value.to_string()))?
            .try_into()?;

        let to = parts
            .next()
            .ok_or_else(|| A1RangeError::InvalidRangeFormat(value.to_string()))?
            .try_into()?;

        Ok(Self::new(from, to))
    }
}

#[cfg(test)]
mod range_tests {
    use super::*;

    #[test]
    fn parse_range__on_valid_range__ok() {
        let range = A1Range::from_str("A1", "C3").unwrap();
        assert_eq!(range.start.to_string(), "A1");
        assert_eq!(range.end.to_string(), "C3");
    }

    #[test]
    fn parse_range__on_invalid_range__err() {
        let range = A1Range::from_str("A1", "C").unwrap_err();
        assert_eq!(
            range,
            A1RangeError::CellParsingError(A1CellIdError::InvalidCellFormat("C".to_string()))
        );
    }

    #[test]
    fn range__to_string__ok() {
        let range = A1Range::from_str("A1", "C3").unwrap();
        assert_eq!(range.to_string(), "A1:C3");
    }

    #[test]
    fn range__into_zero_base_range__already_zero_base__ok() {
        let range = A1Range::from_str("A1", "C3").unwrap();
        let zero_base = range.into_zero_base_range();
        assert_eq!(zero_base.start.to_string(), "A1");
        assert_eq!(zero_base.end.to_string(), "C3");
    }

    #[test]
    fn range__into_zero_base_range__not_zero_base__ok() {
        let range = A1Range::from_str("B2", "D4").unwrap();
        let zero_base = range.into_zero_base_range();
        assert_eq!(zero_base.start.to_string(), "A1");
        assert_eq!(zero_base.end.to_string(), "C3");
    }
}

#[derive(Debug, Clone)]
pub struct SheetA1Range {
    pub sheet: String,
    pub range: A1Range,
}

impl SheetA1Range {
    pub fn new(page: String, range: A1Range) -> Self {
        Self { sheet: page, range }
    }

    pub fn from_str(page: &str, range: &str) -> Result<Self> {
        Ok(Self::new(page.to_string(), range.try_into()?))
    }
}

impl Display for SheetA1Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{}!{}", self.sheet, self.range.to_string()))
    }
}

impl TryFrom<&str> for SheetA1Range {
    type Error = A1RangeError;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let mut parts = value.split('!');

        let page = parts
            .next()
            .ok_or_else(|| A1RangeError::InvalidRangeFormat(value.to_string()))?;

        // Remove leading and traling ' from the page
        let page = page.trim_matches('\'');

        let range = parts
            .next()
            .ok_or_else(|| A1RangeError::InvalidRangeFormat(value.to_string()))?
            .try_into()?;

        Ok(Self::new(page.to_string(), range))
    }
}
