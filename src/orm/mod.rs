use crate::spread_sheet_driver::SharedSpreadSheetDriver;
use crate::types::{A1CellId, A1Range, Entity, EntityEssentials, SheetA1CellId, SheetA1Range};
use error_stack::{FutureExt, ResultExt, bail};
use google_sheets4::api::{AppendValuesResponse, MatchedValueRange};
use google_sheets4::hyper::body::HttpBody;
use std::num::NonZero;
use std::sync::Arc;
use tracing::{debug, info};

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error["Spreadsheet Driver error"]]
    DriverError,
    #[error["Invalid argument: {0}"]]
    InvalidArgument(String),
    #[error["Parsing error"]]
    ParsingError,
    #[error["Unexpected response: {what}. {input}.\nResponse: {response:?}"]]
    UnexpectedResponse {
        what: &'static str,
        input: String,
        response: Box<AppendValuesResponse>,
    },
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
    pub async fn find_in_range<E>(&self, start: &SheetA1CellId, rows: u32) -> Result<Vec<Entity<E>>>
    where
        E: EntityEssentials,
    {
        let range = Self::convert_into_range(start, rows, E::entity_width());
        let matched_value_range = self
            .driver
            .lock()
            .await
            .try_get_range(&range)
            .await
            .change_context(RepositoryError::DriverError)?;

        matched_value_range.parse_positionally()
    }

    // TODO: Fix possible bug with `rows: 1` producing range of 2 rows because of 1-based indexing
    fn convert_into_range(start: &SheetA1CellId, rows: u32, width: u32) -> SheetA1Range {
        // -2 for 1-based offset twice (first time here, second time when calculating end_cell
        let compensation = 2;
        let offset = A1CellId::new(
            start.cell.col.clone() + width - compensation,
            NonZero::new(rows).expect("Expected to have rows to be at least 1"),
        );
        let end_cell = start.cell.clone() + offset;
        let range = SheetA1Range::new(
            start.sheet_name.to_string(),
            A1Range::new(start.cell.clone(), end_cell),
        );
        range
    }

    pub async fn find_by_position<E>(&self, start: SheetA1CellId) -> Result<Option<Entity<E>>>
    where
        E: EntityEssentials,
    {
        let vec = self.find_in_range(&start, 1).await?;
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

        debug!("Updating entity\n{:#?}\nas raw data:{:#?}", entity, data);

        self.driver
            .lock()
            .await
            .try_write_range(range.to_string().as_str(), data)
            .await
            .change_context(RepositoryError::DriverError)?;
        Ok(())
    }

    /// Inserts entity into specified table by appending it to the end of the range.
    pub async fn insert<E>(
        &self,
        start: SheetA1CellId,
        rows: u32,
        entity_data: E,
    ) -> Result<Entity<E>>
    where
        E: EntityEssentials,
    {
        let range = Self::convert_into_range(&start, rows, E::entity_width());

        let data = entity_data
            .clone()
            .serialize()
            .change_context(RepositoryError::DriverError)?;

        let avr = self
            .driver
            .lock()
            .await
            .try_append_row(range.to_string().as_str(), data)
            .await
            .change_context(RepositoryError::DriverError)?;

        info!(
            "For input range: {:?}, data: {:?}\nGot response: {:#?}",
            range, entity_data, avr
        );

        let Some(updates) = &avr.updates else {
            bail!(RepositoryError::UnexpectedResponse {
                what: "AppendValuesResponse doesn't have 'updates'",
                input: format!("Input range: {:?}, data: {:?}", range, entity_data),
                response: Box::new(avr)
            });
        };

        let Some(updated_range) = &updates.updated_range else {
            bail!(RepositoryError::UnexpectedResponse {
                what: "UpdateValuesResponse doesn't have 'updated_range'",
                input: format!("Input range: {:?}, data: {:?}", range, updates),
                response: Box::new(avr)
            });
        };

        let position =
            SheetA1Range::from_raw(updated_range).change_context(RepositoryError::ParsingError)?;
        Ok(Entity {
            position: position.start(),
            data: entity_data,
        })
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
                        data: data,
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

        let sr = SheetA1Range::from_raw(range.as_str())
            .map_err(|e| RepositoryError::InvalidArgument(format!("{e}")))?;

        Ok(sr)
    }
}

#[cfg(test)]
mod orm_tests {
    use super::*;

    use crate::mapper::sheet_row;
    use crate::mapper::sheet_row::{SheetRow, SheetRowExt, SheetRowSerde};
    use google_sheets4::api::{DataFilter, ValueRange};
    use serde_json::Value;
    use std::fmt::Debug;

    #[derive(Debug, Clone, PartialEq)]
    struct User {
        id: i32,
        name: String,
    }

    impl SheetRowSerde for User {
        fn deserialize(row: SheetRow) -> sheet_row::Result<Self>
        where
            Self: Sized,
        {
            Ok(Self {
                id: row.parse_cell(0, "id")?,
                name: row.parse_cell(1, "name")?,
            })
        }
        fn serialize(&self) -> sheet_row::Result<SheetRow> {
            Ok(vec![
                Value::String(self.name.clone()),
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
            println!("{:?}", result);
            assert!(result.is_ok());

            let actual = result.expect("Test: Expected to parse MatchedValueRange");
            assert_eq!(actual.len(), 3);

            let expected = vec![
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
                },
            ];

            println!("{:#?}", expected);
            assert_eq!(actual, expected)
        }
    }
}
