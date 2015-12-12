use serde;
use std;

/// Associative collection for view functions.
pub type ViewFunctionMap = std::collections::HashMap<String, ViewFunction>;

/// JavaScript `map` and `reduce` functions for a CouchDB view.
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
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error> where S: serde::Serializer {

        struct Visitor<'a>(&'a ViewFunction);

        impl <'a> serde::ser::MapVisitor for Visitor<'a> {
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
    fn deserialize<D>(d: &mut D) -> Result<Self, D::Error> where D: serde::Deserializer {

        enum Field {
            Map,
            Reduce,
        }

        impl serde::Deserialize for Field {
            fn deserialize<D>(d: &mut D) -> Result<Self, D::Error> where D: serde::Deserializer {

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
                        None => { break; }
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

    use super::*;
    use jsontest;

    #[test]
    fn test_serialization() {

        // VERIFY: All fields are present.

        let exp = serde_json::builder::ObjectBuilder::new()
            .insert("map", "function(doc) { emit(doc.name, doc.value); }")
            .insert("reduce", "function(keys, values) { return sum(values); }")
            .unwrap();

        let vf = ViewFunction {
            map: "function(doc) { emit(doc.name, doc.value); }".to_string(),
            reduce: Some("function(keys, values) { return sum(values); }".to_string()),
        };

        let s = serde_json::to_string(&vf).unwrap();
        let got = serde_json::from_str::<serde_json::Value>(&s).unwrap();

        assert_eq!(got, exp);

        // VERIFY: The `reduce` field is None.

        let exp = serde_json::builder::ObjectBuilder::new()
            .insert("map", "function(doc) { emit(doc.name, doc.value); }")
            .unwrap();

        let vf = ViewFunction {
            map: "function(doc) { emit(doc.name, doc.value); }".to_string(),
            reduce: None,
        };

        let s = serde_json::to_string(&vf).unwrap();
        let got = serde_json::from_str::<serde_json::Value>(&s).unwrap();

        assert_eq!(got, exp);
    }

    #[test]
    fn test_deserialization() {

        // VERIFY: All fields are present.

        let exp = ViewFunction {
            map: "function(doc) { emit(doc.name, doc.value); }".to_string(),
            reduce: Some("function(keys, values) { return sum(values); }".to_string()),
        };

        let s = r#"{
            "map": "function(doc) { emit(doc.name, doc.value); }",
            "reduce": "function(keys, values) { return sum(values); }"
        }"#;

        let got = serde_json::from_str::<ViewFunction>(&s).unwrap();

        assert_eq!(got, exp);

        // VERIFY: The `reduce` field is missing.

        let exp = ViewFunction {
            map: "function(doc) { emit(doc.name, doc.value); }".to_string(),
            reduce: None,
        };

        let s = r#"{
            "map": "function(doc) { emit(doc.name, doc.value); }"
        }"#;

        let got = serde_json::from_str::<ViewFunction>(&s).unwrap();

        assert_eq!(got, exp);

        // VERIFY: The `map` field is missing.

        let s = r#"{
            "reduce": "function(keys, values) { return sum(values); }"
        }"#;

        let got = serde_json::from_str::<ViewFunction>(&s).unwrap_err();
        jsontest::assert_missing_field(&got, "map");

        // VERIFY: With invalid field.

        let s = r#"{
            "map": "function(doc) { emit(doc.name, doc.value); }",
            "reduce": "function(keys, values) { return sum(values); }",
            "foo": 42,
        }"#;

        let got = serde_json::from_str::<ViewFunction>(&s).unwrap_err();
        jsontest::assert_unknown_field(&got, "foo");
    }
}
