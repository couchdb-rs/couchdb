use hyper;
use std;

use DatabaseName;
use DatabasePath;
use DesignDocumentName;
use Error;
use IntoDatabasePath;
use error::BadPathKind;

// FIXME: Write doc comments.
pub trait IntoDesignDocumentPath {
    fn into_design_document_path(self) -> Result<DesignDocumentPath, Error>;
}

impl<'a> IntoDesignDocumentPath for &'a str {
    fn into_design_document_path(self) -> Result<DesignDocumentPath, Error> {
        use std::str::FromStr;
        DesignDocumentPath::from_str(self)
    }
}

impl<'a> IntoDesignDocumentPath for &'a String {
    fn into_design_document_path(self) -> Result<DesignDocumentPath, Error> {
        use std::str::FromStr;
        DesignDocumentPath::from_str(self)
    }
}

impl IntoDesignDocumentPath for DesignDocumentPath {
    fn into_design_document_path(self) -> Result<DesignDocumentPath, Error> {
        Ok(self)
    }
}

impl<T: IntoDatabasePath> IntoDesignDocumentPath for (T, DesignDocumentName) {
    fn into_design_document_path(self) -> Result<DesignDocumentPath, Error> {
        let ddoc_path = DesignDocumentPath {
            db_name: try!(self.0.into_database_path()).into(),
            ddoc_name: self.1,
        };
        Ok(ddoc_path)
    }
}

// FIXME: Write doc comments.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DesignDocumentPath {
    db_name: DatabaseName,
    ddoc_name: DesignDocumentName,
}

impl DesignDocumentPath {
    // FIXME: Write doc comments.
    pub fn parse<T: AsRef<str>>(path: T) -> Result<Self, Error> {
        use std::str::FromStr;
        DesignDocumentPath::from_str(path.as_ref())
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
            p.reserve(3);
            p.push(percent_encode_uri_path(&self.db_name));
            p.push("_design".to_string());
            p.push(percent_encode_uri_path(&self.ddoc_name));
        }

        uri
    }
}

impl std::fmt::Display for DesignDocumentPath {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use super::percent::percent_encode_uri_path;
        write!(f,
               "/{}/_design/{}",
               percent_encode_uri_path(&self.db_name),
               percent_encode_uri_path(&self.ddoc_name))
    }
}

impl std::str::FromStr for DesignDocumentPath {
    type Err = Error;
    fn from_str(path: &str) -> Result<Self, Self::Err> {

        use super::percent::percent_decode;

        if !path.starts_with("/") {
            return Err(Error::BadDesignDocumentPath(BadPathKind::NoLeadingSlash));
        }

        let path = &path[1..];

        // CouchDB allows database and document names to contain a slash, but we
        // require any slash within a name to be percent-encoded.

        let parts = path.split("/").collect::<Vec<_>>();
        if parts.len() < 3 {
            return Err(Error::BadDesignDocumentPath(BadPathKind::NotDesignDocument));
        }
        if 3 < parts.len() {
            return Err(Error::BadDesignDocumentPath(BadPathKind::NotDesignDocument));
        }
        if parts[0].is_empty() || parts[1] != "_design" || parts[2].is_empty() {
            return Err(Error::BadDesignDocumentPath(BadPathKind::NotDesignDocument));
        }

        let ddoc_path = DesignDocumentPath {
            db_name: DatabaseName::from(try!(percent_decode(parts[0]).map_err(|_| {
                Error::BadDesignDocumentPath(BadPathKind::BadPercentEncoding)
            }))),
            ddoc_name: DesignDocumentName::from(try!(percent_decode(parts[2]).map_err(|_| {
                Error::BadDesignDocumentPath(BadPathKind::BadPercentEncoding)
            }))),
        };

        Ok(ddoc_path)
    }
}

impl<T: Into<DatabasePath>> From<(T, DesignDocumentName)> for DesignDocumentPath {
    fn from(parts: (T, DesignDocumentName)) -> Self {
        DesignDocumentPath {
            db_name: parts.0.into().into(),
            ddoc_name: parts.1,
        }
    }
}

