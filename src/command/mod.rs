//! Sub-module for individual command types.
//!
//! Applications should not need to access the `command` module directly but
//! instead use a `Client` instead to construct individual commands.

mod delete_database;
mod delete_document;
mod get_all_databases;
mod get_database;
mod get_document;
mod get_view;
mod head_database;
mod head_document;
mod post_to_database;
mod put_database;
mod put_document;

pub use self::delete_database::DeleteDatabase;
pub use self::delete_document::DeleteDocument;
pub use self::get_all_databases::GetAllDatabases;
pub use self::get_database::GetDatabase;
pub use self::get_document::GetDocument;
pub use self::get_view::GetView;
pub use self::head_database::HeadDatabase;
pub use self::head_document::HeadDocument;
pub use self::post_to_database::PostToDatabase;
pub use self::put_database::PutDatabase;
pub use self::put_document::PutDocument;
