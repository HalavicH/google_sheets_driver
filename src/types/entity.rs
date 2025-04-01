use std::fmt::Debug;
use crate::types::a1_cell_id::A1CellId;

/// Position aware object which knows its position on the spreadsheet
pub struct Entity<E> where E: Debug {
    position: A1CellId,
    data: E
}

