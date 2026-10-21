# Hermes DataFrame v0.1 — Core Implementation Checklist

**Architecture:** Single Rust crate, multiple modules.  
**Storage:** Apache Arrow.  
**Scope:** Internal DataFrame representation and basic operations. No Python bindings, I/O, lazy execution, or streaming engine yet.

## 1. Project Setup
- [✔] Configure `Cargo.toml`
- [✔] Create `lib.rs` as the library entry point
- [✔] Configure dependencies: `arrow-array`, `arrow-schema`, `arrow-select`, `thiserror`
- [ ] Establish module visibility and public API boundaries

## 2. Core Data Model — `dataframe/`
- [ ] `DataFrame` structure
- [ ] `Series` structure
- [ ] Schema representation
- [ ] Column representation
- [ ] Row and column count tracking
- [ ] Column name and index lookup
- [ ] DataFrame construction and validation
- [ ] Empty DataFrame support
- [ ] Duplicate column name policy
- [ ] Consistent row-length validation

## 3. Type System — `datatypes/`
- [ ] Arrow data type integration
- [ ] Supported primitive types
  - Boolean
  - Signed and unsigned integers
  - Floating-point numbers
  - Strings
  - Binary
- [ ] Null representation
- [ ] Type inspection
- [ ] Type compatibility validation

## 4. Arrow Integration — `arrow/`
- [ ] Arrow array storage using `ArrayRef`
- [ ] Arrow schema integration
- [ ] RecordBatch construction
- [ ] Arrow array access
- [ ] Array type validation
- [ ] Null count and validity handling
- [ ] Sliced-array and offset handling
- [ ] Shared array ownership using `Arc`
- [ ] Conversion between Series and Arrow arrays

## 5. Series API — `series/`
- [ ] Construct Series from Arrow arrays
- [ ] Series name and data type access
- [ ] Series length and null count
- [ ] Value access by index
- [ ] Null-aware value access
- [ ] Series slicing
- [ ] Series renaming
- [ ] Series cloning and shared references
- [ ] Series validation

## 6. DataFrame API — `dataframe/`
- [ ] Construct from Series
- [ ] Construct from RecordBatch
- [ ] Retrieve a column by name
- [ ] Retrieve a column by index
- [ ] Inspect schema
- [ ] Inspect column names and data types
- [ ] Inspect height and width
- [ ] Select columns
- [ ] Slice rows
- [ ] Add a column
- [ ] Replace a column
- [ ] Drop a column
- [ ] Validate structural invariants

## 7. Basic Compute Operations — `compute/`
- [ ] Filter rows using a Boolean mask
- [ ] Select columns
- [ ] Slice arrays without unnecessary copying
- [ ] Cast supported data types
- [ ] Validate filter mask length
- [ ] Define null predicate behavior
- [ ] Preserve column alignment
- [ ] Propagate operation errors

## 8. Error Handling — `error/`
- [ ] `HermesError`
- [ ] Invalid schema errors
- [ ] Duplicate column errors
- [ ] Column not found errors
- [ ] Column length mismatch errors
- [ ] Data type mismatch errors
- [ ] Invalid array errors
- [ ] Invalid filter errors
- [ ] Consistent `Result<T>` alias

## 9. Testing — `tests/`
- [ ] DataFrame construction tests
- [ ] Series construction tests
- [ ] Schema validation tests
- [ ] Type validation tests
- [ ] Column lookup tests
- [ ] Selection tests
- [ ] Slicing tests
- [ ] Filtering tests
- [ ] Null handling tests
- [ ] Empty DataFrame tests
- [ ] Duplicate column tests
- [ ] Arrow interoperability tests
- [ ] Error handling tests

## 10. Folder Structure

```text
hermesdf/
├── Cargo.toml
├── Cargo.lock
├── src/
│   ├── lib.rs
│   ├── dataframe/
│   │   ├── mod.rs
│   │   ├── frame.rs
│   │   ├── series.rs
│   │   ├── schema.rs
│   │   └── column.rs
│   ├── datatypes/
│   │   └── mod.rs
│   ├── arrow/
│   │   └── mod.rs
│   ├── compute/
│   │   ├── mod.rs
│   │   ├── filter.rs
│   │   ├── select.rs
│   │   ├── slice.rs
│   │   └── cast.rs
│   └── error/
│       └── mod.rs
└── tests/
    ├── dataframe_tests.rs
    ├── series_tests.rs
    └── compute_tests.rs
```

## v0.1 Completion Criteria

Version 0.1 is complete when:

- [ ] A DataFrame can hold multiple typed Arrow-backed columns.
- [ ] Every column has a consistent length.
- [ ] Schemas and data types are validated.
- [ ] Null values are handled correctly.
- [ ] Columns can be accessed, selected, added, replaced, and dropped.
- [ ] Rows can be sliced and filtered.
- [ ] Operations preserve schema and column alignment.
- [ ] Errors are explicit and predictable.
- [ ] All core functionality has automated tests.

**Implementation order:** Project setup → Error handling → Data types → Arrow integration → Series → DataFrame → Compute operations → Tests.
