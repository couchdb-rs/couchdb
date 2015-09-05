//! CouchDB thin-wrapper client library.

extern crate hyper;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate tempdir;
extern crate url;

pub mod command;

mod client;
mod database;
mod design;
mod document;
mod error;
mod server;

pub use client::Client;
pub use database::Database;
pub use design::DesignDocument;
pub use design::ViewFunction;
pub use design::ViewResult;
pub use design::ViewRow;
pub use document::Document;
pub use document::Revision;
pub use error::Error;
pub use error::ServerErrorResponse;
pub use server::Server;
