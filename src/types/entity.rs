use crate::spread_sheet_driver::SheetRowSerde;
use crate::types::SheetA1CellId;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

/// Position aware object which knows its position on the spreadsheet
#[derive(Debug, Clone, PartialEq)]
pub struct Entity<E>
where
    E: EntityEssentials,
{
    pub position: SheetA1CellId,
    pub data: E,
}

/// Syntactic sugar to ease work with the wrapped data
impl<E: EntityEssentials> Deref for Entity<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<E: EntityEssentials> DerefMut for Entity<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

pub trait EntityEssentials: Sized + Debug + SheetRowSerde + Clone + PartialEq {
    /// Returns width in columns of the entity
    fn entity_width() -> u32;
}
