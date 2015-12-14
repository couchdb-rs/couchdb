// This module defines raw CouchDB types. These are types with the same layout
// as what the CouchDB API uses. Sometimes these layouts differ from what our
// crate exports.

mod database;
mod errorresponse;
mod writedocresponse;
mod viewresult;
mod viewrow;

pub use self::database::Database;
pub use self::errorresponse::ErrorResponse;
pub use self::writedocresponse::{PostToDatabaseResponse, PutDocumentResponse};
pub use self::viewresult::ViewResult;
pub use self::viewrow::ViewRow;
