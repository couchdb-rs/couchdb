use hyper;
use std;

use DatabaseName;
use DatabasePath;
use DesignDocumentName;
use DesignDocumentPath;
use DocumentId;
use DocumentName;
use Error;
use IntoDatabasePath;
use error::BadPathKind;

/// Trait for converting a type into a `DocumentPath`.
pub trait IntoDocumentPath {
    /// Converts self into a `DocumentPath`.
    fn into_document_path(self) -> Result<DocumentPath, Error>;
}

impl<'a> IntoDocumentPath for &'a str {
    fn into_document_path(self) -> Result<DocumentPath, Error> {
        use std::str::FromStr;
        DocumentPath::from_str(self)
    }
}

impl IntoDocumentPath for DocumentPath {
    fn into_document_path(self) -> Result<DocumentPath, Error> {
        Ok(self)
    }
}

impl IntoDocumentPath for DesignDocumentPath {
    fn into_document_path(self) -> Result<DocumentPath, Error> {
        let (db_name, ddoc_name) = self.into();
        let doc_id = DocumentId::Design(ddoc_name);
        let doc_path = DocumentPath {
            db_name: db_name,
            doc_id: doc_id,
        };
        Ok(doc_path)
    }
}

impl<T: IntoDatabasePath> IntoDocumentPath for (T, DocumentId) {
    fn into_document_path(self) -> Result<DocumentPath, Error> {
        let doc_path = DocumentPath {
            db_name: try!(self.0.into_database_path()).into(),
            doc_id: self.1,
        };
        Ok(doc_path)
    }
}

/// Path part of a URI specifying a document.
///
/// A document path comprises two or three URI path components specifying a
/// database name and document id—the `/db/doc` part of the HTTP request to GET
/// `http://example.com:5984/db/doc` or the `/db/_design/design-doc` part of
/// `http://example.com:5984/db/_design/design-doc`.
///
/// Document paths are percent-encoded. For example, `/foo/bar%2Fqux` identifies the
/// database named `foo` and the document id `bar%2Fqux`. When a `DocumentPath`
/// is constructed from name, id, and path types, the percent-encoding is done
/// automatically. When constructing a `DocumentPath` from a string, the string
/// must be percent-encoded.
///
/// Although the `DocumentPath` type implements the `Ord` and `PartialOrd`
/// traits, it provides no guarantees how that ordering is defined and may
/// change the definition between any two releases of the couchdb crate. That
/// is, for two `DocumentPath` values `a` and `b`, the expression `a < b` may
/// hold true now but not in a subsequent release. Consequently, applications
/// must not rely upon any particular ordering definition.
///
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DocumentPath {
    db_name: DatabaseName,
    doc_id: DocumentId,
}

impl DocumentPath {
    /// Constructs a `DocumentPath` from a given string.
    ///
    /// The `path` string must begin with a leading slash and be
    /// percent-encoded—e.g., `/foo/_design/bar%2Fqux` for the database named
    /// `foo` and the design document named `bar/qux`.
    ///
    pub fn parse<T: AsRef<str>>(path: T) -> Result<Self, Error> {
        use std::str::FromStr;
        DocumentPath::from_str(path.as_ref())
    }

    /// Converts self into a URI.
    pub fn into_uri(self, base_uri: hyper::Url) -> hyper::Url {

        let mut uri = base_uri;

        {
            use super::percent::percent_encode_uri_path;

            let mut p = uri.path_mut().unwrap();
            if p.last().map_or(false, |x| x.is_empty()) {
                p.pop();
            }

            match self.doc_id {
                DocumentId::Normal(doc_name) => {
                    p.reserve(2);
                    p.push(percent_encode_uri_path(&self.db_name));
                    p.push(percent_encode_uri_path(&doc_name));
                }
                DocumentId::Design(doc_name) => {
                    p.reserve(3);
                    p.push(percent_encode_uri_path(&self.db_name));
                    p.push("_design".to_string());
                    p.push(percent_encode_uri_path(&doc_name));
                }
                DocumentId::Local(doc_name) => {
                    p.reserve(3);
                    p.push(percent_encode_uri_path(&self.db_name));
                    p.push("_local".to_string());
                    p.push(percent_encode_uri_path(&doc_name));
                }
            }
        }

        uri
    }
}

