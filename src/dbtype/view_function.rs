use serde;
use std;

/// Associative collection for view functions.
pub type ViewFunctionMap = std::collections::HashMap<String, ViewFunction>;

/// Builder for constructing a view function.
///
/// # Examples
///
/// ```
/// use couchdb::{ViewFunction, ViewFunctionBuilder};
/// let s = "function(doc) { if (doc.foo) { emit(doc.name, doc.foo); } }";
/// let v = ViewFunctionBuilder::new(s)
///             .set_reduce("_sum")
///             .unwrap();
/// assert_eq!(v.map, s);
/// assert_eq!(v.reduce, Some("_sum".to_string()));
/// ```
///
#[derive(Debug)]
pub struct ViewFunctionBuilder {
    view_function: ViewFunction,
}

impl ViewFunctionBuilder {
    /// Constructs a new view function builder.
    ///
    /// The view function contained within the builder has the given map
    /// function and an undefined reduce function.
    ///
    pub fn new<S: Into<String>>(map_function: S) -> Self {
        ViewFunctionBuilder {
            view_function: ViewFunction {
                map: map_function.into(),
                reduce: None,
            },
        }
    }

    /// Returns the view function contained within the builder.
    pub fn unwrap(self) -> ViewFunction {
        self.view_function
    }

    /// Replaces any reduce function in the view function contained within the
    /// builder.
    pub fn set_reduce<S: Into<String>>(mut self, reduce_function: S) -> Self {
        self.view_function.reduce = Some(reduce_function.into());
        self
    }
}

/// JavaScript `map` and `reduce` functions for a CouchDB view.
///
/// DEPRECATION NOTICE: Applications must not directly construct a
/// `ViewFunction`. Instead, they should use `ViewFunctionBuilder` to construct
/// the view function.
///
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ViewFunction {
    /// JavaScript function that takes a document and emits zero or more
    /// key-value pairs.
    pub map: String,

    /// JavaScript function that reduces multiple values emitted from the map
    /// function into a single value.
    pub reduce: Option<String>,
}

impl serde::Serialize for ViewFunction {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        struct Visitor<'a>(&'a ViewFunction);

        impl<'a> serde::ser::MapVisitor for Visitor<'a> {
            fn visit<S>(&mut self, s: &mut S) -> Result<Option<()>, S::Error>
                where S: serde::Serializer
            {
                let Visitor(view_func) = *self;

                try!(s.visit_struct_elt("map", &view_func.map));
                for v in view_func.reduce.iter() {
                    try!(s.visit_struct_elt("reduce", v));
                }

                Ok(None)
            }
        }

        s.visit_struct("ViewFunction", Visitor(self))
    }
}

impl serde::Deserialize for ViewFunction {
    fn deserialize<D>(d: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        enum Field {
            Map,
            Reduce,
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
                            "map" => Ok(Field::Map),
                            "reduce" => Ok(Field::Reduce),
                            _ => Err(E::unknown_field(value)),
                        }
                    }
                }

                d.visit(Visitor)
            }
        }

        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = ViewFunction;

            fn visit_map<V>(&mut self, mut visitor: V) -> Result<ViewFunction, V::Error>
                where V: serde::de::MapVisitor
            {
                let mut map = None;
                let mut reduce = None;
                loop {
                    match try!(visitor.visit_key()) {
                        Some(Field::Map) => {
                            map = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Reduce) => {
                            reduce = Some(try!(visitor.visit_value()));
                        }
                        None => {
                            break;
                        }
                    }
                }

                try!(visitor.end());

                let map = match map {
                    Some(x) => x,
                    None => try!(visitor.missing_field("map")),
                };

                let v = ViewFunction {
                    map: map,
                    reduce: reduce,
                };

                Ok(v)
            }
        }

        static FIELDS: &'static [&'static str] = &["map", "reduce"];
        d.visit_struct("ViewFunction", FIELDS, Visitor)
    }
}

#[cfg(test)]
mod tests {

    use serde_json;

    use ViewFunction;

    #[test]
    fn view_function_serialization_with_all_fields() {
        let expected = serde_json::builder::ObjectBuilder::new()
                           .insert("map", "function(doc) { emit(doc.name, doc.value); }")
                           .insert("reduce", "function(keys, values) { return sum(values); }")
                           .unwrap();
        let source = ViewFunction {
            map: "function(doc) { emit(doc.name, doc.value); }".to_string(),
            reduce: Some("function(keys, values) { return sum(values); }".to_string()),
        };
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn view_function_serialization_with_reduce_field_elided() {
        let expected = serde_json::builder::ObjectBuilder::new()
                           .insert("map", "function(doc) { emit(doc.name, doc.value); }")
                           .unwrap();
        let source = ViewFunction {
            map: "function(doc) { emit(doc.name, doc.value); }".to_string(),
            reduce: None,
        };
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn view_function_deserialization_with_all_fields() {
        let expected = ViewFunction {
            map: "function(doc) { emit(doc.name, doc.value); }".to_string(),
            reduce: Some("function(keys, values) { return sum(values); }".to_string()),
        };
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("map", "function(doc) { emit(doc.name, doc.value); }")
                         .insert("reduce", "function(keys, values) { return sum(values); }")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn view_function_deserialization_with_reduce_field_elided() {
        let expected = ViewFunction {
            map: "function(doc) { emit(doc.name, doc.value); }".to_string(),
            reduce: None,
        };
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("map", "function(doc) { emit(doc.name, doc.value); }")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn view_function_deserialization_with_map_field_elided() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("reduce", "function(keys, values) { return sum(values); }")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<ViewFunction>(&s);
        expect_json_error_missing_field!(got, "value");
    }
}
