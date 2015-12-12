//! The couchdb crate is a thin wrapper around the CouchDB API, providing
//! low-level access to individual CouchDB commands—e.g., PUT database, GET
//! document, etc. The goal is for this crate to deal with the menial task of
//! sending HTTP requests and receiving HTTP responses and allow application
//! writers to focus on their business logic.
//!
//! The couchdb crate provides applications with type-safety beyond working with
//! raw strings. Applications get and put documents as structured types. Other
//! types, such as revisions and views, are strongly typed as well.
//!
//! To deal with JSON, the couchdb crate relies on Serde and its traits
//! `serde::Serialize` and `serde::Deserialize`. These traits are fundamental to
//! the couchdb crate's API—they are not mere implementation details. As such,
//! applications must use these traits when working with documents, views, or
//! any type that is JSON-encoded in the CouchDB API.
//!
//! # Examples
//!
//! Here are some examples of how to use the couchdb crate.
//!
//! ## Example #1: Put a document
//!
//! This example creates a database named “baseball” and puts a single document
//! into it.
//!
//! ```ignore
//! extern crate couchdb;
//!
//! #[derive(Serialize)]
//! struct Player {
//!     name: String,
//!     hits: u32,
//!     home_runs: u32,
//! }
//!
//! let client = couchdb::Client::new("http://your_database/").unwrap();
//!
//! client.put_database("baseball").run().unwrap();
//!
//! let doc = Player {
//!     name: "Babe Ruth".to_string(),
//!     hits: 2873,
//!     home_runs: 714,
//! };
//!
//! client.put_document("cats/babe_ruth", &doc).run()
//!                                            .unwrap();
//! ```
//!
//! ## Example #2: Get a document
//!
//! This example gets the document created in the previous example.
//!
//! ```ignore
//! extern crate couchdb;
//! 
//! #[derive(Deserialize)]
//! struct Player {
//!   name: String,
//!   hits: u32,
//!   home_runs: u32,
//! }
//! 
//! let client = couchdb::Client::new("http://your_database/").unwrap();
//! 
//! let doc = client.get_document::<_, Player>("baseball/babe_ruth").run()
//!                                                                 .unwrap()
//!                                                                 .unwrap();
//! 
//! assert_eq!(doc.path, "baseball/babe_ruth".into());
//! assert_eq!(doc.content.name, "Babe Ruth".to_string());
//! assert_eq!(doc.hits, 2873);
//! assert_eq!(doc.home_runs, 714);
//! ```

extern crate hyper;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate tempdir;
extern crate url;

pub mod command;

mod client;
mod database;
mod dbpath;
mod dbtype;
mod design;
mod docid;
mod docpath;
mod document;
mod error;
mod revision;
mod server;
mod transport;
mod viewfunction;
mod viewpath;
mod viewresult;
mod viewrow;

#[cfg(test)]
mod jsontest;

pub use client::{Client, IntoUrl};
pub use database::Database;
pub use dbpath::DatabasePath;
pub use design::{Design, DesignBuilder};
pub use docid::DocumentId;
pub use docpath::DocumentPath;
pub use document::Document;
pub use error::{Error, ErrorResponse};
pub use revision::Revision;
pub use server::Server;
pub use viewfunction::{ViewFunction, ViewFunctionMap};
pub use viewpath::ViewPath;
pub use viewresult::ViewResult;
pub use viewrow::ViewRow;
