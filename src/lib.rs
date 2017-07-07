extern crate futures;
extern crate regex;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate tempdir;
extern crate url;

mod client;
mod error;
mod request;
pub mod testing;
mod transport;

pub use client::{Client, IntoUrl};
pub use error::{Error, Nok};
