use serde;
use std;

use DocumentId;
use Revision;

/// Response content from the CouchDB server in case of error.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ErrorResponse {
    /// Error string returned by CouchDB Server.
    ///
    /// This is the high-level name of the error—e.g., “file_exists”.
    ///
    pub error: String,

    /// Reason string returned by CouchDB Server.
    ///
    /// This is a low-level description of the error—e.g., “The database could
    /// not be created, the file already exists.”
    ///
    pub reason: String,
}

impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}: {}", self.error, self.reason)
    }
}

impl serde::Deserialize for ErrorResponse {
    fn deserialize<D>(d: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        enum Field {
            Error,
            Reason,
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
                            "error" => Ok(Field::Error),
                            "reason" => Ok(Field::Reason),
                            _ => Err(E::unknown_field(value)),
                        }
                    }
                }

                d.visit(Visitor)
            }
        }

        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = ErrorResponse;

            fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
                where V: serde::de::MapVisitor
            {
                let mut error = None;
                let mut reason = None;
                loop {
                    match try!(visitor.visit_key()) {
                        Some(Field::Error) => {
                            error = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Reason) => {
                            reason = Some(try!(visitor.visit_value()));
                        }
                        None => {
                            break;
                        }
                    }
                }

                try!(visitor.end());

                let x = ErrorResponse {
                    error: match error {
                        Some(x) => x,
                        None => try!(visitor.missing_field("error")),
                    },
                    reason: match reason {
                        Some(x) => x,
                        None => try!(visitor.missing_field("reason")),
                    },
                };

                Ok(x)
            }
        }

        static FIELDS: &'static [&'static str] = &["error", "reason"];
        d.visit_struct("ErrorResponse", FIELDS, Visitor)
    }
}

pub type PostToDatabaseResponse = WriteDocumentResponse;
pub type PutDocumentResponse = WriteDocumentResponse;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct WriteDocumentResponse {
    pub id: DocumentId,
    pub ok: bool,
    pub rev: Revision,
}

impl serde::Deserialize for WriteDocumentResponse {
    fn deserialize<D>(d: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        enum Field {
            Id,
            Ok,
            Rev,
        }

        impl serde::Deserialize for Field {
            fn deserialize<D>(d: &mut D) -> Result<Self, D::Error>
                where D: serde::Deserializer
            {
                struct Visitor;

                impl serde::de::Visitor for Visitor {
                    type Value = Field;

                    fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E>
                        where E: serde::de::Error
                    {
                        match value {
                            "id" => Ok(Field::Id),
                            "ok" => Ok(Field::Ok),
                            "rev" => Ok(Field::Rev),
                            _ => Err(E::unknown_field(value)),
                        }
                    }
                }

                d.visit(Visitor)
            }
        }

        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = WriteDocumentResponse;

            fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
                where V: serde::de::MapVisitor
            {
                let mut id = None;
                let mut ok = None;
                let mut rev = None;
                loop {
                    match try!(visitor.visit_key()) {
                        Some(Field::Id) => {
                            id = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Ok) => {
                            ok = Some(try!(visitor.visit_value()));
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

                let resp = WriteDocumentResponse {
                    id: match id {
                        Some(x) => x,
                        None => try!(visitor.missing_field("id")),
                    },
                    ok: match ok {
                        Some(x) => x,
                        None => try!(visitor.missing_field("ok")),
                    },
                    rev: match rev {
                        Some(x) => x,
                        None => try!(visitor.missing_field("rev")),
                    },
                };

                Ok(resp)
            }
        }

        static FIELDS: &'static [&'static str] = &["id", "ok", "rev"];
        d.visit_struct("WriteDocumentResponse", FIELDS, Visitor)
    }
}

#[cfg(test)]
mod tests {

    use serde_json;

    use DocumentId;
    use ErrorResponse;
    use Revision;
    use super::WriteDocumentResponse;

    #[test]
    fn error_response_display() {
        let expected = "file_exists: The database could not be created, the file already exists.";
        let source = ErrorResponse {
            error: "file_exists".to_string(),
            reason: "The database could not be created, the file already exists.".to_string(),
        };
        let got = format!("{}", source);
        assert_eq!(expected, got);
    }

    #[test]
    fn error_response_deserialization_with_all_fields() {
        let expected = ErrorResponse {
            error: "file_exists".to_string(),
            reason: "The database could not be created, the file already exists.".to_string(),
        };
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "file_exists")
                         .insert("reason",
                                 "The database could not be created, the file already exists.")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn error_response_deserialization_with_no_error_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("reason",
                                 "The database could not be created, the file already exists.")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<ErrorResponse>(&s);
        expect_json_error_missing_field!(got, "error");
    }

    #[test]
    fn error_response_deserialization_with_no_reason_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "file_exists")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<ErrorResponse>(&s);
        expect_json_error_missing_field!(got, "reason");
    }

    #[test]
    fn write_document_response_deserialization_with_all_fields() {
        let expected = WriteDocumentResponse {
            id: DocumentId::Normal("foo".into()),
            ok: true,
            rev: Revision::parse("1-12345678123456781234567812345678").unwrap(),
        };
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("id", "foo")
                         .insert("ok", true)
                         .insert("rev", "1-12345678123456781234567812345678")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn write_document_response_deserialization_with_no_id_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("ok", true)
                         .insert("rev", "1-12345678123456781234567812345678")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<WriteDocumentResponse>(&s);
        expect_json_error_missing_field!(got, "id");
    }

    #[test]
    fn write_document_response_deserialization_with_no_ok_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("id", "foo")
                         .insert("rev", "1-12345678123456781234567812345678")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<WriteDocumentResponse>(&s);
        expect_json_error_missing_field!(got, "ok");
    }

    #[test]
    fn write_document_response_deserialization_with_no_rev_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("id", "foo")
                         .insert("ok", true)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<WriteDocumentResponse>(&s);
        expect_json_error_missing_field!(got, "rev");
    }
}
