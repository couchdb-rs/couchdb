use hyper;

/// Database meta-information.
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

/// Construct a database URI.
pub fn new_uri(base_uri: &hyper::Url, db_name: &str) -> hyper::Url {
    let mut uri = base_uri.clone();
    uri.path_mut().unwrap()[0] = db_name.to_string();
    uri
}
