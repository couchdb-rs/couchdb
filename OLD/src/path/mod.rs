#[cfg(test)]
#[macro_use]
mod test_macro;

// These modules should be private, but there's a Rust bug that prevents
// re-exporting a trait from a private module. See here:
// https://github.com/rust-lang/rust/issues/18241.
//
// If and when the bug is fixed, these modules should be made private.
//
pub mod database_path;
pub mod design_document_path;
pub mod document_path;
pub mod percent;
pub mod view_path;

pub use self::database_path::{DatabasePath, IntoDatabasePath};
pub use self::design_document_path::{DesignDocumentPath, IntoDesignDocumentPath};
pub use self::document_path::{DocumentPath, IntoDocumentPath};
pub use self::view_path::{IntoViewPath, ViewPath};
