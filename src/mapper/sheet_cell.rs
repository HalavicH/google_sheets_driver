use derive_more::Deref;
use derive_more::with_trait::From;
use error_stack::{Context, Report, ResultExt};
use google_sheets4::chrono::{DateTime, Utc};
use std::fmt;

#[derive(Debug)]
pub struct CellParsingError;
impl Context for CellParsingError {}

impl fmt::Display for CellParsingError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("Could not convert cell to type")
    }
}

pub type CellSerdeResult<T> = error_stack::Result<T, CellParsingError>;

#[derive(Debug, Deref, From)]
pub struct SheetRawCell(String);

pub trait SheetRawCellSerde {
    fn serialize(&self) -> SheetRawCell {
        todo!()
    }
    fn deserialize(cell: SheetRawCell) -> CellSerdeResult<Self>
    where
        Self: Sized;
}

// TODO: Implement for all standard primitives
impl SheetRawCellSerde for i32 {
    fn deserialize(cell: SheetRawCell) -> CellSerdeResult<Self> {
        cell.parse::<i32>()
            .map_err(Report::new)
            .change_context(CellParsingError)
            .attach_printable_lazy(|| format!("Source data: {cell:?}"))
    }
}

impl SheetRawCellSerde for f32 {
    fn deserialize(cell: SheetRawCell) -> CellSerdeResult<Self> {
        cell.parse::<f32>()
            .map_err(Report::new)
            .change_context(CellParsingError)
            .attach_printable_lazy(|| format!("Source data: {cell:?}"))
    }
}

impl SheetRawCellSerde for u32 {
    fn deserialize(cell: SheetRawCell) -> CellSerdeResult<Self> {
        cell.parse::<u32>()
            .map_err(Report::new)
            .change_context(CellParsingError)
    }
}

impl SheetRawCellSerde for String {
    fn deserialize(cell: SheetRawCell) -> CellSerdeResult<Self> {
        Ok(cell.to_string())
    }
}

impl SheetRawCellSerde for i64 {
    fn deserialize(cell: SheetRawCell) -> CellSerdeResult<Self> {
        cell.parse::<i64>()
            .map_err(Report::new)
            .change_context(CellParsingError)
    }
}

impl SheetRawCellSerde for DateTime<Utc> {
    fn deserialize(cell: SheetRawCell) -> CellSerdeResult<Self> {
        cell.parse::<DateTime<Utc>>()
            .map_err(Report::new)
            .change_context(CellParsingError)
    }
}
