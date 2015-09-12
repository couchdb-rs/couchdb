# CouchDB-rs Change Log

## v0.1.0 (in development)

API changes:
* Rename: `ServerErrorResponse` → `ErrorResponse`.
* For commands, borrow `Revision` argument instead of taking ownership.
* Hide `Revision` construction from an arbitrary string.

## v0.0.1

* Improve documentation.
* Add this Change Log.

## v0.0.0

* Initial release
* New commands for database manipulation (GET, PUT, HEAD, and DELETE), document
  manipulation (GET, PUT, HEAD, and DELETE), and view execution (GET).
