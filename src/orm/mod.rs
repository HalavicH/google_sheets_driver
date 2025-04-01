use crate::spread_sheet_driver::{SharedSpreadSheetDriver, SpreadSheetDriverError};
use crate::types::{A1CellId, Entity, EntityEssentials};

#[derive(Debug, thiserror::Error, derive_more::Display)]
pub enum RepositoryError {
    DriverError(SpreadSheetDriverError)
}

pub type Result<T> = error_stack::Result<T, RepositoryError>;

pub struct Repository {
    pub driver: SharedSpreadSheetDriver,
}

impl Repository {
    pub fn find_in_range<E>(&self, start: A1CellId, rows: usize) -> Result<Vec<Entity<E>>> where E: EntityEssentials {
        // let range = start.
        // self.driver.lock().await
        //     .read_rows_deserialized()
        todo!()
    }

    pub fn find_one<E>(&self, position: A1CellId) -> Result<Option<Entity<E>>> where E: EntityEssentials{
        todo!()
    }

    pub fn update<E>(&self, entity: &Entity<E>) -> Result<()> where E: EntityEssentials{
        todo!()
    }

    pub fn insert<E>(&self, entity_data: &E) -> Result<Entity<E>> where E: EntityEssentials{
        todo!()
    }

    pub fn delete<E>(&self, entity: &Entity<E>) -> Result<()> where E: EntityEssentials{
        todo!("Brainstorm on how to delete entities properly")
    }
}
