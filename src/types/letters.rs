/////////////////////////// Letters in A1 notation ///////////////////////////

use crate::types::cell::conversions::{dec_to_string_as_base26, string_to_dec_as_base26};
use derive_more::{Deref, Display};
use error_stack::{FutureExt, Report, bail};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};
use thiserror::Error;

#[derive(Debug, Display, Error)]
pub enum LettersError {
    NonAlphanumeric(String),
    EmptyString,
}

/// Encapsulates the letters of the alphabet to use it for the cell id
#[derive(Debug, Eq, PartialEq, Hash, Clone, Deref)]
pub struct Letters(String);

impl Letters {
    /// Panics
    pub fn new(value: String) -> Self {
        assert!(!value.is_empty(), "Expected non-empty letters");
        assert!(
            value.chars().all(char::is_alphabetic),
            "Invalid cell column letters: {:?}",
            value
        );
        Self(value)
    }
}

impl Display for Letters {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for Letters {
    type Error = Report<LettersError>;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            bail!(LettersError::EmptyString)
        }

        if !value.chars().all(char::is_alphabetic) {
            let text = format!(
                "String value: {} must contain only alphabetic characters",
                value
            );
            return Err(Report::new(LettersError::NonAlphanumeric(value)).attach_printable(text));
        }
        Ok(Self::new(value.to_uppercase()))
    }
}

impl Add<u32> for Letters {
    type Output = Letters;

    fn add(self, delta: u32) -> Self::Output {
        let dec_number = string_to_dec_as_base26(&self);
        let result = dec_number + delta;
        let value = dec_to_string_as_base26(result);
        Letters::new(value)
    }
}

impl PartialOrd for Letters {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_number = string_to_dec_as_base26(self);
        let other_number = string_to_dec_as_base26(other);

        self_number.partial_cmp(&other_number)
    }
}

impl Sub<u32> for Letters {
    type Output = Letters;

    fn sub(self, delta: u32) -> Self::Output {
        let dec_number = string_to_dec_as_base26(&self);
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
        let mut letters = self.chars().rev();
        let mut other_letters = other.chars().rev();
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

#[allow(non_snake_case)]
#[cfg(test)]
mod letters_tests {
    use super::*;
    use std::ops::Deref;

    #[test]
    fn letters__new__ok() {
        let letters = Letters::new("A".to_string());
        assert_eq!(letters.deref(), "A");
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
        assert_eq!(result.deref(), "B");
    }

    #[test]
    fn letters__add__with_overflow__ok() {
        let letters = Letters::new("Z".to_string());
        let result = letters + 1;
        assert_eq!(result.deref(), "AA");
    }

    #[test]
    fn letters__add__with_overflow_and_carry__ok() {
        let letters = Letters::new("Z".to_string());
        let result = letters + 2;
        assert_eq!(result.deref(), "AB");
    }

    #[test]
    fn letters__sub__ok() {
        let letters = Letters::new("B".to_string());
        let result = letters - 1;
        assert_eq!(result.deref(), "A");
    }

    #[test]
    fn letters__sub__with_overflow__ok() {
        let letters = Letters::new("AA".to_string());
        let result = letters - 1;
        assert_eq!(result.deref(), "Z");
    }

    #[test]
    fn letters__sub__with_overflow_and_carry__ok() {
        let letters = Letters::new("AA".to_string());
        let result = letters - 2;
        assert_eq!(result.deref(), "Y");
    }

    #[test]
    #[should_panic(expected = "Expected non-empty letters")]
    fn letters__sub__with_underflow__panics() {
        let letters = Letters::new("A".to_string());
        let _ = letters - 1;
    }
}
