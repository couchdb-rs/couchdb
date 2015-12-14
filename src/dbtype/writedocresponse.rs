use serde;

pub type PostToDatabaseResponse = WriteDocumentResponse;
pub type PutDocumentResponse = WriteDocumentResponse;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WriteDocumentResponse {
    pub id: String,
    pub ok: bool,
    pub rev: String,
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

    use super::*;

    #[test]
    fn test_deserialization() {

        // VERIFY: Deserialization succeeds when all fields are present.

        let exp = WriteDocumentResponse {
            id: "foobar".to_string(),
            ok: true,
            rev: "1-abcd".to_string(),
        };

        let s = r#"{
            "id": "foobar",
            "ok": true,
            "rev": "1-abcd"
        }"#;

        let got = serde_json::from_str::<WriteDocumentResponse>(&s).unwrap();

        assert_eq!(got, exp);
    }
}
