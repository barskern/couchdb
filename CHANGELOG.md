# CouchDB-rs Change Log

## [Unreleased (0.3.0)]

### Changed

* Replace string path parameters with stronger path types.
* Replace `Revision` constructor with `From` implementations.
* Change crate dependency versions from `*` to explicit range values.

### Fixed

* Fix `Revision` comparison to be case-insensitive, matching CouchDB
  semantics.
* Fix CPU spin in `Server` on Windows (partially resolve issue #8).

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
