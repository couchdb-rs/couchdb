use serde;

use ViewFunctionMap;
use ViewFunction;

/// Content of a design document.
///
/// CouchDB design documents contain many fields. However, the `Design` type
/// currently supports only views.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Design {
    /// Map of view names to the view function for each name.
    pub views: ViewFunctionMap,
}

impl Design {
    /// Construct an empty design document.
    ///
    /// An empty design document contains no views.
    ///
    pub fn new() -> Self {
        Design { views: ViewFunctionMap::new() }
    }
}

impl serde::Serialize for Design {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {

        struct Visitor<'a>(&'a Design);

        impl<'a> serde::ser::MapVisitor for Visitor<'a> {
            fn visit<S>(&mut self, s: &mut S) -> Result<Option<()>, S::Error>
                where S: serde::Serializer
            {
                let Visitor(design) = *self;

                if !design.views.is_empty() {
                    try!(s.visit_struct_elt("views", &design.views));
                }

                Ok(None)
            }
        }

        s.visit_struct("Design", Visitor(self))
    }
}

impl serde::Deserialize for Design {
    fn deserialize<D>(d: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        enum Field {
            Views,
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
                            "views" => Ok(Field::Views),
                            _ => Err(E::unknown_field(value)),
                        }
                    }
                }

                d.visit(Visitor)
            }
        }

        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = Design;

            fn visit_map<V>(&mut self, mut visitor: V) -> Result<Design, V::Error>
                where V: serde::de::MapVisitor
            {
                let mut views = None;
                loop {
                    match try!(visitor.visit_key()) {
                        Some(Field::Views) => {
                            views = Some(try!(visitor.visit_value()));
                        }
                        None => {
                            break;
                        }
                    }
                }

                try!(visitor.end());

                let views = match views {
                    Some(x) => x,
                    None => ViewFunctionMap::new(),
                };

                let v = Design { views: views };

                Ok(v)
            }
        }

        static FIELDS: &'static [&'static str] = &["views"];
        d.visit_struct("Design", FIELDS, Visitor)
    }
}

/// Builder for constructing a design document.
#[derive(Debug)]
pub struct DesignBuilder {
    design: Design,
}

impl DesignBuilder {
    /// Construct a builder containing an empty design document.
    pub fn new() -> Self {
        DesignBuilder { design: Design { views: ViewFunctionMap::new() } }
    }

    /// Return the design document contained within the builder.
    pub fn unwrap(self) -> Design {
        self.design
    }

    /// Add a view function to the design document contained within the builder.
    ///
    /// If the design document already contains a view with the same name then
    /// the new view function will replace the existing view function.
    ///
    pub fn insert_view<T, U>(mut self, view_name: T, view_function: U) -> Self
        where T: Into<String>,
              U: Into<ViewFunction>
    {
        self.design.views.insert(view_name.into(), view_function.into());
        self
    }
}

#[cfg(test)]
mod tests {

    use serde_json;

    use Design;
    use DesignBuilder;
    use ViewFunction;
    use ViewFunctionMap;

    #[test]
    fn design_builder_empty() {
        let expected = Design { views: ViewFunctionMap::new() };
        let got = DesignBuilder::new().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn design_builder_with_all_fields() {
        let views = {
            let mut x = ViewFunctionMap::new();
            x.insert("foo".to_string(),
                     ViewFunction {
                         map: "function(doc) { emit(doc.name, doc.foo_thing); }".to_string(),
                         reduce: Some("function(keys, values) { return sum(values); }".to_string()),
                     });
            x.insert("bar".to_string(),
                     ViewFunction {
                         map: "function(doc) { emit(doc.name, doc.bar_thing); }".to_string(),
                         reduce: None,
                     });
            x
        };
        let expected = Design { views: views };
        let got = DesignBuilder::new()
                      .insert_view("foo",
                                   ViewFunction {
                                       map: "function(doc) { emit(doc.name, doc.foo_thing); }"
                                                .to_string(),
                                       reduce: Some("function(keys, values) { return sum(values); \
                                                     }"
                                                        .to_string()),
                                   })
                      .insert_view("bar",
                                   ViewFunction {
                                       map: "function(doc) { emit(doc.name, doc.bar_thing); }"
                                                .to_string(),
                                       reduce: None,
                                   })
                      .unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn design_builder_view_replaces_same_name() {
        let views = {
            let mut x = ViewFunctionMap::new();
            x.insert("foo".to_string(),
                     ViewFunction {
                         map: "function(doc) { emit(doc.name, doc.foo_thing); }".to_string(),
                         reduce: Some("function(keys, values) { return sum(values); }".to_string()),
                     });
            x
        };
        let expected = Design { views: views };
        let got = DesignBuilder::new()
                      .insert_view("foo",
                                   ViewFunction {
                                       map: "function(doc) {}".to_string(),
                                       reduce: None,
                                   })
                      .insert_view("foo",
                                   ViewFunction {
                                       map: "function(doc) { emit(doc.name, doc.foo_thing); }"
                                                .to_string(),
                                       reduce: Some("function(keys, values) { return sum(values); \
                                                     }"
                                                        .to_string()),
                                   })
                      .unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn design_serialization_with_all_fields() {
        let expected = serde_json::builder::ObjectBuilder::new()
                           .insert_object("views", |x| {
                               x.insert_object("foo", |x| {
                                    x.insert("map",
                                             "function(doc) { emit(doc.name, doc.foo_thing); }")
                                     .insert("reduce",
                                             "function(keys, values) { return sum(values); }")
                                })
                                .insert_object("bar", |x| {
                                    x.insert("map",
                                             "function(doc) { emit(doc.name, doc.bar_thing); }")
                                })
                           })
                           .unwrap();
        let source = DesignBuilder::new()
                         .insert_view("foo",
                                      ViewFunction {
                                          map: "function(doc) { emit(doc.name, doc.foo_thing); }"
                                                   .to_string(),
                                          reduce: Some("function(keys, values) { return \
                                                        sum(values); }"
                                                           .to_string()),
                                      })
                         .insert_view("bar",
                                      ViewFunction {
                                          map: "function(doc) { emit(doc.name, doc.bar_thing); }"
                                                   .to_string(),
                                          reduce: None,
                                      })
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn design_deserialization_with_view_field_elided() {
        let expected = serde_json::builder::ObjectBuilder::new().unwrap();
        let source = DesignBuilder::new().unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn design_deserialization_with_all_fields() {
        let expected = DesignBuilder::new()
                           .insert_view("foo",
                                        ViewFunction {
                                            map: "function(doc) { emit(doc.name, doc.foo_thing); }"
                                                     .to_string(),
                                            reduce: Some("function(keys, values) { return \
                                                          sum(values); }"
                                                             .to_string()),
                                        })
                           .insert_view("bar",
                                        ViewFunction {
                                            map: "function(doc) { emit(doc.name, doc.bar_thing); }"
                                                     .to_string(),
                                            reduce: None,
                                        })
                           .unwrap();
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert_object("views", |x| {
                             x.insert_object("foo", |x| {
                                  x.insert("map",
                                           "function(doc) { emit(doc.name, doc.foo_thing); }")
                                   .insert("reduce",
                                           "function(keys, values) { return sum(values); }")
                              })
                              .insert_object("bar", |x| {
                                  x.insert("map",
                                           "function(doc) { emit(doc.name, doc.bar_thing); }")
                              })
                         })
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn design_deserialization_with_optional_fields_elided() {
        let expected = DesignBuilder::new().unwrap();
        let source = serde_json::builder::ObjectBuilder::new().unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }
}
