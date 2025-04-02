use crate::halavich_utils_helpers::{AMShared, ErrorStackExt};
use error_stack::{bail, report};
use google_sheets4::api::{
    BatchGetValuesByDataFilterRequest, BatchGetValuesByDataFilterResponse, DataFilter, ValueRange,
};
use google_sheets4::hyper::client::HttpConnector;
use google_sheets4::hyper::{Body, Response};
use google_sheets4::hyper_rustls::HttpsConnector;
use google_sheets4::oauth2::ServiceAccountAuthenticator;
use google_sheets4::{Error, Sheets, hyper, hyper_rustls, oauth2};
use log::error;
use serde_json::Value;
use std::any::type_name;
use std::fmt::{Debug, Formatter};

pub use google_sheets4::api::MatchedValueRange;

#[derive(Debug, thiserror::Error)]
pub enum SpreadSheetDriverError {
    #[error("Range {0} not found")]
    RangeNotFound(String),
    #[error("Spreadsheet API error ({0})")]
    ApiError(String),
    #[error("Parsing error ({0})")]
    ParseError(String),
    #[error("Invalid argument {0}")]
    InvalidArgument(String),
}

pub type SsdResult<T> = error_stack::Result<T, SpreadSheetDriverError>;

pub type SharedSpreadSheetDriver = AMShared<SpreadSheetDriver>;

#[derive(Debug)]
pub struct SpreadSheetDriver {
    document_id: String,
    pub sheets_client: SheetsClient,
}

pub type SheetsClientConnector = Sheets<HttpsConnector<HttpConnector>>;

impl SpreadSheetDriver {
    /// Panics if secret is not provided or is invalid
    pub async fn new(document_id: String, path_to_secret_json: &str) -> Self {
        Self {
            document_id,
            sheets_client: create_sheet_client(path_to_secret_json).await,
        }
    }

    fn client_ref(&self) -> &SheetsClientConnector {
        &self.sheets_client.0
    }
}
pub struct SheetsClient(pub SheetsClientConnector);

impl Debug for SheetsClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SheetsClient").finish()
    }
}

pub async fn create_sheet_client(path: &str) -> SheetsClient {
    // Load the service account key from a file
    let key = oauth2::read_service_account_key(path)
        .await
        .expect("Expected to read service account key");

    // Create a new authenticator
    let auth = ServiceAccountAuthenticator::builder(key)
        .build()
        .await
        .expect("Expected to create authenticator");

    // Create a new HTTPS connector
    let connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .expect("Expected to create HTTPS connector builder")
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .build();

    // Create a new hyper client
    let http_client = hyper::Client::builder().build(connector);

    let sheet_client = Sheets::new(http_client, auth);
    SheetsClient(sheet_client)
}

// TODO: Add API which deserialize `Vec<Vec<Value>>` into structs
// APIs //
impl SpreadSheetDriver {
    /// Read API
    pub async fn get_range<R>(&self, range: R) -> MatchedValueRange
    where
        R: ToString,
    {
        self.try_get_range(range)
            .await
            .expect("Expected to get range")
    }

    pub async fn try_get_range<R>(&self, range: R) -> SsdResult<MatchedValueRange>
    where
        R: ToString,
    {
        let range_str = range.to_string();
        let data = get_data_as_rows(self.client_ref(), &self.document_id, range_str.clone())
            .await
            .map_err(|e| SpreadSheetDriverError::ApiError(e.to_string()))?;
        let maybe_range = data.1.value_ranges.map(|v| v[0].clone());
        log::debug!("Range: {:?} result: {:#?}", range_str, maybe_range);
        maybe_range.ok_or(report!(SpreadSheetDriverError::RangeNotFound(range_str)))
    }

    /// Write api
    pub async fn write_range(&self, range_str: &str, data: Vec<Vec<serde_json::Value>>) {
        self.try_write_range(range_str, data)
            .await
            .unwrap_or_else(|e| panic!("Expected to write to spreadsheet: {:#?}", e))
    }

    pub async fn try_write_range(
        &self,
        range_str: &str,
        data: Vec<Vec<serde_json::Value>>,
    ) -> SsdResult<()> {
        let _ = self
            .client_ref()
            .spreadsheets()
            .values_update(
                ValueRange {
                    major_dimension: None,
                    range: None,
                    values: Some(data),
                },
                self.document_id.as_str(),
                range_str,
            )
            .value_input_option("RAW")
            .doit()
            .await
            .map_err(|e| {
                println!("error: {:#?}", e);
                SpreadSheetDriverError::ApiError(e.to_string())
            })?;

        Ok(())
    }

