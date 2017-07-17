//! The `path` module provides types for identifying databases, documents, etc.
//!
//! # Summary
//!
//! * The `path` module provides a suite of types for names, ids, and paths—all
//!   of which are used to specify the location of CouchDB resources, such as
//!   databases, documents, and views.
//!
//! * Both **names** and **ids** are percent-decoded, but whereas a name
//!   comprises exactly one path segment, an id may comprise more.
//!
//! * Both names and ids are useful for constructing query parameters, headers,
//!   and capturing CouchDB response data.
//!
//! * A **path** is a full percent-encoded URL path and is most useful when
//!   constructing a URL for an HTTP request. It implements neither `Serialize`
//!   nor `Deserialize`.
//!
//! # Remarks
//!
//! The CouchDB API uses strings for specifying the locations of resources, and
//! using these strings “in the raw” can be error-prone. The `path` module
//! provides safer alternatives by way of stronger types.
//!
//! There are two chief kinds of errors that stronger types help eliminate:
//!
//! * **Incorrect percent-encoding.** For example, document names and attachment
//!   names may contain slashes (`/`) and other non-standard characters, and
//!   neglect to percent-encode these characters can cause obscure bugs.
//!
//! * **Type mismatches.** For example, mistakenly using a database path to
//!   delete a document could cause massive data loss.
//!
//! Note, however, that the `path` module merely makes these types available. It
//! is up to the application programmer to make use of them.
//!
//! # Example
//!
//! ```rust
//! extern crate couchdb;
//!
//! // Construct view path: '/alpha/_design/bravo/_view/charlie delta':
//!
//! let view_path = couchdb::DatabaseName::new("alpha")
//!     .with_design_document_id(couchdb::DesignDocumentName::new("bravo"))
//!     .with_view_name("charlie delta");
//!
//! // Paths are percent-encoded and thus well suited for building URLs.
//!
//! let s = view_path.to_string();
//! assert_eq!(s, "/alpha/_design/bravo/_view/charlie%20delta");
//!
//! // Alternatively, paths can be parsed from string.
//!
//! let v2 = couchdb::ViewPath::parse(&s).unwrap();
//! assert_eq!(view_path, v2);
//! ```

use {Error, serde, std};
use serde::Deserialize;
use std::borrow::Cow;
use std::fmt::Display;
use std::str::FromStr;

const DESIGN_PREFIX: &str = "_design";
const LOCAL_PREFIX: &str = "_local";
const VIEW_PREFIX: &str = "_view";

static DOCUMENT_PREFIXES: &[&str] = &[DESIGN_PREFIX, LOCAL_PREFIX];

trait PathEncodable {
    fn encode_path_to(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error>;
}

fn percent_encode_segment(segment: &str, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
    use url::percent_encoding;
    f.write_str("/")?;
    percent_encoding::percent_encode(
        segment.as_bytes(),
        percent_encoding::PATH_SEGMENT_ENCODE_SET,
    ).fmt(f)
}

fn percent_decode<'a>(x: &'a str) -> Result<Cow<'a, str>, Error> {
    use url::percent_encoding;
    percent_encoding::percent_decode(x.as_bytes())
        .decode_utf8()
        .map_err(|_| {
            Error::bad_path("Path is invalid UTF-8 after percent-decoding")
        })
}

// PathDecoder is a utility for parsing a path string into its constituent
// segments while providing consistent error-reporting.
//
// E.g., the string "/alpha/bravo" may be decoded into two parts, "alpha" and
// "bravo".
//
// E.g., the string "/alpha%20bravo" may be decoded into one parts, "alpha
// bravo".
//
#[derive(Clone, Debug, PartialEq)]
struct PathDecoder<'a> {
    cursor: &'a str,
}

trait PathDecodable {
    fn path_decode(s: String) -> Self;
}

impl<T: From<String>> PathDecodable for T {
    fn path_decode(s: String) -> Self {
        Self::from(s)
    }
}

const E_EMPTY_SEGMENT: &str = "Path has an segment";
const E_NO_LEADING_SLASH: &str = "Path does not begin with a slash";
const E_TOO_FEW_SEGMENTS: &str = "Path has too few segments";
const E_TOO_MANY_SEGMENTS: &str = "Path has too many segments";
const E_TRAILING_SLASH: &str = "Path ends with a slash";
const E_UNEXPECTED_SEGMENT: &str = "Path contains unexpected segment";

impl<'a> PathDecoder<'a> {
    pub fn begin(cursor: &'a str) -> Result<Self, Error> {

        if !cursor.starts_with('/') {
            return Err(Error::bad_path(E_NO_LEADING_SLASH));
        }

        Ok(PathDecoder { cursor: cursor })
    }

    pub fn end(self) -> Result<(), Error> {
        match self.cursor {
            "" => Ok(()),
            "/" => Err(Error::bad_path(E_TRAILING_SLASH)),
            _ => Err(Error::bad_path(E_TOO_MANY_SEGMENTS)),
        }
    }

    fn prep(&self) -> Result<&'a str, Error> {
        if self.cursor.is_empty() {
            return Err(Error::bad_path(E_TOO_FEW_SEGMENTS));
        }

        debug_assert!(self.cursor.starts_with('/'));
        let after_slash = &self.cursor['/'.len_utf8()..];

        if after_slash.is_empty() {
            return Err(Error::bad_path(E_TOO_FEW_SEGMENTS));
        }

