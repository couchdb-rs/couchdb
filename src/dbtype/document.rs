use serde;
use serde_json;
use std;

use DocumentId;
use Error;
use Revision;
use error::DecodeErrorKind;
use json;

/// Document, including both meta-information and application-defined content.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Document<T: serde::Deserialize> {
    /// Id of this document.
    pub id: DocumentId,

    /// Revision of this document.
    pub revision: Revision,

    /// Application-defined content of this document.
    pub content: T,
}

impl<T> Document<T> where T: serde::Deserialize
{
    pub fn from_reader<R: std::io::Read>(r: R) -> Result<Self, Error> {

        // The CouchDB API document resource mixes at the top level of the
        // document JSON object both meta fields (e.g., `_id` and `_rev`) and
        // application-defined fields. Serde does not provide a feature for
        // directly deserializing such a doubly typed JSON object. We work
        // around this by deserializing to a generic Value instance, then
        // selectively dividing the fields appropriately.

        fn make_error(what: &'static str) -> Error {
            Error::Decode(DecodeErrorKind::InvalidDocument { what: what })
        }

        let mut top = try!(json::decode_json::<_, serde_json::Value>(r));

        let (doc_id, rev) = {

            let mut dot = match top.as_object_mut() {
                Some(x) => x,
                None => {
                    return Err(make_error("Document is not a JSON object"));
                }
            };

            let rev = match dot.remove("_rev") {
                Some(x) => {
                    match x {
                        serde_json::Value::String(x) => {
                            // TODO: Should reuse the error's description?
                            try!(Revision::parse(&x).map_err(|_| {
                                make_error("The `_rev` field is not a valid revision")
                            }))
                        }
                        _ => {
                            return Err(make_error("The `_rev` field is not a string"));
                        }
                    }
                }
                None => {
                    return Err(make_error("The `_rev` field is missing"));
                }
            };

            let doc_id = match dot.remove("_id") {
                Some(x) => {
                    match x {
                        serde_json::Value::String(x) => DocumentId::from(x),
                        _ => {
                            return Err(make_error("The `_id` field is not a string"));
                        }
                    }
                }
                None => {
                    return Err(make_error("The `_id` field is missing"));
                }
            };

            // Ignore any attachment info.
            dot.remove("_attachments");

            (doc_id, rev)
        };

        let content = try!(serde_json::from_value(top)
                               .map_err(|e| Error::Decode(DecodeErrorKind::Serde { cause: e })));

        let doc = Document {
            id: doc_id,
            revision: rev,
            content: content,
        };

        Ok(doc)
    }
}

#[cfg(test)]
mod tests {

    macro_rules! expect_decode_error {
        ($result:ident) => {
            match $result {
                Ok(..) => {
                    panic!("Got unexpected OK result");
                }
                Err(ref e) => {
                    match *e {
                        Error::Decode(ref kind) => {
                            match *kind {
                                DecodeErrorKind::InvalidDocument{..} => (),
                                _ => {
                                    panic!("Got unexpected error kind: {}", e);
                                }
                            }
                        }
                        _ => {
                            panic!("Got unexpected error: {}", e);
                        }
                    }
                }
            }
        }
    }

    use serde_json;

    use Document;
    use DocumentId;
    use Error;
    use Revision;
    use error::DecodeErrorKind;

    #[test]
    fn document_from_reader_with_all_fields() {
        let expected = Document::<serde_json::Value> {
            id: DocumentId::Normal("foo".into()),
            revision: Revision::parse("42-1234567890abcdef1234567890abcdef").unwrap(),
            content: serde_json::builder::ObjectBuilder::new()
                         .insert("bar", 42)
                         .insert("qux", "baz")
                         .unwrap(),
        };
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("_id", "foo")
                         .insert("_rev", "42-1234567890abcdef1234567890abcdef")
                         .insert("bar", 42)
                         .insert("qux", "baz")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = Document::from_reader(&s.into_bytes()[..]).unwrap();
        assert_eq!(expected, got);

    }

    #[test]
    fn document_from_reader_with_no_id_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("_rev", "42-1234567890abcdef1234567890abcdef")
                         .insert("bar", 42)
                         .insert("qux", "baz")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = Document::<serde_json::Value>::from_reader(&s.into_bytes()[..]);
        expect_decode_error!(got);
    }

    #[test]
    fn document_from_reader_with_bad_id_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("_id", 17)
                         .insert("_rev", "42-1234567890abcdef1234567890abcdef")
                         .insert("bar", 42)
                         .insert("qux", "baz")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = Document::<serde_json::Value>::from_reader(&s.into_bytes()[..]);
        expect_decode_error!(got);
    }

    #[test]
    fn document_from_reader_with_no_rev_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("_id", "foo")
                         .insert("bar", 42)
                         .insert("qux", "baz")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = Document::<serde_json::Value>::from_reader(&s.into_bytes()[..]);
        expect_decode_error!(got);
    }

    #[test]
    fn document_from_reader_with_bad_rev_field_wrong_type() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("_id", "foo")
                         .insert("_rev", 17)
                         .insert("bar", 42)
                         .insert("qux", "baz")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = Document::<serde_json::Value>::from_reader(&s.into_bytes()[..]);
        expect_decode_error!(got);
    }

    #[test]
    fn document_from_reader_with_bad_rev_field_parse_error() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("_id", "foo")
                         .insert("_rev", "bad_revision")
                         .insert("bar", 42)
                         .insert("qux", "baz")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = Document::<serde_json::Value>::from_reader(&s.into_bytes()[..]);
        expect_decode_error!(got);
    }

    #[test]
    fn document_from_reader_with_attachments_field() {
        let expected = Document::<serde_json::Value> {
            id: DocumentId::Normal("foo".into()),
            revision: Revision::parse("42-1234567890abcdef1234567890abcdef").unwrap(),
            content: serde_json::builder::ObjectBuilder::new()
                         .insert("bar", 42)
                         .insert("qux", "baz")
                         .unwrap(),
        };
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("_id", "foo")
                         .insert("_rev", "42-1234567890abcdef1234567890abcdef")
                         .insert_object("_attachments", |x| {
                             x.insert_object("attachment_1", |x| {
                                 x.insert("content-type", "text/plain")
                                  .insert("revpos", 2)
                                  .insert("digest", "md5-abcdefghijklmnopqrstuv==")
                                  .insert("length", 17)
                                  .insert("stub", true)
                             })
                         })
                         .insert("bar", 42)
                         .insert("qux", "baz")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = Document::from_reader(&s.into_bytes()[..]).unwrap();
        assert_eq!(expected, got);

    }
}
