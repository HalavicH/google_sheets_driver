use crate::spread_sheet_driver::{SharedSpreadSheetDriver, SpreadSheetDriverError};
use crate::types::{A1CellId, A1Range, Entity, EntityEssentials, SheetA1CellId, SheetA1Range};
use error_stack::{bail, ResultExt};
use google_sheets4::api::MatchedValueRange;
use google_sheets4::hyper::body::HttpBody;

#[derive(Debug, thiserror::Error, derive_more::Display)]
pub enum RepositoryError {
    DriverError,
    InvalidArgument(String),
    ParsingError,
}

pub type Result<T> = error_stack::Result<T, RepositoryError>;

pub struct Repository {
    pub driver: SharedSpreadSheetDriver,
}

impl Repository {
    pub async fn find_in_range<E>(&self, start: SheetA1CellId, rows: u32) -> Result<Vec<Entity<E>>>
    where
        E: EntityEssentials,
    {
        let offset = A1CellId::new(
            start.cell.col.clone() + E::entity_width(),
            start.cell.row.saturating_add(rows),
        );
        let end_cell = start.cell.clone() + offset;
        let range = SheetA1Range::new(
            start.sheet_name.to_string(),
            A1Range::new(start.cell, end_cell),
        );
        let matched_value_range = self.driver.lock().await
            .try_get_range(&range).await
            .change_context(RepositoryError::DriverError)?;

        matched_value_range.parse_positionally()
    }

    pub async fn find_one<E>(&self, position: A1CellId) -> Result<Option<Entity<E>>>
    where
        E: EntityEssentials,
    {
        todo!()
    }

    pub async fn update<E>(&self, entity: &Entity<E>) -> Result<()>
    where
        E: EntityEssentials,
    {
        todo!()
    }

    pub async fn insert<E>(&self, entity_data: &E) -> Result<Entity<E>>
    where
        E: EntityEssentials,
    {
        todo!()
    }

    pub async fn delete<E>(&self, entity: &Entity<E>) -> Result<()>
    where
        E: EntityEssentials,
    {
        todo!("Brainstorm on how to delete entities properly")
    }
}

pub trait PositionalParsing {
    fn parse_positionally<E>(self) -> Result<Vec<Entity<E>>>
    where
        E: EntityEssentials;
    fn extract_range_from_filters(&self) -> Result<SheetA1Range>;
}
impl PositionalParsing for MatchedValueRange {
    fn parse_positionally<E>(self) -> Result<Vec<Entity<E>>>
    where
        E: EntityEssentials,
    {
        let sr = self.extract_range_from_filters().map_err(|e| e)?;
        let start = sr.range.start;

        let data = self
            .value_range
            .expect("Expected to get range")
            .values
            .unwrap_or_default();

        let data: Result<Vec<Entity<E>>> = data
            .into_iter()
            .enumerate()
            .map(|(i, value)| {
                let result: Result<Entity<E>> = E::try_from_row(value)
                    .map(|data| Entity {
                        position: A1CellId::new(
                            start.col.clone(),
                            start.row.saturating_add(i as u32),
                        ),
                        data,
                    })
                    .change_context(RepositoryError::ParsingError);
                result
            })
            .collect();
        Ok(data?)
    }

    fn extract_range_from_filters(&self) -> Result<SheetA1Range> {
        let Some(filters) = self.data_filters.as_ref() else {
            bail!(RepositoryError::InvalidArgument(
                "MatchedValueRange doesn't have data filters".to_string()
            ));
        };

        if filters.len() != 1 {
            bail!(RepositoryError::InvalidArgument(
                "MatchedValueRange doesn't have exactly one filter".to_string()
            ));
        };

        let filter = filters
            .first()
            .expect("Expected to have exactly one filter");

        let Some(range) = filter.a1_range.as_ref() else {
            bail!(RepositoryError::InvalidArgument(
                "Data filter doesn't have A1 range".to_string()
            ));
        };

        let sr = SheetA1Range::try_from(range.as_str())
            .map_err(|e| RepositoryError::InvalidArgument(format!("{e}")))?;

        Ok(sr)
    }
}
