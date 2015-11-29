use hyper;

use docpath::DocumentPath;

/// View path—i.e., a document path paired with a view name.
///
/// A view path pairs a `DocumentPath` with a `String`—e.g., in the HTTP request
/// `GET http://example.com:5984/db/_design/design-doc/_view/view-name`, the
/// document path comprises the `db/_design/design-doc` part and the view name
/// comprises the `view-name` part. The `_view` part is fixed in the view path.
///
/// `ViewPath` provides additional type-safety over working with raw strings.
///
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ViewPath(DocumentPath, String);

impl ViewPath {
    pub fn new<T, U>(doc_path: T, view_name: U)
        -> Self where T: Into<DocumentPath>, U: Into<String>
    {
        ViewPath(doc_path.into(), view_name.into())
    }

    /// Convert the `ViewPath` into a URI.
    pub fn into_uri(self, base_uri: hyper::Url) -> hyper::Url {

        let ViewPath(doc_path, view_name) = self;
        let mut uri = doc_path.into_uri(base_uri);

        {
            let p = uri.path_mut().unwrap();
            p.push("_view".to_string());
            p.push(view_name);
        }

        uri
    }

    /// Return the document path part of the view path.
    pub fn document_path(&self) -> &DocumentPath {
        &self.0
    }

    /// Return the view name part of the view path.
    pub fn view_name(&self) -> &String {
        &self.1
    }
}

impl From<String> for ViewPath {
    fn from(view_path: String) -> Self {
        ViewPath::from(&view_path as &str)
    }
}

impl<'a> From<&'a str> for ViewPath {
    fn from(view_path: &str) -> Self {
        let mid = "/_view/";
        let n = view_path.find(mid).unwrap();
        let (doc_path, view_name) = view_path.split_at(n);
        let view_name = &view_name[mid.len()..];
        ViewPath::new(doc_path, view_name)
    }
}

impl From<ViewPath> for (DocumentPath, String) {
    fn from(view_path: ViewPath) -> (DocumentPath, String) {
        let ViewPath(doc_path, view_name) = view_path;
        (doc_path, view_name)
    }
}

#[cfg(test)]
mod tests {

    use hyper;

    use docpath::DocumentPath;
    use super::*;

    #[test]
    fn test_view_from_string_ok() {
        let x = ViewPath::from("db/_design/design-doc/_view/view-name".to_string());
        let (doc_path, view_name) = x.into();
        assert_eq!(doc_path, DocumentPath::from("db/_design/design-doc"));
        assert_eq!(view_name, "view-name");
    }

    #[test]
    #[should_panic]
    fn test_view_from_string_panic_missing_view_sep() {
        ViewPath::from("db/_design/design-doc/view-name".to_string());
    }

    #[test]
    #[should_panic]
    fn test_view_from_string_panic_bad_document_path() {
        ViewPath::from("bad_document_path/_view/view_name");
    }

    #[test]
    fn test_view_from_str_ref_ok() {
        let x = ViewPath::from("db/_design/design-doc/_view/view-name");
        let (doc_path, view_name) = x.into();
        assert_eq!(doc_path, DocumentPath::from("db/_design/design-doc"));
        assert_eq!(view_name, "view-name");
    }

    #[test]
    #[should_panic]
    fn test_view_from_str_ref_panic_missing_view_sep() {
        ViewPath::from("db/_design/design-doc/view-name");
    }

    #[test]
    #[should_panic]
    fn test_view_from_str_ref_panic_bad_document_path() {
        ViewPath::from("bad_document_path/_view/view_name");
    }

    #[test]
    fn test_view_new() {
        let got = ViewPath::new(DocumentPath::from("foo/_design/bar"), "baz");
        let exp = ViewPath::from("foo/_design/bar/_view/baz");
        assert_eq!(got, exp);
    }

    #[test]
    fn test_view_path_clone() {
        let x = ViewPath::from("foo/_design/bar/_view/baz");
        let y = x.clone();
        assert_eq!(x, y);
    }

    #[test]
    fn test_view_path_eq() {

        let x = ViewPath::from("foo/_design/bar/_view/baz");
        let y = ViewPath::from("foo/_design/not-bar/_view/baz");
        assert!(x == x);
        assert!(x != y);

        let y = ViewPath::from("foo/_design/not-bar/_view/baz");
        assert!(x != y);

        let y = ViewPath::from("foo/_design/bar/_view/not-baz");
        assert!(x != y);
    }

    #[test]
    fn test_view_path_ord() {

        let x = ViewPath::from("foo/_design/car/_view/caz");
        let y = ViewPath::from("goo/_design/bar/_view/baz");
        assert!(x < y);

        let x = ViewPath::from("foo/_design/bar/_view/caz");
        let y = ViewPath::from("foo/_design/car/_view/baz");
        assert!(x < y);

        let x = ViewPath::from("foo/_design/bar/_view/baz");
        let y = ViewPath::from("foo/_design/bar/_view/caz");
        assert!(x < y);
    }

    #[test]
    fn test_view_path_into_uri() {

        // Verify: A normal URI base yields a normal database URI path.
        let base = hyper::Url::parse("http://example.com:1234").unwrap();
        let uri = ViewPath::from("foo/_design/bar/_view/baz").into_uri(base);
        let exp = hyper::Url::parse("http://example.com:1234/foo/_design/bar/_view/baz").unwrap();
        assert_eq!(uri, exp);

        // Verify: A URI base with a nonempty path yields a URI with the full
        // path.
        let base = hyper::Url::parse("http://example.com:1234/prefix").unwrap();
        let uri = ViewPath::from("foo/_design/bar/_view/baz").into_uri(base);
        let exp = hyper::Url::parse("http://example.com:1234/prefix/foo/_design/bar/_view/baz").unwrap();
        assert_eq!(uri, exp);
    }

    #[test]
    fn test_view_path_accessors() {
        let view_path = ViewPath::from("foo/_design/bar/_view/baz");
        assert_eq!(*view_path.document_path(), DocumentPath::from("foo/_design/bar"));
        assert_eq!(*view_path.view_name(), "baz".to_string());
    }
}
