//! The `attachment` module provides types for working with CouchDB document
//! attachments.

use {Error, base64, serde, std};
use mime::Mime;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

/// `Attachment` is a state-aware representation of a CouchDB document
/// attachment.
///
/// # Summary
///
/// * `Attachment` maintains state about whether it already exists on the server
///   (i.e., _originates from the server_) or not (i.e., _originates from the
///   client_).
///
/// * A CouchDB attachment may be stubbed, meaning it has no content but instead
///   is a placeholder for attachment content that already exists on the server.
///
/// * An `Attachment` instance deserialized from JSON is server-originating.
///
/// * An `Attachment` instance constructed from content (e.g., via the
///   `Attachment::new` method) is client-originating.
///
/// * When serialized to JSON, a server-originating `Attachment` instance emits
///   a stub object—regardless whether the `Attachment` instance is a stub.
///
/// * When serialized to JSON, a client-originating `Attachment` instance emits
///   a non-stub object that uses base64-encoding to encapsulate its content.
///
/// * `Attachment` supports conversion into a stub, which is useful when either:
///
///     * Updating a document but not making changes to its existing
///       attachments, or,
///     * Uploading attachments via multipart-encoding.
///
/// # Remarks
///
/// CouchDB document attachments are versatile but tricky. Generally speaking,
/// there are several things the application must get right:
///
/// * When updating a document on the server, the client must send existing
///   attachments—either stubbed or containing full content—otherwise the server
///   will delete missing attachments as part of the document update.
///
/// * When enclosing attachment content directly in JSON, the content must be
///   base64-encoded.
///
/// * To prevent sending redundant data to the server, the application
///   serializes unmodified attachments as stubs (via `"stub": true` within the
///   attachment object).
///
/// * When using multipart-encoding in lieu of base64-encoding, the application
///   serializes attachments into yet another form (via `"follows": true` within
///   the attachment object).
///
/// # TODO
///
/// * Add a means for applications to construct server-originating attachments
///   from multipart data.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attachment {
    content_type: Mime,
    inner: Inner,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Inner {
    ServerOrigin {
        content: Content,
        digest: Digest,
        encoding: Option<Encoding>,
        revpos: u64,
    },
    ClientOrigin { content: Vec<u8> },
    Follows { content_length: u64 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Content {
    WithBytes(Vec<u8>),
    WithLength(u64),
}

/// `Digest` is a hashed sum of an attachment's content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Digest {
    #[doc(hidden)]
    Md5 { value: Vec<u8> },
    #[doc(hidden)]
    Other { name: String, value: Vec<u8> },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum EncodingCodec {
    Gzip,
    Other(String),
}

/// `Encoding` contains information about the compression the CouchDB server
/// uses to store an attachment's content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Encoding {
    length: u64,
    codec: EncodingCodec,
}

impl Attachment {
    /// Constructs a new attachment.
    ///
    /// The newly constructed `Attachment` is internally marked as having
    /// originated from the client and therefore, when serialized as JSON, will
    /// include all content as a base64-encoded string (as opposed to being
    /// stubbed out). This may incur significant overhead when sent to the
    /// CouchDB server within an enclosed document because base64-encoding uses
    /// four encoded bytes to represent every three encoded bytes.
    ///
    /// One way to reduce base64  overhead is to stub the attachment and instead
    /// use multipart-encoding when uploading the document. See the [CouchDB
    /// documentation](http://docs.couchdb.org/en/2.0.0/api/document/common.html#attachments)
    /// for details.
    ///
    pub fn new(content_type: Mime, content: Vec<u8>) -> Self {
        Attachment {
            content_type: content_type,
            inner: Inner::ClientOrigin { content: content },
        }
    }

    /// Returns whether the attachment originates from the server.
    pub fn is_server_origin(&self) -> bool {
        match self.inner {
            Inner::ServerOrigin { .. } => true,
            Inner::ClientOrigin { .. } => false,
            Inner::Follows { .. } => false,
        }
    }

    /// Returns whether the attachment originates from the client.
    pub fn is_client_origin(&self) -> bool {
        match self.inner {
            Inner::ServerOrigin { .. } => false,
            Inner::ClientOrigin { .. } => true,
            Inner::Follows { .. } => false,
        }
    }

    /// Borrows the attachment's content MIME type.
    pub fn content_type(&self) -> &Mime {
        &self.content_type
    }

    /// Borrows the attachment's content, if available.
    ///
    /// Content is available if and only if:
    ///
    /// * The attachment originates from the client, or,
    /// * The attachment originates from the server and is not a stub.
    ///
    pub fn content(&self) -> Option<&[u8]> {
        match self.inner {
            Inner::ServerOrigin { content: Content::WithBytes(ref bytes), .. } => Some(bytes),
            Inner::ServerOrigin { content: Content::WithLength(_), .. } => None,
            Inner::ClientOrigin { ref content } => Some(content),
            Inner::Follows { .. } => None,
        }
    }

    /// Returns the size of the attachment's content, in bytes.
    pub fn content_length(&self) -> u64 {
        match self.inner {
            Inner::ServerOrigin { content: Content::WithBytes(ref bytes), .. } => bytes.len() as u64,
            Inner::ServerOrigin { content: Content::WithLength(length), .. } => length,
            Inner::ClientOrigin { ref content } => content.len() as u64,
            Inner::Follows { content_length } => content_length,
        }
    }

    /// Constructs a stubbed copy of the attachment.
    ///
    /// A stubbed attachment contains no content, instead marking itself as a
    /// stub and relying on the CouchDB server to already have the content if
    /// the attachment is sent to the server within its enclosing document.
    ///
    /// Hence, only an attachment that originates from the server can be
    /// stubbed. Otherwise, content would be lost, which this method prevents by
    /// instead returning `None` if the attachment originates from the client.
    ///
    /// **Note:** The stub retains all other information about the attachment,
    ///  such as content type and digest.
    ///
    pub fn to_stub(&self) -> Option<Attachment> {
        match self.inner {
            Inner::ServerOrigin {
                ref content,
                ref digest,
                ref encoding,
                ref revpos,
            } => {
                Some(Attachment {
                    content_type: self.content_type.clone(),
                    inner: Inner::ServerOrigin {
                        content: content.to_length_only(),
                        digest: digest.clone(),
                        encoding: encoding.clone(),
                        revpos: revpos.clone(),
                    },
                })
            }
            _ => None,
        }
    }

    /// Constructs a stubbed copy of the attachment suitable for
    /// multipart-encoding.
    ///
    /// The returned attachment loses all information about the attachment
    /// except for its content type and content length. The intention is for the
    /// application to:
    ///
    /// 1. Serialize the attachment stub within an enclosed document, as JSON,
    /// and,
    ///
    /// 2. Send the attachment content as multipart data, within the same HTTP
    /// request.
    ///
    /// See the [CouchDB
    /// documentation](http://docs.couchdb.org/en/2.0.0/api/document/common.html#creating-multiple-attachments)
    /// for details.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate couchdb;
    /// extern crate mime;
    /// extern crate serde_json;
    ///
    /// let att = couchdb::Attachment::new(
    ///     mime::TEXT_PLAIN,
    ///     Vec::from(b"Lorem ipsum dolor sit amet".as_ref())
    /// ).to_multipart_stub();
    ///
    /// let encoded = serde_json::to_vec(&att).unwrap();
    ///
    /// # let decoded = serde_json::from_slice::<serde_json::Value>(&encoded)
    /// #     .unwrap();
    /// #
    /// # let expected = serde_json::Value::Object(
    /// #     vec![
    /// #         (String::from("content_type"), serde_json::Value::String(String::from("text/plain"))),
    /// #         (String::from("follows"), serde_json::Value::Bool(true)),
    /// #         (String::from("length"), serde_json::Value::Number(serde_json::Number::from(26))),
    /// #     ].into_iter().collect::<serde_json::value::Map<String, serde_json::Value>>()
    /// # );
    /// #
    /// # assert_eq!(decoded, expected);
    /// #
    /// // encoded:
    /// //
    /// // {
    /// //     "content_type": "text/plain",
    /// //     "follows": true,
    /// //     "length": 26
    /// // }
    /// ```
    ///
    pub fn to_multipart_stub(&self) -> Attachment {
        Attachment {
            content_type: self.content_type.clone(),
            inner: Inner::Follows { content_length: self.content_length() },
        }
    }

    /// Borrows the attachment's digest, if available.
    ///
    /// An attachment's digest is available if and only if it originates from
    /// the server.
    ///
    pub fn digest(&self) -> Option<&Digest> {
        match self.inner {
            Inner::ServerOrigin { ref digest, .. } => Some(digest),
            Inner::ClientOrigin { .. } => None,
            Inner::Follows { .. } => None,
        }
    }

    /// Returns the attachment's encoding information, if available.
    pub fn encoding(&self) -> Option<&Encoding> {
        match self.inner {
            Inner::ServerOrigin { ref encoding, .. } => encoding.as_ref().clone(),
            Inner::ClientOrigin { .. } => None,
            Inner::Follows { .. } => None,
        }
    }

    /// Returns the attachment's revision sequence number—i.e., the `revpos`
    /// attachment field.
    pub fn revision_sequence(&self) -> Option<u64> {
        match self.inner {
            Inner::ServerOrigin { revpos, .. } => Some(revpos),
            Inner::ClientOrigin { .. } => None,
            Inner::Follows { .. } => None,
        }
    }
}

impl<'a> Deserialize<'a> for Attachment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        #[derive(Deserialize)]
        struct T {
            content_type: SerializableMime,
            data: Option<SerializableBase64>,
            digest: Digest,
            encoded_length: Option<u64>,
            encoding: Option<String>,
            length: Option<u64>,
            revpos: u64,
            // stub: Option<bool>, // unused
        }

        let x = T::deserialize(deserializer)?;

        let encoding = match (x.encoding, x.encoded_length) {
            (Some(codec), Some(length)) => Some(Encoding {
                codec: EncodingCodec::from(codec),
                length: length,
            }),
            (None, None) => None,
            _ => return Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Map,
                &Expectation(
                    "a JSON object with complete CouchDB attachment encoding info OR no such info",
                ),
            )),
        };

        let inner = if let Some(SerializableBase64(bytes)) = x.data {
            Inner::ServerOrigin {
                content: Content::WithBytes(bytes),
                digest: x.digest,
                encoding: encoding,
                revpos: x.revpos,
            }
        } else if let Some(content_length) = x.length {
            Inner::ServerOrigin {
                content: Content::WithLength(content_length),
                digest: x.digest,
                encoding: encoding,
                revpos: x.revpos,
            }
        } else {
            return Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Map,
                &Expectation(
                    "a JSON object with CouchDB attachment content OR content length",
                ),
            ));
        };

        Ok(Attachment {
            content_type: x.content_type.0,
            inner: inner,
        })
    }
}

