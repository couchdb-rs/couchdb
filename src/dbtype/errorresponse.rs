use serde;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ErrorResponse {
    pub error: String,
    pub reason: String,
}

impl serde::Deserialize for ErrorResponse {
    fn deserialize<D>(d: &mut D) -> Result<Self, D::Error> where D: serde::Deserializer {

        enum Field {
            Error,
            Reason,
        }

        impl serde::Deserialize for Field
        {
            fn deserialize<D>(d: &mut D) -> Result<Field, D::Error>
                where D: serde::Deserializer
            {
                struct Visitor;

                impl serde::de::Visitor for Visitor
                {
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

            fn visit_map<V>(&mut self, mut visitor: V)
                -> Result<Self::Value, V::Error> where V: serde::de::MapVisitor
            {
                let mut error = None;
                let mut reason = None;
                loop {
                    match try!(visitor.visit_key()) {
                        Some(Field::Error) => {
                            error = Some(try!(visitor.visit_value()));
                        },
                        Some(Field::Reason) => {
                            reason = Some(try!(visitor.visit_value()));
                        },
                        None => { break; },
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

        static FIELDS: &'static [&'static str] = &[
            "error",
            "reason"
        ];
        d.visit_struct("ErrorResponse", FIELDS, Visitor)
    }
}

#[cfg(test)]
mod tests {

    use serde_json;

    use super::*;
    use dbtype::json::*;

    #[test]
    fn test_deserialization() {

        let fields = [
            r#""error": "stuff happened""#,
            r#""reason": "blah blah blah""#,
        ];

        // Verify: All fields present.
        let s = make_complete_json_object(&fields);
        let v = serde_json::from_str::<ErrorResponse>(&s).unwrap();
        assert_eq!(v.error, "stuff happened".to_string());
        assert_eq!(v.reason, "blah blah blah".to_string());

        // Verify: Each field missing, one at a time.
        let s = make_json_object_with_missing_field(&fields, "error");
        assert!(serde_json::from_str::<ErrorResponse>(&s).is_err());
        let s = make_json_object_with_missing_field(&fields, "reason");
        assert!(serde_json::from_str::<ErrorResponse>(&s).is_err());
    }
}
