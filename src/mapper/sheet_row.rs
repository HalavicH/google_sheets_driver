use crate::{mapper::sheet_cell::SheetRawCellSerde, types::SheetA1CellId};
use error_stack::{Report, ResultExt};
use serde_json::Value;
use std::any::type_name;
use std::ops::Add;
use thiserror::Error;
use tracing::debug;

pub type Result<T> = error_stack::Result<T, ParseError>;

#[derive(Debug, Clone, Error)]
pub enum ParseError {
    #[error("Field {0} is not found in row")]
    FieldIsMissing(&'static str),
    #[error("Can't convert {0} into string")]
    JsonValueToStringError(Value),
    #[error("Can't deserialize JSON string into type")]
    JsonStringDeserializationError,
    #[error(
        "Can't deserialize column '{column_name}' at {location} into type '{type_name}' from string '{input}'"
    )]
    CellDeserializationError {
        location: SheetA1CellId,
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

#[derive(Debug)]
pub struct PositionedSheetRow {
    pub data: Vec<Value>,
    pub start_cell: SheetA1CellId,
}

pub trait SheetRowSerde {
    fn deserialize(row: PositionedSheetRow) -> Result<Self>
    where
        Self: Sized;

    fn serialize(&self) -> Result<Vec<Value>>;
}

impl PositionedSheetRow {
    /// cell_id - 0-based array index
    pub fn parse_cell<T: SheetRawCellSerde>(
        &self,
        cell_id: usize,
        column_name: &'static str,
    ) -> Result<T> {
        let cell = self.data.get(cell_id);

        let type_name = type_name::<T>();
        let result = try_unwrap_value(cell, &self.data, column_name);

        result.and_then(|v| {
            debug!("Parsing {:?} into {}", v, type_name);
            let string = stringify_json_value(v);

            SheetRawCellSerde::deserialize(string.clone().into()).change_context_lazy(|| {
                ParseError::CellDeserializationError {
                    location: self
                        .start_cell
                        .clone()
                        .with_col(self.start_cell.cell.col.clone().add(cell_id as u32)),
                    column_name,
                    type_name,
                    input: string,
                }
            })
        })
    }
}

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

fn stringify_json_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Array(_) => panic!("Array is not supported by this crappy implementation"),
        _ => value.to_string(),
    }
}
