#[macro_use]
mod impl_name_macro;

#[cfg(test)]
#[macro_use]
mod test_macro;

mod database;
mod database_name;
mod design;
mod design_document_name;
mod document;
mod document_id;
mod document_name;
mod response;
mod revision;
mod view_function;
mod view_name;
mod view_result;
mod view_row;

pub use self::database::Database;
pub use self::database_name::DatabaseName;
pub use self::design::{Design, DesignBuilder};
pub use self::design_document_name::DesignDocumentName;
pub use self::document::Document;
pub use self::document_id::DocumentId;
pub use self::document_name::DocumentName;
pub use self::response::{ErrorResponse, PostToDatabaseResponse, PutDocumentResponse};
pub use self::revision::Revision;
pub use self::view_function::{ViewFunction, ViewFunctionMap};
pub use self::view_name::ViewName;
pub use self::view_result::ViewResult;
pub use self::view_row::ViewRow;
