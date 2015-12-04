use serde;

use dbpath::DatabasePath;
use dbtype;
use docpath::DocumentPath;

/// View row.
///
/// `ViewRow` is a single row within the response from getting a view.
///
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ViewRow<K, V>
    where K: serde::Deserialize,
          V: serde::Deserialize
{
    // FIXME: Document these fields.
    pub path: Option<DocumentPath>,
    pub key: Option<K>,
    pub value: V,
}

impl<K, V> ViewRow<K, V>
    where K: serde::Deserialize,
          V: serde::Deserialize
{
    #[doc(hidden)]
    pub fn new_from_db_view_row(
        db_path: &DatabasePath,
        db_row: dbtype::ViewRow<K, V>)
        -> Self
    {
        ViewRow {
            path: db_row.id.map(|doc_id| {
                DocumentPath::new(db_path.clone(), doc_id)
            }),
            key: db_row.key,
            value: db_row.value,
        }
    }
}

#[cfg(test)]
mod tests {

    use dbpath::DatabasePath;
    use dbtype;
    use docpath::DocumentPath;
    use super::*;

    #[test]
    fn test_new_view_row_from_db_type() {

        let db_path = DatabasePath::from("dbpath");

        // Verify: With a document id.
        let src = dbtype::ViewRow {
            id: Some("docid".to_string()),
            key: Some("blah".to_string()),
            value: 42,
        };
        let got = ViewRow::new_from_db_view_row(&db_path, src);
        let exp = ViewRow {
            path: Some(DocumentPath::from("dbpath/docid")),
            key: Some("blah".to_string()),
            value: 42,
        };
        assert_eq!(got, exp);

        // Verify: Without a document id.
        let src = dbtype::ViewRow {
            id: None,
            key: Some("blah".to_string()),
            value: 42,
        };
        let got = ViewRow::new_from_db_view_row(&db_path, src);
        let exp = ViewRow {
            path: None,
            key: Some("blah".to_string()),
            value: 42,
        };
        assert_eq!(got, exp);
    }
}
