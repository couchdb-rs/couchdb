use serde;
use serde_json;
use std::collections::HashMap;

use DocumentId;
use EmbeddedAttachment;
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

    /// Whether this document has been deleted.
    pub deleted: bool,

    /// Attachments to this document.
    pub attachments: HashMap<String, EmbeddedAttachment>,

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
            Attachments,
            Content(String),
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
                            "_attachments" => Ok(Field::Attachments),
                            _ => Ok(Field::Content(value.to_string())),
                        }
                    }
                }

                d.deserialize(Visitor)
            }
        }

        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = Document;

            fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
                where V: serde::de::MapVisitor
            {
                let mut attachments = None;
                let mut deleted = None;
                let mut id = None;
                let mut rev = None;
                let mut content_builder = serde_json::builder::ObjectBuilder::new();

                loop {
                    match try!(visitor.visit_key()) {
                        Some(Field::Attachments) => {
                            attachments = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Content(name)) => {
                            let value = Some(try!(visitor.visit_value::<serde_json::Value>()));
                            content_builder = content_builder.insert(name, value);
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

                let attachments = attachments.unwrap_or(HashMap::new());
                let deleted = deleted.unwrap_or(false);

                let id = match id {
                    Some(x) => x,
                    None => try!(visitor.missing_field("_id")),
                };

                let rev = match rev {
                    Some(x) => x,
                    None => try!(visitor.missing_field("_rev")),
                };


                Ok(Document {
                    attachments: attachments,
                    deleted: deleted,
                    id: id,
                    rev: rev,
                    content: content_builder.unwrap(),
                })
            }
        }

        static FIELDS: &'static [&'static str] = &["_id", "_rev"];
        d.deserialize_struct("Document", FIELDS, Visitor)
    }
}

#[cfg(test)]
mod tests {

    use serde_json;
    use std::collections::HashMap;

    use Document;
    use DocumentId;
    use Revision;
    use dbtype::EmbeddedAttachmentBuilder;

    #[test]
    fn decode_json_ok_with_attachments() {

        let expected = Document {
            attachments: {
                let mut m = HashMap::new();
                m.insert("bar".to_owned(),
                         EmbeddedAttachmentBuilder::new("text/plain".parse().unwrap(),
                                                        "md5-iMaiC8wqiFlD2NjLTemvCQ==".to_owned(),
                                                        11)
                             .length(5)
                             .unwrap());
                m
            },
            content: serde_json::builder::ObjectBuilder::new().unwrap(),
            deleted: false,
            id: DocumentId::Normal("foo".into()),
            rev: Revision::parse("42-1234567890abcdef1234567890abcdef").unwrap(),
        };

        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("_id", "foo")
                         .insert("_rev", "42-1234567890abcdef1234567890abcdef")
                         .insert_object("_attachments", |x| {
                             x.insert_object("bar", |x| {
                                 x.insert("content_type", "text/plain")
                                  .insert("digest", "md5-iMaiC8wqiFlD2NjLTemvCQ==")
                                  .insert("revpos", 11)
                                  .insert("length", 5)
                                  .insert("stub", true)
                             })
                         })
                         .unwrap();

        let source = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&source).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn decode_json_ok_as_deleted() {
        let expected = Document {
            attachments: HashMap::new(),
            content: serde_json::builder::ObjectBuilder::new().unwrap(),
            deleted: true,
            id: DocumentId::Normal("foo".into()),
            rev: Revision::parse("42-1234567890abcdef1234567890abcdef").unwrap(),
        };
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("_id", "foo")
                         .insert("_rev", "42-1234567890abcdef1234567890abcdef")
                         .insert("_deleted", true)
                         .unwrap();
        let source = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&source).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn decode_json_ok_with_content() {
        let expected = Document {
            attachments: HashMap::new(),
            content: serde_json::builder::ObjectBuilder::new()
                         .insert("bar", 17)
                         .insert("qux", "ipsum lorem")
                         .unwrap(),
            deleted: false,
            id: DocumentId::Normal("foo".into()),
            rev: Revision::parse("42-1234567890abcdef1234567890abcdef").unwrap(),
        };
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("_id", "foo")
                         .insert("_rev", "42-1234567890abcdef1234567890abcdef")
                         .insert("bar", 17)
                         .insert("qux", "ipsum lorem")
                         .unwrap();
        let source = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&source).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn decode_json_nok_with_no_id_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("_rev", "42-1234567890abcdef1234567890abcdef")
                         .unwrap();
        let source = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Document>(&source);
        expect_json_error_missing_field!(got, "_id");
    }

    #[test]
    fn decode_json_nok_with_no_rev_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("_id", "foo")
                         .unwrap();
        let source = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Document>(&source);
        expect_json_error_missing_field!(got, "_rev");
    }
}
