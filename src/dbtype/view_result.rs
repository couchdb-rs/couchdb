use serde;
use std;

use ViewRow;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ViewResult<K, V>
    where K: serde::Deserialize,
          V: serde::Deserialize
{
    pub total_rows: Option<u64>,
    pub offset: Option<u64>,
    pub rows: Vec<ViewRow<K, V>>,
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
                })
            }
        }

        static FIELDS: &'static [&'static str] = &["total_rows", "offset", "rows"];
        d.visit_struct("ViewResult",
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

    use ViewResult;
    use ViewRow;

    #[test]
    fn view_result_deserialization_non_reduced_view() {
        let expected = ViewResult::<String, i32> {
            total_rows: Some(42),
            offset: Some(17),
            rows: vec![ViewRow {
                           id: Some("foo".into()),
                           key: Some("bar".into()),
                           value: 5,
                       },
                       ViewRow {
                           id: Some("qux".into()),
                           key: Some("kit".into()),
                           value: 13,
                       }],
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
            rows: vec![ViewRow {
                           id: Some("foo".into()),
                           key: Some("bar".into()),
                           value: 5,
                       }],
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
        let got = serde_json::from_str::<ViewRow<String, i32>>(&s);
        expect_json_error_missing_field!(got, "rows");
    }
}
