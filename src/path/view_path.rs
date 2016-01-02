use hyper;
use std;

use DatabaseName;
use DatabasePath;
use DesignDocumentName;
use DesignDocumentPath;
use Error;
use ViewName;
use error::BadPathKind;

// FIXME: Write doc comments.
pub trait IntoViewPath {
    fn into_view_path(self) -> Result<ViewPath, Error>;
}

impl<'a> IntoViewPath for &'a str {
    fn into_view_path(self) -> Result<ViewPath, Error> {
        use std::str::FromStr;
        ViewPath::from_str(self)
    }
}

impl<'a> IntoViewPath for &'a String {
    fn into_view_path(self) -> Result<ViewPath, Error> {
        use std::str::FromStr;
        ViewPath::from_str(self)
    }
}

impl IntoViewPath for ViewPath {
    fn into_view_path(self) -> Result<ViewPath, Error> {
        Ok(self)
    }
}

impl<T: Into<DesignDocumentPath>> IntoViewPath for (T, ViewName) {
    fn into_view_path(self) -> Result<ViewPath, Error> {
        let ddoc_path = self.0.into();
        let (db_name, ddoc_name) = ddoc_path.into();
        let view_path = ViewPath {
            db_name: db_name,
            ddoc_name: ddoc_name,
            view_name: self.1,
        };
        Ok(view_path)
    }
}

// FIXME: Write doc comments.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ViewPath {
    db_name: DatabaseName,
    ddoc_name: DesignDocumentName,
    view_name: ViewName,
}

impl ViewPath {
    // FIXME: Write doc comments.
    pub fn parse<T: AsRef<str>>(path: T) -> Result<Self, Error> {
        use std::str::FromStr;
        ViewPath::from_str(path.as_ref())
    }

    // FIXME: Write doc comments.
    pub fn into_uri(self, base_uri: hyper::Url) -> hyper::Url {

        let mut uri = base_uri;

        {
            use super::percent::percent_encode_uri_path;

            let mut p = uri.path_mut().unwrap();
            if p.last().map_or(false, |x| x.is_empty()) {
                p.pop();
            }

            p.reserve(5);
            p.push(percent_encode_uri_path(&self.db_name));
            p.push("_design".to_string());
            p.push(percent_encode_uri_path(&self.ddoc_name));
            p.push("_view".to_string());
            p.push(percent_encode_uri_path(&self.view_name));
        }

        uri
    }
}

impl std::fmt::Display for ViewPath {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use super::percent::percent_encode_uri_path;
        write!(f,
               "/{}/_design/{}/_view/{}",
               percent_encode_uri_path(&self.db_name),
               percent_encode_uri_path(&self.ddoc_name),
               percent_encode_uri_path(&self.view_name))
    }
}

impl std::str::FromStr for ViewPath {
    type Err = Error;
    fn from_str(path: &str) -> Result<Self, Self::Err> {

        use super::percent::percent_decode;

        if !path.starts_with("/") {
            return Err(Error::BadViewPath(BadPathKind::NoLeadingSlash));
        }

        let path = &path[1..];

        // CouchDB allows database, document, and view names to contain a slash,
        // but we require any slash within a name to be percent-encoded.

        let parts = path.split("/").collect::<Vec<_>>();
        if parts.len() < 5 {
            return Err(Error::BadViewPath(BadPathKind::NotView));
        }
        if 5 < parts.len() {
            return Err(Error::BadViewPath(BadPathKind::NotView));
        }
        if parts[0].is_empty() || parts[1] != "_design" || parts[2].is_empty() ||
           parts[3] != "_view" || parts[4].is_empty() {
            return Err(Error::BadViewPath(BadPathKind::NotView));
        }

        let db_name = DatabaseName::from(try!(percent_decode(parts[0]).map_err(|_| {
            Error::BadViewPath(BadPathKind::BadPercentEncoding)
        })));
        let ddoc_name = DesignDocumentName::from(try!(percent_decode(parts[2]).map_err(|_| {
            Error::BadViewPath(BadPathKind::BadPercentEncoding)
        })));
        let view_name = ViewName::from(try!(percent_decode(parts[4]).map_err(|_| {
            Error::BadViewPath(BadPathKind::BadPercentEncoding)
        })));

        let view_path = ViewPath {
            db_name: db_name,
            ddoc_name: ddoc_name,
            view_name: view_name,
        };

        Ok(view_path)
    }
}

