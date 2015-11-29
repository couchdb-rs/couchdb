use hyper;
use serde;
use std;

use dbpath::DatabasePath;

/// Document identifier.
///
/// A document id specifies a document's type and name. For example, given the
/// HTTP request `GET http://example.com:5984/db/_design/foo`, the document id
/// part is `_design/foo` and specifies a design document with the name `foo`.
///
/// There are three types of documents: normal, design (i.e., `_design`), and
/// local (i.e., `_local`). Each type is expressed as an enum variant that owns
/// a `String` specifying the document name.
///
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DocumentId {

    /// Normal document—i.e., neither a design document nor a local document.
    Normal(String),

    /// Design document (i.e., `_design`).
    Design(String),

    /// Local document (i.e., `_local`).
    Local(String),
}

impl<'a> From<&'a str> for DocumentId {
    fn from(doc_id: &str) -> Self {
        let design = "_design/";
        let local = "_local/";
        if doc_id.starts_with(design) {
            DocumentId::Design(doc_id[design.len() ..].to_string())
        } else if doc_id.starts_with(local) {
            DocumentId::Local(doc_id[local.len() ..].to_string())
        } else {
            DocumentId::Normal(doc_id.to_string())
        }
    }
}

impl From<String> for DocumentId {
    fn from(doc_id: String) -> Self {
        // TODO: Move doc_id string into new DocumentId if a normal document.
        (&doc_id as &str).into()
    }
}

impl IntoIterator for DocumentId {
    type Item = String;
    type IntoIter = DocumentIdIterator;
    fn into_iter(self) -> DocumentIdIterator {
        match self {
            DocumentId::Normal(s) => DocumentIdIterator::Root(s),
            DocumentId::Design(s) => DocumentIdIterator::Prefix("_design", s),
            DocumentId::Local(s) => DocumentIdIterator::Prefix("_local", s),
        }
    }
}

impl serde::Serialize for DocumentId {
    fn serialize<S>(&self, serializer: &mut S)
        -> Result<(), S::Error> where S: serde::Serializer
    {
        match *self {
            DocumentId::Normal(ref s) => serializer.visit_str(s),
            DocumentId::Design(ref s) => {
                let s = format!("_design/{}", s);
                serializer.visit_str(&s)
            },
            DocumentId::Local(ref s) => {
                let s = format!("_local/{}", s);
                serializer.visit_str(&s)
            },
        }
    }
}

impl serde::Deserialize for DocumentId {
    fn deserialize<D>(deserializer: &mut D)
        -> Result<Self, D::Error> where D: serde::Deserializer
    {
        struct Visitor;

        impl serde::de::Visitor for Visitor {

            type Value = DocumentId;

            fn visit_str<E>(&mut self, v: &str)
                -> Result<Self::Value, E> where E: serde::de::Error
            {
                Ok(DocumentId::from(v))
            }
        }

        deserializer.visit(Visitor)
    }
}

