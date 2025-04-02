use crate::mapper::sheet_cell::SheetRawCellSerde;
use error_stack::{Report, ResultExt};
use serde_json::Value;
use std::any::type_name;
use thiserror::Error;

pub type SheetRow = Vec<Value>;

pub trait SheetRowSerde {
    fn deserialize(row: SheetRow) -> Result<Self>
    where
        Self: Sized;

    fn serialize(self) -> Result<SheetRow>;
}

pub trait SheetRowExt {
    /// cell_id - 0-based array index
    fn parse_cell<T: SheetRawCellSerde + Default>(
        &self,
        cell_id: usize,
        column_name: &'static str,
    ) -> Result<T>;
}

impl SheetRowExt for SheetRow {
    fn parse_cell<T: SheetRawCellSerde + Default>(
        &self,
        cell_id: usize,
        column_name: &'static str,
    ) -> Result<T> {
        self.get(cell_id).parse_optional_value(self, column_name)
    }
}

pub type Result<T> = error_stack::Result<T, ParseError>;
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Field {0} is not found in row")]
    FieldIsMissing(&'static str),
    #[error("Can't convert {0} into string")]
    JsonValueToStringError(Value),
    #[error("Can't deserialize JSON string into type")]
    JsonStringDeserializationError,
    #[error("Can't deserialize Cell '{column_name}' into type {type_name} from string {input}")]
    CellDeserializationError {
        column_name: &'static str,
        type_name: &'static str,
        input: String,
    },
    #[error("Expected row length {min}-{max}, but it's {actual}")]
    InvalidRowLength {
        min: usize,
        max: usize,
        actual: usize,
    },
}

trait ParseOptionalValue {
    fn parse_optional_value<T>(self, row: &Vec<Value>, field_name: &'static str) -> Result<T>
    where
        T: SheetRawCellSerde + Default;
    // TODO: Make it nicer by bounding receiver (self) to be an Option<Value>

    fn try_unwrap_value<'a>(
        value: Option<&'a Value>,
        row: &Vec<Value>,
        field_name: &'static str,
    ) -> Result<&'a Value> {
        value.ok_or_else(|| {
            Report::new(ParseError::FieldIsMissing(field_name))
                .attach_printable(format!("Input row: {row:?}"))
        })
    }
}

impl ParseOptionalValue for Option<&Value> {
    fn parse_optional_value<T: SheetRawCellSerde + Default>(
        self,
        row: &Vec<Value>,
        field_name: &'static str,
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
                .ok_or_else(|| ParseError::JsonValueToStringError(v.clone()))?
                .to_owned();

            SheetRawCellSerde::deserialize(string.clone().into()).change_context_lazy(|| {
                ParseError::CellDeserializationError {
                    column_name: field_name,
                    type_name,
                    input: string,
                }
            })
        })
    }
}
