# CouchDB-rs Change Log

## v0.3.0 (in development)

Release date: n/a

API changes:

* Replace string path parameters with stronger path types.
* Change crate dependency versions from * to explicit range values.

Other changes:

* Fix CPU spin in `Server` on Windows (partially resolve issue #8).

## v0.2.0

Release date: 2015-10-17

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

Release date: 2015-09-21

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

Release date: 2015-09-07

* Improve documentation.
* Add this Change Log.

## v0.0.0

Release date: 2015-09-05

* Initial release
* New commands for database manipulation (GET, PUT, HEAD, and DELETE),
	document manipulation (GET, PUT, HEAD, and DELETE), and view execution
  (GET).
