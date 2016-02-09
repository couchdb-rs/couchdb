# CouchDB-rs Change Log

## v0.5.1 (unreleased)

### Deprecated

* Deprecated the method `Client::post_to_database`, replaced by
  `Client::post_database`.
* Deprecated the action type `PostToDatabase`, replaced by
  `PostDatabase`.

### New

* New action `GetChanges` for getting database changes—i.e., `GET
  /db/_changes`.
* New action `GetRoot` for getting the CouchDB server's root resource
  (`/`)—e.g., for getting the server's version.
* New support for getting documents at a specific revision via the `rev`
  query parameter—i.e., `GET /db/doc?rev=<revision>`.
* New support for getting embedded attachments via the `GetDocument`
  action.
* New field `deleted` in the `Document` type for signifying whether the
  document has been deleted.
* New feature table in the `action` module showing, in detail, this
  crate's coverage of the CouchDB API.

## v0.5.0 (2016-01-17)

This release makes a few API changes to continue the library's progress
towards optimal type-safety and convenience.

### Breaking changes

* The `Document` type has been refactored to make it easier to use.
    * The `Document` type is no longer a generic type, nor is the
      `content` field publicly accessible. Applications now access
      document content via a new `into_content` method, which does the
      JSON-decoding. See issue [#28][issue_28] for more information.
    * The `revision` field has been renamed to `rev`, which more closely
      matches the CouchDB name.
    * The `Document` type implements `serde::Deserialize` instead of a
      custom `from_reader` deserialization method. This should _not_
      affect applications.
    * The `Document` type no longer implements these traits: `Eq`,
      `Hash`, `Ord`, and `PartialOrd`.
* Throughout the project, the term “command” has been replaced with
  “action”. The only API change is that the
  `command` module is now named the `action` module. This should _not_
  affect applications. See issue [#32][issue_32] for more information.
* The `PostToDatabase` action now returns `(DocumentId, Revision)`, not
  `(Revision, DocumentId)`.
* The following types now have at least one private field and can no
  longer be directly constructed by applications:
    * `Database`,
    * `Design`,
    * `ErrorResponse`,
    * `ViewFunction`,
    * `ViewResult`, and
    * `ViewRow`.
* The `DeleteDocument` action now returns the revision of the deleted
  document. Previously the action returned nothing.
* The `Server` type has been moved/renamed to `testing::FakeServer`.

### New

* New `ViewFunctionBuilder` type for constructing a `ViewFunction`
  instance.
* New `Revision::update_number` method for getting the _update number_
  part of a revision.

### Additional notes

* The project is now dual-licensed under Apache-2.0 and MIT. See issue
  [#31][issue_31] for more information.
* Actions are now tested as unit tests _and_ integration tests.
  Previously, actions were tested only as integration tests.
  Unit-testing now provides good test coverage without having the
  CouchDB server installed on the local machine.
* The project now has support for Travis CI.

## v0.4.0 (2016-01-03)

This release introduces several breaking changes to improve type-safety
and ease-of-use, as well as to fix inconsistencies between the crate's
API and the CouchDB API.

### Breaking changes

* The _path_ types of v0.3.x (e.g., `DocumentPath`, etc.) are now split
  into _path_, _id_, and _name_ types (e.g., `DocumentPath`,
  `DocumentId`, and `DocumentName`, etc.). Client commands use path
  types as input; id and name types are used everywhere else to match
  what the CouchDB API uses.
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
      CouchDB server does.
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

### Fixes

* When getting a document, the client now ignores any `_attachments`
  field in the CouchDB response. Previously, the client included the
  attachment info in the document content.
* The client no longer tries to decode the server's response as JSON
  when the client receives an "unauthorized" error as a result of
  executing a client command to HEAD a document.

### Additional notes

* Test coverage has expanded, and test cases have been broken out into
  smaller cases. Consequently, there are now more than 200 additional
  test cases than in the v0.3.1 release.
* The source code has been reorganized to be more hierarchical. CouchDB
  types, path types, and client commands now reside within distinct
  submodules.

## v0.3.1 (2015-12-21)

This release expands the crate's coverage of the CouchDB API.

### New

* There's a new client command to POST to a database.
* The `Revision` type now implements `serde::Serialize` and
  `serde::Deserialize`.

## v0.3.0 (2015-12-12)

This release overhauls the crate's API to provide stronger type-safety
and to be more Rust-idiomatic.

### Breaking changes

* There are new types for specifying databases, documents, and views.
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
* The `Command` trait is now private.
* Crate dependencies now specify explicit version ranges instead of `*`.

### Fixes

* All JSON-decoding errors are now reported as the `Decode` error
  variant. Previously, some decoding errors were reported as a hidden
  variant.
* The `Revision` type now compares as case-insensitive, matching CouchDB
  semantics.
* A bug has been fixed that caused CPU spin on Windows in the `Server`
  type.

### New

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
  from the resulting document id.

### Additional notes

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
* Disallow construction of a `Revision` from an arbitrary string.
* The `ServerErrorResponse` type has been renamed to `ErrorResponse`,
  which is now used consistently for reporting CouchDB server errors.
* The `DesignDocument` type has been renamed to `Design`.
* There's a new `IntoUrl` trait that aliases `hyper::IntoUrl`.

### Fixes

* The `views` field of the `Design` struct is now public.

### New

* There's a new `ViewFunctionMap` collection type.

## v0.0.1 (2015-09-07)

This release adds and improves API doc comments.

## v0.0.0 (2015-09-05)

This is the first release. It provides support for client commands to
manipulate databases (HEAD, GET, PUT, and DELETE), to manipulate
documents (HEAD, GET, PUT, and DELETE), and to execute views (GET).

[issue_28]: https://github.com/couchdb-rs/couchdb/issues/28 "Issue #28"
[issue_31]: https://github.com/couchdb-rs/couchdb/issues/31 "Issue #31"
[issue_32]: https://github.com/couchdb-rs/couchdb/issues/32 "Issue #32"
