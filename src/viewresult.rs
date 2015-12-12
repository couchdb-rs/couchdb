use serde;
use std;

use dbpath::DatabasePath;
use dbtype;
use viewrow::ViewRow;

/// Response resulting from executing a view.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ViewResult<K, V>
    where K: serde::Deserialize,
          V: serde::Deserialize
{
    /// Total number of rows in the view, including those not contained within
    /// this result.
    pub total_rows: Option<u64>,

    /// Zero-based offset of the first row contained within this result.
    ///
    /// The `offset` field specifies the number of rows in the view whose key is
    /// less than the key of the first row in the `rows` field.
    ///
    pub offset: Option<u64>,

    /// Rows contained within this result.
    ///
    /// A view may have rows not contained within this result. For example, this
    /// may happen when using the `startkey` or `endkey` parameters when
    /// executing the view.
    ///
    pub rows: Vec<ViewRow<K, V>>,
}

impl<K, V> ViewResult<K, V>
    where K: serde::Deserialize,
          V: serde::Deserialize
{
    #[doc(hidden)]
    pub fn from_db_view_result(db_path: &DatabasePath, mut db_result: dbtype::ViewResult<K, V>) -> Self {
        let db_rows = std::mem::replace(&mut db_result.rows, Vec::new());
        let dst_rows = db_rows.into_iter()
                              .map(|db_row| ViewRow::from_db_view_row(db_path, db_row))
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
        let got = ViewResult::from_db_view_result(&db_path, src);
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
        let got = ViewResult::from_db_view_result(&db_path, src);
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
