use std::fmt::Debug;
use std::ops::Deref;
use crate::types::a1_cell_id::A1CellId;

/// Position aware object which knows its position on the spreadsheet
pub struct Entity<E> where E: EntityEssentials {
    pub position: A1CellId,
    pub data: E
}

/// Syntactic sugar to ease work with the wrapped data
impl<E: EntityEssentials> Deref for Entity<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub trait EntityEssentials where Self: Sized + Debug {
    /// Returns width in columns of the entity
    fn entity_width() -> usize;
}
