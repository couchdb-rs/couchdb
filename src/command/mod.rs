//! Sub-module for individual command types.

mod delete_database;
mod delete_document;
mod get_all_databases;
mod get_database;
mod get_document;
mod get_view;
mod head_database;
mod head_document;
mod put_database;
mod put_document;

pub use self::delete_database::{DeleteDatabase, new_delete_database};
pub use self::delete_document::{DeleteDocument, new_delete_document};
pub use self::get_all_databases::{GetAllDatabases, new_get_all_databases};
pub use self::get_database::{GetDatabase, new_get_database};
pub use self::get_document::{GetDocument, new_get_document};
pub use self::get_view::{GetView, new_get_view};
pub use self::head_database::{HeadDatabase, new_head_database};
pub use self::head_document::{HeadDocument, new_head_document};
pub use self::put_database::{PutDatabase, new_put_database};
pub use self::put_document::{PutDocument, new_put_document};
