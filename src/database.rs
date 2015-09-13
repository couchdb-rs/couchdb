/// Database meta-information, as returned by the CouchDB server.
#[derive(Debug)]
pub struct Database {
    // TODO: Add more fields as needed.

    /// Database name.
    pub db_name: String,

    /// Number of documents in the database.
    pub doc_count: u64,

    /// Number of deleted documents.
    pub doc_del_count: u64,
}
