# CouchDB-rs Change Log

## v0.2.0 (in development)

Backwards-incompatible API changes:

* Command-construction methods of the `Client` type now bind the
  lifetime of the returned command to the lifetimes of all `&str`
  arguments.

Fixes:

* Strip `"_design/"` from document id for `GetDesignDocument` command.

Other changes:

* Refactor integration tests.
* New `Command` trait to simplify command implementations.

## v0.1.0

This release fixes many small problems in the API.

Backwards-incompatible API changes:

* Improve `Revision` type-safety:
	* Remove `as_str` method and instead implement the `AsRef<str>` trait.
	* CouchDB commands that have a revision parameter now borrow the
	  `Revision` argument instead of taking ownership.
	* Hide `Revision` construction from an arbitrary string. Applications
		now may only construct revisions via the API, e.g., getting a
    document.
* New `ViewFunctionMap` collection type.
* The `views` member of the `DesignDocument` struct is now publicly
  accessible.
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
