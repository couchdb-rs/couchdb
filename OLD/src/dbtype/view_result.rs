use serde;
use std;

use ViewRow;

/// Response resulting from executing a view.
///
/// A `ViewResult` contains all content in the response from the CouchDB server
/// as a result of executing a view.
///
/// A `ViewResult` takes one of two forms. The first form is that the view has
/// been reduced, in which case the `total_rows` and `offset` fields are `None`
/// and the `rows` field contains a single row containing the reduced result.
/// The second form is that the view has not been reduced, in which case the
/// `total_rows` and `offset` fields are `Some` and the `rows` field contains
/// zero or more rows containing the unreduced result.
///
/// Although the `ViewResult` type implements the `Ord` and `PartialOrd` traits,
/// it provides no guarantees how that ordering is defined and may change the
/// definition between any two releases of the couchdb crate. That is, for two
/// `ViewResult` values `a` and `b`, the expression `a < b` may hold true now
/// but not in a subsequent release. Consequently, applications must not rely
/// upon any particular ordering definition.
///
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ViewResult<K, V>
    where K: serde::Deserialize,
          V: serde::Deserialize
{
    /// Number of rows in an unreduced view, including rows excluded in the
    /// `rows` field.
    pub total_rows: Option<u64>,

    /// Number of rows in an unreduced view excluded before the first row in the
    /// `rows` field.
    pub offset: Option<u64>,

    /// All rows included in the response content for an unreduced view, or, for
    /// a reduced view, the one row containing the reduced result.
    pub rows: Vec<ViewRow<K, V>>,

    // Include a private field to prevent applications from directly
    // constructing this struct. This allows us to add new fields without
    // breaking applications.
    _dummy: std::marker::PhantomData<()>,
}

impl<K, V> ViewResult<K, V>
    where K: serde::Deserialize,
          V: serde::Deserialize
{
    /// Constructs an empty view result.
    pub fn new() -> Self {
        ViewResult {
            total_rows: None,
            offset: None,
            rows: Vec::new(),
            _dummy: std::marker::PhantomData,
        }
    }

    /// Constructs a view result containing only the given value.
    pub fn new_reduced<T: Into<V>>(value: T) -> Self {
        ViewResult { rows: vec![ViewRow::new(value)], ..Self::new() }
    }
}

impl<K, V> serde::Deserialize for ViewResult<K, V>
    where K: serde::Deserialize,
          V: serde::Deserialize
{
    fn deserialize<D>(d: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        enum Field {
            TotalRows,
            Offset,
            Rows,
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
                            "total_rows" => Ok(Field::TotalRows),
                            "offset" => Ok(Field::Offset),
                            "rows" => Ok(Field::Rows),
                            _ => Err(E::unknown_field(value)),
                        }
                    }
                }

                d.deserialize(Visitor)
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
            type Value = ViewResult<K2, V2>;

            fn visit_map<Vis>(&mut self, mut visitor: Vis) -> Result<Self::Value, Vis::Error>
                where Vis: serde::de::MapVisitor
            {
                let mut total_rows = None;
                let mut offset = None;
                let mut rows = None;
                loop {
                    match try!(visitor.visit_key()) {
                        Some(Field::TotalRows) => {
                            total_rows = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Offset) => {
                            offset = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Rows) => {
                            rows = Some(try!(visitor.visit_value()));
                        }
                        None => {
                            break;
                        }
                    }
                }

                try!(visitor.end());

                let rows = match rows {
                    Some(x) => x,
                    None => try!(visitor.missing_field("rows")),
                };

                Ok(ViewResult {
                    total_rows: total_rows,
                    offset: offset,
                    rows: rows,
                    _dummy: std::marker::PhantomData,
                })
            }
        }

        static FIELDS: &'static [&'static str] = &["total_rows", "offset", "rows"];
        d.deserialize_struct("ViewResult",
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

    use ViewResult;
    use ViewRow;

    #[test]
    fn view_result_new() {
        let expected = ViewResult::<String, i32> {
            total_rows: None,
            offset: None,
            rows: vec![],
            _dummy: std::marker::PhantomData,
        };
        let got = ViewResult::new();
        assert_eq!(expected, got);
    }

    #[test]
    fn view_result_new_reduced() {
        let expected = ViewResult::<String, i32> {
            total_rows: None,
            offset: None,
            rows: vec![ViewRow::new(42)],
            _dummy: std::marker::PhantomData,
        };
        let got = ViewResult::new_reduced(42);
        assert_eq!(expected, got);
    }

    #[test]
    fn view_result_deserialization_unreduced_view() {
        let expected = ViewResult::<String, i32> {
            total_rows: Some(42),
            offset: Some(17),
            rows: vec![{
                           let mut v = ViewRow::new(5);
                           v.id = Some("foo".into());
                           v.key = Some("bar".into());
                           v
                       },
                       {
                           let mut v = ViewRow::new(13);
                           v.id = Some("qux".into());
                           v.key = Some("kit".into());
                           v
                       }],
            _dummy: std::marker::PhantomData,
        };
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("total_rows", 42)
                         .insert("offset", 17)
                         .insert_array("rows", |x| {
                             x.push_object(|x| {
                                  x.insert("id", "foo")
                                   .insert("key", "bar")
                                   .insert("value", 5)
                              })
                              .push_object(|x| {
                                  x.insert("id", "qux")
                                   .insert("key", "kit")
                                   .insert("value", 13)
                              })
                         })
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn view_result_deserialization_reduce_view() {
        let expected = ViewResult::<String, i32> {
            total_rows: None,
            offset: None,
            rows: vec![{
                           let mut v = ViewRow::new(5);
                           v.id = Some("foo".into());
                           v.key = Some("bar".into());
                           v
                       }],
            _dummy: std::marker::PhantomData,
        };
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert_array("rows", |x| {
                             x.push_object(|x| {
                                 x.insert("id", "foo")
                                  .insert("key", "bar")
                                  .insert("value", 5)
                             })
                         })
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn view_result_deserialization_with_no_rows_field() {
        let source = serde_json::builder::ObjectBuilder::new().unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<ViewResult<String, i32>>(&s);
        expect_json_error_missing_field!(got, "rows");
    }
}