        Ok(after_slash)
    }

    pub fn decode_exact(&mut self, key: &str) -> Result<(), Error> {

        let p = self.prep()?;

        let slash = p.find('/').unwrap_or(p.len());
        if slash == 0 {
            return Err(Error::bad_path(E_EMPTY_SEGMENT));
        }

        if &p[..slash] != key {
            return Err(Error::bad_path(E_UNEXPECTED_SEGMENT));
        }

        self.cursor = &p[slash..];

        Ok(())
    }

    pub fn decode_segment<T: PathDecodable>(&mut self) -> Result<T, Error> {

        // TODO: We could use From<Cow<'a, str>> instead of From<String> to
        // eliminate a temporary memory allocation when no percent decoding
        // takes place.

        let p = self.prep()?;

        let slash = p.find('/').unwrap_or(p.len());
        if slash == 0 {
            return Err(Error::bad_path(E_EMPTY_SEGMENT));
        }

        let segment = percent_decode(&p[..slash])?;
        self.cursor = &p[slash..];

        Ok(T::path_decode(segment.into_owned()))
    }

    pub fn decode_with_prefix<T: PathDecodable>(&mut self, prefix: &str) -> Result<T, Error> {
        // TODO: We could use From<Cow<'a, str>> instead of From<String> to
        // eliminate a temporary memory allocation when no percent decoding
        // takes place.

        let p = self.prep()?;

        let slash = p.find('/').unwrap_or(p.len());
        if slash + 1 >= p.len() {
            return Err(Error::bad_path(E_TOO_FEW_SEGMENTS));
        }

        if &p[..slash] != prefix {
            return Err(Error::bad_path(E_UNEXPECTED_SEGMENT));
        }

        let p = &p[slash + 1..];

        let slash = p.find('/').unwrap_or(p.len());
        if slash == 0 {
            return Err(Error::bad_path(E_EMPTY_SEGMENT));
        }

        let segment = percent_decode(&p[..slash])?;
        self.cursor = &p[slash..];

        Ok(T::path_decode(format!("{}/{}", prefix, segment)))
    }

    pub fn decode_with_optional_prefix<T, I, S>(&mut self, prefixes: I) -> Result<T, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
        T: PathDecodable,
    {
        // TODO: We could use From<Cow<'a, str>> instead of From<String> to
        // eliminate a temporary memory allocation when no percent decoding
        // takes place.

        let p = self.prep()?;
        let slash = p.find('/').unwrap_or(p.len());

        for prefix in prefixes.into_iter() {
            if &p[..slash] != prefix.as_ref() {
                continue;
            }

            if slash + 1 >= p.len() {
                return Err(Error::bad_path(E_TOO_FEW_SEGMENTS));
            }

            let p = &p[slash + 1..];

            let slash = p.find('/').unwrap_or(p.len());
            if slash == 0 {
                return Err(Error::bad_path(E_EMPTY_SEGMENT));
            }

            let segment = percent_decode(&p[..slash])?;
            self.cursor = &p[slash..];

            return Ok(T::path_decode(format!("{}/{}", prefix.as_ref(), segment)));
        }

        self.decode_segment()
    }
}

macro_rules! define_name_type {
    ($type_name:ident,
     $param_name:ident,
     #[$doc_description:meta],
     #[$doc_content:meta]) =>
    {
        #[$doc_content]
        ///
        /// For more information about path-related types, see the [module-level
        /// documentation](index.html).
        ///
        #[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        pub struct $type_name(String);

        impl $type_name {
            /// Constructs a new
            #[$doc_description]
            /// name.
            pub fn new<T: Into<String>>(s: T) -> Self {
                $type_name(s.into())
            }

            /// Converts the
            #[$doc_description]
            /// name into a string.
            pub fn into_string(self) -> String {
                self.0
            }
        }

        impl PathEncodable for $type_name {
            fn encode_path_to(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                percent_encode_segment(&self.0, f)
            }
        }

        impl AsRef<str> for $type_name {
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }

        impl From<$type_name> for String {
            fn from($param_name: $type_name) -> Self {
                String::from($param_name.0)
            }
        }

        impl Display for $type_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                self.0.fmt(f)
            }
        }

        impl<'a> From<&'a str> for $type_name {
            fn from(s: &'a str) -> Self {
                $type_name(String::from(s))
            }
        }

        impl From<String> for $type_name {
            fn from(s: String) -> Self {
                $type_name(s)
            }
        }

    };
}