impl std::fmt::Display for DocumentPath {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use super::percent::percent_encode_uri_path;
        match self.doc_id {
            DocumentId::Normal(ref doc_name) => {
                write!(f,
                       "/{}/{}",
                       percent_encode_uri_path(&self.db_name),
                       percent_encode_uri_path(doc_name))
            }
            DocumentId::Design(ref doc_name) => {
                write!(f,
                       "/{}/_design/{}",
                       percent_encode_uri_path(&self.db_name),
                       percent_encode_uri_path(doc_name))
            }
            DocumentId::Local(ref doc_name) => {
                write!(f,
                       "/{}/_local/{}",
                       percent_encode_uri_path(&self.db_name),
                       percent_encode_uri_path(doc_name))
            }
        }
    }
}

impl std::str::FromStr for DocumentPath {
    type Err = Error;
    fn from_str(path: &str) -> Result<Self, Self::Err> {

        use super::percent::percent_decode;

        if !path.starts_with("/") {
            return Err(Error::BadDocumentPath(BadPathKind::NoLeadingSlash));
        }

        let path = &path[1..];

        // CouchDB allows database and document names to contain a slash, but we
        // require any slash within a name to be percent-encoded.

        let parts = path.split("/").collect::<Vec<_>>();
        let doc_path = {
            if 3 == parts.len() && parts[1] == "_design" {
                if parts[0].is_empty() || parts[2].is_empty() {
                    return Err(Error::BadDocumentPath(BadPathKind::NotDocument));
                }
                DocumentPath {
                    db_name: DatabaseName::from(try!(percent_decode(parts[0]).map_err(|_| {
                        Error::BadDocumentPath(BadPathKind::BadPercentEncoding)
                    }))),
                    doc_id: DocumentId::Design(DesignDocumentName::from(try!(percent_decode(parts[2]).map_err(|_| {
                        Error::BadDocumentPath(BadPathKind::BadPercentEncoding)
                    })))),
                }
            } else if 3 == parts.len() && parts[1] == "_local" {
                if parts[0].is_empty() || parts[2].is_empty() {
                    return Err(Error::BadDocumentPath(BadPathKind::NotDocument));
                }
                DocumentPath {
                    db_name: DatabaseName::from(try!(percent_decode(parts[0]).map_err(|_| {
                        Error::BadDocumentPath(BadPathKind::BadPercentEncoding)
                    }))),
                    doc_id: DocumentId::Local(DocumentName::from(try!(percent_decode(parts[2]).map_err(|_| {
                        Error::BadDocumentPath(BadPathKind::BadPercentEncoding)
                    })))),
                }
            } else if 2 == parts.len() {
                if parts[0].is_empty() || parts[1].is_empty() || parts[1] == "_design" ||
                   parts[1] == "_local" {
                    return Err(Error::BadDocumentPath(BadPathKind::NotDocument));
                }
                DocumentPath {
                    db_name: DatabaseName::from(try!(percent_decode(parts[0]).map_err(|_| {
                        Error::BadDocumentPath(BadPathKind::BadPercentEncoding)
                    }))),
                    doc_id: DocumentId::Normal(DocumentName::from(try!(percent_decode(parts[1]).map_err(|_| {
                        Error::BadDocumentPath(BadPathKind::BadPercentEncoding)
                    })))),
                }
            } else {
                return Err(Error::BadDocumentPath(BadPathKind::NotDocument));
            }
        };

        Ok(doc_path)
    }
}

impl<T: Into<DatabasePath>> From<(T, DocumentId)> for DocumentPath {
    fn from(parts: (T, DocumentId)) -> Self {
        DocumentPath {
            db_name: parts.0.into().into(),
            doc_id: parts.1,
        }
    }
}

impl From<DocumentPath> for (DatabaseName, DocumentId) {
    fn from(doc_path: DocumentPath) -> Self {
        (doc_path.db_name, doc_path.doc_id)
    }
}

#[cfg(test)]
mod tests {

    use hyper;

    use DatabaseName;
    use DatabasePath;
    use DesignDocumentName;
    use DesignDocumentPath;
    use DocumentId;
    use DocumentPath;
    use Error;
    use IntoDatabasePath;
    use IntoDocumentPath;
    use error::BadPathKind;