impl From<DesignDocumentPath> for (DatabaseName, DesignDocumentName) {
    fn from(ddoc_path: DesignDocumentPath) -> Self {
        (ddoc_path.db_name, ddoc_path.ddoc_name)
    }
}

#[cfg(test)]
mod tests {

    use hyper;

    use DatabaseName;
    use DatabasePath;
    use DesignDocumentName;
    use DesignDocumentPath;
    use Error;
    use IntoDatabasePath;
    use IntoDesignDocumentPath;
    use error::BadPathKind;

    fn make_design_document_path<T: Into<DatabaseName>, U: Into<DesignDocumentName>>
        (db_name: T,
         ddoc_name: U)
         -> DesignDocumentPath {
        DesignDocumentPath {
            db_name: db_name.into(),
            ddoc_name: ddoc_name.into(),
        }
    }

    #[test]
    fn into_design_document_path_from_str_ref_ok() {
        let expected = make_design_document_path("foo", "bar");
        let got = "/foo/_design/bar".into_design_document_path().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn into_design_document_path_from_str_ref_nok() {
        "bad_path".into_design_document_path().unwrap_err();
    }

    #[test]
    fn into_design_document_path_from_string_ok() {
        let expected = make_design_document_path("foo", "bar");
        let got = "/foo/_design/bar".to_string().into_design_document_path().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn into_design_document_path_from_string_nok() {
        "bad_path".to_string().into_design_document_path().unwrap_err();
    }

    #[test]
    fn into_design_document_path_from_design_document_path() {
        let expected = make_design_document_path("foo", "bar");
        let got = make_design_document_path("foo", "bar").into_design_document_path().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn into_design_document_path_from_database_name_and_design_document_name() {
        let expected = make_design_document_path("foo", "bar");
        let src = (DatabaseName::from("foo"), DesignDocumentName::from("bar"));
        let got = src.into_design_document_path().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn into_document_path_from_custom_database_path_and_document_id() {

        struct Db;

        impl IntoDatabasePath for Db {
            fn into_database_path(self) -> Result<DatabasePath, Error> {
                DatabasePath::parse("/foo")
            }
        }

        let expected = make_design_document_path("foo", "bar");
        let src = (Db, DesignDocumentName::from("bar"));
        let got = src.into_design_document_path().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn design_document_path_parse_ok() {
        let expected = make_design_document_path("foo", "bar");
        let got = DesignDocumentPath::parse("/foo/_design/bar").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn design_document_path_parse_nok() {
        DesignDocumentPath::parse("bad_path").unwrap_err();
    }

    #[test]
    fn design_document_path_into_uri_basic() {
        let expected = "http://example.com:1234/foo/_design/bar";
        let base = hyper::Url::parse("http://example.com:1234").unwrap();
        let uri = make_design_document_path("foo", "bar").into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn design_document_path_into_uri_trailing_slash() {
        let expected = "http://example.com:1234/foo/_design/bar";
        let base = hyper::Url::parse("http://example.com:1234/").unwrap();
        let uri = make_design_document_path("foo", "bar").into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn design_document_path_into_uri_nonempty_path() {
        let expected = "http://example.com:1234/nonempty_path/foo/_design/bar";
        let base = hyper::Url::parse("http://example.com:1234/nonempty_path").unwrap();
        let uri = make_design_document_path("foo", "bar").into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn design_document_path_into_uri_nonempty_path_with_trailing_slash() {
        let expected = "http://example.com:1234/nonempty_path/foo/_design/bar";
        let base = hyper::Url::parse("http://example.com:1234/nonempty_path/").unwrap();
        let uri = make_design_document_path("foo", "bar").into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn design_document_path_into_uri_percent_encoded() {
        let expected = "http://example.com:1234/foo%2F%25%20bar/_design/qux%2F%25%20kit";
        let base = hyper::Url::parse("http://example.com:1234").unwrap();
        let uri = make_design_document_path("foo/% bar", "qux/% kit").into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn design_document_path_display() {
        let expected = "/foo%2F%25%20bar/_design/qux%2F%25%20kit";
        let got = format!("{}", make_design_document_path("foo/% bar", "qux/% kit"));
        assert_eq!(expected, got);
    }

    #[test]
    fn design_document_path_from_str_ok() {
        use std::str::FromStr;
        let expected = make_design_document_path("foo/% bar", "qux/% kit");
        let got = DesignDocumentPath::from_str("/foo%2F%25%20bar/_design/qux%2F%25%20kit").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn design_document_path_from_str_nok_no_leading_slash() {
        use std::str::FromStr;
        let got = DesignDocumentPath::from_str("foo/_design/bar");
        expect_path_parse_error!(got, BadDesignDocumentPath, NoLeadingSlash);
    }

    #[test]
    fn design_document_path_from_str_nok_normal_document() {
        use std::str::FromStr;
        let got = DesignDocumentPath::from_str("/foo/bar");
        expect_path_parse_error!(got, BadDesignDocumentPath, NotDesignDocument);
    }

    #[test]
    fn design_document_path_from_str_nok_local_document() {
        use std::str::FromStr;
        let got = DesignDocumentPath::from_str("/foo/_local/bar");
        expect_path_parse_error!(got, BadDesignDocumentPath, NotDesignDocument);
    }

    #[test]
    fn design_document_path_from_str_nok_too_many_path_components() {
        use std::str::FromStr;
        let got = DesignDocumentPath::from_str("/foo/_design/bar/qux");
        expect_path_parse_error!(got, BadDesignDocumentPath, NotDesignDocument);
    }

    #[test]
    fn design_document_path_from_str_nok_empty_database_name() {
        use std::str::FromStr;
        let got = DesignDocumentPath::from_str("//_design/foo");
        expect_path_parse_error!(got, BadDesignDocumentPath, NotDesignDocument);
    }

    #[test]
    fn design_document_path_from_str_nok_empty_document_name() {
        use std::str::FromStr;
        let got = DesignDocumentPath::from_str("/foo/_design/");
        expect_path_parse_error!(got, BadDesignDocumentPath, NotDesignDocument);
    }

    #[test]
    fn design_document_path_from_str_nok_bad_percent_encoded_database_name() {
        use std::str::FromStr;
        let got = DesignDocumentPath::from_str("/foo%/_design/bar");
        expect_path_parse_error!(got, BadDesignDocumentPath, BadPercentEncoding);
    }

    #[test]
    fn design_document_path_from_str_nok_bad_percent_encoded_document_name() {
        use std::str::FromStr;
        let got = DesignDocumentPath::from_str("/foo/_design/bar%");
        expect_path_parse_error!(got, BadDesignDocumentPath, BadPercentEncoding);
    }

    #[test]
    fn design_document_path_from_database_name_and_design_document_name() {
        let expected = make_design_document_path("foo/% bar", "qux/% kit");
        let source = (DatabaseName::from("foo/% bar"),
                      DesignDocumentName::from("qux/% kit"));
        let got = DesignDocumentPath::from(source);
        assert_eq!(expected, got);
    }

    #[test]
    fn design_document_path_from_database_path_and_design_document_name() {
        let expected = make_design_document_path("foo/% bar", "qux/% kit");
        let source = (DatabasePath::parse("/foo%2F%25%20bar").unwrap(),
                      DesignDocumentName::from("qux/% kit"));
        let got = DesignDocumentPath::from(source);
        assert_eq!(expected, got);
    }

    #[test]
    fn database_name_and_design_document_name_from_design_document_path() {
        let expected = (DatabaseName::from("foo/% bar"),
                        DesignDocumentName::from("qux/% kit"));
        let source = make_design_document_path("foo/% bar", "qux/% kit");
        let got: (DatabaseName, DesignDocumentName) = source.into();
        assert_eq!(expected, got);
    }
}