define_name_type!(DatabaseName, db_name, #[doc="database"],
#[doc="`DatabaseName` is a single URL path segment that specifies the name of a
database.

For example, given the document path `/db/_design/doc`, the database name is
`db`."]);

impl DatabaseName {
    /// Joins the database name with a document id to construct a document path.
    pub fn with_document_id<T: Into<DocumentId>>(self, doc_id: T) -> DocumentPath {
        DocumentPath {
            db_name: self,
            doc_id: doc_id.into(),
        }
    }

    /// Joins the database name with a design document id to construct a design
    /// document path.
    pub fn with_design_document_id<T: Into<DesignDocumentId>>(self, ddoc_id: T) -> DesignDocumentPath {
        DesignDocumentPath {
            db_name: self,
            ddoc_id: ddoc_id.into(),
        }
    }

    /// Converts the database name into a database path.
    pub fn into_database_path(self) -> DatabasePath {
        DatabasePath { db_name: self }
    }
}

define_name_type!(NormalDocumentName, doc_name, #[doc="normal document"],
#[doc="`NormalDocumentName` is a single URL path segment that specifies the name
of a document that is neither a design document nor a local document.

For example, given the document path `/db/doc`, the document name is `doc`."]);

define_name_type!(DesignDocumentName, ddoc_name, #[doc="design document"],
#[doc="`DesignDocumentName` is a single URL path segment that specifies the name
of a design document.

For example, given the design document path `/db/_design/doc`, the document name
is `doc`."]);

define_name_type!(LocalDocumentName, ldoc_name, #[doc="local document"],
#[doc="`LocalDocumentName` is a single URL path segment that specifies the name
of a local document.

For example, given the local document path `/db/_local/doc`, the document name
is `doc`."]);

define_name_type!(AttachmentName, att_name, #[doc="attachment"],
#[doc="`AttachmentName` is a single URL path segment that specifies the name of
an attachment.

For example, given the attachment path `/db/doc/att`, the attachment name is
`att`."]);

define_name_type!(ViewName, view_name, #[doc="view"], #[doc="`ViewName` is a
single URL path segment that specifies the name of a view.

For example, given the view path `/db/_design/doc/_view/view`, the view name is
`view`."]);

/// `DocumentId` comprises one or more URL path segments that, together,
/// identify a document.
///
/// For example, given the document path `/db/_design/doc`, the document id is
/// `_design/doc`.
///
/// For more information about path-related types, see the [module-level
/// documentation](index.html).
///
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct DocumentId(String);

impl DocumentId {
    /// Constructs a new document id.
    pub fn new<T: Into<String>>(s: T) -> Self {
        DocumentId(s.into())
    }

    /// Converts the document id into a string.
    pub fn into_string(self) -> String {
        self.0
    }

    /// Returns whether the document id specifies a normal document—i.e.,
    /// neither a design document nor a local document.
    pub fn is_normal(&self) -> bool {
        self.split_prefix().0.is_none()
    }

    /// Tries to convert the document id into a normal document name.
    ///
    /// The conversion fails if and only if the document id does not specify a
    /// normal document. In other words, the conversion succeeds if and only if
    /// the document id begins with neither the `_design/` prefix nor the
    /// `_local` prefix.
    ///
    pub fn into_normal_document_name(self) -> Result<NormalDocumentName, DocumentId> {
        if let (None, base) = self.split_prefix() {
            return Ok(NormalDocumentName::from(String::from(base)));
        }
        Err(self)
    }

    /// Returns whether the document id specifies a design document—i.e., the
    /// document begins with the `_design/` prefix.
    pub fn is_design(&self) -> bool {
        DocumentId::has_given_prefix(&self.0, DESIGN_PREFIX)
    }

    /// Tries to convert the document id into a design document name.
    pub fn into_design_document_name(self) -> Result<DesignDocumentName, DocumentId> {
        match self.split_prefix() {
            (Some(prefix), base) if prefix == DESIGN_PREFIX => return Ok(DesignDocumentName::from(String::from(base))),
            _ => {}
        }
        Err(self)
    }

    /// Tries to converts the document id into a design document id.
    ///
    /// The conversion fails if and only if the document id does not specify a
    /// design document. In other words, the conversion succeeds if and only if
    /// the document id begins with the `_design/` prefix.
    ///
    pub fn into_design_document_id(self) -> Result<DesignDocumentId, DocumentId> {
        if self.is_design() { Ok(DesignDocumentId(self)) } else { Err(self) }
    }

    /// Returns whether the document id specifies a local document—i.e., the
    /// document begins with the `_local/` prefix.
    pub fn is_local(&self) -> bool {
        DocumentId::has_given_prefix(&self.0, LOCAL_PREFIX)
    }

    /// Tries to convert the document id into a local document name.
    ///
    /// The conversion fails if and only if the document id does not specify a
    /// local document. In other words, the conversion succeeds if and only if
    /// the document id begins with the `_local/` prefix.
    ///
    pub fn into_local_document_name(self) -> Result<LocalDocumentName, DocumentId> {
        match self.split_prefix() {
            (Some(prefix), base) if prefix == LOCAL_PREFIX => return Ok(LocalDocumentName::from(String::from(base))),
            _ => {}
        }
        Err(self)
    }

    fn has_given_prefix(s: &str, prefix: &str) -> bool {
        s.starts_with(prefix) && s[prefix.len()..].starts_with('/')
    }

    fn split_prefix(&self) -> (Option<&str>, &str) {
        for &prefix in DOCUMENT_PREFIXES.iter() {
            if DocumentId::has_given_prefix(&self.0, prefix) {
                return (Some(prefix), &self.0[prefix.len() + 1..]);
            }
        }
        (None, &self.0)
    }
}

impl PathEncodable for DocumentId {
    fn encode_path_to(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let (prefix, base) = self.split_prefix();
        if let Some(prefix) = prefix {
            percent_encode_segment(prefix, f)?;
        }
        percent_encode_segment(base, f)
    }
}

impl AsRef<str> for DocumentId {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<DocumentId> for String {
    fn from(doc_id: DocumentId) -> Self {
        String::from(doc_id.0)
    }
}

impl Display for DocumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl<'a> From<&'a str> for DocumentId {
    fn from(s: &'a str) -> Self {
        DocumentId(String::from(s))
    }
}

impl From<String> for DocumentId {
    fn from(s: String) -> Self {
        DocumentId(s)
    }
}

impl From<DesignDocumentId> for DocumentId {
    fn from(ddoc_id: DesignDocumentId) -> Self {
        ddoc_id.0
    }
}

impl From<NormalDocumentName> for DocumentId {
    fn from(doc_name: NormalDocumentName) -> Self {
        DocumentId(doc_name.0)
    }
}

impl From<DesignDocumentName> for DocumentId {
    fn from(ddoc_name: DesignDocumentName) -> Self {
        DocumentId(format!("{}/{}", DESIGN_PREFIX, ddoc_name.0))
    }
}

impl From<LocalDocumentName> for DocumentId {
    fn from(ldoc_name: LocalDocumentName) -> Self {
        DocumentId(format!("{}/{}", LOCAL_PREFIX, ldoc_name.0))
    }
}

/// `DesignDocumentId` comprises URL path segments that, together, identify a
/// design document.
///
/// For example, given the document path `/db/_design/doc`, the design document
/// id is `_design/doc`.
///
/// `DesignDocumentId` is a special form of
/// [`DocumentId`](struct.DocumentId.html). All design document ids are document
/// ids, but not all document ids are design document ids.
///
/// For more information about path-related types, see the [module-level
/// documentation](index.html).
///
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct DesignDocumentId(DocumentId);

impl DesignDocumentId {
    /// Tries to construct a design document id from a string.
    pub fn parse(s: &str) -> Result<Self, Error> {
        DesignDocumentId::from_str(s)
    }

    /// Converts the design document id into a `String`.
    pub fn into_string(self) -> String {
        self.0.into_string()
    }

    fn validate(s: &str) -> Result<(), Error> {
        if s.len() <= DESIGN_PREFIX.len() + '/'.len_utf8() || !s.starts_with(DESIGN_PREFIX) ||
            !s[DESIGN_PREFIX.len()..].starts_with('/')
        {
            return Err(Error::BadDesignDocumentId);
        }
        Ok(())
    }

    /// Converts the design document id into a general document id.
    pub fn into_document_id(self) -> DocumentId {
        self.0
    }

    /// Converts the design document id into a design document name.
    pub fn into_design_document_name(self) -> DesignDocumentName {
        self.into_document_id().into_design_document_name().unwrap()
    }
}

impl PathEncodable for DesignDocumentId {
    fn encode_path_to(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.0.encode_path_to(f)
    }
}

impl AsRef<str> for DesignDocumentId {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<DesignDocumentId> for String {
    fn from(ddoc_id: DesignDocumentId) -> Self {
        String::from(ddoc_id.0)
    }
}

impl Display for DesignDocumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl FromStr for DesignDocumentId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        DesignDocumentId::validate(s)?;
        Ok(DesignDocumentId(DocumentId::from(s)))
    }
}

impl From<DesignDocumentName> for DesignDocumentId {
    fn from(ddoc_name: DesignDocumentName) -> Self {
        DesignDocumentId(DocumentId::new(format!("{}/{}", DESIGN_PREFIX, ddoc_name)))
    }
}

impl<'a> Deserialize<'a> for DesignDocumentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        struct Visitor;

        impl<'b> serde::de::Visitor<'b> for Visitor {
            type Value = DesignDocumentId;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                write!(f, "a string specifying a CouchDB design document id")
            }

            fn visit_str<E>(self, source: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if !DocumentId::has_given_prefix(source, DESIGN_PREFIX) {
                    return Err(E::invalid_value(serde::de::Unexpected::Str(source), &self));
                }
                Ok(DesignDocumentId(DocumentId::from(source)))
            }

            fn visit_string<E>(self, source: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if !DocumentId::has_given_prefix(&source, DESIGN_PREFIX) {
                    return Err(E::invalid_value(serde::de::Unexpected::Str(&source), &self));
                }
                Ok(DesignDocumentId(DocumentId::from(source)))
            }
        }

        deserializer.deserialize_string(Visitor)
    }
}

impl PathDecodable for DesignDocumentId {
    fn path_decode(s: String) -> Self {
        debug_assert!(DocumentId::has_given_prefix(&s, DESIGN_PREFIX));
        DesignDocumentId(DocumentId::from(s))
    }
}

/// `ViewId` comprises URL path segments that combine a design document name and
/// a view name.
///
/// For example, given the view path `/db/_design/doc/_view/view`, the view id
/// is `doc/view`.
///
/// An application can use `ViewId` when specifying a view filter by, for
/// example, sending an HTTP request to `GET
/// /{db}/_changes?filter=_view&view=doc/view`, where `doc/view` is a view id.
///
/// **NOTE:** As of version 1.6.1, the CouchDB server does not support view
///  filters where either the design document name or view name contain a slash
///  character (`/`). As such, `ViewId` makes no attempt to correctly
///  percent-encode the names.
///
/// For more information about path-related types, see the [module-level
/// documentation](index.html).
///
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ViewId(String);

impl ViewId {
    /// Constructs a view id from a design document name and a view name.
    pub fn new<T, U>(ddoc_name: T, view_name: U) -> Self
    where
        T: Into<DesignDocumentName>,
        U: Into<ViewName>,
    {
        ViewId(format!("{}/{}", ddoc_name.into(), view_name.into()))
    }

    /// Converts the view id into a `String`.
    pub fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<str> for ViewId {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<ViewId> for String {
    fn from(view_id: ViewId) -> Self {
        String::from(view_id.0)
    }
}

impl Display for ViewId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

/// `DatabasePath` is the full URL path of a database.
///
/// For more information about path-related types, see the [module-level
/// documentation](index.html).
///
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DatabasePath {
    db_name: DatabaseName,
}

impl DatabasePath {
    /// Tries to construct a database path from a string.
    pub fn parse(s: &str) -> Result<Self, Error> {
        DatabasePath::from_str(s)
    }

    /// Borrows the path's database name.
    pub fn database_name(&self) -> &DatabaseName {
        &self.db_name
    }

    /// Joins the path with a document id to construct a document path.
    pub fn with_document_id<T: Into<DocumentId>>(self, doc_id: T) -> DocumentPath {
        DocumentPath {
            db_name: self.db_name,
            doc_id: doc_id.into(),
        }
    }

    /// Joins the path with a design document id to construct a design document
    /// path.
    pub fn with_design_document_id<T: Into<DesignDocumentId>>(self, ddoc_id: T) -> DesignDocumentPath {
        DesignDocumentPath {
            db_name: self.db_name,
            ddoc_id: ddoc_id.into(),
        }
    }
}

impl FromStr for DatabasePath {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut p = PathDecoder::begin(s)?;
        let db_name = p.decode_segment()?;
        p.end()?;
        Ok(DatabasePath { db_name: db_name })
    }
}

impl Display for DatabasePath {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.db_name.encode_path_to(f)?;
        Ok(())
    }
}

/// `DocumentPath` is the full URL path of a document.
///
/// For more information about path-related types, see the [module-level
/// documentation](index.html).
///
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DocumentPath {
    db_name: DatabaseName,
    doc_id: DocumentId,
}

impl DocumentPath {
    /// Tries to construct a document path from a string.
    pub fn parse(s: &str) -> Result<Self, Error> {
        DocumentPath::from_str(s)
    }

    /// Borrows the path's database name.
    pub fn database_name(&self) -> &DatabaseName {
        &self.db_name
    }

    /// Borrows the path's document id.
    pub fn document_id(&self) -> &DocumentId {
        &self.doc_id
    }

    /// Joins the path with an attachment name to construct an attachment path.
    pub fn with_attachment_name<T: Into<AttachmentName>>(self, att_name: T) -> AttachmentPath {
        AttachmentPath {
            db_name: self.db_name,
            doc_id: self.doc_id,
            att_name: att_name.into(),
        }
    }
}

impl FromStr for DocumentPath {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut p = PathDecoder::begin(s)?;
        let db_name = p.decode_segment()?;
        let doc_id = p.decode_with_optional_prefix(DOCUMENT_PREFIXES)?;
        p.end()?;
        Ok(DocumentPath {
            db_name: db_name,
            doc_id: doc_id,
        })
    }
}

impl Display for DocumentPath {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.db_name.encode_path_to(f)?;
        self.doc_id.encode_path_to(f)?;
        Ok(())
    }
}

/// `DesignDocumentPath` is the full URL path of a design document.
///
/// For more information about path-related types, see the [module-level
/// documentation](index.html).
///
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DesignDocumentPath {
    db_name: DatabaseName,
    ddoc_id: DesignDocumentId,
}

impl DesignDocumentPath {
    /// Tries to construct a design database path from a string.
    pub fn parse(s: &str) -> Result<Self, Error> {
        DesignDocumentPath::from_str(s)
    }

    /// Borrows this design document path's database name.
    pub fn database_name(&self) -> &DatabaseName {
        &self.db_name
    }

    /// Borrows this design document path's design document id.
    pub fn design_document_id(&self) -> &DesignDocumentId {
        &self.ddoc_id
    }

    /// Joins the path with an attachment name to construct an attachment path.
    pub fn with_attachment_name<T: Into<AttachmentName>>(self, att_name: T) -> AttachmentPath {
        AttachmentPath {
            db_name: self.db_name,
            doc_id: self.ddoc_id.into_document_id(),
            att_name: att_name.into(),
        }
    }

    /// Joins the path with a view name to construct a view path.
    pub fn with_view_name<T: Into<ViewName>>(self, view_name: T) -> ViewPath {
        ViewPath {
            db_name: self.db_name,
            ddoc_id: self.ddoc_id,
            view_name: view_name.into(),
        }
    }
}

impl FromStr for DesignDocumentPath {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut p = PathDecoder::begin(s)?;
        let db_name = p.decode_segment()?;
        let ddoc_id = p.decode_with_prefix(DESIGN_PREFIX)?;
        p.end()?;
        Ok(DesignDocumentPath {
            db_name: db_name,
            ddoc_id: ddoc_id,
        })
    }
}

