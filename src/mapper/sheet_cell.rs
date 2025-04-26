use crate::types::{Letters, SpreadSheetDateTime};
use derive_more::Deref;
use derive_more::with_trait::From;
use error_stack::{Context, Report, ResultExt};
use google_sheets4::chrono::{DateTime, NaiveDate, Utc};
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

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
        panic!("Serialization is not supported by default. You need explicitly opt in for it")
    }
    fn deserialize(cell: SheetRawCell) -> CellSerdeResult<Self>
    where
        Self: Sized;
}
/// Standard library types
impl SheetRawCellSerde for String {
    fn deserialize(cell: SheetRawCell) -> CellSerdeResult<Self> {
        Ok(cell.to_string())
    }
}

macro_rules! impl_sheet_raw_cell_serde {
    ($($type:ty), *) => {
        $(
            impl SheetRawCellSerde for $type {
                fn deserialize(cell: SheetRawCell) -> CellSerdeResult<Self> {
                    cell.parse::<Self>()
                        .map_err(Report::new)
                        .change_context(CellParsingError)
                        .attach_printable_lazy(||format!("Input: {:?}", cell))
                }
            }
        )*
    };
}

impl_sheet_raw_cell_serde!(
    i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64, bool
);

/// Own types

impl SheetRawCellSerde for Letters {
    fn deserialize(cell: SheetRawCell) -> CellSerdeResult<Self>
    where
        Self: Sized,
    {
        Letters::try_from(cell.deref().to_owned()).change_context(CellParsingError)
    }
}

/// Third party types
impl SheetRawCellSerde for DateTime<Utc> {
    fn deserialize(cell: SheetRawCell) -> CellSerdeResult<Self> {
        cell.parse::<DateTime<Utc>>()
            .map_err(Report::new)
            .change_context(CellParsingError)
    }
}

impl SheetRawCellSerde for NaiveDate {
    fn deserialize(cell: SheetRawCell) -> CellSerdeResult<Self>
    where
        Self: Sized,
    {
        NaiveDate::from_str(&cell).change_context(CellParsingError)
    }
}

impl SheetRawCellSerde for SpreadSheetDateTime {
    fn deserialize(cell: SheetRawCell) -> CellSerdeResult<Self>
    where
        Self: Sized,
    {
        let val = SheetRawCellSerde::deserialize(cell)?;
        let date = SpreadSheetDateTime::from_raw(val)
            .ok_or(CellParsingError)
            .attach_printable_lazy(|| format!("Date number {} is out of range", val))?;
        Ok(date)
    }
}
