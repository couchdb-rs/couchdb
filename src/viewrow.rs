use serde;

use dbpath::DatabasePath;
use dbtype;
use docpath::DocumentPath;

/// Single row contained within the response resulting from executing a view.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ViewRow<K, V>
    where K: serde::Deserialize,
          V: serde::Deserialize
{
    /// Path for this document.
    ///
    /// The `path` field replaces the `id` field returned by the CouchDB server.
    /// The `path` field provides stronger type-safety because it binds the
    /// database name with the document id.
    ///
    pub path: Option<DocumentPath>,

    /// Key emitted by the view function.
    pub key: Option<K>,

    /// Value emitted by the view function.
    pub value: V,
}

impl<K, V> ViewRow<K, V>
    where K: serde::Deserialize,
          V: serde::Deserialize
{
    #[doc(hidden)]
    pub fn from_db_view_row(
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
        let got = ViewRow::from_db_view_row(&db_path, src);
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
        let got = ViewRow::from_db_view_row(&db_path, src);
        let exp = ViewRow {
            path: None,
            key: Some("blah".to_string()),
            value: 42,
        };
        assert_eq!(got, exp);
    }
}
