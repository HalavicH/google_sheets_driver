
////////////////////////// Letters in A1 notation ///////////////////////////

use std::ops::Add;
use crate::types::conversions::{dec_to_string_as_base26, string_to_dec_as_base26};

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
