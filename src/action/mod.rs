mod delete_database;
mod head_database;
mod put_database;

pub use self::delete_database::DeleteDatabase;
pub use self::head_database::HeadDatabase;
pub use self::put_database::PutDatabase;

const E_ACTION_USED: &str = "Cannot use action more than once";
