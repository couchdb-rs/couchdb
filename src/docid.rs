use serde;
use std;

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

#[cfg(test)]
mod tests {

    use serde_json;

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

        // Verify: Document ids are case-sensitive.
        let x = DocumentId::Normal("foo".to_string());
        let y = DocumentId::Normal("FOO".to_string());
        assert!(x != y);
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

        // Verify: Document ids are case-sensitive.
        let x = DocumentId::Normal("foo".to_string());
        let y = DocumentId::Normal("FOO".to_string());
        assert!(y < x);
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
}
