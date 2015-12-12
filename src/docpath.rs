use hyper;

use dbpath::DatabasePath;
use docid::DocumentId;

/// Document path—i.e., a database path paired with a document id.
///
/// A document path pairs a `DatabasePath` with a `DocumentId`—e.g., the `db`
/// and `docid` parts in the HTTP request `GET
/// http://example.com:5984/db/docid`.
///
/// `DocumentPath` provides additional type-safety over working with raw
/// strings.
///
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DocumentPath(DatabasePath, DocumentId);

impl DocumentPath {
    pub fn new<T, U>(db_path: T, doc_id: U) -> Self
        where T: Into<DatabasePath>,
              U: Into<DocumentId>
    {
        DocumentPath(db_path.into(), doc_id.into())
    }

    /// Convert the `DocumentPath` into a URI.
    pub fn into_uri(self, base_uri: hyper::Url) -> hyper::Url {

        let DocumentPath(db_path, doc_id) = self;
        let mut uri = db_path.into_uri(base_uri);

        {
            let p = uri.path_mut().unwrap();
            let doc_id_iter = doc_id.into_iter();

            // Use size hint to preallocate.
            if let Some(doc_id_size) = doc_id_iter.size_hint().1 {
                p.reserve(doc_id_size);
            }

            for s in doc_id_iter {
                p.push(s);
            }
        }

        uri
    }

    /// Return the database path part of the document path.
    pub fn database_path(&self) -> &DatabasePath {
        &self.0
    }

    /// Return the document id part of the document path.
    pub fn document_id(&self) -> &DocumentId {
        &self.1
    }
}

impl From<String> for DocumentPath {
    fn from(db_path: String) -> Self {
        DocumentPath::from(&db_path as &str)
    }
}

impl<'a> From<&'a str> for DocumentPath {
    fn from(db_path: &str) -> Self {
        let n = db_path.find('/').unwrap();
        let (db_name, doc_id) = db_path.split_at(n);
        let doc_id = &doc_id[1..];
        DocumentPath::new(db_name, doc_id)
    }
}

impl From<DocumentPath> for (DatabasePath, DocumentId) {
    fn from(doc_path: DocumentPath) -> (DatabasePath, DocumentId) {
        let DocumentPath(db_path, doc_id) = doc_path;
        (db_path, doc_id)
    }
}

#[cfg(test)]
mod tests {

    use hyper;

    use dbpath::DatabasePath;
    use docid::DocumentId;
    use super::*;

    #[test]
    fn test_document_path_from_str_ref_ok() {

        let got = DocumentPath::from("db/docid");
        let exp = DocumentPath(DatabasePath::from("db"),
                               DocumentId::Normal("docid".to_string()));
        assert_eq!(got, exp);

        let got = DocumentPath::from("db/_design/docid");
        let exp = DocumentPath(DatabasePath::from("db"),
                               DocumentId::Design("docid".to_string()));
        assert_eq!(got, exp);
    }

    #[test]
    #[should_panic]
    fn test_document_path_from_str_ref_panic() {
        DocumentPath::from("missing_doc_id");
    }

    #[test]
    fn test_document_path_from_string_ok() {

        let got = DocumentPath::from("db/docid".to_string());
        let exp = DocumentPath(DatabasePath::from("db"),
                               DocumentId::Normal("docid".to_string()));
        assert_eq!(got, exp);

        let got = DocumentPath::from("db/_design/docid".to_string());
        let exp = DocumentPath(DatabasePath::from("db"),
                               DocumentId::Design("docid".to_string()));
        assert_eq!(got, exp);
    }

    #[test]
    #[should_panic]
    fn test_document_path_from_string_panic() {
        DocumentPath::from("missing_doc_id".to_string());
    }

    #[test]
    fn test_document_path_clone() {
        let x = DocumentPath::from("foo/bar");
        let y = x.clone();
        assert_eq!(x, y);

        let x = DocumentPath::from("foo/_design/bar");
        let y = x.clone();
        assert_eq!(x, y);
    }

    #[test]
    fn test_document_path_eq() {
        let x = DocumentPath::from("foo/bar");
        let y = DocumentPath::from("foo/bar");
        assert!(x == x);
        assert!(x == y);

        let x = DocumentPath::from("foo/bar");
        let y = DocumentPath::from("foo/biz");
        assert!(x != y);

        let x = DocumentPath::from("foo/bar");
        let y = DocumentPath::from("foo/_design/bar");
        assert!(x != y);

        let x = DocumentPath::from("foo/bar");
        let y = DocumentPath::from("biz/bar");
        assert!(x != y);
    }

    #[test]
    fn test_document_path_ord() {
        let x = DocumentPath::from("foo/bar");
        let y = DocumentPath::from("biz/bar");
        assert!(y < x);

        let x = DocumentPath::from("foo/bar");
        let y = DocumentPath::from("foo/biz");
        assert!(x < y);

        let x = DocumentPath::from("foo/bar");
        let y = DocumentPath::from("foo/_design/alpha");
        assert!(x < y);
    }

    #[test]
    fn test_document_path_new() {
        let x = DocumentPath::new("foo", "_design/bar");
        let (db_path, doc_id) = x.into();
        assert_eq!(db_path, DatabasePath::from("foo"));
        assert_eq!(doc_id, DocumentId::Design("bar".to_string()));
    }

    #[test]
    fn test_document_path_into_uri() {

        // Verify: A normal URI base yields a normal database URI path.
        let base = hyper::Url::parse("http://example.com:1234").unwrap();
        let uri = DocumentPath::new("foo", DocumentId::from("_design/bar")).into_uri(base);
        let exp = hyper::Url::parse("http://example.com:1234/foo/_design/bar").unwrap();
        assert_eq!(uri, exp);

        // Verify: A URI base with a nonempty path yields a URI with the full
        // path.
        let base = hyper::Url::parse("http://example.com:1234/bar").unwrap();
        let uri = DocumentPath::new("foo", DocumentId::from("_design/bar")).into_uri(base);
        let exp = hyper::Url::parse("http://example.com:1234/bar/foo/_design/bar").unwrap();
        assert_eq!(uri, exp);
    }

    #[test]
    fn test_document_path_accessors() {
        let doc_path = DocumentPath::from("foo/bar");
        assert_eq!(*doc_path.database_path(), DatabasePath::from("foo"));
        assert_eq!(*doc_path.document_id(), DocumentId::from("bar"));
    }
}
