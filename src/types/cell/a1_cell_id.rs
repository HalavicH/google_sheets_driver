use crate::types::cell::num_cell_id::NumCellId;
use crate::types::cell::conversions::string_to_dec_as_base26;
use crate::types::letters::Letters;
use error_stack::IntoReportCompat;
use std::cmp::Ordering;
use std::num::{NonZero, NonZeroU32};
use std::ops::{Add, Sub};


pub type Result<T> = error_stack::Result<T, A1CellIdError>;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SheetA1CellId {
    pub sheet_name: String,
    pub cell: A1CellId
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum A1CellIdError {
    #[error("Invalid cell format: {0}")]
    InvalidCellFormat(String)
}

/// Defines a cell id in A1 notation.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct A1CellId {
    pub col: Letters,
    pub row: NonZeroU32,
}

impl Add for A1CellId {
    type Output = A1CellId;

    fn add(self, other: Self) -> Self::Output {
        /// Add two cell ids together
        /// Example: A1 + B2 = C3
        /// Example: A1 + A1 = A2
        let number = self.row.get() + other.row.get() - 1;
        let letter = self.col + string_to_dec_as_base26(&other.col.value) - 1;
        A1CellId::new(
            letter,
            NonZero::new(number).expect("Expected a non-zero cell row number"),
        )
    }
}

impl A1CellId {
    /// Convert the cell id to a 1-indexed row index
    /// Example: A1 -> 1
    /// Example: A2 -> 2
    pub(crate) fn row(&self) -> NonZeroU32 {
        self.row
    }

    /// Convert the cell id to a 1-indexed column index
    /// Example: A1 -> 1
    /// Example: B1 -> 2
    pub(crate) fn column(&self) -> NonZeroU32 {
        NonZero::new(string_to_dec_as_base26(&self.col.value))
            .expect("Expected a non-zero cell column number")
    }
}

impl A1CellId {
    pub fn new(letter: Letters, number: NonZeroU32) -> Self {
        Self {
            col: letter,
            row: number,
        }
    }
    pub fn from_primitives(letter: &str, number: u32) -> Self {
        Self {
            col: Letters::new(letter.to_string()),
            row: NonZero::new(number).expect("Expected a non-zero cell row number"),
        }
    }

    /// Convert the cell id to a 1-indexed row and column indices
    pub fn as_indices(&self) -> NumCellId {
        NumCellId {
            col: string_to_dec_as_base26(&self.col.value),
            row: self.row.get(),
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}{}", self.col.value, self.row)
    }

    pub(crate) fn delta(&self, columns: i32, rows: i32) -> A1CellId {
        let number = self.row.get() as i32 + rows;
        let letter = if columns < 0 {
            self.col.clone() - columns.unsigned_abs()
        } else {
            self.col.clone() + columns as u32
        };

        A1CellId::new(
            letter,
            NonZero::new(number as u32).expect("Expected a non-zero cell row number"),
        )
    }

    fn append_letter(letters: &String, plus: u32) -> String {
        let mut letters = letters.chars();
        let mut result = String::new();
        let mut carry = plus;

        while let Some(letter) = letters.next_back() {
            let mut letter = letter;
            let mut value = letter as u32 - 'A' as u32 + carry;

            if value >= 26 {
                carry = value / 26;
                value %= 26;
                letter = (value as u8 + b'A') as char;
            } else {
                carry = 0;
                letter = (letter as u8 + plus as u8) as char;
            }

            result.push(letter);
        }

        if carry > 0 {
            result.push((carry as u8 + b'A') as char);
        }

        result.chars().rev().collect()
    }
}

impl TryFrom<&str> for A1CellId {
    type Error = A1CellIdError;

    fn try_from(value: &str) -> std::result::Result<A1CellId, A1CellIdError> {
        let mut letter = String::new();
        let mut number = String::new();

        for c in value.chars() {
            if c.is_alphabetic() {
                letter.push(c);
            } else if c.is_numeric() {
                number.push(c);
            } else {
                return Err(A1CellIdError::InvalidCellFormat(value.to_string()));
            }
        }

        if letter.is_empty() || number.is_empty() {
            return Err(A1CellIdError::InvalidCellFormat(value.to_string()));
        }

        Ok(Self {
            col: Letters::new(letter),
            row: number.parse().unwrap(),
        })
    }
}

impl PartialOrd for A1CellId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let Some(x_ord) = self.col.partial_cmp(&other.col) else {
            return None;
        };
        let Some(y_ord) = self.row.partial_cmp(&other.row) else {
            return None;
        };

        Some(y_ord.then(x_ord))
    }
}

#[cfg(test)]
mod a1_cell_id_tests {
    use super::*;

    #[cfg(test)]
    mod cell_creation_tests {
        use super::*;

        #[test]
        fn cell_id__new__ok() {
            let cell_id = A1CellId::from_primitives("A", 1);
            assert_eq!(cell_id.col.value, "A");
            assert_eq!(cell_id.row.get(), 1);
        }

        #[test]
        #[should_panic(expected = "Expected a non-zero cell row number")]
        fn cell_id__new__panics_on_zero_number() {
            A1CellId::from_primitives("A", 0);
        }

        #[test]
        #[should_panic(expected = "Invalid cell column letters: \"1\"")]
        fn cell_id__new__panics_on_invalid_letter() {
            A1CellId::from_primitives("1", 1);
        }

        #[test]
        fn cell_id__to_string__ok() {
            let cell_id = A1CellId::from_primitives("A", 1);
            assert_eq!(cell_id.to_string(), "A1");
        }

        #[test]
        fn cell_id__cell_at__ok() {
            let cell_id = A1CellId::from_primitives("A", 1);
            let result = cell_id.delta(1, 1);
            assert_eq!(result.col.value, "B");
            assert_eq!(result.row.get(), 2);
        }

        #[test]
        fn cell_id__cell_at__with_overflow__ok() {
            let cell_id = A1CellId::from_primitives("Z", 26);
            let result = cell_id.delta(1, 1);
            assert_eq!(result.col.value, "AA");
            assert_eq!(result.row.get(), 27);
        }
    }
    #[cfg(test)]
    mod add_tests {
        use super::*;

        #[test]
        fn cell_id__add__ok() {
            let cell_id = A1CellId::from_primitives("A", 1);
            let other = A1CellId::from_primitives("B", 2);
            let result = cell_id + other;
            assert_eq!(result.col.value, "C");
            assert_eq!(result.row.get(), 3);
        }
    }

    #[cfg(test)]
    mod partial_cmp__tests {
        use super::*;

        #[test]
        fn a1_less_than_b2() {
            let cell_id = A1CellId::from_primitives("A", 1);
            let other = A1CellId::from_primitives("B", 2);
            assert_eq!(cell_id.partial_cmp(&other), Some(Ordering::Less));
        }

        #[test]
        fn b2_greater_than_a1() {
            let cell_id = A1CellId::from_primitives("B", 2);
            let other = A1CellId::from_primitives("A", 1);
            assert_eq!(cell_id.partial_cmp(&other), Some(Ordering::Greater));
        }

        #[test]
        fn a1_equal_to_a1() {
            let cell_id = A1CellId::from_primitives("A", 1);
            let other = A1CellId::from_primitives("A", 1);
            assert_eq!(cell_id.partial_cmp(&other), Some(Ordering::Equal));
        }

        #[test]
        fn a3_greater_than_b2() {
            let cell_id = A1CellId::from_primitives("A", 3);
            let other = A1CellId::from_primitives("B", 2);
            assert_eq!(cell_id.partial_cmp(&other), Some(Ordering::Greater));
        }
    }
}
