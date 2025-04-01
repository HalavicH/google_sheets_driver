/// Defines a cell id as 0-indexed 2D coordinates
#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub struct NumCellId {
    pub col: u32,
    pub row: u32,
}

impl NumCellId {
    pub fn from_primitives(col: u32, row: u32) -> Self {
        Self { col, row }
    }
}

#[cfg(test)]
mod num_cell_id_tests {
    use super::*;

    #[test]
    fn given_valid_primitives_when_created_then_correct() {
        let cell_id = NumCellId::from_primitives(1, 1);
        assert_eq!(1, cell_id.col);
        assert_eq!(1, cell_id.row);
    }
}
