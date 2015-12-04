use serde;
use std;

/// Associative collection for view functions.
pub type ViewFunctionMap = std::collections::BTreeMap<String, ViewFunction>;

/// Design document content.
#[derive(Debug)]
pub struct Design {
    pub views: ViewFunctionMap,
}

impl Design {
    pub fn new() -> Self{
        Design {
            views: ViewFunctionMap::new(),
        }
    }
}

impl std::cmp::Eq for Design {}

impl std::cmp::PartialEq for Design {
    fn eq(&self, other: &Design) -> bool {
        self.views == other.views
    }
}

impl serde::Serialize for Design {

    fn serialize<S: serde::Serializer>(&self, s: &mut S) -> Result<(), S::Error> {

        struct Visitor<'a> {
            value: &'a Design,
            state: u8,
        }

        impl <'a> serde::ser::MapVisitor for Visitor<'a> {
            fn visit<S: serde::Serializer>(&mut self, s: &mut S) -> Result<Option<()>, S::Error> {
                loop {
                    match self.state {
                        0 => {
                            self.state += 1;
                            if !self.value.views.is_empty() {
                                return Ok(Some(try!(s.visit_struct_elt("views",
                                                                       &self.value.views))));
                            }
                        }
                        _ => {
                            return Ok(None);
                        }
                    }
                }
            }
        }

        s.visit_struct("Design", Visitor {
            value: self,
            state: 0,
        })
    }
}

impl serde::Deserialize for Design {

    fn deserialize<D: serde::Deserializer>(d: &mut D) -> Result<Self, D::Error> {

        enum Field {
            Views,
        }

        impl serde::Deserialize for Field {

            fn deserialize<D: serde:: Deserializer>(d: &mut D) -> Result<Field, D::Error> {

                struct Visitor;

                impl serde::de::Visitor for Visitor {
                    type Value = Field;

                    fn visit_str<E: serde::de::Error>(&mut self, value: &str) -> Result<Field, E> {
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

            fn visit_map<V: serde::de::MapVisitor>(
                            &mut self,
                            mut visitor: V) -> Result<Design, V::Error> {

                let mut views = None;
                loop {
                    match try!(visitor.visit_key()) {
                        Some(Field::Views) => {
                            views = Some(try!(visitor.visit_value()));
                        }
                        None => { break; }
                    }
                }

                try!(visitor.end());

                let views = match views {
                    Some(x) => x,
                    None => ViewFunctionMap::new(),
                };

                Ok(Design {
                    views: views,
                })
            }
        }

        static FIELDS: &'static[&'static str] = &["views"];
        d.visit_struct("Design", FIELDS, Visitor)
    }
}

/// View function.
#[derive(Debug)]
pub struct ViewFunction {
    pub map: String,
    pub reduce: Option<String>,
}

impl std::cmp::Eq for ViewFunction {}

impl std::cmp::PartialEq for ViewFunction {
    fn eq(&self, other: &ViewFunction) -> bool {
        self.map == other.map &&
            self.reduce == other.reduce
    }
}

impl serde::Serialize for ViewFunction {
    fn serialize<S: serde::Serializer>(&self, s: &mut S) -> Result<(), S::Error> {

        struct Visitor<'a> {
            value: &'a ViewFunction,
            state: u8,
        }

        impl <'a> serde::ser::MapVisitor for Visitor<'a> {
            fn visit<S: serde::Serializer>(&mut self, s: &mut S) -> Result<Option<()>, S::Error> {
                loop {
                    match self.state {
                        0 => {
                            self.state += 1;
                            return Ok(Some(try!(s.visit_struct_elt("map",
                                                                   &self.value.map))));
                        }
                        1 => {
                            self.state += 1;
                            if self.value.reduce.is_some() {
                                return Ok(Some(try!(s.visit_struct_elt("reduce",
                                                                       self.value.reduce.as_ref()))));
                            }
                        }
                        _ => {
                            return Ok(None);
                        }
                    }
                }
            }
        }

        s.visit_struct("ViewFunction", Visitor {
            value: self,
            state: 0,
        })
    }
}

impl serde::Deserialize for ViewFunction {
    fn deserialize<D: serde::Deserializer>(d: &mut D) -> Result<ViewFunction, D::Error> {

        enum Field {
            Map,
            Reduce,
        }

        impl serde::Deserialize for Field {
            fn deserialize<D: serde::Deserializer>(d: &mut D) -> Result<Field, D::Error> {

                struct Visitor;

                impl serde::de::Visitor for Visitor {
                    type Value = Field;

                    fn visit_str<E: serde::de::Error>(&mut self, value: &str) -> Result<Field, E> {
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

            fn visit_map<V: serde::de::MapVisitor>(
                            &mut self,
                            mut visitor: V) -> Result<ViewFunction, V::Error> {
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

                Ok(ViewFunction {
                    map: map,
                    reduce: reduce,
                })
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

    #[test]
    fn test_serialization_design() {

        let views = ViewFunctionMap::new();
        let v1 = Design {
            views: views,
        };
        let s = serde_json::to_string(&v1).unwrap();
        let v2 = serde_json::from_str(&s).unwrap();
        assert_eq!(v1, v2);

        let mut views = ViewFunctionMap::new();
        views.insert("alpha".to_string(), ViewFunction {
            map: "function(doc) { emit(doc.alpha); }".to_string(),
            reduce: None,
        });
        views.insert("bravo".to_string(), ViewFunction {
            map: "function(doc) { emit(doc.bravo); }".to_string(),
            reduce: None,
        });
        let v1 = Design {
            views: views,
        };
        let s = serde_json::to_string(&v1).unwrap();
        let v2 = serde_json::from_str(&s).unwrap();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_serialization_view_function() {

        let v1 = ViewFunction {
            map: "function(doc) { emit(doc.name); }".to_string(),
            reduce: None,
        };
        let s = serde_json::to_string(&v1).unwrap();
        let v2 = serde_json::from_str(&s).unwrap();
        assert_eq!(v1, v2);

        let v1 = ViewFunction {
            map: "function(doc) { emit(doc.name); }".to_string(),
            reduce: Some("function(keys, values) { return sum(values); }".to_string()),
        };
        let s = serde_json::to_string(&v1).unwrap();
        let v2 = serde_json::from_str(&s).unwrap();
        assert_eq!(v1, v2);
    }
}
