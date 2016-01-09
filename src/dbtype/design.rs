use serde;
use std;

use ViewFunction;
use ViewFunctionBuilder;
use ViewFunctionMap;

/// Content of a design document.
///
/// CouchDB design documents contain many fields. However, the `Design` type
/// currently supports only views.
///
/// Applications may construct a `Design` by using a `DesignBuilder`.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Design {
    /// Map of view names to the view function for each name.
    pub views: ViewFunctionMap,

    // Include a private field to prevent applications from directly
    // constructing this struct. This allows us to add new fields without
    // breaking applications.
    _dummy: std::marker::PhantomData<()>,
}

impl Design {
    /// Construct an empty design document.
    ///
    /// An empty design document contains no views.
    ///
    pub fn new() -> Self {
        Design {
            _dummy: std::marker::PhantomData,
            views: ViewFunctionMap::new(),
        }
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

                let v = Design {
                    _dummy: std::marker::PhantomData,
                    views: views,
                };

                Ok(v)
            }
        }

        static FIELDS: &'static [&'static str] = &["views"];
        d.visit_struct("Design", FIELDS, Visitor)
    }
}

/// Builder for constructing a design document.
///
/// # Examples
///
/// ```
/// use couchdb::{DesignBuilder, ViewFunction};
/// let design = DesignBuilder::new()
///                  .build_view("foo",
///                              "function(doc) { if (doc.foo) { emit(doc.name, doc.foo); } }",
///                              |x| x)
///                  .build_view("bar",
///                              "function(doc) { if (doc.foo) { emit(doc.name, doc.bar); } }",
///                              |x| x.set_reduce("_sum"))
///                  .unwrap();
/// assert_eq!(design.views.get("foo").unwrap().map,
///            "function(doc) { if (doc.foo) { emit(doc.name, doc.foo); } }".to_string());
/// assert_eq!(design.views.get("bar").unwrap().reduce,
///            Some("_sum".to_string()));
/// ```
///
#[derive(Debug)]
pub struct DesignBuilder {
    design: Design,
}

impl DesignBuilder {
    /// Constructs a builder containing an empty design document.
    pub fn new() -> Self {
        DesignBuilder {
            design: Design {
                _dummy: std::marker::PhantomData,
                views: ViewFunctionMap::new(),
            },
        }
    }

    /// Returns the design document contained within the builder.
    pub fn unwrap(self) -> Design {
        self.design
    }

    /// Adds a view function to the design document contained within the
    /// builder.
    ///
    /// If the design document already contains a view with the same name then
    /// the new view function replaces the existing view function.
    ///
    pub fn insert_view<T, U>(mut self, view_name: T, view_function: U) -> Self
        where T: Into<String>,
              U: Into<ViewFunction>
    {
        self.design.views.insert(view_name.into(), view_function.into());
        self
    }

    /// Builds a view function and adds it to the design document contained
    /// within the builder.
    ///
    /// If the design document already contains a view with the same name then
    /// the new view function replaces the existing view function.
    ///
    pub fn build_view<T, U, F>(mut self, view_name: T, map_function: U, f: F) -> Self
        where T: Into<String>,
              U: Into<String>,
              F: FnOnce(ViewFunctionBuilder) -> ViewFunctionBuilder
    {
        let b = ViewFunctionBuilder::new(map_function.into());
        self.design.views.insert(view_name.into(), f(b).unwrap());
        self
    }
}

#[cfg(test)]
mod tests {

    use serde_json;
    use std;

    use Design;
    use DesignBuilder;
    use ViewFunctionBuilder;
    use ViewFunctionMap;

