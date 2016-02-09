use mime;
use serde;
use std;

use dbtype;

// The `EmbeddedAttachmentBuilder` type is not ready for the public API. Its
// `digest` and `encodings` fields should be made into stronger types.

#[derive(Debug)]
#[doc(hidden)]
pub struct EmbeddedAttachmentBuilder {
    attachment: EmbeddedAttachment,
}

impl EmbeddedAttachmentBuilder {
    pub fn new(content_type: mime::Mime, digest: String, revpos: u64) -> Self {
        EmbeddedAttachmentBuilder {
            attachment: EmbeddedAttachment {
                _dummy: std::marker::PhantomData,
                content_type: content_type,
                data: None,
                _digest: digest,
                _encoded_length: None,
                _encoding: None,
                _length: None,
                revpos: revpos,
            },
        }
    }

    pub fn unwrap(self) -> EmbeddedAttachment {
        self.attachment
    }

    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.attachment.data = Some(data);
        self.attachment._length = None;
        self
    }

    pub fn length(mut self, length: u64) -> Self {
        self.attachment.data = None;
        self.attachment._length = Some(length);
        self
    }

    pub fn encoding_info(mut self, encoded_length: u64, encoding: String) -> Self {
        self.attachment._encoded_length = Some(encoded_length);
        self.attachment._encoding = Some(encoding);
        self
    }
}

/// An embedded attachment is an attachment contained within a document.
///
/// Broadly speaking, CouchDB provides two ways of accessing an attachment:
///
/// * Separately from its document, via the special attachment actions—e.g.,
///   `GET /db/doc/attachment`, and,
///
/// * Embedded within the JSON object of a document, via the document
///   actions—e.g., `GET /db/doc?attachments=true`.
///
/// By default, an embedded attachment is a stub containing only
/// meta-information about the attachment, but the attachment can be made to
/// also contain its content via the `attachments` query parameter of the
/// `GetDocument` action.
///
#[derive(Clone, Debug, PartialEq)]
pub struct EmbeddedAttachment {
    _dummy: std::marker::PhantomData<()>,

    /// The content type is the MIME type describing the attachment's content.
    pub content_type: mime::Mime,

    /// The `data` field is the attachment's full content.
    ///
    /// This field is `Some` only if the attachment is not a stub and instead
    /// contains its content. By default, an embedded attachment is a stub and
    /// does not contain its content.
    ///
    /// Note that embedded attachments encode their content using Base64 during
    /// transmission as a JSON object over HTTP. This leads to a lot of overhead
    /// for large attachments.
    ///
    pub data: Option<Vec<u8>>,

    _digest: String,
    _encoded_length: Option<u64>,
    _encoding: Option<String>,
    _length: Option<u64>,

    /// The revision position is the revision number of the attachment's
    /// document when the attachment was added.
    pub revpos: u64,
}

impl serde::Deserialize for EmbeddedAttachment {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        enum Field {
            ContentType,
            Data,
            Digest,
            EncodedLength,
            Encoding,
            Length,
            Revpos,
            Stub,
        }

        impl serde::Deserialize for Field {
            fn deserialize<D>(deserializer: &mut D) -> Result<Field, D::Error>
                where D: serde::Deserializer
            {
                struct Visitor;

                impl serde::de::Visitor for Visitor {
                    type Value = Field;

                    fn visit_str<E>(&mut self, value: &str) -> Result<Field, E>
                        where E: serde::de::Error
                    {
                        match value {
                            "content_type" => Ok(Field::ContentType),
                            "data" => Ok(Field::Data),
                            "digest" => Ok(Field::Digest),
                            "encoded_length" => Ok(Field::EncodedLength),
                            "encoding" => Ok(Field::Encoding),
                            "length" => Ok(Field::Length),
                            "revpos" => Ok(Field::Revpos),
                            "stub" => Ok(Field::Stub),
                            _ => Err(E::unknown_field(value)),
                        }
                    }
                }

                deserializer.visit(Visitor)
            }
        }

        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = EmbeddedAttachment;

            fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
                where V: serde::de::MapVisitor
            {
                let mut content_type = None;
                let mut data = None;
                let mut digest = None;
                let mut encoded_length = None;
                let mut encoding = None;
                let mut length = None;
                let mut revpos = None;

                loop {
                    match try!(visitor.visit_key()) {
                        Some(Field::ContentType) => {
                            content_type = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Data) => {
                            data = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Digest) => {
                            digest = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::EncodedLength) => {
                            encoded_length = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Encoding) => {
                            encoding = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Length) => {
                            length = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Revpos) => {
                            revpos = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Stub) => {
                            // Ignore this field.
                            try!(visitor.visit_value::<bool>());
                        }
                        None => {
                            break;
                        }
                    }
                }

                try!(visitor.end());

                let content_type = match content_type {
                    Some(dbtype::ContentType(x)) => x,
                    None => try!(visitor.missing_field("content_type")),
                };

                let data = data.map(|dbtype::Base64Blob(x)| x);

                let digest = match digest {
                    Some(x) => x,
                    None => try!(visitor.missing_field("digest")),
                };

                let revpos = match revpos {
                    Some(x) => x,
                    None => try!(visitor.missing_field("revpos")),
                };

                Ok(EmbeddedAttachment {
                    _dummy: std::marker::PhantomData,
                    content_type: content_type,
                    data: data,
                    _digest: digest,
                    _encoded_length: encoded_length,
                    _encoding: encoding,
                    _length: length,
                    revpos: revpos,
                })
            }
        }

        static FIELDS: &'static [&'static str] = &["content_type",
                                                   "data",
                                                   "digest",
                                                   "encoded_length",
                                                   "encoding",
                                                   "length",
                                                   "revpos",
                                                   "stub"];
        deserializer.visit_struct("EmbeddedAttachment", FIELDS, Visitor)
    }
}

#[cfg(test)]
mod tests {

    use serde_json;
    use std;

    use super::{EmbeddedAttachment, EmbeddedAttachmentBuilder};

    #[test]
    fn builder_basic() {

        let expected = EmbeddedAttachment {
            _dummy: std::marker::PhantomData,
            content_type: "text/plain".parse().unwrap(),
            data: None,
            _digest: "md5-iMaiC8wqiFlD2NjLTemvCQ==".to_owned(),
            _encoded_length: None,
            _encoding: None,
            _length: None,
            revpos: 11,
        };

        let got = EmbeddedAttachmentBuilder::new("text/plain".parse().unwrap(),
                                                 "md5-iMaiC8wqiFlD2NjLTemvCQ==".to_string(),
                                                 11)
                      .unwrap();

        assert_eq!(expected, got);
    }

    #[test]
    fn builder_with_data() {

        let expected = EmbeddedAttachment {
            _dummy: std::marker::PhantomData,
            content_type: "text/plain".parse().unwrap(),
            data: Some("hello".to_owned().into_bytes()),
            _digest: "md5-iMaiC8wqiFlD2NjLTemvCQ==".to_owned(),
            _encoded_length: None,
            _encoding: None,
            _length: None,
            revpos: 11,
        };

        let got = EmbeddedAttachmentBuilder::new("text/plain".parse().unwrap(),
                                                 "md5-iMaiC8wqiFlD2NjLTemvCQ==".to_string(),
                                                 11)
                      .length(5)
                      .data("hello".to_owned().into_bytes())
                      .unwrap();

        assert_eq!(expected, got);
    }

