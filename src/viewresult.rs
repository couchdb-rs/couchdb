use serde;
use std;

use dbpath::DatabasePath;
use dbtype;
use viewrow::ViewRow;

/// View result.
///
/// `ViewResult` is the response from getting a view.
///
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ViewResult<K, V>
    where K: serde::Deserialize,
          V: serde::Deserialize
{
    // FIXME: Document these fields.
    pub total_rows: Option<u64>,
    pub offset: Option<u64>,
    pub rows: Vec<ViewRow<K, V>>,
}

impl<K, V> ViewResult<K, V>
    where K: serde::Deserialize,
          V: serde::Deserialize
{
    #[doc(hidden)]
    pub fn new_from_db_view_result(
        db_path: &DatabasePath,
        mut db_result: dbtype::ViewResult<K, V>)
        -> Self
    {
        let db_rows = std::mem::replace(&mut db_result.rows, Vec::new());
        let dst_rows = db_rows.into_iter()
            .map(|db_row| { ViewRow::new_from_db_view_row(db_path, db_row) })
            .collect();
        ViewResult {
            total_rows: db_result.total_rows,
            offset: db_result.offset,
            rows: dst_rows,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use dbpath::DatabasePath;
    use dbtype;
    use docpath::DocumentPath;
    use viewrow::ViewRow;

    #[test]
    fn test_new_view_result_from_db_type() {

        let db_path = DatabasePath::from("dbpath");

        // Verify: With a document id.
        let src = dbtype::ViewResult {
            total_rows: Some(123),
            offset: Some(66),
            rows: vec![
                dbtype::ViewRow {
                    id: Some("alpha".to_string()),
                    key: Some("bravo".to_string()),
                    value: 42,
                },
                dbtype::ViewRow {
                    id: Some("charlie".to_string()),
                    key: Some("delta".to_string()),
                    value: 17,
                },
            ],
        };
        let got = ViewResult::new_from_db_view_result(&db_path, src);
        let exp = ViewResult {
            total_rows: Some(123),
            offset: Some(66),
            rows: vec![
                ViewRow {
                    path: Some(DocumentPath::from("dbpath/alpha")),
                    key: Some("bravo".to_string()),
                    value: 42,
                },
                ViewRow {
                    path: Some(DocumentPath::from("dbpath/charlie")),
                    key: Some("delta".to_string()),
                    value: 17,
                },
            ],
        };
        assert_eq!(got, exp);

        // Verify: Without a document id.
        let src = dbtype::ViewResult {
            total_rows: Some(123),
            offset: Some(66),
            rows: vec![
                dbtype::ViewRow {
                    id: None,
                    key: Some("bravo".to_string()),
                    value: 42,
                },
            ],
        };
        let got = ViewResult::new_from_db_view_result(&db_path, src);
        let exp = ViewResult {
            total_rows: Some(123),
            offset: Some(66),
            rows: vec![
                ViewRow {
                    path: None,
                    key: Some("bravo".to_string()),
                    value: 42,
                },
            ],
        };
        assert_eq!(got, exp);

    }
}