impl<T: Into<DatabasePath>> From<(T, DesignDocumentName, ViewName)> for ViewPath {
    fn from(parts: (T, DesignDocumentName, ViewName)) -> Self {
        ViewPath {
            db_name: parts.0.into().into(),
            ddoc_name: parts.1,
            view_name: parts.2,
        }
    }
}

impl From<ViewPath> for (DatabaseName, DesignDocumentName, ViewName) {
    fn from(view_path: ViewPath) -> Self {
        (view_path.db_name, view_path.ddoc_name, view_path.view_name)
    }
}

#[cfg(test)]
mod tests {

    use hyper;

    use DatabaseName;
    use DesignDocumentName;
    use DesignDocumentPath;
    use Error;
    use IntoViewPath;
    use ViewName;
    use ViewPath;
    use error::BadPathKind;

    fn make_view_path<T: Into<DatabaseName>, U: Into<DesignDocumentName>, V: Into<ViewName>>
        (db_name: T,
         ddoc_name: U,
         view_name: V)
         -> ViewPath {
        ViewPath {
            db_name: db_name.into(),
            ddoc_name: ddoc_name.into(),
            view_name: view_name.into(),
        }
    }

    #[test]
    fn into_view_path_from_str_ref_ok() {
        let expected = make_view_path("foo", "bar", "qux");
        let got = "/foo/_design/bar/_view/qux".into_view_path().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn into_view_path_from_str_ref_nok() {
        "bad_path".into_view_path().unwrap_err();
    }

    #[test]
    fn into_view_path_from_string_ok() {
        let expected = make_view_path("foo", "bar", "qux");
        let got = "/foo/_design/bar/_view/qux".to_string().into_view_path().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn into_view_path_from_string_nok() {
        "bad_path".to_string().into_view_path().unwrap_err();
    }

    #[test]
    fn into_view_path_from_view_path() {
        let expected = make_view_path("foo", "bar", "qux");
        let got = make_view_path("foo", "bar", "qux").into_view_path().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn into_view_path_from_design_document_path_and_view_name() {
        let expected = make_view_path("foo", "bar", "qux");
        let source = (DesignDocumentPath::parse("/foo/_design/bar").unwrap(),
                      ViewName::from("qux"));
        let got = source.into_view_path().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn view_path_parse_ok() {
        let expected = make_view_path("foo", "bar", "qux");
        let got = ViewPath::parse("/foo/_design/bar/_view/qux").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn view_path_parse_nok() {
        ViewPath::parse("bad_path").unwrap_err();
    }

    #[test]
    fn view_path_into_uri_basic() {
        let expected = "http://example.com:1234/foo/_design/bar/_view/qux";
        let base = hyper::Url::parse("http://example.com:1234").unwrap();
        let uri = make_view_path("foo", "bar", "qux").into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn view_path_into_uri_trailing_slash() {
        let expected = "http://example.com:1234/foo/_design/bar/_view/qux";
        let base = hyper::Url::parse("http://example.com:1234/").unwrap();
        let uri = make_view_path("foo", "bar", "qux").into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn view_path_into_uri_nonempty_path() {
        let expected = "http://example.com:1234/nonempty_path/foo/_design/bar/_view/qux";
        let base = hyper::Url::parse("http://example.com:1234/nonempty_path").unwrap();
        let uri = make_view_path("foo", "bar", "qux").into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn view_path_into_uri_nonempty_path_with_trailing_slash() {
        let expected = "http://example.com:1234/nonempty_path/foo/_design/bar/_view/qux";
        let base = hyper::Url::parse("http://example.com:1234/nonempty_path/").unwrap();
        let uri = make_view_path("foo", "bar", "qux").into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn view_path_into_uri_percent_encoded() {
        let expected = "http://example.com:\
                        1234/foo%2F%25%20bar/_design/qux%2F%25%20baz/_view/kit%2F%25%20lea";
        let base = hyper::Url::parse("http://example.com:1234").unwrap();
        let uri = make_view_path("foo/% bar", "qux/% baz", "kit/% lea").into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn view_path_display() {
        let expected = "/foo%2F%25%20bar/_design/qux%2F%25%20baz/_view/kit%2F%25%20lea";
        let got = format!("{}", make_view_path("foo/% bar", "qux/% baz", "kit/% lea"));
        assert_eq!(expected, got);
    }

    #[test]
    fn view_path_from_str_ok() {
        use std::str::FromStr;
        let expected = make_view_path("foo/% bar", "qux/% baz", "kit/% lea");
        let got = ViewPath::from_str("/foo%2F%25%20bar/_design/qux%2F%25%20baz/_view/kit%2F%25%20\
                                      lea")
                      .unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn view_path_from_str_nok_no_leading_slash() {
        use std::str::FromStr;
        let got = ViewPath::from_str("foo/_design/bar/_view/qux");
        expect_path_parse_error!(got, BadViewPath, NoLeadingSlash);
    }

    #[test]
    fn view_path_from_str_nok_bad_design_path_component() {
        use std::str::FromStr;
        let got = ViewPath::from_str("/foo/not_a_design/bar/_view/qux");
        expect_path_parse_error!(got, BadViewPath, NotView);
    }

    #[test]
    fn view_path_from_str_nok_no_design_path_component() {
        use std::str::FromStr;
        let got = ViewPath::from_str("/foo/bar/_view/qux");
        expect_path_parse_error!(got, BadViewPath, NotView);
    }

    #[test]
    fn view_path_from_str_nok_bad_view_path_component() {
        use std::str::FromStr;
        let got = ViewPath::from_str("/foo/_design/bar/not_a_view/qux");
        expect_path_parse_error!(got, BadViewPath, NotView);
    }

    #[test]
    fn view_path_from_str_nok_no_view_path_component() {
        use std::str::FromStr;
        let got = ViewPath::from_str("/foo/_design/bar/qux");
        expect_path_parse_error!(got, BadViewPath, NotView);
    }

    #[test]
    fn view_path_from_str_nok_slash_in_database_name() {
        use std::str::FromStr;
        let got = ViewPath::from_str("/foo/bar/_design/qux/_view/kit");
        expect_path_parse_error!(got, BadViewPath, NotView);
    }

    #[test]
    fn view_path_from_str_nok_slash_in_document_name() {
        use std::str::FromStr;
        let got = ViewPath::from_str("/foo/_design/bar/qux/_view/kit");
        expect_path_parse_error!(got, BadViewPath, NotView);
    }

    #[test]
    fn view_path_from_str_nok_slash_in_view_name() {
        use std::str::FromStr;
        let got = ViewPath::from_str("/foo/_design/bar/_view/qux/kit");
        expect_path_parse_error!(got, BadViewPath, NotView);
    }

    #[test]
    fn view_path_from_str_nok_no_path_component_after_view() {
        use std::str::FromStr;
        let got = ViewPath::from_str("/foo/_design/bar/_view");
        expect_path_parse_error!(got, BadViewPath, NotView);
    }

    #[test]
    fn view_path_from_str_nok_empty_database_name() {
        use std::str::FromStr;
        let got = ViewPath::from_str("//_design/bar/_view/qux");
        expect_path_parse_error!(got, BadViewPath, NotView);
    }

    #[test]
    fn view_path_from_str_nok_empty_document_name() {
        use std::str::FromStr;
        let got = ViewPath::from_str("/foo/_design//_view/qux");
        expect_path_parse_error!(got, BadViewPath, NotView);
    }

    #[test]
    fn view_path_from_str_nok_empty_view_name() {
        use std::str::FromStr;
        let got = ViewPath::from_str("/foo/_design/bar/_view/");
        expect_path_parse_error!(got, BadViewPath, NotView);
    }

    #[test]
    fn view_path_from_str_nok_bad_percent_encoded_database_name() {
        use std::str::FromStr;
        let got = ViewPath::from_str("/foo%/_design/bar/_view/qux");
        expect_path_parse_error!(got, BadViewPath, BadPercentEncoding);
    }

    #[test]
    fn view_path_from_str_nok_bad_percent_encoded_document_name() {
        use std::str::FromStr;
        let got = ViewPath::from_str("/foo/_design/bar%/_view/qux");
        expect_path_parse_error!(got, BadViewPath, BadPercentEncoding);
    }

    #[test]
    fn view_path_from_str_nok_bad_percent_encoded_view_name() {
        use std::str::FromStr;
        let got = ViewPath::from_str("/foo/_design/bar/_view/qux%");
        expect_path_parse_error!(got, BadViewPath, BadPercentEncoding);
    }

    #[test]
    fn view_name_from_database_path_and_design_document_name_and_view_name() {
        let expected = make_view_path("foo", "bar", "qux");
        let source = (DatabaseName::from("foo"),
                      DesignDocumentName::from("bar"),
                      ViewName::from("qux"));
        let got = source.into();
        assert_eq!(expected, got);
    }

    #[test]
    fn database_name_and_design_document_name_and_view_name_from_view_path() {
        let expected = (DatabaseName::from("foo"),
                        DesignDocumentName::from("bar"),
                        ViewName::from("qux"));
        let source = make_view_path("foo", "bar", "qux");
        let got = source.into();
        assert_eq!(expected, got);
    }
}
