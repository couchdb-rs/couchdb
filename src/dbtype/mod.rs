// This module defines raw CouchDB types. These are types with the same layout
// as what the CouchDB API uses. Sometimes these layouts differ from what our
// crate exports.

mod viewresult;
mod viewrow;

pub use self::viewresult::ViewResult;
pub use self::viewrow::ViewRow;
