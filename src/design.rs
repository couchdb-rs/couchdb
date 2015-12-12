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
    /// Construct a builder with an empty design document.
    pub fn new() -> Self {
        DesignBuilder { design: Design { views: ViewFunctionMap::new() } }
    }

    /// Return the design document.
    pub fn unwrap(self) -> Design {
        self.design
    }

    /// Add a view function to the design document.
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

    use super::*;
    use jsontest;
    use ViewFunction;

    #[test]
    fn test_serialization() {

        use serde_json::builder::ObjectBuilder;

        // VERIFY: All fields are present.

        let exp = ObjectBuilder::new()
                      .insert("views",
                              ObjectBuilder::new()
                                  .insert("alpha",
                                          ObjectBuilder::new()
                                              .insert("map", "function(doc) { emit(doc.name, doc.value); }")
                                              .insert("reduce", "function(keys, values) { return sum(values); }")
                                              .unwrap())
                                  .insert("bravo",
                                          ObjectBuilder::new()
                                              .insert("map", "function(doc) { emit(doc.name, doc.other_value); }")
                                              .unwrap())
                                  .unwrap())
                      .unwrap();

        let design = DesignBuilder::new()
                         .insert_view("alpha",
                                      ViewFunction {
                                          map: "function(doc) { emit(doc.name, doc.value); }".to_string(),
                                          reduce: Some("function(keys, values) { return sum(values); }".to_string()),
                                      })
                         .insert_view("bravo",
                                      ViewFunction {
                                          map: "function(doc) { emit(doc.name, doc.other_value); }".to_string(),
                                          reduce: None,
                                      })
                         .unwrap();

        let s = serde_json::to_string(&design).unwrap();
        let got = serde_json::from_str::<serde_json::Value>(&s).unwrap();

        assert_eq!(got, exp);

        // VERIFY: The `views` field is empty.

        let exp = ObjectBuilder::new().unwrap();

        let design = DesignBuilder::new().unwrap();

        let s = serde_json::to_string(&design).unwrap();
        let got = serde_json::from_str::<serde_json::Value>(&s).unwrap();
        assert_eq!(got, exp);
    }


    #[test]
    fn test_deserialization() {

        // VERIFY: All fields are present.

        let exp = DesignBuilder::new()
                      .insert_view("alpha",
                                   ViewFunction {
                                       map: "function(doc) { emit(doc.name, doc.value); }".to_string(),
                                       reduce: Some("function(keys, values) { return sum(values); }".to_string()),
                                   })
                      .insert_view("bravo",
                                   ViewFunction {
                                       map: "function(doc) { emit(doc.name, doc.other_value); }".to_string(),
                                       reduce: None,
                                   })
                      .unwrap();

        let s = r#"{
            "views": {
                "alpha": {
                    "map": "function(doc) { emit(doc.name, doc.value); }",
                    "reduce": "function(keys, values) { return sum(values); }"
                },
                "bravo": {
                    "map": "function(doc) { emit(doc.name, doc.other_value); }"
                }
            }
        }"#;

        let got = serde_json::from_str::<Design>(&s).unwrap();

        assert_eq!(got, exp);

        // VERIFY: The `views` field is missing.

        let exp = Design::new();
        let s = "{}";
        let got = serde_json::from_str::<Design>(&s).unwrap();
        assert_eq!(got, exp);

        // VERIFY: With invalid field.

        let s = r#"{
            "views": {
                "alpha": {
                    "map": "function(doc) { emit(doc.name, doc.value); }",
                    "reduce": "function(keys, values) { return sum(values); }"
                },
                "bravo": {
                    "map": "function(doc) { emit(doc.name, doc.other_value); }"
                }
            },
            "foo": 42
        }"#;

        let got = serde_json::from_str::<Design>(&s).unwrap_err();
        jsontest::assert_unknown_field(&got, "foo");
    }
}