impl Display for DesignDocumentPath {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.db_name.encode_path_to(f)?;
        self.ddoc_id.encode_path_to(f)?;
        Ok(())
    }
}

/// `AttachmentPath` is the full URL path of an attachment.
///
/// For more information about path-related types, see the [module-level
/// documentation](index.html).
///
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AttachmentPath {
    db_name: DatabaseName,
    doc_id: DocumentId,
    att_name: AttachmentName,
}

impl AttachmentPath {
    /// Tries to construct an attachment path from a string.
    pub fn parse(s: &str) -> Result<Self, Error> {
        AttachmentPath::from_str(s)
    }

    /// Borrows the path's database name.
    pub fn database_name(&self) -> &DatabaseName {
        &self.db_name
    }

    /// Borrows the path's document id.
    pub fn document_id(&self) -> &DocumentId {
        &self.doc_id
    }

    /// Borrows the path's attachment name.
    pub fn attachment_name(&self) -> &AttachmentName {
        &self.att_name
    }
}

impl FromStr for AttachmentPath {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut p = PathDecoder::begin(s)?;
        let db_name = p.decode_segment()?;
        let doc_id = p.decode_with_optional_prefix(DOCUMENT_PREFIXES)?;
        let att_name = p.decode_segment()?;
        p.end()?;
        Ok(AttachmentPath {
            db_name: db_name,
            doc_id: doc_id,
            att_name: att_name,
        })
    }
}

