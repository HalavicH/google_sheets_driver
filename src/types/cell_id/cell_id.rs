
/// Defines a cell id as 0-indexed 2D coordinates
#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub struct CellId {
    pub col: u32,
    pub row: u32,
}

impl CellId {
    pub fn new(col: u32, row: u32) -> Self {
        Self { col, row }
    }
}

#[cfg(test)]
mod cell_id_tests {
    use super::*;

}
