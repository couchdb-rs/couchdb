// Types used in the CouchDB API.
//
// This module contains “CouchDB type,” which are types used in the requests
// and/or responses when communicating with the CouchDB server. Usually these
// types are serialized as JSON.

#[macro_use]
mod hex16;

#[macro_use]
mod impl_name_macro;

#[cfg(test)]
#[macro_use]
mod test_macro;

mod base64_blob;
mod change_item;
mod change_line;
mod change_result;
mod changes;
mod content_type;
mod database;
mod database_name;
mod design;
mod design_document_name;
mod document;
mod document_id;
mod document_name;
mod embedded_attachment;
mod error_response;
mod revision;
mod root;
mod since;
mod uuid;
mod view_function;
mod view_name;
mod view_result;
mod view_row;
mod write_document_response;

pub use self::base64_blob::Base64Blob;
pub use self::change_item::{ChangeItem, ChangeItemBuilder};
pub use self::change_line::ChangeLine;
pub use self::change_result::{ChangeResult, ChangeResultBuilder};
pub use self::changes::{Changes, ChangesBuilder};
pub use self::content_type::ContentType;
pub use self::database::Database;
pub use self::database_name::DatabaseName;
pub use self::design::{Design, DesignBuilder};
pub use self::design_document_name::DesignDocumentName;
pub use self::document::Document;
pub use self::document_id::DocumentId;
pub use self::document_name::DocumentName;
pub use self::embedded_attachment::{EmbeddedAttachment, EmbeddedAttachmentBuilder};
pub use self::error_response::ErrorResponse;
pub use self::revision::Revision;
pub use self::root::{Root, RootBuilder, Vendor};
pub use self::since::Since;
pub use self::uuid::Uuid;
pub use self::view_function::{ViewFunction, ViewFunctionBuilder, ViewFunctionMap};
pub use self::view_name::ViewName;
pub use self::view_result::ViewResult;
pub use self::view_row::ViewRow;
pub use self::write_document_response::{DeleteDocumentResponse, PostDatabaseResponse,
                                        PutDocumentResponse};
