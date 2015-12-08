use serde;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PutDocumentResponse {
    pub id: String,
    pub ok: bool,
    pub rev: String,
}

impl serde::Deserialize for PutDocumentResponse {
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
            type Value = PutDocumentResponse;

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
                        },
                        Some(Field::Ok) => {
                            ok = Some(try!(visitor.visit_value()));
                        },
                        Some(Field::Rev) => {
                            rev = Some(try!(visitor.visit_value()));
                        },
                        None => { break; },
                    }
                }

                try!(visitor.end());

                let resp = PutDocumentResponse {
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
        d.visit_struct("PutDocumentResponse", FIELDS, Visitor)
    }
}

#[cfg(test)]
mod tests {

    use serde_json;

    use super::*;
    use jsontest;

    #[test]
    fn test_deserialization() {

        let fields = [
            r#""id": "stuff""#,
            r#""ok": true"#,
            r#""rev": "1-1234abcd""#,
        ];

        // Verify: All fields present.
        let s = jsontest::make_complete_json_object(&fields);
        let v = serde_json::from_str::<PutDocumentResponse>(&s).unwrap();
        assert_eq!(v.id, "stuff".to_string());
        assert_eq!(v.ok, true);
        assert_eq!(v.rev, "1-1234abcd".to_string());

        // Verify: Each field missing, one at a time.
        let s = jsontest::make_json_object_with_missing_field(&fields, "id");
        assert!(serde_json::from_str::<PutDocumentResponse>(&s).is_err());
        let s = jsontest::make_json_object_with_missing_field(&fields, "ok");
        assert!(serde_json::from_str::<PutDocumentResponse>(&s).is_err());
        let s = jsontest::make_json_object_with_missing_field(&fields, "rev");
        assert!(serde_json::from_str::<PutDocumentResponse>(&s).is_err());
    }
}
