use crate::spread_sheet_driver::SpreadSheetDriverError;
use derive_more::Deref;
use derive_more::with_trait::From;
use error_stack::{Context, Report, ResultExt};
use google_sheets4::chrono::{DateTime, Utc};
use serde_json::Value;
use std::any::type_name;
use std::fmt;

pub type Result<T> = error_stack::Result<T, SpreadSheetDriverError>;

pub trait ParseOptionalValue {
    fn parse_optional_value<T>(self, row: &Vec<Value>, field_name: &str) -> Result<T>
    where
        T: CellSerde + Default;
    // TODO: Make it nicer by bounding receiver (self) to be an Option<Value>
    fn try_unwrap_value<'a>(
        value: Option<&'a Value>,
        row: &Vec<Value>,
        field_name: &str,
    ) -> Result<&'a Value> {
        value.ok_or_else(|| {
            Report::new(SpreadSheetDriverError::ParseError(format!(
                "Field {field_name:?} is not present in row: {row:?}"
            )))
        })
    }
}

impl ParseOptionalValue for Option<&Value> {
    fn parse_optional_value<T: CellSerde + Default>(
        self,
        row: &Vec<Value>,
        field_name: &str,
    ) -> Result<T> {
        let type_name = type_name::<T>();
        let result = Self::try_unwrap_value(self, row, field_name);

        if type_name.starts_with("core::option::Option<") {
            return Ok(T::default()); // Returns None
        }

        result.and_then(|v| {
            log::debug!("Parsing {:?} into {}", v, type_name);
            let string = v
                .clone()
                .as_str()
                .ok_or(SpreadSheetDriverError::ParseError(format!(
                    "Expected to convert value to string: {v:?}"
                )))?
                .to_owned();

            CellSerde::deserialize(string.into()).change_context(
                SpreadSheetDriverError::ParseError(format!(
                    "Can't parse {field_name:?} into {type_name} in row: {row:?}"
                )),
            )
        })
    }
}

#[derive(Debug)]
pub struct TryFromCellError;
impl Context for TryFromCellError {}

impl fmt::Display for TryFromCellError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("Could not convert cell to type")
    }
}

pub type TryFromCellResult<T> = error_stack::Result<T, TryFromCellError>;

#[derive(Debug, Deref, From)]
pub struct Cell(String);

pub trait CellSerde {
    fn serialize(&self) -> Cell {
        todo!()
    }
    fn deserialize(cell: Cell) -> TryFromCellResult<Self>
    where
        Self: Sized;
}

impl CellSerde for i32 {
    fn deserialize(cell: Cell) -> TryFromCellResult<Self> {
        cell.parse::<i32>()
            .map_err(Report::new)
            .change_context(TryFromCellError)
    }
}

impl CellSerde for String {
    fn deserialize(cell: Cell) -> TryFromCellResult<Self> {
        Ok(cell.to_string())
    }
}

impl CellSerde for i64 {
    fn deserialize(cell: Cell) -> TryFromCellResult<Self> {
        cell.parse::<i64>()
            .map_err(Report::new)
            .change_context(TryFromCellError)
    }
}

impl<T: CellSerde> CellSerde for Option<T> {
    fn deserialize(cell: Cell) -> TryFromCellResult<Self> {
        Ok(T::deserialize(cell).ok())
    }
}

impl CellSerde for DateTime<Utc> {
    fn deserialize(cell: Cell) -> TryFromCellResult<Self> {
        cell.parse::<DateTime<Utc>>()
            .map_err(Report::new)
            .change_context(TryFromCellError)
    }
}
