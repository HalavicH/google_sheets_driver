
/// Defines a cell id as 0-indexed 2D coordinates
#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub struct NumCellId {
    pub col: u32,
    pub row: u32,
}

impl NumCellId {
    pub fn new(col: u32, row: u32) -> Self {
        Self { col, row }
    }
}

#[cfg(test)]
mod num_cell_id_tests {
    use super::*;

}