    #[test]
    fn builder_with_length() {

        let expected = EmbeddedAttachment {
            _dummy: std::marker::PhantomData,
            content_type: "text/plain".parse().unwrap(),
            data: None,
            _digest: "md5-iMaiC8wqiFlD2NjLTemvCQ==".to_owned(),
            _encoded_length: None,
            _encoding: None,
            _length: Some(5),
            revpos: 11,
        };

        let got = EmbeddedAttachmentBuilder::new("text/plain".parse().unwrap(),
                                                 "md5-iMaiC8wqiFlD2NjLTemvCQ==".to_string(),
                                                 11)
                      .data("hello".to_owned().into_bytes())
                      .length(5)
                      .unwrap();

        assert_eq!(expected, got);
    }

    #[test]
    fn builder_with_encoding_info() {

        let expected = EmbeddedAttachment {
            _dummy: std::marker::PhantomData,
            content_type: "text/plain".parse().unwrap(),
            data: None,
            _digest: "md5-iMaiC8wqiFlD2NjLTemvCQ==".to_owned(),
            _encoded_length: Some(25),
            _encoding: Some("gzip".to_owned()),
            _length: None,
            revpos: 11,
        };

        let got = EmbeddedAttachmentBuilder::new("text/plain".parse().unwrap(),
                                                 "md5-iMaiC8wqiFlD2NjLTemvCQ==".to_string(),
                                                 11)
                      .encoding_info(25, "gzip".to_owned())
                      .unwrap();

        assert_eq!(expected, got);
    }

    #[test]
    fn decode_json_ok_with_data_and_encoding_info() {

        let expected = EmbeddedAttachment {
            _dummy: std::marker::PhantomData,
            content_type: "text/plain".parse().unwrap(),
            data: Some("hello".to_owned().into_bytes()),
            _digest: "md5-iMaiC8wqiFlD2NjLTemvCQ==".to_owned(),
            _encoded_length: Some(25),
            _encoding: Some("gzip".to_owned()),
            _length: None,
            revpos: 11,
        };

        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("content_type", "text/plain")
                         .insert("data", "aGVsbG8=")
                         .insert("digest", "md5-iMaiC8wqiFlD2NjLTemvCQ==")
                         .insert("encoded_length", 25)
                         .insert("encoding", "gzip")
                         .insert("revpos", 11)
                         .unwrap();

        let source = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&source).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn decode_json_ok_as_stub() {

        let expected = EmbeddedAttachment {
            _dummy: std::marker::PhantomData,
            content_type: "text/plain".parse().unwrap(),
            data: None,
            _digest: "md5-iMaiC8wqiFlD2NjLTemvCQ==".to_owned(),
            _encoded_length: None,
            _encoding: None,
            _length: Some(5),
            revpos: 11,
        };

        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("content_type", "text/plain")
                         .insert("digest", "md5-iMaiC8wqiFlD2NjLTemvCQ==")
                         .insert("length", 5)
                         .insert("revpos", 11)
                         .insert("stub", true)
                         .unwrap();

        let source = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&source).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn decode_json_nok_content_type_missing() {

        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("digest", "md5-iMaiC8wqiFlD2NjLTemvCQ==")
                         .insert("length", 5)
                         .insert("revpos", 11)
                         .insert("stub", true)
                         .unwrap();

        let source = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<EmbeddedAttachment>(&source);
        expect_json_error_missing_field!(got, "content_type");
    }

    #[test]
    fn decode_json_nok_digest_missing() {

        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("content_type", "text/plain")
                         .insert("length", 5)
                         .insert("revpos", 11)
                         .insert("stub", true)
                         .unwrap();

        let source = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<EmbeddedAttachment>(&source);
        expect_json_error_missing_field!(got, "digest");
    }

    #[test]
    fn decode_json_nok_revpos_missing() {

        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("content_type", "text/plain")
                         .insert("digest", "md5-iMaiC8wqiFlD2NjLTemvCQ==")
                         .insert("length", 5)
                         .insert("stub", true)
                         .unwrap();

        let source = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<EmbeddedAttachment>(&source);
        expect_json_error_missing_field!(got, "revpos");
    }
}
