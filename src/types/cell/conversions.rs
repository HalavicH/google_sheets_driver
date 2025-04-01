use crate::types::{A1CellId, NumCellId};

///////////////////////// CellId <-> A1CellId conversions /////////////////////////
impl From<A1CellId> for NumCellId {
    fn from(value: A1CellId) -> Self {
        Self {
            col: string_to_dec_as_base26(&value.col) - 1,
            row: value.row.get() - 1,
        }
    }
}

#[cfg(test)]
mod from_a1_cell_id_tests {
    use super::*;

    #[test]
    fn on_a1_cell_id__ok() {
        let a1_cell_id = A1CellId::from_primitives("A", 1);
        let cell_id = NumCellId::from(a1_cell_id);
        assert_eq!(cell_id.col, 0);
        assert_eq!(cell_id.row, 0);
    }

    #[test]
    fn on_b2_cell_id__ok() {
        let a1_cell_id = A1CellId::from_primitives("B", 2);
        let cell_id = NumCellId::from(a1_cell_id);
        assert_eq!(cell_id.col, 1);
        assert_eq!(cell_id.row, 1);
    }

    #[test]
    fn on_z26_cell_id__ok() {
        let a1_cell_id = A1CellId::from_primitives("Z", 26);
        let cell_id = NumCellId::from(a1_cell_id);
        assert_eq!(cell_id.col, 25);
        assert_eq!(cell_id.row, 25);
    }
}

impl From<NumCellId> for A1CellId {
    fn from(value: NumCellId) -> Self {
        Self::from_primitives(
            &dec_to_string_as_base26(value.col + 1),
            value.row + 1,
        )
    }
}

#[cfg(test)]
mod from_cell_id_tests {
    use std::ops::Deref;
    use super::*;

    #[test]
    fn on_cell_id__ok() {
        let cell_id = NumCellId { col: 0, row: 0 };
        let a1_cell_id = A1CellId::from(cell_id);
        assert_eq!(a1_cell_id.col.deref(), "A");
        assert_eq!(a1_cell_id.row.get(), 1);
    }

    #[test]
    fn on_b2_cell_id__ok() {
        let cell_id = NumCellId { col: 1, row: 1 };
        let a1_cell_id = A1CellId::from(cell_id);
        assert_eq!(a1_cell_id.col.deref(), "B");
        assert_eq!(a1_cell_id.row.get(), 2);
    }

    #[test]
    fn on_z26_cell_id__ok() {
        let cell_id = NumCellId { col: 25, row: 25 };
        let a1_cell_id = A1CellId::from(cell_id);
        assert_eq!(a1_cell_id.col.deref(), "Z");
        assert_eq!(a1_cell_id.row.get(), 26);
    }
}


/////////////////////////// Conversion functions ///////////////////////////

/// Convert a string of letters to a decimal number in 1-indexed base-26.
pub fn string_to_dec_as_base26(string: &str) -> u32 {
    let mut result = 0;
    for letter in string.chars() {
        result = result * 26 + (letter as u32 - 'A' as u32 + 1);
    }
    result
}

/// Convert a decimal number to a string of letters in 1-indexed base-26.
pub fn dec_to_string_as_base26(mut dec_number: u32) -> String {
    let mut result = String::new();
    while dec_number > 0 {
        dec_number -= 1; // Adjust for 1-based indexing
        let letter = (dec_number % 26) as u8 + b'A';
        result.push(letter as char);
        dec_number /= 26;
    }
    result.chars().rev().collect()
}

#[cfg(test)]
mod base26_tests {
    use super::*;

    #[test]
    fn test_string_to_dec_as_base26() {
        assert_eq!(string_to_dec_as_base26("A"), 1);
        assert_eq!(string_to_dec_as_base26("B"), 2);
        assert_eq!(string_to_dec_as_base26("Z"), 26);
        assert_eq!(string_to_dec_as_base26("AA"), 27);
        assert_eq!(string_to_dec_as_base26("AB"), 28);
        assert_eq!(string_to_dec_as_base26("BA"), 53);
        assert_eq!(string_to_dec_as_base26("AAA"), 703);
    }

    #[test]
    fn test_dec_to_string_as_base26() {
        assert_eq!(dec_to_string_as_base26(1), "A");
        assert_eq!(dec_to_string_as_base26(2), "B");
        assert_eq!(dec_to_string_as_base26(26), "Z");
        assert_eq!(dec_to_string_as_base26(27), "AA");
        assert_eq!(dec_to_string_as_base26(28), "AB");
        assert_eq!(dec_to_string_as_base26(53), "BA");
        assert_eq!(dec_to_string_as_base26(703), "AAA");
    }
}
