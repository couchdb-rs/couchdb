extern crate futures;
extern crate regex;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
#[macro_use]
extern crate serde_json;
#[cfg(not(test))]
extern crate serde_json;
#[cfg(test)]
extern crate serde_test;
extern crate tempdir;
extern crate tokio_core;
extern crate url;
extern crate uuid;

mod transport;
pub mod action;
mod client;
pub mod db;
mod error;
pub mod path;
mod revision;
pub mod testing;

pub use client::{Client, ClientOptions, IntoUrl};
pub use db::{Database, Nok, Root};
pub use error::Error;
pub use path::{AttachmentName, AttachmentPath, DatabaseName, DatabasePath, DesignDocumentId, DesignDocumentPath,
               DocumentId, DocumentPath, IntoAttachmentPath, IntoDatabasePath, IntoDesignDocumentPath,
               IntoDocumentPath, IntoViewPath, ViewName, ViewPath};
pub use revision::Revision;
pub use transport::ActionFuture;