impl Display for AttachmentPath {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.db_name.encode_path_to(f)?;
        self.doc_id.encode_path_to(f)?;
        self.att_name.encode_path_to(f)?;
        Ok(())
    }
}

/// `ViewPath` is the full URL path of a view.
///
/// For more information about path-related types, see the [module-level
/// documentation](index.html).
///
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ViewPath {
    db_name: DatabaseName,
    ddoc_id: DesignDocumentId,
    view_name: ViewName,
}

impl ViewPath {
    /// Tries to construct a view path from a string.
    pub fn parse(s: &str) -> Result<Self, Error> {
        ViewPath::from_str(s)
    }

    /// Borrows the path's database name.
    pub fn database_name(&self) -> &DatabaseName {
        &self.db_name
    }

    /// Borrows the path's design document id.
    pub fn design_document_id(&self) -> &DesignDocumentId {
        &self.ddoc_id
    }

    /// Borrows the path's view name.
    pub fn view_name(&self) -> &ViewName {
        &self.view_name
    }
}

impl FromStr for ViewPath {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut p = PathDecoder::begin(s)?;
        let db_name = p.decode_segment()?;
        let ddoc_id = p.decode_with_prefix(DESIGN_PREFIX)?;
        p.decode_exact(VIEW_PREFIX)?;
        let view_name = p.decode_segment()?;
        p.end()?;
        Ok(ViewPath {
            db_name: db_name,
            ddoc_id: ddoc_id,
            view_name: view_name,
        })
    }
}

impl Display for ViewPath {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.db_name.encode_path_to(f)?;
        self.ddoc_id.encode_path_to(f)?;
        percent_encode_segment(VIEW_PREFIX, f)?;
        self.view_name.encode_path_to(f)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use {serde_json, std};

