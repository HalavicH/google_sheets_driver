use error_stack::{Context, Report, ResultExt};
use google_sheets4::chrono::{DateTime, FixedOffset};
use serde_json::Value;
use std::any::type_name;
use std::fmt;
use crate::spread_sheet_driver::SpreadSheetDriverError;

pub type ParseResult<T> = error_stack::Result<T, SpreadSheetDriverError>;

pub trait ParseOptionalValue {
    fn parse_optional_value<T>(self, row: &Vec<Value>, field_name: &str) -> ParseResult<T>
    where
        T: TryFromCell + Default;
    // TODO: Make it nicer by bounding receiver (self) to be an Option<Value>
    fn try_unwrap_value<'a>(
        value: Option<&'a Value>,
        row: &Vec<Value>,
        field_name: &str,
    ) -> ParseResult<&'a Value> {
        value.ok_or_else(|| {
            Report::new(SpreadSheetDriverError::ParseError(format!(
                "Field {field_name:?} is not present in row: {row:?}"
            )))
        })
    }
}

impl ParseOptionalValue for Option<&Value> {
    fn parse_optional_value<T: TryFromCell + Default>(
        self,
        row: &Vec<Value>,
        field_name: &str,
    ) -> ParseResult<T> {
        let type_name = type_name::<T>();
        let result = Self::try_unwrap_value(self, row, field_name);

        // TODO: If 'T' Is Option<?> -> return None
        if type_name.starts_with("core::option::Option<") {
            return Ok(T::default());
        }

        result.and_then(|v| {
            let str = v.as_str().unwrap_or_else(|| panic!("Can't convert {v:?}"));
            log::debug!("Parsing {:?} into {}", str, type_name);
            TryFromCell::try_from_cell(str).change_context(SpreadSheetDriverError::ParseError(
                format!("Can't parse {field_name:?} into {type_name} in row: {row:?}"),
            ))
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

pub trait TryFromCell {
    fn try_from_cell(cell: &str) -> TryFromCellResult<Self>
    where
        Self: Sized;
}

impl TryFromCell for i32 {
    fn try_from_cell(cell: &str) -> TryFromCellResult<Self> {
        cell.parse::<i32>()
            .map_err(Report::new)
            .change_context(TryFromCellError)
    }
}

impl TryFromCell for String {
    fn try_from_cell(cell: &str) -> TryFromCellResult<Self> {
        Ok(cell.to_string())
    }
}

impl TryFromCell for i64 {
    fn try_from_cell(cell: &str) -> TryFromCellResult<Self> {
        cell.parse::<i64>()
            .map_err(Report::new)
            .change_context(TryFromCellError)
    }
}

impl<T: TryFromCell> TryFromCell for Option<T> {
    fn try_from_cell(cell: &str) -> TryFromCellResult<Self> {
        Ok(T::try_from_cell(cell).ok())
    }
}

impl TryFromCell for DateTime<FixedOffset> {
    fn try_from_cell(cell: &str) -> TryFromCellResult<Self> {
        DateTime::parse_from_rfc3339(cell)
            .map_err(Report::new)
            .change_context(TryFromCellError)
    }
}