/// Document identifier iterator.
///
/// `DocumentIdIterator` iterates through the URI path components of a document
/// id. Normal documents have one URI path component—the document name—whereas
/// design documents and local documents have two components—a type prefix
/// (`_design` or `_local`) and a document name.
///
pub enum DocumentIdIterator {
    Prefix(&'static str, String),
    Root(String),
    Done,
}

impl Iterator for DocumentIdIterator {

    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let (item, next) = match *self {
            DocumentIdIterator::Prefix(prefix, ref mut id_root) => {
                let id_root = std::mem::replace(id_root, String::new());
                (Some(prefix.to_string()), DocumentIdIterator::Root(id_root))
            },
            DocumentIdIterator::Root(ref mut id_root) => {
                let id_root = std::mem::replace(id_root, String::new());
                (Some(id_root), DocumentIdIterator::Done)
            },
            DocumentIdIterator::Done => {
                (None, DocumentIdIterator::Done)
            }
        };
        *self = next;
        item
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match *self {
            DocumentIdIterator::Prefix(..) => (2, Some(2)),
            DocumentIdIterator::Root(..) => (1, Some(1)),
            DocumentIdIterator::Done => (0, Some(0)),
        }
    }
}

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

    pub fn new<T, U>(db_path: T, doc_id: U)
        -> Self where T: Into<DatabasePath>, U: Into<DocumentId>
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
    use serde_json;

    use dbpath::DatabasePath;
    use super::*;

    #[test] 
    fn test_document_id_iterator() {

        let got = DocumentIdIterator::Prefix("_design", "foo".to_string()).collect::<Vec<String>>();
        assert_eq!(got, vec!["_design", "foo"]);

        // Verify the size hint's upper bound is correct. Higher-level code uses
        // the upper bound for preallocating a URI path vector.

        let got = DocumentIdIterator::Prefix("_design", "foo".to_string()).size_hint();
        assert_eq!(got.1, Some(2));

        let got = DocumentIdIterator::Root("foo".to_string()).size_hint();
        assert_eq!(got.1, Some(1));
    }

    #[test]
    fn test_document_id_clone() {

        let x = DocumentId::Normal("foo".to_string());
        assert_eq!(x, x.clone());

        let x = DocumentId::Design("foo".to_string());
        assert_eq!(x, x.clone());

        let x = DocumentId::Local("foo".to_string());
        assert_eq!(x, x.clone());
    }

    #[test]
    fn test_document_id_eq() {

        let x = DocumentId::Normal("foo".to_string());
        let y = DocumentId::Normal("bar".to_string());
        assert!(x == x);
        assert!(x != y);

        let y = DocumentId::Design("bar".to_string());
        assert!(x != y);
        let x = DocumentId::Design("foo".to_string());
        assert!(x != y);
        assert!(x == x);

        let x = DocumentId::Local("bar".to_string());
        assert!(x != y);
        let y = DocumentId::Local("foo".to_string());
        assert!(x != y);
        assert!(x == x);
    }

    #[test]
    fn test_document_id_ord() {

        let x = DocumentId::Normal("foo".to_string());
        let y = DocumentId::Normal("bar".to_string());
        assert!(y < x);
        assert!(x <= x);

        let x = DocumentId::Normal("foo".to_string());
        let y = DocumentId::Design("bar".to_string());
        assert!(x < y);
        assert!(x <= x);
    }

    #[test]
    fn test_document_id_from_str_ref() {

        let doc_id = DocumentId::from("foo");
        assert_eq!(doc_id, DocumentId::Normal("foo".to_string()));

        let doc_id = DocumentId::from("_design/foo");
        assert_eq!(doc_id, DocumentId::Design("foo".to_string()));

        let doc_id = DocumentId::from("_local/foo");
        assert_eq!(doc_id, DocumentId::Local("foo".to_string()));
    }

    #[test]
    fn test_document_id_from_string() {

        let doc_id = DocumentId::from("foo".to_string());
        assert_eq!(doc_id, DocumentId::Normal("foo".to_string()));

        let doc_id = DocumentId::from("_design/foo".to_string());
        assert_eq!(doc_id, DocumentId::Design("foo".to_string()));

        let doc_id = DocumentId::from("_local/foo".to_string());
        assert_eq!(doc_id, DocumentId::Local("foo".to_string()));
    }

    #[test]
    fn test_document_id_iteration() {

        let doc_id = DocumentId::from("foobar");
        let iter = doc_id.into_iter();
        assert_eq!((1, Some(1)), iter.size_hint());
        assert_eq!(iter.collect::<Vec<String>>(), vec!["foobar"]
        );

        let doc_id = DocumentId::from("_design/foobar");
        let iter = doc_id.into_iter();
        assert_eq!((2, Some(2)), iter.size_hint());
        assert_eq!(iter.collect::<Vec<String>>(), vec!["_design", "foobar"]);

        let doc_id = DocumentId::from("_local/foobar");
        let iter = doc_id.into_iter();
        assert_eq!((2, Some(2)), iter.size_hint());
        assert_eq!(iter.collect::<Vec<String>>(), vec!["_local", "foobar"]);
    }

    #[test]
    fn test_document_id_serialization() {

        let pre = DocumentId::from("foo");
        let j = serde_json::to_string(&pre).unwrap();
        let post = serde_json::from_str::<DocumentId>(&j).unwrap();
        assert_eq!(post, DocumentId::from("foo"));

        let pre = DocumentId::from("_design/foo");
        let j = serde_json::to_string(&pre).unwrap();
        let post = serde_json::from_str::<DocumentId>(&j).unwrap();
        assert_eq!(post, DocumentId::from("_design/foo"));

        let pre = DocumentId::from("_local/foo");
        let j = serde_json::to_string(&pre).unwrap();
        let post = serde_json::from_str::<DocumentId>(&j).unwrap();
        assert_eq!(post, DocumentId::from("_local/foo"));
    }

    #[test]
    fn test_document_path_from_str_ref_ok() {

        let got = DocumentPath::from("db/docid");
        let exp = DocumentPath(
            DatabasePath::from("db"),
            DocumentId::Normal("docid".to_string()));
        assert_eq!(got, exp);

        let got = DocumentPath::from("db/_design/docid");
        let exp = DocumentPath(
            DatabasePath::from("db"),
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
        let exp = DocumentPath(
            DatabasePath::from("db"),
            DocumentId::Normal("docid".to_string()));
        assert_eq!(got, exp);

        let got = DocumentPath::from("db/_design/docid".to_string());
        let exp = DocumentPath(
            DatabasePath::from("db"),
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
        let uri = DocumentPath::new("foo", DocumentId::from("_design/bar"))
            .into_uri(base);
        let exp = hyper::Url::parse("http://example.com:1234/foo/_design/bar").unwrap();
        assert_eq!(uri, exp);

        // Verify: A URI base with a nonempty path yields a URI with the full
        // path.
        let base = hyper::Url::parse("http://example.com:1234/bar").unwrap();
        let uri = DocumentPath::new("foo", DocumentId::from("_design/bar"))
            .into_uri(base);
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