    define_name_type!(TestName, test_name, #[doc=""], #[doc=""]);

    #[test]
    fn path_decoding_must_begin_with_leading_slash() {
        PathDecoder::begin("/").unwrap();
        PathDecoder::begin("").unwrap_err().to_string().contains(
            E_NO_LEADING_SLASH,
        );
        PathDecoder::begin("alpha")
            .unwrap_err()
            .to_string()
            .contains(E_NO_LEADING_SLASH);
        PathDecoder::begin("alpha/bravo")
            .unwrap_err()
            .to_string()
            .contains(E_NO_LEADING_SLASH);
    }

    #[test]
    fn path_decoding_must_end_with_empty_string() {

        let mut p = PathDecoder::begin("/alpha").unwrap();
        assert_eq!(p.decode_segment::<String>().unwrap(), "alpha");
        p.end().unwrap();

        let p = PathDecoder::begin("/").unwrap();
        assert!(p.end().unwrap_err().to_string().contains(E_TRAILING_SLASH));

        let p = PathDecoder::begin("//").unwrap();
        assert!(p.end().unwrap_err().to_string().contains(
            E_TOO_MANY_SEGMENTS,
        ));

        let p = PathDecoder::begin("/alpha").unwrap();
        assert!(p.end().unwrap_err().to_string().contains(
            E_TOO_MANY_SEGMENTS,
        ));
    }

    #[test]
    fn path_decoding_enforces_nonemptiness_for_segments() {
        let mut p = PathDecoder::begin("/alpha//bravo").unwrap();
        assert_eq!(p.decode_segment::<String>().unwrap(), "alpha");
        assert!(
            p.decode_segment::<String>()
                .unwrap_err()
                .to_string()
                .contains(E_EMPTY_SEGMENT)
        );

        let mut p = PathDecoder::begin("/alpha//bravo").unwrap();
        assert!(
            p.decode_with_prefix::<String>("alpha")
                .unwrap_err()
                .to_string()
                .contains(E_EMPTY_SEGMENT)
        );

        let mut p = PathDecoder::begin("/alpha//bravo").unwrap();
        assert_eq!(p.decode_segment::<String>().unwrap(), "alpha");
        assert!(
            p.decode_with_optional_prefix::<String, _, _>(&["charlie"])
                .unwrap_err()
                .to_string()
                .contains(E_EMPTY_SEGMENT)
        );

        println!("CHECK go time");
        let mut p = PathDecoder::begin("/alpha//bravo").unwrap();
        assert!(
            p.decode_with_optional_prefix::<String, _, _>(&["alpha"])
                .unwrap_err()
                .to_string()
                .contains(E_EMPTY_SEGMENT)
        );
    }

    #[test]
    fn path_decoding_fails_on_a_path_having_too_few_segments() {
        let mut p = PathDecoder::begin("/alpha").unwrap();
        assert_eq!(p.decode_segment::<String>().unwrap(), "alpha");
        assert!(
            p.decode_segment::<String>()
                .unwrap_err()
                .to_string()
                .contains(E_TOO_FEW_SEGMENTS)
        );

        let mut p = PathDecoder::begin("/alpha").unwrap();
        assert_eq!(p.decode_segment::<String>().unwrap(), "alpha");
        assert!(
            p.decode_with_prefix::<String>("bravo")
                .unwrap_err()
                .to_string()
                .contains(E_TOO_FEW_SEGMENTS)
        );

        // I.e., once we find the prefix in the input string, we're committed to
        // decoding with that prefix and will not fall back to not using the
        // prefix.
        //
        // This helps enforce additional strictness so that don't, say, end up
        // with a non-design document named "_design" but instead yield an
        // error.

        let mut p = PathDecoder::begin("/alpha").unwrap();
        assert_eq!(p.decode_segment::<String>().unwrap(), "alpha");
        assert!(
            p.decode_with_optional_prefix::<String, _, _>(&["alpha"])
                .unwrap_err()
                .to_string()
                .contains(E_TOO_FEW_SEGMENTS)
        );
    }

    #[test]
    fn path_decoding_fails_on_an_unexpected_segment() {
        let mut p = PathDecoder::begin("/alpha/bravo").unwrap();
        assert!(
            p.decode_with_prefix::<String>("bravo")
                .unwrap_err()
                .to_string()
                .contains(E_UNEXPECTED_SEGMENT)
        );

        let mut p = PathDecoder::begin("/alpha/bravo").unwrap();
        assert!(p.decode_exact("bravo").unwrap_err().to_string().contains(
            E_UNEXPECTED_SEGMENT,
        ));
    }

    #[test]
    fn path_decoding_succeeds_on_a_prefix() {
        let mut p = PathDecoder::begin("/alpha/bravo/charlie").unwrap();
        assert_eq!(
            p.decode_with_prefix::<String>("alpha").unwrap(),
            "alpha/bravo"
        );
        assert_eq!(p.decode_segment::<String>().unwrap(), "charlie");
        p.end().unwrap();
    }

    #[test]
    fn path_decoding_succeeds_on_an_optional_prefix() {
        let mut p = PathDecoder::begin("/alpha/bravo/charlie").unwrap();
        assert_eq!(
            p.decode_with_optional_prefix::<String, _, _>(&["alpha"])
                .unwrap(),
            "alpha/bravo"
        );
        assert_eq!(p.decode_segment::<String>().unwrap(), "charlie");
        p.end().unwrap();

        let mut p = PathDecoder::begin("/bravo/charlie").unwrap();
        assert_eq!(
            p.decode_with_optional_prefix::<String, _, _>(&["alpha", "bravo"])
                .unwrap(),
            "bravo/charlie"
        );
        p.end().unwrap();

        let mut p = PathDecoder::begin("/bravo").unwrap();
        assert_eq!(
            p.decode_with_optional_prefix::<String, _, _>(&["alpha"])
                .unwrap(),
            "bravo"
        );
        p.end().unwrap();
    }

    #[test]
    fn path_decoding_percent_decodes() {
        let mut p = PathDecoder::begin("/alpha%20bravo%2fcharlie").unwrap();
        assert_eq!(p.decode_segment::<String>().unwrap(), "alpha bravo/charlie");
        p.end().unwrap();
    }

    fn encode_path<T: PathEncodable>(x: &T) -> String {
        struct PathEncoder<'a, T: PathEncodable + 'a>(&'a T);
        impl<'a, T: PathEncodable> std::fmt::Display for PathEncoder<'a, T> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                self.0.encode_path_to(f)
            }
        }

        // Ensure that the percent-encodings are uppercase, e.g., "%2F" not
        // "%2f".

        let encoded = PathEncoder(x).to_string();
        let mut iter = encoded.split('%');
        let first = iter.next().unwrap();

        iter.map(|s| {
            // str has no split_mut!
            let mut a = s[..2].to_uppercase();
            a.push_str(&s[2..]);
            a
        }).fold(String::from(first), |mut a, b| {
                a.push('%');
                a.push_str(&b);
                a
            })
    }

