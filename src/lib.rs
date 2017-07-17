//! The `couchdb` library provides types for working with CouchDB.
//!
//! # Summary
//!
//! * The `couchdb` library is not a CouchDB client. Rather, it makes it easier
//!   for applications to communicate with a CouchDB server using existing HTTP
//!   client libraries (such as [hyper](https://crates.io/crates/hyper) and
//!   [reqwest](https://crates.io/crates/reqwest)).
//!
//! * The `couchdb` library is a toolkit, not a framework. Applications may opt
//!   in to using as much or as little of the library as makes the most sense.
//!
//! # Prerequisites
//!
//! * The application programmer is familiar with CouchDB and its API.
//!
//! Though the `couchdb` library aims to be easy to use, it does not aim to
//! teach programmers about CouchDB or how to use the CouchDB API. For more
//! information about CouchDB, consult its
//! [documentation](http://docs.couchdb.org/en/2.0.0/index.html#).
//!
//! # Remarks
//!
//! The CouchDB API, like most HTTP interfaces, uses a lot of stringly types and
//! requires client applications to do a lot of text-parsing and
//! text-formatting. The `couchdb` library makes working with these stringly
//! types easier.
//!
//! In earlier versions, the `couchdb` library provided a fledgling CouchDB
//! client for communicating with a CouchDB server, but now the library is
//! purely a passive collection of types, as well as testing tools, that's
//! intended to be used in conjunction with other HTTP libraries, such as
//! [hyper](https://crates.io/crates/hyper) or
//! [reqwest](https://crates.io/crates/reqwest).

extern crate base64;
extern crate mime;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
#[macro_use]
extern crate serde_json;
extern crate tempdir;
extern crate url;
extern crate uuid;

pub mod attachment;
pub mod path;
pub mod testing;

mod database;
mod error;
mod nok;
mod revision;
mod root;

pub use attachment::Attachment;
pub use database::Database;
pub use error::Error;
pub use nok::Nok;
pub use path::*;
pub use revision::Revision;
pub use root::{Root, Vendor, Version};