    /// Append API
    pub async fn try_append_row<R>(&self, range: R, row: Vec<serde_json::Value>) -> SsdResult<()>
    where
        R: Into<String>,
    {
        let range = range.into();
        let req = ValueRange {
            major_dimension: Some("ROWS".to_string()),
            range: Some(range.clone()),
            values: Some(vec![row]),
        };
        self.client_ref()
            .spreadsheets()
            .values_append(req, self.document_id.as_str(), range.as_str())
            .value_input_option("RAW")
            .doit()
            .await
            .map_err(|e| report!(SpreadSheetDriverError::ApiError(e.to_string())))
            .map(|_| ())
    }

    /// Returns row number in spreadsheet by row index of resulting data
    pub async fn try_get_row_num_by_row_index(
        data: &MatchedValueRange,
        row_index: usize,
    ) -> SsdResult<u32> {
        let Some(filters) = data.data_filters.as_ref() else {
            bail!(SpreadSheetDriverError::InvalidArgument(
                "MatchedValueRange doesn't have data filters".to_string()
            ));
        };

        if filters.len() != 1 {
            bail!(SpreadSheetDriverError::InvalidArgument(
                "MatchedValueRange doesn't have exactly one filter".to_string()
            ));
        };

        let filter = filters
            .first()
            .expect("Expected to have exactly one filter");

        let Some(range) = filter.a1_range.as_ref() else {
            bail!(SpreadSheetDriverError::InvalidArgument(
                "Data filter doesn't have A1 range".to_string()
            ));
        };

        todo!("Parse range into parts and calculate row index");
        // let maybe_range_start  = range.split(":").collect::<Vec<&str>>()
        //     .first()
        // .and_then(|&part| if part.contains("!") {
        //     part.split("!").collect::<Vec<&str>>().get(1).expect("Expected to have 2 parts after split by '!'")
        // } else {
        //     part
        // }
        // )
        // .and_then(|part| parse::<>);
        Ok(5)
    }

    /// Typed API ///
    pub async fn read_rows_deserialized_ignore_errors<T>(&self, range_str: &str) -> Vec<T>
    where
        T: SheetRowSerde,
    {
        let result = self.try_get_range(range_str).await;
        let range = match result {
            Ok(range) => range,
            Err(_) => {
                return vec![];
            }
        };

        range
            .into_vec()
            .into_iter()
            // TODO: use .filter_map(|v| v.....
            .filter_map(|row| {
                let result = T::deserialize(row);
                match result {
                    Ok(v) => Some(v),
                    Err(err) => {
                        log::error!(
                            "Failed to create {:?} from row.\nError: {}",
                            type_name::<T>(),
                            err.to_string_no_bt()
                        );
                        None
                    }
                }
            })
            .collect()
    }

    pub async fn read_rows_deserialized<T>(&self, range_str: &str) -> SsdResult<Vec<T>>
    where
        T: SheetRowSerde,
    {
        let range = self.get_range(range_str).await;
        let result: SsdResult<Vec<T>> = range
            .into_vec()
            .into_iter()
            // TODO: use .filter_map(|v| v.....
            .map(|row| T::deserialize(row))
            // .flatten()
            .collect();
        result
    }
}

pub async fn get_data_as_rows(
    client: &Sheets<HttpsConnector<HttpConnector>>,
    sheet: &str,
    range_str: String,
) -> Result<(Response<Body>, BatchGetValuesByDataFilterResponse), Error> {
    let req = BatchGetValuesByDataFilterRequest {
        data_filters: Some(vec![DataFilter {
            a1_range: Some(range_str),
            developer_metadata_lookup: None,
            grid_range: None,
        }]),
        date_time_render_option: None,
        major_dimension: Some("ROWS".to_string()),
        value_render_option: None,
    };

    let result = client
        .spreadsheets()
        .values_batch_get_by_data_filter(req, sheet)
        .doit()
        .await;

    let data = match result {
        Ok(data) => data,
        Err(err) => return Err(err),
    };
    Ok(data)
}

pub trait IntoStrVec {
    fn into_str_vec(self) -> Vec<Vec<String>>;
    fn into_vec(self) -> Vec<Vec<Value>>;
}

impl IntoStrVec for MatchedValueRange {
    fn into_str_vec(self) -> Vec<Vec<String>> {
        self.into_vec()
            .into_iter()
            .map(|v| v.iter().map(|v| v.as_str().unwrap().to_owned()).collect())
            .collect()
    }

    fn into_vec(self) -> Vec<Vec<Value>> {
        self.value_range
            .expect("Expected to get range")
            .values
            .unwrap_or_default()
    }
}

pub type RawRow = Vec<Value>;

pub trait SheetRowSerde {
    fn deserialize(row: RawRow) -> SsdResult<Self>
    where
        Self: Sized;

    fn serialize(self) -> SsdResult<RawRow>;
}
