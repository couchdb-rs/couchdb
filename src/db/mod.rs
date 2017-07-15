//! The `db` module provides types that mirror CouchDB server types.

mod nok;
mod root;

pub use self::nok::Nok;
pub use self::root::Root;
