use serde;
use std;

use DocumentId;

/// Single row contained in the response resulting from executing a view.
///
/// A `ViewRow` takes one of two forms. The first form is that the view has been
/// reduced, in which case the `id` and `key` fields are `None` and the `value`
/// field contains the reduced result. The second form is that the view has not
/// been reduced, in which case the `id` and `key` fields are `Some` and the
/// `value` field contains one of the rows of the view result.
///
/// Although the `ViewRow` type implements the `Ord` and `PartialOrd` traits, it
/// provides no guarantees how that ordering is defined and may change the
/// definition between any two releases of the couchdb crate. That is, for two
/// `ViewRow` values `a` and `b`, the expression `a < b` may hold true now but
/// not in a subsequent release. Consequently, applications must not rely upon
/// any particular ordering definition.
///
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ViewRow<K, V>
    where K: serde::Deserialize,
          V: serde::Deserialize
{
    /// Id of the document, if and only if the view has not been reduced.
    pub id: Option<DocumentId>,

    /// Emitted key, if and only if the view has not been reduced.
    pub key: Option<K>,

    /// Emitted value, either reduced or not.
    pub value: V,

    // Include a private field to prevent applications from directly
    // constructing this struct. This allows us to add new fields without
    // breaking applications.
    _dummy: std::marker::PhantomData<()>,
}

impl<K, V> ViewRow<K, V>
    where K: serde::Deserialize,
          V: serde::Deserialize
{
    /// Constructs a minimum view row containing only a value.
    ///
    /// The newly constructed view row has no id and no key.
    ///
    pub fn new<T: Into<V>>(value: T) -> Self {
        ViewRow {
            id: None,
            key: None,
            value: value.into(),
            _dummy: std::marker::PhantomData,
        }
    }
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
                    _dummy: std::marker::PhantomData,
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
    use std;

    use DocumentId;
    use ViewRow;

    #[test]
    fn view_row_new() {
        let expected = ViewRow::<String, i32> {
            id: None,
            key: None,
            value: 42,
            _dummy: std::marker::PhantomData,
        };
        let got = ViewRow::new(42);
        assert_eq!(expected, got);
    }

    #[test]
    fn view_row_deserialization_with_all_fields() {
        let expected = ViewRow {
            id: Some(DocumentId::Normal("foo".into())),
            key: Some("bar".to_string()),
            value: 42,
            _dummy: std::marker::PhantomData,
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
            _dummy: std::marker::PhantomData,
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
            _dummy: std::marker::PhantomData,
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