    #[test]
    fn name_type_percent_encodes_self() {
        assert_eq!(
            encode_path(&TestName::new("alpha/bravo?charlie")),
            "/alpha%2Fbravo%3Fcharlie"
        );
    }

    #[test]
    fn document_id_distinguishes_by_document_type() {
        let doc_id = DocumentId::new("alpha");
        assert!(doc_id.is_normal());
        assert!(!doc_id.is_design());
        assert!(!doc_id.is_local());

        let doc_id = DocumentId::new("_design/alpha");
        assert!(!doc_id.is_normal());
        assert!(doc_id.is_design());
        assert!(!doc_id.is_local());

        let doc_id = DocumentId::new("_local/alpha");
        assert!(!doc_id.is_normal());
        assert!(!doc_id.is_design());
        assert!(doc_id.is_local());
    }

    #[test]
    fn document_id_converts_into_normal_document_name() {
        assert_eq!(
            DocumentId::new("alpha").into_normal_document_name(),
            Ok(NormalDocumentName::new("alpha"))
        );
        assert_eq!(
            DocumentId::new("alpha/bravo?charlie").into_normal_document_name(),
            Ok(NormalDocumentName::new("alpha/bravo?charlie"))
        );
        assert_eq!(
            DocumentId::new("_design/alpha").into_normal_document_name(),
            Err(DocumentId::new("_design/alpha"))
        );
        assert_eq!(
            DocumentId::new("_local/alpha").into_normal_document_name(),
            Err(DocumentId::new("_local/alpha"))
        );
    }

    #[test]
    fn document_id_converts_into_design_document_name() {
        assert_eq!(
            DocumentId::new("alpha").into_design_document_name(),
            Err(DocumentId::new("alpha"))
        );
        assert_eq!(
            DocumentId::new("_design/alpha").into_design_document_name(),
            Ok(DesignDocumentName::new("alpha"))
        );
        assert_eq!(
            DocumentId::new("_design/alpha/bravo?charlie").into_design_document_name(),
            Ok(DesignDocumentName::new("alpha/bravo?charlie"))
        );
        assert_eq!(
            DocumentId::new("_local/alpha").into_design_document_name(),
            Err(DocumentId::new("_local/alpha"))
        );
    }

    #[test]
    fn document_id_converts_into_local_document_name() {
        assert_eq!(
            DocumentId::new("alpha").into_local_document_name(),
            Err(DocumentId::new("alpha"))
        );
        assert_eq!(
            DocumentId::new("_design/alpha").into_local_document_name(),
            Err(DocumentId::new("_design/alpha"))
        );
        assert_eq!(
            DocumentId::new("_local/alpha").into_local_document_name(),
            Ok(LocalDocumentName::new("alpha"))
        );
        assert_eq!(
            DocumentId::new("_local/alpha/bravo?charlie").into_local_document_name(),
            Ok(LocalDocumentName::new("alpha/bravo?charlie"))
        );
    }

    #[test]
    fn document_id_converts_from_document_name() {
        assert_eq!(
            DocumentId::from(NormalDocumentName::new("alpha")),
            DocumentId::new("alpha")
        );
        assert_eq!(
            DocumentId::from(DesignDocumentName::new("alpha")),
            DocumentId::new("_design/alpha")
        );
        assert_eq!(
            DocumentId::from(LocalDocumentName::new("alpha")),
            DocumentId::new("_local/alpha")
        );
    }

    #[test]
    fn document_id_percent_encodes_self() {
        assert_eq!(
            encode_path(&DocumentId::new("alpha/bravo?charlie")),
            "/alpha%2Fbravo%3Fcharlie"
        );
        assert_eq!(
            encode_path(&DocumentId::new("_design/alpha/bravo?charlie")),
            "/_design/alpha%2Fbravo%3Fcharlie"
        );
        assert_eq!(
            encode_path(&DocumentId::new("_local/alpha/bravo?charlie")),
            "/_local/alpha%2Fbravo%3Fcharlie"
        );
    }

    #[test]
    fn design_document_id_converts_into_document_id() {
        assert_eq!(
            DesignDocumentId::parse("_design/alpha")
                .unwrap()
                .into_document_id(),
            DocumentId::new("_design/alpha")
        )
    }

    #[test]
    fn design_document_id_converts_into_design_document_name() {
        assert_eq!(
            DesignDocumentId::parse("_design/alpha")
                .unwrap()
                .into_design_document_name(),
            DesignDocumentName::new("alpha")
        )
    }

    #[test]
    fn design_document_id_converts_from_design_document_name() {
        assert_eq!(
            DesignDocumentId::from(DesignDocumentName::new("alpha")),
            DesignDocumentId::parse("_design/alpha").unwrap()
        );
    }

    #[test]
    fn design_document_id_deserialization_enforces_design_prefix() {

        let source = r#""_design/alpha""#;
        let got: DesignDocumentId = serde_json::from_str(&source).unwrap();
        let expected = DesignDocumentId(DocumentId::new("_design/alpha"));
        assert_eq!(got, expected);

        let source = r#""alpha""#;
        match serde_json::from_str::<DesignDocumentId>(&source) {
            Err(ref e) if e.is_data() => {}
            x => panic!("Got unexpected result {:?}", x),
        }
    }

    #[test]
    fn database_path_percent_encodes_itself() {
        let got = DatabaseName::new("alpha/bravo?charlie")
            .into_database_path()
            .to_string();
        let expected = "/alpha%2Fbravo%3Fcharlie";
        assert_eq!(got, expected);
    }

    #[test]
    fn database_path_decodes_str() {
        let got = DatabasePath::from_str("/alpha%2Fbravo%3Fcharlie").unwrap();
        let expected = DatabaseName::new("alpha/bravo?charlie").into_database_path();
        assert_eq!(got, expected);
    }