impl Serialize for Attachment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Debug, Default, Deserialize, Serialize)]
        struct T {
            content_type: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            data: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            stub: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none")]
            follows: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none")]
            length: Option<u64>,
        }

        let mut x = T::default();
        x.content_type = self.content_type.to_string();

        match self.inner {
            Inner::ServerOrigin { .. } => {
                x.stub = Some(true);
            }
            Inner::ClientOrigin { ref content } => {
                x.data = Some(base64::encode(content));
            }
            Inner::Follows { content_length } => {
                x.follows = Some(true);
                x.length = Some(content_length);
            }
        };

        x.serialize(serializer)
    }
}

struct Expectation(&'static str);

impl serde::de::Expected for Expectation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.write_str(self.0)
    }
}

struct SerializableBase64(Vec<u8>);

impl<'a> Deserialize<'a> for SerializableBase64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let s = String::deserialize(deserializer)?;

        let v = base64::decode(&s).map_err(|_| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(&s),
                &Expectation("a base64-encoded string containing CouchDB attachment data"),
            )
        })?;

        Ok(SerializableBase64(v))
    }
}

impl<'a> Deserialize<'a> for Digest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let s = String::deserialize(deserializer)?;

        let v = Digest::from_str(&s).map_err(|_| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(&s),
                &Expectation("a CouchDB attachment digest"),
            )
        })?;

        Ok(v)
    }
}

