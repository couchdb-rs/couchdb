use serde;
use std;

use DocumentId;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ViewRow<K, V>
    where K: serde::Deserialize,
          V: serde::Deserialize
{
    pub id: Option<DocumentId>,
    pub key: Option<K>,
    pub value: V,
}

impl<K, V> serde::Deserialize for ViewRow<K, V>
    where K: serde::Deserialize,
          V: serde::Deserialize
{
    fn deserialize<D>(d: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        enum Field {
            Id,
            Key,
            Value,
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
                            "key" => Ok(Field::Key),
                            "value" => Ok(Field::Value),
                            _ => Err(E::unknown_field(value)),
                        }
                    }
                }

                d.visit(Visitor)
            }
        }

        struct Visitor<K2, V2>
            where K2: serde::Deserialize,
                  V2: serde::Deserialize
        {
            _phantom_key: std::marker::PhantomData<K2>,
            _phantom_value: std::marker::PhantomData<V2>,
        }

        impl<K2, V2> serde::de::Visitor for Visitor<K2, V2>
            where K2: serde::Deserialize,
                  V2: serde::Deserialize
{
            type Value = ViewRow<K2, V2>;

            fn visit_map<Vis>(&mut self, mut visitor: Vis) -> Result<Self::Value, Vis::Error>
                where Vis: serde::de::MapVisitor
            {
                let mut id = None;
                let mut key = None;
                let mut value = None;
                loop {
                    match try!(visitor.visit_key()) {
                        Some(Field::Id) => {
                            id = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Key) => {
                            key = try!(visitor.visit_value()); // allow null
                        }
                        Some(Field::Value) => {
                            value = Some(try!(visitor.visit_value()));
                        }
                        None => {
                            break;
                        }
                    }
                }

                try!(visitor.end());

                let value = match value {
                    Some(x) => x,
                    None => try!(visitor.missing_field("value")),
                };

                Ok(ViewRow {
                    id: id,
                    key: key,
                    value: value,
                })
            }
        }

        static FIELDS: &'static [&'static str] = &["id", "key", "value"];
        d.visit_struct("ViewRow",
                       FIELDS,
                       Visitor::<K, V> {
                           _phantom_key: std::marker::PhantomData,
                           _phantom_value: std::marker::PhantomData,
                       })
    }
}

#[cfg(test)]
mod tests {

    use serde_json;

    use DocumentId;
    use ViewRow;

    #[test]
    fn view_row_deserialization_with_all_fields() {
        let expected = ViewRow {
            id: Some(DocumentId::Normal("foo".into())),
            key: Some("bar".to_string()),
            value: 42,
        };
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("id", "foo")
                         .insert("key", "bar")
                         .insert("value", 42)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn view_row_deserialization_with_no_id_field() {
        let expected = ViewRow {
            id: None,
            key: Some("bar".to_string()),
            value: 42,
        };
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("key", "bar")
                         .insert("value", 42)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn view_row_deserialization_with_no_key_field() {
        let expected = ViewRow::<String, _> {
            id: Some(DocumentId::Normal("foo".into())),
            key: None,
            value: 42,
        };
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("id", "foo")
                         .insert("key", serde_json::Value::Null)
                         .insert("value", 42)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn view_row_deserialization_with_no_value_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("id", "foo")
                         .insert("key", "bar")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<ViewRow<String, i32>>(&s);
        expect_json_error_missing_field!(got, "value");
    }
}
