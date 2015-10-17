# CouchDB-rs Change Log

## v0.2.1 (in development)

(nothing yet!)

## v0.2.0

This release fixes a few small problems in the API, as well as
refactoring a lot of code to make it easier to add support for covering
more CouchDB commands.

API changes:

* Command-construction methods (e.g., `put_document`, `get_database`,
	etc.) now bind the lifetime of the returned command to the lifetimes
  of all `&str` arguments.

Other changes:

* Fix `GetDesignDocument` to strip `"_design/"` from document id.
* Refactor integration tests.
	* Separate integration test into separate test cases, one for each
	  CouchDB command.
  * Add support for running on Windows. (See issue #8.)
* New `Command` trait to simplify command implementations.

## v0.1.0

This release fixes many small problems in the API.

API changes:

* Improve `Revision` type-safety:
	* Remove `as_str` method and instead implement the `AsRef<str>` trait.
	* CouchDB commands that have a revision parameter now borrow the
	  `Revision` argument instead of taking ownership.
	* Hide `Revision` construction from an arbitrary string. Applications
		now may only construct revisions via the API, e.g., getting a
    document.
* New `ViewFunctionMap` collection type.
* Make public the `views` member of the `DesignDocument` struct.
* New `IntoUrl` trait to alias the trait of the same name from the hyper
  crate.
* Rename `ServerErrorResponse` to `ErrorResponse` and use the type
  consistently for errors.
* Rename `DesignDocument` to `Design`.

## v0.0.1

* Improve documentation.
* Add this Change Log.

## v0.0.0

* Initial release
* New commands for database manipulation (GET, PUT, HEAD, and DELETE),
	document manipulation (GET, PUT, HEAD, and DELETE), and view execution
  (GET).
