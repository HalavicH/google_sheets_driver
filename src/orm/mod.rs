use crate::spread_sheet_driver::SharedSpreadSheetDriver;
use crate::types::{A1CellId, A1Range, Entity, EntityEssentials, SheetA1CellId, SheetA1Range};
use error_stack::{FutureExt, ResultExt, bail};
use google_sheets4::api::MatchedValueRange;
use google_sheets4::hyper::body::HttpBody;
use std::sync::Arc;

#[derive(Debug, thiserror::Error, derive_more::Display)]
pub enum RepositoryError {
    DriverError,
    InvalidArgument(String),
    ParsingError,
}

pub type Result<T> = error_stack::Result<T, RepositoryError>;

pub type SharedRepository = Arc<Repository>;
pub struct Repository {
    pub driver: SharedSpreadSheetDriver,
}

impl Repository {
    pub fn new(driver: SharedSpreadSheetDriver) -> Self {
        Self { driver }
    }
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
        let matched_value_range = self
            .driver
            .lock()
            .await
            .try_get_range(&range)
            .await
            .change_context(RepositoryError::DriverError)?;

        matched_value_range.parse_positionally()
    }

    pub async fn find_by_position<E>(&self, start: SheetA1CellId) -> Result<Option<Entity<E>>>
    where
        E: EntityEssentials,
    {
        let vec = self.find_in_range(start, 1).await?;
        Ok(vec.first().cloned())
    }

    pub async fn update<E>(&self, entity: &Entity<E>) -> Result<()>
    where
        E: EntityEssentials,
    {
        let new_row = entity.position.cell.row.get() + 1;
        let end_col = entity.position.cell.col.clone() + E::entity_width();
        let range = entity.position.clone().into_range(end_col, new_row);

        let data = vec![
            entity
                .data
                .clone()
                .serialize()
                .change_context(RepositoryError::DriverError)?,
        ];

        self.driver
            .lock()
            .await
            .try_write_range(range.to_string().as_str(), data)
            .await
            .change_context(RepositoryError::DriverError)?;
        Ok(())
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
        let sr = self.extract_range_from_filters()?;
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
                let result: Result<Entity<E>> = E::deserialize(value)
                    .map(|data| Entity {
                        position: SheetA1CellId::from_primitives(
                            &sr.sheet,
                            &start.col,
                            start.row.get() + i as u32,
                        ),
                        data,
                    })
                    .change_context(RepositoryError::ParsingError);
                result
            })
            .collect();
        data
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

#[cfg(test)]
mod orm_tests {
    use super::*;
    use crate::mapper::ParseOptionalValue;
    use crate::spread_sheet_driver::{RawRow, SheetRowSerde, SsdResult};
    use google_sheets4::api::{DataFilter, ValueRange};
    use serde_json::Value;
    use std::fmt::Debug;

    #[derive(Debug, Clone, PartialEq)]
    struct User {
        id: i32,
        name: String,
    }

    impl SheetRowSerde for User {
        fn deserialize(row: RawRow) -> SsdResult<Self>
        where
            Self: Sized,
        {
            Ok(Self {
                id: row.first().parse_optional_value(&row, "id")?,
                name: row.get(1).parse_optional_value(&row, "name")?,
            })
        }
        fn serialize(self) -> SsdResult<RawRow> {
            Ok(vec![
                Value::String(self.name),
                Value::String(self.id.to_string()),
            ])
        }
    }

    impl EntityEssentials for User {
        fn entity_width() -> u32 {
            2
        }
    }

    #[cfg(test)]
    mod positional_parsing_tests {
        use super::*;

        fn get_mocked_query_response() -> MatchedValueRange {
            MatchedValueRange {
                data_filters: Some(vec![DataFilter {
                    a1_range: Some("users!A1:B3".to_string()),
                    ..Default::default()
                }]),
                value_range: Some(ValueRange {
                    values: Some(vec![
                        vec![
                            Value::String("1".to_string()),
                            Value::String("Joe".to_string()),
                        ],
                        vec![
                            Value::String("2".to_string()),
                            Value::String("John".to_string()),
                        ],
                        vec![
                            Value::String("3".to_string()),
                            Value::String("Jane".to_string()),
                        ],
                    ]),
                    ..Default::default()
                }),
            }
        }

        #[test]
        fn given_valid_mvr__when_parse__then_success() {
            let input = get_mocked_query_response();

            let result: Result<Vec<Entity<User>>> = input.parse_positionally();
            assert!(result.is_ok());

            let vec = result.expect("Test: Expected to parse MatchedValueRange");
            assert_eq!(vec.len(), 3);

            assert_eq!(vec, vec![
                Entity {
                    position: SheetA1CellId::from_primitives("users", "A", 1),
                    data: User {
                        id: 1,
                        name: "Joe".to_string(),
                    },
                },
                Entity {
                    position: SheetA1CellId::from_primitives("users", "A", 2),
                    data: User {
                        id: 2,
                        name: "John".to_string(),
                    },
                },
                Entity {
                    position: SheetA1CellId::from_primitives("users", "A", 3),
                    data: User {
                        id: 3,
                        name: "Jane".to_string(),
                    },
                }
            ])
        }
    }
}
