use crate::types::range::a1_range::A1Range;
use crate::types::range::CellRange;

impl From<CellRange> for A1Range {
    fn from(value: CellRange) -> Self {
        let start = value.start.into();
        let end = value.end.into();
        Self::new(start, end)
    }
}

impl From<A1Range> for CellRange {
    fn from(value: A1Range) -> Self {
        let start = value.start.into();
        let end = value.end.into();
        Self::new(start, end)
    }
}

#[cfg(test)]
mod range_tests {
    use crate::types::CellId;
    use super::*;

    #[test]
    fn from_range__on_valid_range__ok() {
        let start = CellId::new(0, 0);
        let end = CellId::new(1, 1);
        let range = CellRange::new(start, end);
        let a1_range = A1Range::from(range);
        assert_eq!(a1_range.start.to_string(), "A1");
        assert_eq!(a1_range.end.to_string(), "B2");
    }

    #[test]
    fn from_a1_range__on_valid_range__ok() {
        let a1_range = A1Range::from_str("A1", "B2").unwrap();
        let range = CellRange::from(a1_range);
        assert_eq!(range.start, CellId::new(0, 0));
        assert_eq!(range.end, CellId::new(1, 1));
    }
}
