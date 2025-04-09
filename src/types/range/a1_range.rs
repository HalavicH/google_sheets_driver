use crate::types::letters::Letters;
use crate::types::{A1CellId, SheetA1CellId};
use error_stack::{ResultExt, bail};
use std::fmt::Display;
use std::num::NonZero;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum A1RangeError {
    #[error("Invalid range format: {0}")]
    InvalidRangeFormat(String),
    #[error("Can't parse cell")]
    CellParsingError,
}

pub type Result<T> = error_stack::Result<T, A1RangeError>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct A1Range {
    pub start: A1CellId,
    pub end: A1CellId,
}

impl A1Range {
    pub fn iter(&self) -> A1RangeIterator {
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

#[allow(non_snake_case)]
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
    pub fn into_zero_base_range(self) -> A1Range {
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
        let start = A1CellId::from_raw(from)
            .change_context(A1RangeError::CellParsingError)
            .attach_printable_lazy(|| format!("Input cell (from): {}", from))?;

        let end = A1CellId::from_raw(to)
            .change_context(A1RangeError::CellParsingError)
            .attach_printable_lazy(|| format!("Input cell (to): {}", to))?;

        Ok(Self::new(start, end))
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.start.to_string(), self.end.to_string())
    }
}

impl A1Range {
    fn from_raw<S>(value: S) -> Result<Self>
    where
        S: Display,
    {
        let string = value.to_string();
        let parts = string.split(':').collect::<Vec<_>>();

        if parts.len() != 2 {
            bail!(A1RangeError::InvalidRangeFormat(value.to_string()));
        }

        let from = A1CellId::from_raw(parts[0])
            .change_context(A1RangeError::CellParsingError)
            .attach_printable_lazy(|| format!("Input range str: {}", string))?;

        let to = A1CellId::from_raw(parts[1])
            .change_context(A1RangeError::CellParsingError)
            .attach_printable_lazy(|| format!("Input range str: {}", string))?;

        Ok(Self::new(from, to))
    }
}

#[allow(non_snake_case)]
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
        assert_eq!(*range.current_context(), A1RangeError::CellParsingError);
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
    pub(crate) fn start(&self) -> SheetA1CellId {
        SheetA1CellId::new(self.sheet.clone(), self.range.start.clone())
    }
}

impl SheetA1Range {
    pub fn from_raw<S>(value: S) -> Result<Self>
    where
        S: Display,
    {
        let string = value.to_string();
        let parts = string.split('!').collect::<Vec<_>>();

        if parts.len() != 2 {
            bail!(A1RangeError::InvalidRangeFormat(value.to_string()));
        }

        // Remove leading and traling ' from the page
        let page = parts[0].trim_matches('\'');
        let range = A1Range::from_raw(parts[1])?;

        Ok(Self::new(page.to_string(), range))
    }
}

impl SheetA1Range {
    pub fn new<N>(page: N, range: A1Range) -> Self
    where
        N: Display,
    {
        Self {
            sheet: page.to_string(),
            range,
        }
    }

    pub fn from_str(page: &str, range: &str) -> Result<Self> {
        Ok(Self::new(page.to_string(), A1Range::from_raw(range)?))
    }
}

impl Display for SheetA1Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!("{}!{}", self.sheet, self.range.to_string())
        )
    }
}
