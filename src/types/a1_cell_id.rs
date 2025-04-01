use std::cmp::Ordering;
use std::num::{NonZero, NonZeroU32};
use std::ops::{Add, Sub};
use error_stack::{bail, IntoReportCompat, Report};
use crate::types::cell_id::CellId;
use crate::types::conversions::{dec_to_string_as_base26, string_to_dec_as_base26};

pub type Result<T> = error_stack::Result<T, A1CellIdError>;

#[derive(Debug, Clone, thiserror::Error)]
pub enum A1CellIdError {
    #[error("Invalid cell format: {0}")]
    InvalidCellFormat(String)
}

/// Defines a cell id in A1 notation.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct A1CellId {
    pub sheet_name: Option<String>,
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
            sheet_name: None,
            col: letter,
            row: number,
        }
    }
    pub fn from_primitives(letter: &str, number: u32) -> Self {
        Self {
            sheet_name: None,
            col: Letters::new(letter.to_string()),
            row: NonZero::new(number).expect("Expected a non-zero cell row number"),
        }
    }

    pub fn with_sheet_name(self, sheet_name: &str) -> Self {
        Self {
            sheet_name: Some(sheet_name.to_owned()),
            ..self
        }
    }

    /// Convert the cell id to a 1-indexed row and column indices
    pub fn as_indices(&self) -> CellId {
        CellId {
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
            sheet_name: todo!("Parse sheet name"),
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

/////////////////////////// Letters in A1 notation ///////////////////////////

/// Encapsulates the letters of the alphabet to use it for the cell id
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Letters {
    pub value: String,
}

impl Letters {
    pub fn new(value: String) -> Self {
        assert!(!value.is_empty(), "Expected non-empty letters");
        assert!(
            value.chars().all(char::is_alphabetic),
            "Invalid cell column letters: {:?}",
            value
        );
        Self { value }
    }
}

impl From<&str> for Letters {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

impl Add<u32> for Letters {
    type Output = Letters;

    fn add(self, delta: u32) -> Self::Output {
        let dec_number = string_to_dec_as_base26(&self.value);
        let result = dec_number + delta;
        let value = dec_to_string_as_base26(result);
        Letters::new(value)
    }
}

impl PartialOrd for Letters {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_number = string_to_dec_as_base26(&self.value);
        let other_number = string_to_dec_as_base26(&other.value);

        self_number.partial_cmp(&other_number)
    }
}

impl Sub<u32> for Letters {
    type Output = Letters;

    fn sub(self, delta: u32) -> Self::Output {
        let dec_number = string_to_dec_as_base26(&self.value);
        let result = dec_number - delta;
        let value = dec_to_string_as_base26(result);
        Letters::new(value)
    }
}
impl Sub<&Letters> for Letters {
    type Output = i32;

    fn sub(self, rhs: &Letters) -> Self::Output {
        &self - rhs
    }
}

impl Sub<&Letters> for &Letters {
    type Output = i32;

    fn sub(self, other: &Letters) -> Self::Output {
        let mut letters = self.value.chars().rev();
        let mut other_letters = other.value.chars().rev();
        let mut result = 0;
        let mut carry = 0;

        while let (Some(letter), Some(other_letter)) = (letters.next(), other_letters.next()) {
            let letter = letter as i32 - 'A' as i32;
            let other_letter = other_letter as i32 - 'A' as i32;

            let value = letter - other_letter - carry;

            if value < 0 {
                carry = 1;
                result += 26 + value;
            } else {
                carry = 0;
                result += value;
            }
        }

        result
    }
}

#[cfg(test)]
mod letters_tests {
    use super::*;

    #[test]
    fn letters__new__ok() {
        let letters = Letters::new("A".to_string());
        assert_eq!(letters.value, "A");
    }

    #[test]
    #[should_panic(expected = "Invalid cell column letters: \"1\"")]
    fn letters__new__panics_on_invalid_letters() {
        Letters::new("1".to_string());
    }

    #[test]
    fn letters__add__ok() {
        let letters = Letters::new("A".to_string());
        let result = letters + 1;
        assert_eq!(result.value, "B");
    }

    #[test]
    fn letters__add__with_overflow__ok() {
        let letters = Letters::new("Z".to_string());
        let result = letters + 1;
        assert_eq!(result.value, "AA");
    }

    #[test]
    fn letters__add__with_overflow_and_carry__ok() {
        let letters = Letters::new("Z".to_string());
        let result = letters + 2;
        assert_eq!(result.value, "AB");
    }

    #[test]
    fn letters__sub__ok() {
        let letters = Letters::new("B".to_string());
        let result = letters - 1;
        assert_eq!(result.value, "A");
    }

    #[test]
    fn letters__sub__with_overflow__ok() {
        let letters = Letters::new("AA".to_string());
        let result = letters - 1;
        assert_eq!(result.value, "Z");
    }

    #[test]
    fn letters__sub__with_overflow_and_carry__ok() {
        let letters = Letters::new("AA".to_string());
        let result = letters - 2;
        assert_eq!(result.value, "Y");
    }

    #[test]
    #[should_panic(expected = "Expected non-empty letters")]
    fn letters__sub__with_underflow__panics() {
        let letters = Letters::new("A".to_string());
        let result = letters - 1;
    }
}
