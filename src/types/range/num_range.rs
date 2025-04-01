/// Re-exporting conversion functions
use crate::types::NumCellId;

/// Defines a 0-indexed range in 2D coordinates
/// Both start and end are inclusive
/// Example: Range { start: CellId { col: 0, row: 0 }, end: CellId { col: 1, row: 1 } }
/// Start must be less or equal to end
#[derive(Debug, PartialEq, Clone, Eq)]
pub struct NumRange {
    pub start: NumCellId,
    pub end: NumCellId,
}

impl NumRange {
    pub fn new(start: NumCellId, end: NumCellId) -> Self {
        assert!(
            start.col <= end.col,
            "Start column must be less or equal to end column"
        );
        assert!(
            start.row <= end.row,
            "Start row must be less or equal to end row"
        );
        Self { start, end }
    }
}

#[cfg(test)]
mod range_tests {
    use super::*;

    #[test]
    fn new__on_valid_range__ok() {
        let start = NumCellId::from_primitives(0, 0);
        let end = NumCellId::from_primitives(1, 1);
        let range = NumRange::new(start, end);
        assert_eq!(range.start, start);
        assert_eq!(range.end, end);
    }

    #[test]
    #[should_panic(expected = "Start column must be less or equal to end column")]
    fn new__on_invalid_range__panic() {
        let start = NumCellId::from_primitives(1, 0);
        let end = NumCellId::from_primitives(0, 1);
        NumRange::new(start, end);
    }
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct SheetRange {
    /// The sheet name where range is located. If None, the range is located in the default sheet
    pub sheet: Option<String>,
    /// The range in 2D coordinates
    pub range: NumRange,
}
