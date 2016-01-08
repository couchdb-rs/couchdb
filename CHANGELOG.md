# CouchDB-rs Change Log

## v0.4.1 (unreleased)

### Deprecations

* Direct construction of a `ViewFunction` instance. Applications should
  instead use the new `ViewFunctionBuilder` type to construct a
  `ViewFunction` instance.

### Additions

* New `ViewFunctionBuilder` type for constructing a `ViewFunction`
  instance.

## v0.4.0 (2016-01-03)

This release introduces several breaking changes to improve type-safety
and ease-of-use, as well as to fix inconsistencies between the crate's
API and the CouchDB API.

### Breaking changes

* The _path_ types of v0.3.x (e.g., `DocumentPath`, etc.) are now split
  into _path_, _id_, and _name_ types (e.g., `DocumentPath`,
  `DocumentId`, and `DocumentName`, etc.). Client commands use path
  types as input; id and name types are used everywhere else to match
  what the CouchDB API uses. This resolves issue #15.
    * Paths now must begin with a slash (e.g., `/db/docid` vs the
      `db/docid` format of v0.3.x).
    * Path types now implement `std::str::FromStr` instead of
      `From<String>`. This means string-to-path conversions now may
      fail.
* The `Revision` type now fully understands CouchDB revisions.
    * The `Revision` type now implements `std::str::FromStr` instead of
      `From<&str>` and `From<String>`. This means string-to-revision
      conversion now may fail.
    * The `Revision` type no longer implements `AsRef<str>`.
    * Revisions now compare as numbers, not strings, to match what the
      CouchDB server does. This resolves issue #16.
* The `Error` enum has been refactored to be simpler.
    * Many error variants documented in v0.3.x are now hidden or
      removed. The remaining variants are either CouchDB response errors
      or are for path-parsing.
    * All CouchDB response error values are now wrapped in an `Option`
      to reflect how the CouchDB server returns no detailed error
      information for HEAD requests.
    * All non-hidden error variant values are now tuples, not structs.
    * The `InvalidRequest` error variant has been renamed to
      `BadRequest`. The new name matches HTTP status code 400 of the
      same name.

### Other fixes

* When getting a document, the client now ignores any `_attachments`
  field in the CouchDB response. Previously, the client included the
  attachment info in the document content. This resolves issue #17.
* The client no longer tries to decode the server's response as JSON
  when the client receives an "unauthorized" error as a result of
  executing a client command to HEAD a document.

### Implementation notes

* Test coverage has expanded, and test cases have been broken out into
  smaller cases. Consequently, there are now more than 200 additional
  test cases than in the v0.3.1 release.
* The source code has been reorganized to be more hierarchical. CouchDB
  types, path types, and client commands now reside within distinct
  submodules.

## v0.3.1 (2015-12-21)

This release expands the crate's coverage of the CouchDB API.

### Additions

* There's a new client command to POST to a database. This resolves
  issue #11.
* The `Revision` type now implements `serde::Serialize` and
  `serde::Deserialize`.

## v0.3.0 (2015-12-12)

This release overhauls the crate's API to provide stronger type-safety
and to be more Rust-idiomatic.

### Breaking changes

* There are new types for specifying databases, documents, and views.
  This resolves issue #4.
    * All raw-string path parameters have been replaced with new _path_
      types: `DatabasePath`, `DocumentPath`, and `ViewPath`. The
      signatures of all client commands have changed, as well as the
      `Document` and `ViewRow` types.
    * There's a new `DocumentId` type that combines a document name with
      its type (i.e., _normal_ document vs _design_ document vs _local_
      document).
* All client commands specific to design documents (e.g.,
  `get_design_document`) have been removed. Design documents are now
  accessible via generic document commands (e.g., `get_document`).
* The `ViewResult` struct now wraps its `total_rows` and `offset` fields
  in an `Option`.
* The underlying type for `ViewFunctionMap` is now `HashMap`, not
  `BTreeMap`.
* The `Command` trait is now private. This resolves issue #9.
* Crate dependencies now specify explicit version ranges instead of `*`.

### Other fixes

* All JSON-decoding errors are now reported as the `Decode` error
  variant. Previously, some decoding errors were reported as a hidden
  variant.
* The `Revision` type now compares as case-insensitive, matching CouchDB
  semantics.
* A bug has been fixed that caused CPU spin on Windows in the `Server`
  type. This partially resolves issue #8.

### Other additions

* The `Database` type now includes all fields returned by the CouchDB
  server as a result of a client command to GET a database.
* There's a new `DesignBuilder` type to make it easier to construct
  `Design` instances.
* The `Clone`, `Hash`, `Eq`, `PartialEq`, `Ord`, and `PartialOrd` traits
  have been implemented for all types where appropriate.

## v0.2.0 (2015-10-17)

### Breaking changes

* Client command-construction methods (e.g., `put_document`,
  `get_database`, etc.) now bind the lifetime of the returned command to
  the lifetimes of all `&str` parameters.
* The client command to GET a design document now strips `"_design/"`
  from the resulting document id. This resolves issue #7.

### Implementation notes

* The integration test has been split into separate test cases, one for
  each CouchDB command.
* Some support has been added for running tests on Windows. See issue
  #8.

## v0.1.0 (2015-09-21)

### Breaking changes

* The `Revision` type now implements the `AsRef<str>` trait instead of
  implementing the `as_str` method.
* Client commands that have a revision parameter now borrow the
  `Revision` argument instead of taking ownership. This resolves issue
  #1.
* Disallow construction of a `Revision` from an arbitrary string. This
  resolves issue #3.
* The `ServerErrorResponse` type has been renamed to `ErrorResponse`,
  which is now used consistently for reporting CouchDB server errors.
* The `DesignDocument` type has been renamed to `Design`. This resolves
  issue #5.
* There's a new `IntoUrl` trait that aliases `hyper::IntoUrl`. This
  resolves issue #2.

### Other fixes

* The `views` member of the `Design` struct is now public.

### Other additions

* There's a new `ViewFunctionMap` collection type.

## v0.0.1 (2015-09-07)

This release adds and improves API doc comments.

## v0.0.0 (2015-09-05)

This is the first release. It provides support for client commands to
manipulate databases (HEAD, GET, PUT, and DELETE), to manipulate
documents (HEAD, GET, PUT, and DELETE), and to execute views (GET).
