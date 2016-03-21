use std;
use serde;

use ChangeResult;
use ChangeResultBuilder;
use DocumentId;

/// Builder for constructing a change list.
#[derive(Debug)]
pub struct ChangesBuilder {
    target: Changes,
}

impl ChangesBuilder {
    /// Constructs a new change list builder.
    ///
    /// The change list contained within the builder has the given sequence
    /// number and no change results.
    ///
    pub fn new(last_seq: u64) -> Self {
        ChangesBuilder {
            target: Changes {
                last_seq: last_seq,
                results: Vec::new(),
                _dummy: std::marker::PhantomData,
            },
        }
    }

    /// Returns the change list contained within the builder.
    pub fn unwrap(self) -> Changes {
        self.target
    }

    /// Adds a change result to the change list contained within the builder.
    pub fn push_result(mut self, result: ChangeResult) -> Self {
        self.target.results.push(result);
        self
    }

    /// Builds and adds a change result to the change list contained within the
    /// builder.
    pub fn build_result<I, F>(mut self, seq: u64, doc_id: I, f: F) -> Self
        where I: Into<DocumentId>,
              F: FnOnce(ChangeResultBuilder) -> ChangeResultBuilder
    {
        let builder = ChangeResultBuilder::new(seq, doc_id);
        self.target.results.push(f(builder).unwrap());
        self
    }
}

/// List of changes to documents within a database.
#[derive(Clone, Debug, PartialEq)]
pub struct Changes {
    /// Sequence number for the most recent change.
    pub last_seq: u64,

    /// Changes in the change list.
    pub results: Vec<ChangeResult>,
    _dummy: std::marker::PhantomData<()>,
}

impl serde::Deserialize for Changes {
    fn deserialize<D>(d: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        enum Field {
            LastSeq,
            Results,
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
                            "last_seq" => Ok(Field::LastSeq),
                            "results" => Ok(Field::Results),
                            _ => Err(E::unknown_field(value)),
                        }
                    }
                }

                d.deserialize(Visitor)
            }
        }

        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = Changes;

            fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
                where V: serde::de::MapVisitor
            {
                let mut last_seq = None;
                let mut results = None;
                loop {
                    match try!(visitor.visit_key()) {
                        Some(Field::LastSeq) => {
                            last_seq = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Results) => {
                            results = Some(try!(visitor.visit_value()));
                        }
                        None => {
                            break;
                        }
                    }
                }

                try!(visitor.end());

                let last_seq = match last_seq {
                    Some(x) => x,
                    None => try!(visitor.missing_field("last_seq")),
                };

                let results = match results {
                    Some(x) => x,
                    None => try!(visitor.missing_field("results")),
                };

                Ok(Changes {
                    last_seq: last_seq,
                    results: results,
                    _dummy: std::marker::PhantomData,
                })
            }
        }

        static FIELDS: &'static [&'static str] = &["last_seq", "results"];
        d.deserialize_struct("Changes", FIELDS, Visitor)
    }
}

#[cfg(test)]
mod tests {

    use serde_json;
    use std;

    use ChangeResultBuilder;
    use super::Changes;
    use super::ChangesBuilder;

    #[test]
    fn builder_default() {
        let expected = Changes {
            last_seq: 42,
            results: Vec::new(),
            _dummy: std::marker::PhantomData,
        };
        let got = ChangesBuilder::new(42).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn builder_push_result() {
        let r = ChangeResultBuilder::new(6, "6478c2ae800dfc387396d14e1fc39626")
                    .build_change_from_rev_str("2-7051cbe5c8faecd085a3fa619e6e6337", |x| x)
                    .unwrap();
        let expected = Changes {
            last_seq: 42,
            results: vec![r.clone()],
            _dummy: std::marker::PhantomData,
        };
        let got = ChangesBuilder::new(42)
                      .push_result(r)
                      .unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn builder_build_result() {
        let expected = Changes {
            last_seq: 42,
            results: vec![ChangeResultBuilder::new(6, "6478c2ae800dfc387396d14e1fc39626")
                              .build_change_from_rev_str("2-7051cbe5c8faecd085a3fa619e6e6337",
                                                         |x| x)
                              .unwrap()],
            _dummy: std::marker::PhantomData,
        };
        let got = ChangesBuilder::new(42)
                      .build_result(6, "6478c2ae800dfc387396d14e1fc39626", |x| {
                          x.build_change_from_rev_str("2-7051cbe5c8faecd085a3fa619e6e6337", |x| x)
                      })
                      .unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn deserialization_ok_with_all_fields() {
        let expected = ChangesBuilder::new(11)
                           .build_result(6, "6478c2ae800dfc387396d14e1fc39626", |x| {
                               x.build_change_from_rev_str("2-7051cbe5c8faecd085a3fa619e6e6337",
                                                           |x| x)
                           })
                           .build_result(9, "5bbc9ca465f1b0fcd62362168a7c8831", |x| {
                               x.deleted(true)
                                .build_change_from_rev_str("3-7379b9e515b161226c6559d90c4dc49f",
                                                           |x| x)
                           })
                           .build_result(11, "729eb57437745e506b333068fff665ae", |x| {
                               x.build_change_from_rev_str("6-460637e73a6288cb24d532bf91f32969",
                                                           |x| x)
                                .build_change_from_rev_str("5-eeaa298781f60b7bcae0c91bdedd1b87",
                                                           |x| x)
                           })
                           .unwrap();
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("last_seq", 11)
                         .insert_array("results", |x| {
                             x.push_object(|x| {
                                  x.insert_array("changes", |x| {
                                       x.push_object(|x| {
                                           x.insert("rev", "2-7051cbe5c8faecd085a3fa619e6e6337")
                                       })
                                   })
                                   .insert("id", "6478c2ae800dfc387396d14e1fc39626")
                                   .insert("seq", 6)
                              })
                              .push_object(|x| {
                                  x.insert_array("changes", |x| {
                                       x.push_object(|x| {
                                           x.insert("rev", "3-7379b9e515b161226c6559d90c4dc49f")
                                       })
                                   })
                                   .insert("deleted", true)
                                   .insert("id", "5bbc9ca465f1b0fcd62362168a7c8831")
                                   .insert("seq", 9)
                              })
                              .push_object(|x| {
                                  x.insert_array("changes", |x| {
                                       x.push_object(|x| {
                                            x.insert("rev", "6-460637e73a6288cb24d532bf91f32969")
                                        })
                                        .push_object(|x| {
                                            x.insert("rev", "5-eeaa298781f60b7bcae0c91bdedd1b87")
                                        })
                                   })
                                   .insert("id", "729eb57437745e506b333068fff665ae")
                                   .insert("seq", 11)
                              })
                         })
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn deserialization_nok_without_last_seq_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert_array("results", |x| x)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Changes>(&s);
        expect_json_error_missing_field!(got, "last_seq");
    }

    #[test]
    #[should_panic] // because serde_json issue #29 (https://github.com/serde-rs/json/issues/29)
    fn deserialization_nok_without_results_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("last_seq", 11)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Changes>(&s);
        expect_json_error_missing_field!(got, "results");
    }
}