    #[test]
    fn design_builder_empty() {
        let expected = Design {
            _dummy: std::marker::PhantomData,
            views: ViewFunctionMap::new(),
        };
        let got = DesignBuilder::new().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn design_builder_insert_view() {
        let views = {
            let mut x = ViewFunctionMap::new();
            x.insert("foo".to_string(),
                     ViewFunctionBuilder::new("function(doc) { emit(doc.name, doc.foo); }")
                         .set_reduce("function(keys, values) { return sum(values); }")
                         .unwrap());
            x.insert("bar".to_string(),
                     ViewFunctionBuilder::new("function(doc) { emit(doc.name, doc.bar); }")
                         .unwrap());
            x
        };
        let expected = Design {
            _dummy: std::marker::PhantomData,
            views: views,
        };
        let got = DesignBuilder::new()
                      .insert_view("foo",
                                   ViewFunctionBuilder::new("function(doc) { emit(doc.name, \
                                                             doc.foo); }")
                                       .set_reduce("function(keys, values) { return \
                                                    sum(values); }")
                                       .unwrap())
                      .insert_view("bar",
                                   ViewFunctionBuilder::new("function(doc) { emit(doc.name, \
                                                             doc.bar); }")
                                       .unwrap())
                      .unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn design_builder_insert_view_replaces_same_name() {
        let views = {
            let mut x = ViewFunctionMap::new();
            x.insert("foo".to_string(),
                     ViewFunctionBuilder::new("function(doc) { emit(doc.name, doc.foo); }")
                         .unwrap());
            x
        };
        let expected = Design {
            _dummy: std::marker::PhantomData,
            views: views,
        };
        let got = DesignBuilder::new()
                      .insert_view("foo", ViewFunctionBuilder::new("function(doc) {}").unwrap())
                      .insert_view("foo",
                                   ViewFunctionBuilder::new("function(doc) { emit(doc.name, \
                                                             doc.foo); }")
                                       .unwrap())
                      .unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn design_builder_build_view() {
        let views = {
            let mut x = ViewFunctionMap::new();
            x.insert("foo".to_string(),
                     ViewFunctionBuilder::new("function(doc) { emit(doc.name, doc.foo); }")
                         .set_reduce("function(keys, values) { return sum(values); }")
                         .unwrap());
            x.insert("bar".to_string(),
                     ViewFunctionBuilder::new("function(doc) { emit(doc.name, doc.bar); }")
                         .unwrap());
            x
        };
        let expected = Design {
            _dummy: std::marker::PhantomData,
            views: views,
        };
        let got = DesignBuilder::new()
                      .build_view("foo", "function(doc) { emit(doc.name, doc.foo); }", |x| {
                          x.set_reduce("function(keys, values) { return sum(values); }")
                      })
                      .build_view("bar", "function(doc) { emit(doc.name, doc.bar); }", |x| x)
                      .unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn design_builder_build_view_replaces_same_name() {
        let views = {
            let mut x = ViewFunctionMap::new();
            x.insert("foo".to_string(),
                     ViewFunctionBuilder::new("function(doc) { emit(doc.name, doc.foo); }")
                         .unwrap());
            x
        };
        let expected = Design {
            _dummy: std::marker::PhantomData,
            views: views,
        };
        let got = DesignBuilder::new()
                      .build_view("foo", "function(doc) {}", |x| x)
                      .build_view("foo", "function(doc) { emit(doc.name, doc.foo); }", |x| x)
                      .unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn design_serialization_with_all_fields() {
        let expected = serde_json::builder::ObjectBuilder::new()
                           .insert_object("views", |x| {
                               x.insert_object("foo", |x| {
                                    x.insert("map", "function(doc) { emit(doc.name, doc.foo); }")
                                     .insert("reduce",
                                             "function(keys, values) { return sum(values); }")
                                })
                                .insert_object("bar", |x| {
                                    x.insert("map", "function(doc) { emit(doc.name, doc.bar); }")
                                })
                           })
                           .unwrap();
        let source = DesignBuilder::new()
                         .build_view("foo", "function(doc) { emit(doc.name, doc.foo); }", |x| {
                             x.set_reduce("function(keys, values) { return sum(values); }")
                         })
                         .build_view("bar", "function(doc) { emit(doc.name, doc.bar); }", |x| x)
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
                           .build_view("foo", "function(doc) { emit(doc.name, doc.foo); }", |x| {
                               x.set_reduce("function(keys, values) { return sum(values); }")
                           })
                           .build_view("bar", "function(doc) { emit(doc.name, doc.bar); }", |x| x)
                           .unwrap();
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert_object("views", |x| {
                             x.insert_object("foo", |x| {
                                  x.insert("map", "function(doc) { emit(doc.name, doc.foo); }")
                                   .insert("reduce",
                                           "function(keys, values) { return sum(values); }")
                              })
                              .insert_object("bar", |x| {
                                  x.insert("map", "function(doc) { emit(doc.name, doc.bar); }")
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
