use hyper;
use std;

use DatabaseName;
use Error;
use error::BadPathKind;

// FIXME: Write doc comments describing the peculiarities of path strings--all
// path strings, not just database paths--containing slashes and
// percent-encodable characters.

// FIXME: Write doc comments.
pub trait IntoDatabasePath {
    fn into_database_path(self) -> Result<DatabasePath, Error>;
}

impl<'a> IntoDatabasePath for &'a str {
    fn into_database_path(self) -> Result<DatabasePath, Error> {
        use std::str::FromStr;
        DatabasePath::from_str(self)
    }
}

impl<'a> IntoDatabasePath for &'a String {
    fn into_database_path(self) -> Result<DatabasePath, Error> {
        use std::str::FromStr;
        DatabasePath::from_str(self)
    }
}

impl<T: Into<DatabasePath>> IntoDatabasePath for T {
    fn into_database_path(self) -> Result<DatabasePath, Error> {
        Ok(self.into())
    }
}

// FIXME: Write doc comments.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DatabasePath {
    db_name: DatabaseName,
}

impl DatabasePath {
    // FIXME: Write doc comments.
    pub fn parse<T: AsRef<str>>(path: T) -> Result<Self, Error> {
        use std::str::FromStr;
        DatabasePath::from_str(path.as_ref())
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

            p.push(percent_encode_uri_path(&self.db_name));
        }

        uri
    }
}

impl std::fmt::Display for DatabasePath {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use super::percent::percent_encode_uri_path;
        write!(f, "/{}", percent_encode_uri_path(&self.db_name))
    }
}

impl std::str::FromStr for DatabasePath {
    type Err = Error;
    fn from_str(path: &str) -> Result<Self, Self::Err> {

        if !path.starts_with("/") {
            return Err(Error::BadDatabasePath(BadPathKind::NoLeadingSlash));
        }

        let path = &path[1..];

        // CouchDB allows database names to contain a slash, but we require any
        // slash within a name to be percent-encoded.
        if path.contains("/") {
            return Err(Error::BadDatabasePath(BadPathKind::NotDatabase));
        }

        if path.is_empty() {
            return Err(Error::BadDatabasePath(BadPathKind::NotDatabase));
        }

        let path = try!(super::percent::percent_decode(&path)
                            .map_err(|_| Error::BadDatabasePath(BadPathKind::BadPercentEncoding)));
        let db_path = DatabasePath { db_name: DatabaseName::from(path) };

        Ok(db_path)
    }
}

impl From<DatabaseName> for DatabasePath {
    fn from(db_name: DatabaseName) -> Self {
        DatabasePath { db_name: db_name }
    }
}

impl From<DatabasePath> for DatabaseName {
    fn from(db_path: DatabasePath) -> Self {
        db_path.db_name
    }
}

#[cfg(test)]
mod tests {

    use hyper;

    use DatabaseName;
    use DatabasePath;
    use Error;
    use IntoDatabasePath;
    use error::BadPathKind;

    #[test]
    fn into_database_path_from_str_ref_ok() {
        let expected = DatabasePath { db_name: "foo".into() };
        let got = "/foo".into_database_path().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn into_database_path_from_str_ref_nok() {
        "bad_path".into_database_path().unwrap_err();
    }

    #[test]
    fn into_database_path_from_string_ok() {
        let expected = DatabasePath { db_name: "foo".into() };
        let got = "/foo".to_string().into_database_path().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn into_database_path_from_string_nok() {
        "bad_path".to_string().into_database_path().unwrap_err();
    }

    #[test]
    fn into_database_path_from_database_path() {
        let expected = DatabasePath { db_name: "foo".into() };
        let got = DatabasePath { db_name: "foo".into() }.into_database_path().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn into_database_path_from_database_name_basic() {
        let expected = DatabasePath { db_name: "foo/% bar".into() };
        let got = DatabaseName::from("foo/% bar").into_database_path().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn database_path_parse_ok() {
        let expected = DatabasePath { db_name: "foo".into() };
        let got = DatabasePath::parse("/foo").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn database_path_parse_nok() {
        DatabasePath::parse("bad_path").unwrap_err();
    }

    #[test]
    fn database_path_into_uri_basic() {
        let expected = "http://example.com:1234/foo";
        let base = hyper::Url::parse("http://example.com:1234").unwrap();
        let uri = DatabasePath { db_name: "foo".into() }.into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn database_path_into_uri_trailing_slash() {
        let expected = "http://example.com:1234/foo";
        let base = hyper::Url::parse("http://example.com:1234/").unwrap();
        let uri = DatabasePath { db_name: "foo".into() }.into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn database_path_into_uri_nonempty_path() {
        let expected = "http://example.com:1234/nonempty_path/foo";
        let base = hyper::Url::parse("http://example.com:1234/nonempty_path").unwrap();
        let uri = DatabasePath { db_name: "foo".into() }.into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn database_path_into_uri_nonempty_path_with_trailing_slash() {
        let expected = "http://example.com:1234/nonempty_path/foo";
        let base = hyper::Url::parse("http://example.com:1234/nonempty_path/").unwrap();
        let uri = DatabasePath { db_name: "foo".into() }.into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn database_path_into_uri_percent_encoded() {
        let expected = "http://example.com:1234/foo%2F%25%20bar";
        let base = hyper::Url::parse("http://example.com:1234").unwrap();
        let uri = DatabasePath { db_name: "foo/% bar".into() }.into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn database_path_display() {
        let expected = "/foo%2F%25%20bar";
        let got = format!("{}", DatabasePath { db_name: "foo/% bar".into() });
        assert_eq!(expected, got);
    }

    #[test]
    fn database_path_from_str_ok() {
        use std::str::FromStr;
        let expected = DatabasePath { db_name: "foo/% bar".into() };
        let got = DatabasePath::from_str("/foo%2F%25%20bar").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn database_path_from_str_nok_no_leading_slash() {
        use std::str::FromStr;
        let got = DatabasePath::from_str("foo");
        expect_path_parse_error!(got, BadDatabasePath, NoLeadingSlash);
    }

    #[test]
    fn database_path_from_str_nok_too_many_path_components() {
        use std::str::FromStr;
        let got = DatabasePath::from_str("/foo/bar");
        expect_path_parse_error!(got, BadDatabasePath, NotDatabase);
    }

    #[test]
    fn database_path_from_str_nok_empty_database_name() {
        use std::str::FromStr;
        let got = DatabasePath::from_str("/");
        expect_path_parse_error!(got, BadDatabasePath, NotDatabase);
    }

    #[test]
    fn database_path_from_str_nok_bad_percent_encoding() {
        use std::str::FromStr;
        let got = DatabasePath::from_str("/foo%");
        expect_path_parse_error!(got, BadDatabasePath, BadPercentEncoding);
    }

    #[test]
    fn database_path_from_database_name() {
        let expected = DatabasePath { db_name: "foo/% bar".into() };
        let got = DatabasePath::from(DatabaseName::from("foo/% bar"));
        assert_eq!(expected, got);
    }

    #[test]
    fn database_name_from_database_path() {
        let expected = DatabaseName::from("foo/% bar");
        let got = DatabaseName::from(DatabasePath { db_name: "foo/% bar".into() });
        assert_eq!(expected, got);
    }
}
