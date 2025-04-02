# google_sheets_driver

**google_sheets_driver** is a Rust library that wraps the [google-sheets4](https://crates.io/crates/google-sheets4) API, providing a more convenient—and in some cases, type-safe—interface for using Google Sheets as a persistent storage mechanism. It offers a simple ORM-like abstraction, enabling you to treat spreadsheets almost like a relational database.

> **Note:** This library is in an early alpha stage. Expect frequent changes to the API, possible renaming or repurposing, and even the chance that the project may be abandoned. Use it for experimentation and feedback only.

## Features

- **Driver Abstraction:**  
  A thin layer over the Google Sheets API for reading, writing, appending, and batch operations.

- **Repository & ORM-like API:**  
  Provides a repository layer with typed methods to fetch and deserialize rows, update records, and eventually support full CRUD operations.

- **Type-Safe Cell and Range Handling:**  
  Uses A1-notation cell IDs and ranges with operator overloading and basic math capabilities for cell manipulation.

- **Error Handling:**  
  Integrates with the [error-stack](https://crates.io/crates/error-stack) library and uses [thiserror](https://crates.io/crates/thiserror) and [derive_more](https://crates.io/crates/derive_more) for ergonomic error definitions and conversions.

- **Typed Deserialization:**  
  Easily convert spreadsheet rows into your custom structs via a simple trait-based mechanism.

## Usage

Add the dependency in your `Cargo.toml`:

```toml
[dependencies]
google_sheets_driver = "0.0.1"
error-stack = "0.5.0"
google-sheets4 = "5.0.5"
log = "0.4.27"
serde = "1.0.219"
serde_json = "1.0.140"
thiserror = "2.0.12"
tokio = "1.44.1"
derive_more = { version = "2.0.1", features = ["display", "deref", "from", "from_str"] }
```

### Example

Below is a brief example showing how you might set up and use the driver and repository:

```rust
use google_sheets_driver::{SpreadSheetDriver, SharedSpreadSheetDriver};
use std::sync::Arc;

/// Entity to use in ORM
#[derive(Debug, Clone, PartialEq)]
struct User {
  id: i32,
  name: String,
}

/// From row and Into row conversions
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
  fn serialize(self) -> sheet_row::Result<SheetRow> {
    Ok(vec![
      Value::String(self.name),
      Value::String(self.id.to_string()),
    ])
  }
}

/// Entity metadata
impl EntityEssentials for User {
  fn entity_width() -> u32 {
    2
  }
}

#[tokio::main]
async fn main() {
  // Create a new SpreadSheetDriver using a service account JSON key.
  let driver = SpreadSheetDriver::new(
    "your_gs_id".to_string(),
    "path/to/service_account_key.json",
  ).await;

  // Wrap the driver in a shared reference if needed
  let shared_driver: SharedSpreadSheetDriver = Arc::new(driver);

  // Create a Repository to access ORM-like features.
  let repository = Repository::new(shared_driver);

  // Example of reading rows as deserialized structs.
  let start_cell = &SheetA1CellId::from_primitives("users", "A", 2);
  let rows_to_fetch = 3;
  // Row width is calculated automatically based on entity metadata
  let users: Vec<User> = repository
          .find_in_range(start_cell, rows_to_fetch)
          .await
          .expect("Expected to fetch users");

  println!("Fetched users: {:?}", users);
}
```
Output:
```
[
    Entity {
        position: SheetA1CellId {
            sheet_name: "users",
            cell: A1CellId {
                col: Letters(
                    "A",
                ),
                row: 1,
            },
        },
        data: User {
            id: 1,
            name: "Joe",
        },
    },
    ...    
]
```
In this example, `User` would be your custom struct implementing the required traits (such as `SheetRowSerde` and `EntityEssentials`) for proper serialization and deserialization of spreadsheet rows.

## Development Direction

The goal of **google_sheets_driver** is to evolve into a robust, database-like interface for working with Google Sheets. Future enhancements may include:

- **Full CRUD Operations:**  
  Expanding repository functions to support insert, update, delete, and more complex queries.

- **Improved Error Handling:**  
  Richer error reporting and easier debugging during interactions with the Google Sheets API.

- **Enhanced Type Safety:**  
  Additional type-level guarantees and conversions for working with cells, ranges, and row data.

- **Better API Ergonomics:**  
  Simplifying the API surface and adding more helper functions to reduce boilerplate code when interacting with spreadsheets.

- **Community Feedback:**  
  Given the early alpha stage, feedback is welcome. Please open issues or submit pull requests if you have suggestions or improvements.

## Contributing

Contributions are welcome—even though this library is in super early development, community feedback is valuable. Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct, and the process for submitting pull requests.

## License

This project is dual-licensed under the terms of both the MIT License and the Apache License, Version 2.0. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
