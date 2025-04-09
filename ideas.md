## Features
- [ ] Implement from_str for every cell/range structs (for convenience)
- [ ] Implement Display for every cell/range structs (for nicer logs)
- [ ] Implement more high-level repository (or update current) which:
  - relies more on entity metadata like start cell
  - automatically tracks exact number of rows in the table
  - automatically inserts more empty rows if approaching the row limit
  - implement optional caching (using metadata)

## Fixes/Improvements
- [ ] Fix cell offset calculations
- [ ] Skip empty rows during deserialization

