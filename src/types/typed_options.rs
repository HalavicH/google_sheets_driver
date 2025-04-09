//////////////////////// Typed options ////////////////////////
// TODO: Use derive_more to reduce boilerplate

use derive_more::{Display, FromStr};

#[derive(Debug, Display, Clone, FromStr)]
pub enum MajorDimension {
    /// Resulting Vec<Vec<_>> vector will represent rows
    #[display("ROWS")]
    Rows,
    /// Resulting Vec<Vec<_>> vector will represent columns
    #[display("COLUMNS")]
    Columns,
}

impl MajorDimension {
    pub fn as_str(&self) -> &'static str {
        match self {
            MajorDimension::Rows => "ROWS",
            MajorDimension::Columns => "COLUMNS",
        }
    }
}

#[derive(Debug, Display, Clone, FromStr)]
pub enum InputMode {
    /// Will add ' before the numeric operations to avoid Google Sheets to interpret them as formulas
    #[display("RAW")]
    Raw,
    /// Will add the data as if it was entered by the user from the UI
    #[display("USER_ENTERED")]
    UserEntered,
}

impl InputMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            InputMode::Raw => "RAW",
            InputMode::UserEntered => "USER_ENTERED",
        }
    }
}

#[derive(Debug, Display, Clone, FromStr)]
pub enum ValueRenderOption {
    /// The values will be calculated
    #[display("FORMATTED_VALUE")]
    FormattedValue,
    /// The values will be the raw values
    #[display("UNFORMATTED_VALUE")]
    UnformattedValue,
    /// The values will be the formulas
    #[display("FORMULA")]
    Formula,
}

impl ValueRenderOption {
    pub fn as_str(&self) -> &'static str {
        match self {
            ValueRenderOption::FormattedValue => "FORMATTED_VALUE",
            ValueRenderOption::UnformattedValue => "UNFORMATTED_VALUE",
            ValueRenderOption::Formula => "FORMULA",
        }
    }
}

pub type SheetId = String;
