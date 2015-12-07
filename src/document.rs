use serde;
use serde_json;
use std;

use dbpath::DatabasePath;
use docid::DocumentId;
use docpath::DocumentPath;
use error::{DecodeKind, Error};
use revision::Revision;
use transport;

/// Document, including meta-information and content.
#[derive(Debug)]
pub struct Document<T: serde::Deserialize> {
    pub path: DocumentPath,
    pub revision: Revision,
    pub content: T,
}

impl<T> Document<T> where T: serde::Deserialize {
    pub fn from_reader<R>(r: R, db_path: DatabasePath) -> Result<Self, Error>
        where R: std::io::Read
    {
        // The CouchDB API document resource mixes at the top level of the same
        // JSON object both meta fields (e.g., `id` and `rev`) and
        // application-defined fields. Serde does not provide a feature for
        // directly deserializing a doubly typed JSON object. We work around
        // this by deserializing to a generic Value instance, then selectively
        // dividing the fields appropriately.

        fn make_error(what: &'static str) -> Error {
            Error::Decode { kind: DecodeKind::InvalidDocument { what: what } }
        }

        let mut top = try!(transport::decode_json::<_, serde_json::Value>(r));

        let (id, rev) = {

            let mut dot = match top.as_object_mut() {
                Some(x) => x,
                None => { return Err(make_error("Document is not a JSON object")); },
            };

            let rev = match dot.remove("_rev") {
                Some(x) => match x {
                    serde_json::Value::String(x) => Revision::from(x),
                    _ => { return Err(make_error("The `_rev` field is not a string")); },
                },
                None => { return Err(make_error("The `_rev` field is missing")); },
            };

            let id = match dot.remove("_id") {
                Some(x) => match x {
                    serde_json::Value::String(x) => { DocumentId::from(x) },
                    _ => { return Err(make_error("The `_id` field is not a string")); },
                },
                None => { return Err(make_error("The `_id` field is missing")); },
            };

            (id, rev)
        };

        let content = try!(
            serde_json::from_value(top)
                .map_err(|e| { Error::Decode { kind: DecodeKind::Serde { cause: e } } })
        );

        let doc = Document {
            path: DocumentPath::new(db_path, id),
            revision: rev,
            content: content,
        };

        Ok(doc)
    }
}

#[cfg(test)]
mod tests {

    use serde_json;

    use super::*;
    use dbpath::DatabasePath;
    use docpath::DocumentPath;
    use revision::Revision;

    #[test]
    fn test_deserialization() {

        let db_path = DatabasePath::from("db");

        let exp_content = serde_json::builder::ObjectBuilder::new()
            .insert("foo", 42)
            .insert("bar", "yep")
            .unwrap();

        // Verify: All fields are present.

        let s = r#"{
            "_id": "docid",
            "_rev": "1-abcd",
            "foo": 42,
            "bar": "yep"
        }"#;

        let got: Document<serde_json::Value> =
            Document::from_reader(s.as_bytes(), db_path.clone()).unwrap();
        assert_eq!(got.path, DocumentPath::from("db/docid"));
        assert_eq!(got.revision, Revision::from("1-abcd"));
        assert_eq!(got.content, exp_content);

        // Verify: JSON string is not an object.

        let s = r#"["stuff"]"#;

        Document::<serde_json::Value>::from_reader(s.as_bytes(), db_path.clone()).unwrap_err();

        // Verify: Missing the "_id" field.

        let s = r#"{
            "_rev": "1-abcd",
            "foo": 42,
            "bar": "yep"
        }"#;

        Document::<serde_json::Value>::from_reader(s.as_bytes(), db_path.clone()).unwrap_err();

        // Verify: The "_id" field is not a string.

        let s = r#"{
            "_id": 17,
            "_rev": "1-abcd",
            "foo": 42,
            "bar": "yep"
        }"#;

        Document::<serde_json::Value>::from_reader(s.as_bytes(), db_path.clone()).unwrap_err();

        // Verify: Missing the "_rev" field.

        let s = r#"{
            "_id": "docid",
            "foo": 42,
            "bar": "yep"
        }"#;

        Document::<serde_json::Value>::from_reader(s.as_bytes(), db_path.clone()).unwrap_err();

        // Verify: The "_rev" field is not a string.

        let s = r#"{
            "_id": "docid",
            "_rev": 17,
            "foo": 42,
            "bar": "yep"
        }"#;

        Document::<serde_json::Value>::from_reader(s.as_bytes(), db_path.clone()).unwrap_err();
    }
}
