use serde;

use ChangeResult;

// A ChangeLine is a single line returned from the CouchDB server as part of a
// continuous `/db/_changes` result.
#[derive(Debug, PartialEq)]
pub enum ChangeLine {
    Event(ChangeResult),
    End {
        last_seq: u64,
    },
}

impl serde::Deserialize for ChangeLine {
    fn deserialize<D>(d: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        enum Field {
            Changes,
            Deleted,
            Id,
            LastSeq,
            Seq,
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
                            "changes" => Ok(Field::Changes),
                            "deleted" => Ok(Field::Deleted),
                            "id" => Ok(Field::Id),
                            "last_seq" => Ok(Field::LastSeq),
                            "seq" => Ok(Field::Seq),
                            _ => Err(E::unknown_field(value)),
                        }
                    }
                }

                d.deserialize(Visitor)
            }
        }

        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = ChangeLine;

            fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
                where V: serde::de::MapVisitor
            {
                enum Type {
                    Event,
                    End,
                }

                let mut which_type = None;

                let mut last_seq = None;
                let mut changes = None;
                let mut deleted = None;
                let mut id = None;
                let mut seq = None;

                loop {
                    match try!(visitor.visit_key()) {
                        Some(Field::LastSeq) => {
                            if let None = which_type {
                                which_type = Some(Type::End);
                            } else if let Some(Type::Event) = which_type {
                                use serde::de::Error;
                                return Err(V::Error::unknown_field("last_seq"));
                            }
                            last_seq = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Changes) => {
                            if let None = which_type {
                                which_type = Some(Type::Event);
                            } else if let Some(Type::End) = which_type {
                                use serde::de::Error;
                                return Err(V::Error::unknown_field("changes"));
                            }
                            changes = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Deleted) => {
                            if let None = which_type {
                                which_type = Some(Type::Event);
                            } else if let Some(Type::End) = which_type {
                                use serde::de::Error;
                                return Err(V::Error::unknown_field("deleted"));
                            }
                            deleted = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Id) => {
                            if let None = which_type {
                                which_type = Some(Type::Event);
                            } else if let Some(Type::End) = which_type {
                                use serde::de::Error;
                                return Err(V::Error::unknown_field("id"));
                            }
                            id = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Seq) => {
                            if let None = which_type {
                                which_type = Some(Type::Event);
                            } else if let Some(Type::End) = which_type {
                                use serde::de::Error;
                                return Err(V::Error::unknown_field("seq"));
                            }
                            seq = Some(try!(visitor.visit_value()));
                        }
                        None => {
                            break;
                        }
                    }
                }

                try!(visitor.end());

                Ok(match which_type {
                    None => try!(visitor.missing_field("last_seq")),
                    Some(Type::Event) => {
                        let c = ChangeResult::new(match seq {
                                                      None => try!(visitor.missing_field("seq")),
                                                      Some(x) => x,
                                                  },
                                                  match id {
                                                      None => try!(visitor.missing_field("id")),
                                                      Some(x) => x,
                                                  },
                                                  match changes {
                                                      None => {
                                                          try!(visitor.missing_field("changes"))
                                                      }
                                                      Some(x) => x,
                                                  },
                                                  deleted.unwrap_or(false));
                        ChangeLine::Event(c)
                    }
                    Some(Type::End) => {
                        match last_seq {
                            None => try!(visitor.missing_field("last_seq")),
                            Some(x) => ChangeLine::End { last_seq: x },
                        }
                    }
                })
            }
        }

        static FIELDS: &'static [&'static str] = &["changes", "deleted", "id", "last_seq", "seq"];
        d.deserialize_struct("ChangeLine", FIELDS, Visitor)
    }
}

#[cfg(test)]
mod tests {

    use serde_json;

    use ChangeResultBuilder;
    use super::ChangeLine;

    #[test]
    fn deserialization_ok_end() {
        let expected = ChangeLine::End { last_seq: 42 };
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("last_seq", 42)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn deserialization_ok_event_with_deleted_field() {
        let c = ChangeResultBuilder::new(9, "5bbc9ca465f1b0fcd62362168a7c8831")
                    .build_change_from_rev_str("3-7379b9e515b161226c6559d90c4dc49f", |x| x)
                    .deleted(true)
                    .unwrap();
        let expected = ChangeLine::Event(c);
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert_array("changes", |x| {
                             x.push_object(|x| {
                                 x.insert("rev", "3-7379b9e515b161226c6559d90c4dc49f")
                             })
                         })
                         .insert("deleted", true)
                         .insert("id", "5bbc9ca465f1b0fcd62362168a7c8831")
                         .insert("seq", 9)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn deserialization_ok_event_without_deleted_field() {
        let c = ChangeResultBuilder::new(9, "5bbc9ca465f1b0fcd62362168a7c8831")
                    .build_change_from_rev_str("3-7379b9e515b161226c6559d90c4dc49f", |x| x)
                    .deleted(false)
                    .unwrap();
        let expected = ChangeLine::Event(c);
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert_array("changes", |x| {
                             x.push_object(|x| {
                                 x.insert("rev", "3-7379b9e515b161226c6559d90c4dc49f")
                             })
                         })
                         .insert("id", "5bbc9ca465f1b0fcd62362168a7c8831")
                         .insert("seq", 9)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn deserialization_nok_no_fields() {
        let source = serde_json::builder::ObjectBuilder::new().unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<ChangeLine>(&s);
        expect_json_error_missing_field!(got, "last_seq");
    }

    #[test]
    fn deserialization_nok_end_with_seq_field() {
        let s = r#"{"last_seq": 42, "seq": 17}"#;
        let got = serde_json::from_str::<ChangeLine>(&s);
        expect_json_error_unknown_field!(got, "seq");
    }

    #[test]
    fn deserialization_nok_end_with_id_field() {
        let s = r#"{"last_seq": 42, "id": "5bbc9ca465f1b0fcd62362168a7c8831"}"#;
        let got = serde_json::from_str::<ChangeLine>(&s);
        expect_json_error_unknown_field!(got, "id");
    }

    #[test]
    fn deserialization_nok_end_with_changes_field() {
        let s = r#"{"last_seq": 42, "changes" [{"rev": "3-7379b9e515b161226c6559d90c4dc49f"}]}"#;
        let got = serde_json::from_str::<ChangeLine>(&s);
        expect_json_error_unknown_field!(got, "changes");
    }

    #[test]
    fn deserialization_nok_end_with_deleted_field() {
        let s = r#"{"last_seq": 42, "deleted": true}"#;
        let got = serde_json::from_str::<ChangeLine>(&s);
        expect_json_error_unknown_field!(got, "deleted");
    }

    #[test]
    fn deserialization_nok_event_with_last_seq_field() {
        let s = "{\"seq\": 9, \"id\": \"5bbc9ca465f1b0fcd62362168a7c8831\",\"changes\": \
                 [{\"rev\": \"3-7379b9e515b161226c6559d90c4dc49f\"}],\"last_seq\": 42}";
        let got = serde_json::from_str::<ChangeLine>(&s);
        expect_json_error_unknown_field!(got, "last_seq");
    }
}