impl Content {
    /// Returns a length-only copy of the content.
    pub fn to_length_only(&self) -> Content {
        match *self {
            Content::WithBytes(ref bytes) => Content::WithLength(bytes.len() as u64),
            Content::WithLength(length) => Content::WithLength(length),
        }
    }
}

impl Digest {
    /// Borrows the encoded digest value.
    ///
    /// For example, with an MD5 digest, the value is the 16-byte MD5 sum of the
    /// attachment's content.
    ///
    pub fn bytes(&self) -> &[u8] {
        match *self {
            Digest::Md5 { ref value } => value,
            Digest::Other { ref value, .. } => value,
        }
    }

    /// Returns whether the digest is MD5-encoded.
    pub fn is_md5(&self) -> bool {
        match *self {
            Digest::Md5 { .. } => true,
            _ => false,
        }
    }
}

impl FromStr for Digest {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {

        let mut iter = s.splitn(2, '-');
        let name = iter.next().unwrap();
        let value = iter.next().ok_or(Error::BadDigest)?;
        let value = base64::decode(&value).map_err(|_| Error::BadDigest)?;

        Ok(match name {
            "md5" => Digest::Md5 { value: value },
            _ => Digest::Other {
                name: String::from(name),
                value: value,
            },
        })
    }
}

