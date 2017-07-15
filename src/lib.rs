extern crate futures;
extern crate regex;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
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
pub mod testing;

pub use client::{Client, ClientOptions, IntoUrl};
pub use db::{Nok, Root};
pub use error::Error;
pub use path::{AttachmentName, AttachmentPath, DatabaseName, DatabasePath, DesignDocumentId, DesignDocumentPath,
               DocumentId, DocumentPath, IntoAttachmentPath, IntoDatabasePath, IntoDesignDocumentPath,
               IntoDocumentPath, IntoViewPath, ViewName, ViewPath};
pub use transport::ActionFuture;