    #[test]
    fn document_path_percent_encodes_itself() {
        let got = DatabaseName::new("alpha/bravo?charlie")
            .with_document_id("delta/echo?foxtrot")
            .to_string();
        let expected = "/alpha%2Fbravo%3Fcharlie/delta%2Fecho%3Ffoxtrot";
        assert_eq!(got, expected);

        let got = DatabaseName::new("alpha/bravo?charlie")
            .with_document_id(DesignDocumentName::new("delta/echo?foxtrot"))
            .to_string();
        let expected = "/alpha%2Fbravo%3Fcharlie/_design/delta%2Fecho%3Ffoxtrot";
        assert_eq!(got, expected);

        let got = DatabaseName::new("alpha/bravo?charlie")
            .with_document_id(LocalDocumentName::new("delta/echo?foxtrot"))
            .to_string();
        let expected = "/alpha%2Fbravo%3Fcharlie/_local/delta%2Fecho%3Ffoxtrot";
        assert_eq!(got, expected);
    }

    #[test]
    fn document_path_decodes_str() {
        let got = DocumentPath::from_str("/alpha%2Fbravo%3Fcharlie/delta%2Fecho%3Ffoxtrot").unwrap();
        let expected = DatabaseName::new("alpha/bravo?charlie").with_document_id("delta/echo?foxtrot");
        assert_eq!(got, expected);

        let got = DocumentPath::from_str("/alpha%2Fbravo%3Fcharlie/_design/delta%2Fecho%3Ffoxtrot").unwrap();
        let expected =
            DatabaseName::new("alpha/bravo?charlie").with_document_id(DesignDocumentName::new("delta/echo?foxtrot"));
        assert_eq!(got, expected);

        let got = DocumentPath::from_str("/alpha%2Fbravo%3Fcharlie/_local/delta%2Fecho%3Ffoxtrot").unwrap();
        let expected =
            DatabaseName::new("alpha/bravo?charlie").with_document_id(LocalDocumentName::new("delta/echo?foxtrot"));
        assert_eq!(got, expected);
    }

    #[test]
    fn design_document_path_percent_encodes_itself() {
        let got = DatabaseName::new("alpha/bravo?charlie")
            .with_document_id(DesignDocumentName::new("delta/echo?foxtrot"))
            .to_string();
        let expected = "/alpha%2Fbravo%3Fcharlie/_design/delta%2Fecho%3Ffoxtrot";
        assert_eq!(got, expected);
    }

    #[test]
    fn design_document_path_decodes_str() {
        let got = DesignDocumentPath::from_str("/alpha%2Fbravo%3Fcharlie/_design/delta%2Fecho%3Ffoxtrot").unwrap();
        let expected = DatabaseName::new("alpha/bravo?charlie")
            .with_design_document_id(DesignDocumentName::new("delta/echo?foxtrot"));
        assert_eq!(got, expected);
    }

    #[test]
    fn attachment_path_percent_encodes_itself() {
        let got = DatabaseName::new("alpha/bravo?charlie")
            .with_document_id("delta/echo?foxtrot")
            .with_attachment_name("golf")
            .to_string();
        let expected = "/alpha%2Fbravo%3Fcharlie/delta%2Fecho%3Ffoxtrot/golf";
        assert_eq!(got, expected);

        let got = DatabaseName::new("alpha/bravo?charlie")
            .with_document_id(DesignDocumentName::new("delta/echo?foxtrot"))
            .with_attachment_name("golf")
            .to_string();
        let expected = "/alpha%2Fbravo%3Fcharlie/_design/delta%2Fecho%3Ffoxtrot/golf";
        assert_eq!(got, expected);

        let got = DatabaseName::new("alpha/bravo?charlie")
            .with_document_id(LocalDocumentName::new("delta/echo?foxtrot"))
            .with_attachment_name("golf")
            .to_string();
        let expected = "/alpha%2Fbravo%3Fcharlie/_local/delta%2Fecho%3Ffoxtrot/golf";
        assert_eq!(got, expected);
    }

    #[test]
    fn attachment_path_decodes_str() {
        let got = AttachmentPath::from_str("/alpha%2Fbravo%3Fcharlie/delta%2Fecho%3Ffoxtrot/golf").unwrap();
        let expected = DatabaseName::new("alpha/bravo?charlie")
            .with_document_id("delta/echo?foxtrot")
            .with_attachment_name("golf");
        assert_eq!(got, expected);

        let got = AttachmentPath::from_str(
            "/alpha%2Fbravo%3Fcharlie/_design/delta%2Fecho%3Ffoxtrot/golf",
        ).unwrap();
        let expected = DatabaseName::new("alpha/bravo?charlie")
            .with_document_id(DesignDocumentName::new("delta/echo?foxtrot"))
            .with_attachment_name("golf");
        assert_eq!(got, expected);

        let got = AttachmentPath::from_str(
            "/alpha%2Fbravo%3Fcharlie/_local/delta%2Fecho%3Ffoxtrot/golf",
        ).unwrap();
        let expected = DatabaseName::new("alpha/bravo?charlie")
            .with_document_id(LocalDocumentName::new("delta/echo?foxtrot"))
            .with_attachment_name("golf");
        assert_eq!(got, expected);
    }

    #[test]
    fn view_path_percent_encodes_itself() {
        let got = DatabaseName::new("alpha/bravo?charlie")
            .with_design_document_id(DesignDocumentName::new("delta/echo?foxtrot"))
            .with_view_name("golf")
            .to_string();
        let expected = "/alpha%2Fbravo%3Fcharlie/_design/delta%2Fecho%3Ffoxtrot/_view/golf";
        assert_eq!(got, expected);
    }

    #[test]
    fn view_path_decodes_str() {
        let got = ViewPath::from_str(
            "/alpha%2Fbravo%3Fcharlie/_design/delta%2Fecho%3Ffoxtrot/_view/golf",
        ).unwrap();
        let expected = DatabaseName::new("alpha/bravo?charlie")
            .with_design_document_id(DesignDocumentName::new("delta/echo?foxtrot"))
            .with_view_name("golf");
        assert_eq!(got, expected);
    }
}
