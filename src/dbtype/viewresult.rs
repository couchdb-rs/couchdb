use serde;
use std;

use dbtype::viewrow::ViewRow;

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
                        },
                        Some(Field::Offset) => {
                            offset = Some(try!(visitor.visit_value()));
                        },
                        Some(Field::Rows) => {
                            rows = Some(try!(visitor.visit_value()));
                        },
                        None => { break; },
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
        d.visit_struct("ViewResult", FIELDS, Visitor::<K, V> {
            _phantom_key: std::marker::PhantomData,
            _phantom_value: std::marker::PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {

    use serde_json;

    use dbtype::viewrow::ViewRow;
    use super::*;

    #[test]
    fn test_view_result_serialization() {

        // Verify: All fields present--i.e., a non-reduced view.
        let s = r#"{"total_rows": 42, "offset": 17, "rows": [
                {"id": "alpha", "key": "bravo", "value": 5},
                {"id": "charlie", "key": "delta", "value": 37}
            ]}"#;
        let v = serde_json::from_str::<ViewResult<String, i32>>(&s).unwrap();
        assert_eq!(v.total_rows, Some(42));
        assert_eq!(v.offset, Some(17));
        let exp_rows = vec![
            ViewRow {
                id: Some("alpha".to_string()),
                key: Some("bravo".to_string()),
                value: 5,
            },
            ViewRow {
                id: Some("charlie".to_string()),
                key: Some("delta".to_string()),
                value: 37,
            },
        ];
        assert_eq!(v.rows, exp_rows);

        // Verify: Only one row--i.e., a reduced view.
        let s = r#"{"rows": [ {"key": null, "value": 42} ]}"#;
        let v = serde_json::from_str::<ViewResult<String, i32>>(&s).unwrap();
        assert_eq!(v.total_rows, None);
        assert_eq!(v.offset, None);
        let exp_rows = vec![
            ViewRow {
                id: None,
                key: None,
                value: 42,
            },
        ];
        assert_eq!(v.rows, exp_rows);
    }
}
