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

mod transport;
pub mod action;
mod client;
mod error;
pub mod testing;

pub use client::{AsyncClient, Client, ClientOptions, IntoUrl, SyncClient};
pub use error::{Error, Nok};
pub use transport::ActionFuture;