impl Encoding {
    /// Returns the size of the attachment's compressed content, in bytes.
    pub fn length(&self) -> u64 {
        self.length
    }

    /// Returns whether the compression codec is gzip.
    pub fn is_gzip(&self) -> bool {
        self.codec == EncodingCodec::Gzip
    }
}

impl From<String> for EncodingCodec {
    fn from(s: String) -> Self {
        match s.as_str() {
            "gzip" => EncodingCodec::Gzip,
            _ => EncodingCodec::Other(s),
        }
    }
}

struct SerializableMime(Mime);

impl<'a> Deserialize<'a> for SerializableMime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let s = String::deserialize(deserializer)?;

        let v = Mime::from_str(&s).map_err(|_| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(&s),
                &Expectation("a string specifying a MIME type"),
            )
        })?;

        Ok(SerializableMime(v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use {mime, serde_json};

    #[test]
    fn attachment_deserializes_as_stub() {

        let source = r#"{
            "content_type": "text/plain",
            "digest": "md5-Ids41vtv725jyrN7iUvMcQ==",
            "length": 1872,
            "revpos": 4,
            "stub": true
        }"#;

        let expected = Attachment {
            content_type: mime::TEXT_PLAIN,
            inner: Inner::ServerOrigin {
                content: Content::WithLength(1872),
                digest: Digest::Md5 {
                    value: Vec::from(
                        b"\x21\xdb\x38\xd6\
                         \xfb\x6f\xef\x6e\
                         \x63\xca\xb3\x7b\
                         \x89\x4b\xcc\x71"
                            .as_ref(),
                    ),
                },
                encoding: None,
                revpos: 4,
            },
        };

        let got: Attachment = serde_json::from_str(source).unwrap();
        assert_eq!(got, expected);
    }

    #[test]
    fn attachment_deserializes_with_data() {

        let source = r#"{
            "content_type": "image/gif",
            "data": "R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7",
            "digest": "md5-2JdGiI2i2VELZKnwMers1Q==",
            "revpos": 2
        }"#;

        let expected = Attachment {
            content_type: mime::IMAGE_GIF,
            inner: Inner::ServerOrigin {
                content: Content::WithBytes(Vec::from(
                    b"\x47\x49\x46\x38\
                    \x39\x61\x01\x00\
                    \x01\x00\x80\x00\
                    \x00\x00\x00\x00\
                    \xff\xff\xff\x21\
                    \xf9\x04\x01\x00\
                    \x00\x00\x00\x2c\
                    \x00\x00\x00\x00\
                    \x01\x00\x01\x00\
                    \x00\x02\x01\x44\
                    \x00\x3b"
                        .as_ref(),
                )),
                digest: Digest::Md5 {
                    value: Vec::from(
                        b"\xd8\x97\x46\x88\
                        \x8d\xa2\xd9\x51\
                        \x0b\x64\xa9\xf0\
                        \x31\xea\xec\xd5"
                            .as_ref(),
                    ),
                },
                encoding: None,
                revpos: 2,
            },
        };

        let got: Attachment = serde_json::from_str(source).unwrap();
        assert_eq!(got, expected);
    }

    #[test]
    fn attachment_deserializes_with_encoding_info() {

        let source = r#"{
            "content_type": "text/plain",
            "digest": "md5-Ids41vtv725jyrN7iUvMcQ==",
            "encoded_length": 693,
            "encoding": "gzip",
            "length": 1872,
            "revpos": 4,
            "stub": true
        }"#;

        let expected = Attachment {
            content_type: mime::TEXT_PLAIN,
            inner: Inner::ServerOrigin {
                content: Content::WithLength(1872),
                digest: Digest::Md5 {
                    value: Vec::from(
                        b"\x21\xdb\x38\xd6\
                         \xfb\x6f\xef\x6e\
                         \x63\xca\xb3\x7b\
                         \x89\x4b\xcc\x71"
                            .as_ref(),
                    ),
                },
                encoding: Some(Encoding {
                    length: 693,
                    codec: EncodingCodec::Gzip,
                }),
                revpos: 4,
            },
        };

        let got: Attachment = serde_json::from_str(source).unwrap();
        assert_eq!(got, expected);
    }

    #[test]
    fn client_origin_attachment_serializes_with_content() {

        let source = Attachment::new(
            mime::TEXT_PLAIN,
            Vec::from(b"Lorem ipsum dolor sit amet".as_ref()),
        );

        let encoded = serde_json::to_vec(&source).unwrap();
        let decoded: serde_json::Value = serde_json::from_slice(&encoded).unwrap();

        let expected = json!({
            "content_type": "text/plain",
            "data": "TG9yZW0gaXBzdW0gZG9sb3Igc2l0IGFtZXQ=",
        });

        assert_eq!(decoded, expected);
    }

    #[test]
    fn server_origin_attachment_serializes_as_stub() {

        let source = Attachment {
            content_type: mime::TEXT_PLAIN,
            inner: Inner::ServerOrigin {
                content: Content::WithLength(1872),
                digest: Digest::Md5 {
                    value: Vec::from(
                        b"\x21\xdb\x38\xd6\
                         \xfb\x6f\xef\x6e\
                         \x63\xca\xb3\x7b\
                         \x89\x4b\xcc\x71"
                            .as_ref(),
                    ),
                },
                encoding: Some(Encoding {
                    length: 693,
                    codec: EncodingCodec::Gzip,
                }),
                revpos: 4,
            },
        };

        let encoded = serde_json::to_vec(&source).unwrap();
        let decoded: serde_json::Value = serde_json::from_slice(&encoded).unwrap();

        let expected = json!({
            "content_type": "text/plain",
            "stub": true,
        });

        assert_eq!(decoded, expected);
    }
}
