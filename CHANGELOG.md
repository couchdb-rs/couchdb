# CouchDB-rs Change Log

## [Unreleased (0.3.1)]

No changes yet.

## [0.3.0] - 2015-12-12

An API overhaul, strengthening type-safety and making the crate more
Rust-idiomatic.

### Added

* New path types: `DatabasePath`, `DocumentPath`, and `ViewPath`. These
  types provide stronger type-safety than raw strings.
* New `DocumentId` type to specify name _and_ distinguish type of
  document (e.g., normal document vs design document).
* Add missing fields to the `Database` type. The `Database` type now
  includes all information returned by the CouchDB server.
* New `DesignBuilder` type to make it easier to construct `Design`
  instances.
* Implement `Clone`, `Hash`, `Eq`, `PartialEq`, `Ord`, and `PartialOrd`
  for the `Database` type.
* Implement `Clone` for the `Design` type.
* Implement `Clone`, `Hash`, `Eq`, `PartialEq`, `Ord`, and `PartialOrd`
  for the `Database` type.
* Implement `Clone`, `Display` `Eq`, `Hash`, `Ord`, `PartialEq`, and
  `PartialOrd` for the `ErrorResponse` type.
* Implement `Hash`, `From<String>`, and `From<&str>` for the `Revision`
  type.
* Implement `Clone`, `Hash`, `Ord`, and `PartialOrd` for the
  `ViewFunction` type.
* Implement `Clone`, `Hash`, `Eq`, `PartialEq`, `Ord`, and `PartialOrd`
  for the `ViewResult` type.
* Implement `Clone`, `Hash`, `Ord`, and `PartialOrd` for the `ViewRow`
  type.

### Changed

* Replace raw-string path types with new type-safe path types.
  * Client commands.
  * `Document` type.
  * `ViewRow` type.
* Wrap the `total_rows` and `offset` fields with `Option` in the
  `ViewResult` type.
* Return the `Error::Decode` variant for all JSON-decoding errors.
  Previously, some decoding errors were reported as an undocumented
  variant.
* Change underlying type of `ViewFunctionMap` from `BTreeMap` to
  `HashMap`.
* Change crate dependency versions from `*` to explicit range values.

### Fixed

* Change `Revision` comparison to be case-insensitive, matching CouchDB
  semantics.
* Eliminate CPU spin in `Server` on Windows. This partially resolves
  issue #8.

### Removed

* Make the `Command` trait private. This is an implementation detail.
* Remove client commands specific to design documents, e.g.,
  `get_design_document`, etc.. These commands are made redundant by the
  new path types.

## [0.2.0] - 2015-10-17

### Changed

* Command-construction methods (e.g., `put_document`, `get_database`,
  etc.) now bind the lifetime of the returned command to the lifetimes
  of all `&str` arguments.
* Fix `GetDesignDocument` to strip `"_design/"` from document id.
* Refactor integration tests.
  * Separate integration test into separate test cases, one for each
    CouchDB command.
  * Add support for running on Windows. (See issue #8.)

## [0.1.0] - 2015-09-21

### Changed

* Remove `as_str` method from the `Revision` type and instead implement the
  `AsRef<str>` trait.
* CouchDB commands that have a revision parameter now borrow the `Revision`
  argument instead of taking ownership.
* Hide `Revision` construction from an arbitrary string. Applications now may
  only construct revisions via the API, e.g., getting a document.
* New `ViewFunctionMap` collection type.
* Make public the `views` member of the `DesignDocument` struct.
* New `IntoUrl` trait to alias the trait of the same name from the hyper
  crate.
* Rename `ServerErrorResponse` to `ErrorResponse` and use the type
  consistently for errors.
* Rename `DesignDocument` to `Design`.

## [0.0.1] - 2015-09-07

### Changed

* Improve documentation.

## [0.0.0] - 2015-09-05

### Added

* Initial release
* New commands for database manipulation (GET, PUT, HEAD, and DELETE),
  document manipulation (GET, PUT, HEAD, and DELETE), and view execution
  (GET).