    fn make_document_path<T: Into<DatabaseName>, U: Into<DocumentId>>(db_name: T,
                                                                      doc_id: U)
                                                                      -> DocumentPath {
        DocumentPath {
            db_name: db_name.into(),
            doc_id: doc_id.into(),
        }
    }

    #[test]
    fn into_document_path_from_str_ref_ok() {
        let expected = make_document_path("foo", "bar");
        let got = "/foo/bar".into_document_path().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn into_document_path_from_str_ref_nok() {
        "bad_path".into_document_path().unwrap_err();
    }

    #[test]
    fn into_document_path_from_document_path() {
        let expected = make_document_path("foo", "bar");
        let got = make_document_path("foo", "bar").into_document_path().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn into_document_path_from_design_document_path() {
        let expected = make_document_path("foo", "_design/bar");
        let got = DesignDocumentPath::from((DatabaseName::from("foo"),
                                            DesignDocumentName::from("bar")))
                      .into_document_path()
                      .unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn into_document_path_from_database_name_and_document_id_normal() {
        let expected = make_document_path("foo", "bar");
        let source = (DatabaseName::from("foo"), DocumentId::from("bar"));
        let got = source.into_document_path().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn into_document_path_from_database_name_and_document_id_design() {
        let expected = make_document_path("foo", "_design/bar");
        let source = (DatabaseName::from("foo"), DocumentId::from("_design/bar"));
        let got = source.into_document_path().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn into_document_path_from_database_name_and_document_id_local() {
        let expected = make_document_path("foo/bar", "qux/kit");
        let source = (DatabaseName::from("foo/bar"), DocumentId::from("qux/kit"));
        let got = source.into_document_path().unwrap();
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

        let expected = make_document_path("foo", "bar");
        let source = (Db, DocumentId::from("bar"));
        let got = source.into_document_path().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn document_path_parse_ok() {
        let expected = make_document_path("foo", "bar");
        let got = DocumentPath::parse("/foo/bar").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn document_path_parse_nok() {
        DocumentPath::parse("bad_path").unwrap_err();
    }

    #[test]
    fn document_path_into_uri_basic_normal() {
        let expected = "http://example.com:1234/foo/bar";
        let base = hyper::Url::parse("http://example.com:1234").unwrap();
        let uri = make_document_path("foo", "bar").into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn document_path_into_uri_basic_design() {
        let expected = "http://example.com:1234/foo/_design/bar";
        let base = hyper::Url::parse("http://example.com:1234").unwrap();
        let uri = make_document_path("foo", "_design/bar").into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn document_path_into_uri_basic_local() {
        let expected = "http://example.com:1234/foo/_local/bar";
        let base = hyper::Url::parse("http://example.com:1234").unwrap();
        let uri = make_document_path("foo", "_local/bar").into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn document_path_into_uri_trailing_slash() {
        let expected = "http://example.com:1234/foo/bar";
        let base = hyper::Url::parse("http://example.com:1234/").unwrap();
        let uri = make_document_path("foo", "bar").into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn document_path_into_uri_nonempty_path() {
        let expected = "http://example.com:1234/nonempty_path/foo/bar";
        let base = hyper::Url::parse("http://example.com:1234/nonempty_path").unwrap();
        let uri = make_document_path("foo", "bar").into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn document_path_into_uri_nonempty_path_with_trailing_slash() {
        let expected = "http://example.com:1234/nonempty_path/foo/bar";
        let base = hyper::Url::parse("http://example.com:1234/nonempty_path/").unwrap();
        let uri = make_document_path("foo", "bar").into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn document_path_into_uri_percent_encoded_normal() {
        let expected = "http://example.com:1234/foo%2F%25%20bar/qux%2F%25%20kit";
        let base = hyper::Url::parse("http://example.com:1234").unwrap();
        let uri = make_document_path("foo/% bar", "qux/% kit").into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn document_path_into_uri_percent_encoded_design() {
        let expected = "http://example.com:1234/foo%2F%25%20bar/_design/qux%2F%25%20kit";
        let base = hyper::Url::parse("http://example.com:1234").unwrap();
        let uri = make_document_path("foo/% bar", "_design/qux/% kit").into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn document_path_into_uri_percent_encoded_local() {
        let expected = "http://example.com:1234/foo%2F%25%20bar/_local/qux%2F%25%20kit";
        let base = hyper::Url::parse("http://example.com:1234").unwrap();
        let uri = make_document_path("foo/% bar", "_local/qux/% kit").into_uri(base);
        assert_eq!(expected, uri.to_string());
    }

    #[test]
    fn document_path_display_normal() {
        let expected = "/foo%2F%25%20bar/qux%2F%25%20kit";
        let got = format!("{}", make_document_path("foo/% bar", "qux/% kit"));
        assert_eq!(expected, got);
    }

    #[test]
    fn document_path_display_design() {
        let expected = "/foo%2F%25%20bar/_design/qux%2F%25%20kit";
        let got = format!("{}", make_document_path("foo/% bar", "_design/qux/% kit"));
        assert_eq!(expected, got);
    }

    #[test]
    fn document_path_display_local() {
        let expected = "/foo%2F%25%20bar/_local/qux%2F%25%20kit";
        let got = format!("{}", make_document_path("foo/% bar", "_local/qux/% kit"));
        assert_eq!(expected, got);
    }

    #[test]
    fn document_path_from_str_ref_ok_normal() {
        use std::str::FromStr;
        let expected = make_document_path("foo/% bar", "qux/% kit");
        let got = DocumentPath::from_str("/foo%2F%25%20bar/qux%2F%25%20kit").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn document_path_from_str_ref_ok_design() {
        use std::str::FromStr;
        let expected = make_document_path("foo/% bar", "_design/qux/% kit");
        let got = DocumentPath::from_str("/foo%2F%25%20bar/_design/qux%2F%25%20kit").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn document_path_from_str_ref_ok_local() {
        use std::str::FromStr;
        let expected = make_document_path("foo/% bar", "_local/qux/% kit");
        let got = DocumentPath::from_str("/foo%2F%25%20bar/_local/qux%2F%25%20kit").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn document_path_from_str_ref_nok_no_leading_slash() {
        use std::str::FromStr;
        let got = DocumentPath::from_str("foo/bar");
        expect_path_parse_error!(got, BadDocumentPath, NoLeadingSlash);
    }

    #[test]
    fn document_path_from_str_ref_nok_normal_too_many_path_components() {
        use std::str::FromStr;
        let got = DocumentPath::from_str("/foo/bar/qux");
        expect_path_parse_error!(got, BadDocumentPath, NotDocument);
    }

    #[test]
    fn document_path_from_str_ref_nok_design_slash_in_database_name() {
        use std::str::FromStr;
        let got = DocumentPath::from_str("/foo/bar/_design/qux");
        expect_path_parse_error!(got, BadDocumentPath, NotDocument);
    }

    #[test]
    fn document_path_from_str_ref_nok_design_slash_in_document_name() {
        use std::str::FromStr;
        let got = DocumentPath::from_str("/foo/_design/bar/qux");
        expect_path_parse_error!(got, BadDocumentPath, NotDocument);
    }

    #[test]
    fn document_path_fom_str_ref_nok_local_slash_in_database_name() {
        use std::str::FromStr;
        let got = DocumentPath::from_str("/foo/bar/_local/qux");
        expect_path_parse_error!(got, BadDocumentPath, NotDocument);
    }

    #[test]
    fn document_path_from_str_ref_nok_local_slash_in_document_name() {
        use std::str::FromStr;
        let got = DocumentPath::from_str("/foo/_local/bar/qux");
        expect_path_parse_error!(got, BadDocumentPath, NotDocument);
    }

    #[test]
    fn document_path_from_str_ref_nok_nothing_after_database_name() {
        use std::str::FromStr;
        let got = DocumentPath::from_str("/foo");
        expect_path_parse_error!(got, BadDocumentPath, NotDocument);
    }

    #[test]
    fn document_path_from_str_ref_nok_nothing_after_design() {
        use std::str::FromStr;
        let got = DocumentPath::from_str("/foo/_design");
        expect_path_parse_error!(got, BadDocumentPath, NotDocument);
    }

    #[test]
    fn document_path_from_str_ref_nok_nothing_after_local() {
        use std::str::FromStr;
        let got = DocumentPath::from_str("/foo/_local");
        expect_path_parse_error!(got, BadDocumentPath, NotDocument);
    }

    #[test]
    fn document_path_from_str_ref_nok_normal_empty_database_name() {
        use std::str::FromStr;
        let got = DocumentPath::from_str("//foo");
        expect_path_parse_error!(got, BadDocumentPath, NotDocument);
    }

    #[test]
    fn document_path_from_str_ref_nok_normal_empty_document_name() {
        use std::str::FromStr;
        let got = DocumentPath::from_str("/foo/");
        expect_path_parse_error!(got, BadDocumentPath, NotDocument);
    }

    #[test]
    fn document_path_from_str_ref_nok_design_empty_database_name() {
        use std::str::FromStr;
        let got = DocumentPath::from_str("//_design/foo");
        expect_path_parse_error!(got, BadDocumentPath, NotDocument);
    }

    #[test]
    fn document_path_from_str_ref_nok_design_empty_document_name() {
        use std::str::FromStr;
        let got = DocumentPath::from_str("/foo/_design/");
        expect_path_parse_error!(got, BadDocumentPath, NotDocument);
    }

    #[test]
    fn document_path_from_str_ref_nok_local_empty_database_name() {
        use std::str::FromStr;
        let got = DocumentPath::from_str("//_local/foo");
        expect_path_parse_error!(got, BadDocumentPath, NotDocument);
    }

    #[test]
    fn document_path_from_str_ref_nok_local_empty_document_name() {
        use std::str::FromStr;
        let got = DocumentPath::from_str("/foo/_local/");
        expect_path_parse_error!(got, BadDocumentPath, NotDocument);
    }

    #[test]
    fn document_path_from_str_ref_nok_normal_bad_percent_encoded_database_name() {
        use std::str::FromStr;
        let got = DocumentPath::from_str("/foo%/bar");
        expect_path_parse_error!(got, BadDocumentPath, BadPercentEncoding);
    }

    #[test]
    fn document_path_from_str_ref_nok_normal_bad_percent_encoded_document_name() {
        use std::str::FromStr;
        let got = DocumentPath::from_str("/foo/bar%");
        expect_path_parse_error!(got, BadDocumentPath, BadPercentEncoding);
    }

    #[test]
    fn document_path_from_str_ref_nok_design_bad_percent_encoded_database_name() {
        use std::str::FromStr;
        let got = DocumentPath::from_str("/foo%/_design/bar");
        expect_path_parse_error!(got, BadDocumentPath, BadPercentEncoding);
    }

    #[test]
    fn document_path_from_str_ref_nok_design_bad_percent_encoded_document_name() {
        use std::str::FromStr;
        let got = DocumentPath::from_str("/foo/_design/bar%");
        expect_path_parse_error!(got, BadDocumentPath, BadPercentEncoding);
    }

    #[test]
    fn document_path_from_str_ref_nok_local_bad_percent_encoded_database_name() {
        use std::str::FromStr;
        let got = DocumentPath::from_str("/foo%/_local/bar");
        expect_path_parse_error!(got, BadDocumentPath, BadPercentEncoding);
    }

    #[test]
    fn document_path_from_str_ref_nok_local_bad_percent_encoded_document_name() {
        use std::str::FromStr;
        let got = DocumentPath::from_str("/foo/_local/bar%");
        expect_path_parse_error!(got, BadDocumentPath, BadPercentEncoding);
    }

    #[test]
    fn document_path_from_database_name_and_document_id() {
        let expected = make_document_path("foo/% bar", "qux/% kit");
        let source = (DatabaseName::from("foo/% bar"),
                      DocumentId::from("qux/% kit"));
        let got = DocumentPath::from(source);
        assert_eq!(expected, got);
    }

    #[test]
    fn document_path_from_database_path_and_document_id() {
        let expected = make_document_path("foo/% bar", "qux/% kit");
        let source = (DatabasePath::parse("/foo%2F%25%20bar").unwrap(),
                      DocumentId::from("qux/% kit"));
        let got = DocumentPath::from(source);
        assert_eq!(expected, got);
    }

    #[test]
    fn database_name_and_document_id_from_document_path() {
        let expected = (DatabaseName::from("foo/% bar"),
                        DocumentId::from("qux/% kit"));
        let source = make_document_path("foo/% bar", "qux/% kit");
        let got: (DatabaseName, DocumentId) = source.into();
        assert_eq!(expected, got);
    }
}
