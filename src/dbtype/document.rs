use serde;
use serde_json;

use DocumentId;
use Error;
use Revision;
use error::DecodeErrorKind;

/// Document, including both meta-information and application-defined content.
///
/// Applications construct documents indirectly, by retrieving them from the
/// CouchDB server. See the `Client::get_document` method for more information.
///
#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    /// Id of this document.
    pub id: DocumentId,

    /// Revision of this document.
    pub rev: Revision,

    /// Whether the document has been deleted.
    pub deleted: bool,

    content: serde_json::Value,
}

impl Document {
    /// Converts self into the document's content, decoding from JSON to do so.
    pub fn into_content<T: serde::Deserialize>(self) -> Result<T, Error> {
        serde_json::from_value(self.content)
            .map_err(|e| Error::Decode(DecodeErrorKind::Serde { cause: e }))
    }
}

impl serde::Deserialize for Document {
    fn deserialize<D>(d: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        enum Field {
            Content(String),
            Ignored,
            Deleted,
            Id,
            Rev,
        }

        impl serde::Deserialize for Field {
            fn deserialize<D>(d: &mut D) -> Result<Field, D::Error>
                where D: serde::Deserializer
            {
                struct Visitor;

                impl serde::de::Visitor for Visitor {
                    type Value = Field;

                    fn visit_str<E>(&mut self, value: &str) -> Result<Field, E>
                        where E: serde::de::Error
                    {
                        match value {
                            "_deleted" => Ok(Field::Deleted),
                            "_id" => Ok(Field::Id),
                            "_rev" => Ok(Field::Rev),
                            "_attachments" => Ok(Field::Ignored),
                            _ => Ok(Field::Content(value.to_string())),
                        }
                    }
                }

                d.visit(Visitor)
            }
        }

        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = Document;

            fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
                where V: serde::de::MapVisitor
            {
                let mut deleted = None;
                let mut id = None;
                let mut rev = None;
                let mut content_builder = serde_json::builder::ObjectBuilder::new();

                loop {
                    match try!(visitor.visit_key()) {
                        Some(Field::Content(name)) => {
                            let value = Some(try!(visitor.visit_value::<serde_json::Value>()));
                            content_builder = content_builder.insert(name, value);
                        }
                        Some(Field::Ignored) => {
                            try!(visitor.visit_value::<serde_json::Value>());
                        }
                        Some(Field::Deleted) => {
                            deleted = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Id) => {
                            id = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Rev) => {
                            rev = Some(try!(visitor.visit_value()));
                        }
                        None => {
                            break;
                        }
                    }
                }

                try!(visitor.end());

                let deleted = deleted.unwrap_or(false);

                let id = match id {
                    Some(x) => x,
                    None => try!(visitor.missing_field("id")),
                };

                let rev = match rev {
                    Some(x) => x,
                    None => try!(visitor.missing_field("rev")),
                };


                Ok(Document {
                    deleted: deleted,
                    id: id,
                    rev: rev,
                    content: content_builder.unwrap(),
                })
            }
        }

        static FIELDS: &'static [&'static str] = &["_id", "_rev"];
        d.visit_struct("Document", FIELDS, Visitor)
    }
}

#[cfg(test)]
mod tests {

    use serde_json;

    use Document;
    use DocumentId;
    use Revision;

    #[test]
    fn deserialization_ok_with_all_fields() {
        let expected = Document {
            id: DocumentId::Normal("foo".into()),
            rev: Revision::parse("42-1234567890abcdef1234567890abcdef").unwrap(),
            deleted: true,
            content: serde_json::builder::ObjectBuilder::new().unwrap(),
        };
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("_id", "foo")
                         .insert("_rev", "42-1234567890abcdef1234567890abcdef")
                         .insert("_deleted", true)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn deserialization_ok_with_no_deleted_field() {
        let expected = Document {
            id: DocumentId::Normal("foo".into()),
            rev: Revision::parse("42-1234567890abcdef1234567890abcdef").unwrap(),
            deleted: false,
            content: serde_json::builder::ObjectBuilder::new().unwrap(),
        };
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("_id", "foo")
                         .insert("_rev", "42-1234567890abcdef1234567890abcdef")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn deserialization_ok_with_attachments_field() {
        let expected = Document {
            id: DocumentId::Normal("foo".into()),
            rev: Revision::parse("42-1234567890abcdef1234567890abcdef").unwrap(),
            deleted: false,
            content: serde_json::builder::ObjectBuilder::new().unwrap(),
        };
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("_id", "foo")
                         .insert("_rev", "42-1234567890abcdef1234567890abcdef")
                         .insert_object("_attachments", |x| {
                             x.insert_object("bar", |x| {
                                 x.insert("content_type", "text/plain")
                                  .insert("revpos", 3)
                                  .insert("digest", "md5-ABCDEFGHIJKLMNOPQRSTUV==")
                                  .insert("length", 17)
                                  .insert("stub", true)
                             })
                         })
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        println!("I: {}", s);
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn deserialization_ok_with_content() {
        let expected = Document {
            id: DocumentId::Normal("foo".into()),
            rev: Revision::parse("42-1234567890abcdef1234567890abcdef").unwrap(),
            deleted: false,
            content: serde_json::builder::ObjectBuilder::new()
                         .insert("bar", 17)
                         .insert("qux", "ipsum lorem")
                         .unwrap(),
        };
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("_id", "foo")
                         .insert("_rev", "42-1234567890abcdef1234567890abcdef")
                         .insert("bar", 17)
                         .insert("qux", "ipsum lorem")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn deserialization_nok_with_no_id_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("_rev", "42-1234567890abcdef1234567890abcdef")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Document>(&s);
        expect_json_error_missing_field!(got, "_id");
    }

    #[test]
    fn deserialization_nok_with_no_rev_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("_id", "foo")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Document>(&s);
        expect_json_error_missing_field!(got, "_rev");
    }
}
