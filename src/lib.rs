/*
//! The couchdb crate is a thin wrapper around the CouchDB API, providing direct
//! access to individual CouchDB commands—e.g., PUT database, GET document, etc.
//!
//! The couchdb crate provides a medium amount of type-safety. Applications get
//! and put documents as structured types, not raw strings, and other types,
//! such as revisions and views, are strongly typed as well. However, the
//! couchdb crate does not enforce a schema for document content. Applications
//! must ensure they use the correct names and types when getting and putting
//! documents, as any type-mismatch regarding document content will cause a
//! run-time error, such as a failure to encode or decode JSON.
//!
//! To deal with JSON, the couchdb crate uses the `serde` traits `Serialize` and
//! `Deserialize`. These traits are fundamental to the couchdb crate's API and
//! are _not_ mere implementation details. As such, applications must use these
//! traits when working with documents.
//!
//! # Example: Put a document using serde macros
//!
//! Here's an example that creates a database named “cats” and puts a single
//! document into it.
//!
//! ```ignore
//! extern crate couchdb;
//!
//! #[derive(Serialize)]
//! struct Cat {
//!     name: String,
//!     color: String,
//!     years_old: u32,
//! }
//!
//! let client = couchdb::Client::new("http://your_database/").unwrap();
//!
//! client.put_database("cats").run().unwrap();
//!
//! let doc = Cat {
//!     name: "Nutmeg".to_string(),
//!     color: "orange".to_string(),
//!     years_old: 7,
//! };
//! client.put_document("cats", "nutmeg", &doc).run().unwrap();
//! ```
//!
//! # Example: Put a document using generic JSON placeholder
//!
//! Here's the same example, using the generic `serde_json::Value` type instead
//! of the `Cat` type.
//!
//! ```no_run
//! extern crate couchdb;
//! extern crate serde_json;
//!
//! let client = couchdb::Client::new("http://your_database/").unwrap();
//!
//! client.put_database("cats").run().unwrap();
//!
//! let mut doc = std::collections::BTreeMap::<String, serde_json::Value>::new();
//! doc.insert("name".to_string(), serde_json::to_value(&"Nutmeg"));
//! doc.insert("color".to_string(), serde_json::to_value(&"orange"));
//! doc.insert("years_old".to_string(), serde_json::to_value(&7));
//!
//! let doc = serde_json::Value::Object(doc);
//! client.put_document("cats", "nutmeg", &doc).run().unwrap();
//! ```
//!
//! This example works because the `serde_json::Value` type implements the
//! `Serialize` trait. Thus, the generic `Value` type allows an application to
//! avoid dealing with the serde macros to auto-derive the `Serialize` and
//! `Deserialize` traits.
//!
//! # Example: Get a document using serde macros
//!
//! The API for getting documents is similar to putting documents. Here's an
//! example. 
//!
//! ```ignore
//! extern crate couchdb;
//!
//! #[derive(Deserialize)]
//! struct Cat {
//!     name: String,
//!     color: String,
//!     years_old: u32,
//! }
//!
//! let client = couchdb::Client::new("http://your_database/").unwrap();
//!
//! let doc = client.get_document::<Cat>("cats", "nutmeg").run().unwrap()
//!     .unwrap().content;
//!
//! assert_eq!(doc.name, "Nutmeg".to_string());
//! assert_eq!(doc.color, "orange".to_string());
//! assert_eq!(doc.years_old, 7);
//! ```
//!
//! Applications may use the generic `serde_json::Value` type for getting
//! documents, too.
//!
//! # Example: construct a complex CouchDB command
//!
//! Applications build complex CouchDB commands one function call at a time,
//! adding non-default arguments as needed. Here's an example using
//! If-None-Match when getting a document to cause the server to return `304 Not
//! Modified` in case of a match, which the couchdb crate returns as `None`.
//!
//! ```ignore
//! extern crate couchdb;
//!
//! #[derive(Serialize, Deserialize)]
//! struct Cat {
//!     name: String,
//!     color: String,
//!     years_old: u32,
//! }
//!
//! let client = couchdb::Client::new("http://your_database/").unwrap();
//!
//! let doc = Cat {
//!     name: "Nutmeg".to_string(),
//!     color: "orange".to_string(),
//!     years_old: 7,
//! };
//! let rev = client.put_document("cats", "nutmeg", &doc).run().unwrap();
//!
//! let doc = client.get_document::<Cat>("cats", "nutmeg")
//!     .if_none_match(rev)
//!     .run()
//!     .unwrap();
//! assert!(doc.is_none());
//! ```
*/

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
mod design;
mod docid;
mod docpath;
mod document;
mod error;
mod revision;
mod server;
mod transport;
mod viewpath;

pub use client::{Client, IntoUrl};
pub use database::Database;
pub use dbpath::DatabasePath;
pub use design::{
    Design,
    ViewFunction,
    ViewFunctionMap,
    ViewResult,
    ViewRow};
pub use docid::DocumentId;
pub use docpath::DocumentPath;
pub use document::Document;
pub use error::{Error, ErrorResponse};
pub use revision::Revision;
pub use server::Server;
pub use viewpath::ViewPath;
