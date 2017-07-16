//! The `path` module provides types for identifying databases, documents, etc.
//!
//! # Overview
//!
//! The `couchdb` crate provides a large suite of types for identifying various
//! CouchDB resources, such as databases, documents, and views. These types are
//! collectively called **path-related types**.
//!
//! The purpose of path-related types is to improve type-safety for
//! applications. Here are two design aims:
//!
//! * To prevent type-mismatch errors, such as an application mistakenly trying
//!   to create a document using a database name, and,
//!
//! * To prevent formatting errors, such as an application neglecting to
//!   percent-encode a URL path.
//!
//! This additional type-safety incurs a small additional run-time cost as
//! compared to working with raw strings, but the cost is negligible compared to
//! the cost of round-tripping an HTTP request with a CouchDB server.
//!
//! # Path types vs component types
//!
//! The most important thing to know about path-related types is the distinction
//! between **path types** and **component types**. A path type is an “on the
//! wire” representation of the path part of a URL, whereas a component is a
//! non-encoded representation of one—or some cases, two–path segments. Here are
//! a couple examples:
//!
//! * The document path `"/db/doc"` is made up of two components, the database
//!   name `"db"` and the document id `"doc"`.
//!
//! * The view path `"/db/_design/design-doc/_view/view%20name%20has%20spaces"`
//!   is made up of three components, the database name `"db"`, the document id
//!   `"_design/design-doc"`, and the view name `"view name has spaces"`.
//!
//! Notice that paths begin with a slash and are percent-encoded, whereas path
//! components do not begin with a slash and are not percent-encoded.
//!
//! Path types have “Path” in their name, such as `DatabasePath` and
//! `DocumentPath`. Whereas, components types do not, such as `DatabaseName` and
//! `DocumentId`.
//!
//! # Path conversion traits
//!
//! Whereas path and component types improve type-safety, **path conversion
//! traits** improve convenience.
//!
//! Each path type has a corresponding conversion trait, which converts other
//! types into that path type. The naming scheme for conversion traits is
//! consistent across types—e.g., the `IntoDatabasePath` trait converts types
//! into `DatabasePath`.
//!
//! These conversions are fallible, meaning they return a `Result` containing an
//! error if the conversion fails. However, conversions fail only when parsing a
//! string. A conversion from constituent component types always succeeds.
//!
//! Applications should avoid doing their own custom path-encoding at runtime
//! and instead rely on constructing paths from component types. This is because
//! path-encoding is easy to get wrong, especially with corner cases such as
//! percent-encoding. However, hard-coding a path is reasonable.

use {Error, serde};
use serde::Deserialize;
use std::borrow::Cow;
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;
use url::percent_encoding;

const DESIGN_PREFIX: &str = "_design";
const LOCAL_PREFIX: &str = "_local";
const VIEW_PREFIX: &str = "_view";

static DOCUMENT_PREFIXES: &[&str] = &[DESIGN_PREFIX, LOCAL_PREFIX];

fn percent_decode<'a>(x: &'a str) -> Result<Cow<'a, str>, Error> {
    use url::percent_encoding;
    percent_encoding::percent_decode(x.as_bytes())
        .decode_utf8()
        .map_err(|e| {
            Error::from(("Path is not valid UTF-8 after percent-decoding", e))
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
            return Err(Error::from(E_NO_LEADING_SLASH));
        }

        Ok(PathDecoder { cursor: cursor })
    }

    pub fn end(self) -> Result<(), Error> {
        match self.cursor {
            "" => Ok(()),
            "/" => Err(Error::from(E_TRAILING_SLASH)),
            _ => Err(Error::from(E_TOO_MANY_SEGMENTS)),
        }
    }

    fn prep(&self) -> Result<&'a str, Error> {
        if self.cursor.is_empty() {
            return Err(Error::from(E_TOO_FEW_SEGMENTS));
        }

        debug_assert!(self.cursor.starts_with('/'));
        let after_slash = &self.cursor['/'.len_utf8()..];

        if after_slash.is_empty() {
            return Err(Error::from(E_TOO_FEW_SEGMENTS));
        }

        Ok(after_slash)
    }

    pub fn decode_exact(&mut self, key: &str) -> Result<(), Error> {

        let p = self.prep()?;

        let slash = p.find('/').unwrap_or(p.len());
        if slash == 0 {
            return Err(Error::from(E_EMPTY_SEGMENT));
        }

        if &p[..slash] != key {
            return Err(Error::from(format!(
                "{} (got: {:?}, expected: {:?})",
                E_UNEXPECTED_SEGMENT,
                &p[..slash],
                key
            )));
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
            return Err(Error::from(E_EMPTY_SEGMENT));
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
            return Err(Error::from(E_TOO_FEW_SEGMENTS));
        }

        if &p[..slash] != prefix {
            return Err(Error::from(format!(
                "{} (got: {:?}, expected: {:?})",
                E_UNEXPECTED_SEGMENT,
                &p[..slash],
                prefix
            )));
        }

        let p = &p[slash + 1..];

        let slash = p.find('/').unwrap_or(p.len());
        if slash == 0 {
            return Err(Error::from(E_EMPTY_SEGMENT));
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
                return Err(Error::from(E_TOO_FEW_SEGMENTS));
            }

            let p = &p[slash + 1..];

            let slash = p.find('/').unwrap_or(p.len());
            if slash == 0 {
                return Err(Error::from(E_EMPTY_SEGMENT));
            }

            let segment = percent_decode(&p[..slash])?;
            self.cursor = &p[slash..];

            return Ok(T::path_decode(format!("{}/{}", prefix.as_ref(), segment)));
        }

        self.decode_segment()
    }
}

/// `DatabaseName` specifies the name of a database.
///
/// For example, given the document path `/db/_design/doc`, the database name is
/// `db`.
///
/// For more information about path-related types, see the [module-level
/// documentation](index.html).
///
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct DatabaseName(String);

impl DatabaseName {
    /// Joins this database name with the given document id to form a document
    /// path.
    pub fn with_document_id(self, doc_id: DocumentId) -> DocumentPath {
        DocumentPath::from((self, doc_id))
    }

    /// Joins this database name with the given design document id to form a
    /// design document path.
    pub fn with_design_document_id(self, ddoc_id: DesignDocumentId) -> DesignDocumentPath {
        DesignDocumentPath::from((self, ddoc_id))
    }

    /// Converts this database name into a database path.
    pub fn into_database_path(self) -> DatabasePath {
        DatabasePath::from(self)
    }

    fn encode_to(&self, f: &mut fmt::Formatter) -> fmt::Result {
        percent_encoding::percent_encode(self.0.as_bytes(), percent_encoding::PATH_SEGMENT_ENCODE_SET).fmt(f)
    }
}

impl AsRef<str> for DatabaseName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<DatabaseName> for String {
    fn from(db_name: DatabaseName) -> Self {
        String::from(db_name.0)
    }
}

impl Display for DatabaseName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a> From<&'a str> for DatabaseName {
    fn from(s: &'a str) -> Self {
        DatabaseName(String::from(s))
    }
}

impl From<String> for DatabaseName {
    fn from(s: String) -> Self {
        DatabaseName(s)
    }
}

/// `DatabasePath` specifies the full path of a database.
///
/// For more information about path-related types, see the [module-level
/// documentation](index.html).
///
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DatabasePath {
    db_name: DatabaseName,
}

impl DatabasePath {
    /// Borrows this database path's database name.
    pub fn database_name(&self) -> &DatabaseName {
        &self.db_name
    }

    /// Joins this database path with the given document id to form a document
    /// path.
    pub fn with_document_id(self, doc_id: DocumentId) -> DocumentPath {
        DocumentPath::from((self.db_name, doc_id))
    }

    /// Joins this database path with the given design document id to form a
    /// design document path.
    pub fn with_design_document_id(self, ddoc_id: DesignDocumentId) -> DesignDocumentPath {
        DesignDocumentPath::from((self.db_name, ddoc_id))
    }
}

// DatabasePath is a special path type in that it can be constructed from only
// its component. The other path types require tuples because they require
// multiple components.
impl From<DatabaseName> for DatabasePath {
    fn from(db_name: DatabaseName) -> Self {
        DatabasePath { db_name: db_name }
    }
}

impl<T> From<(T,)> for DatabasePath
where
    T: Into<DatabaseName>,
{
    fn from(db_name: (T,)) -> Self {
        DatabasePath { db_name: db_name.0.into() }
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        '/'.fmt(f)?;
        self.db_name.encode_to(f)
    }
}

/// The `IntoDatabasePath` trait converts types into a `DatabasePath`.
///
/// **TODO:** Replace this trait with `TryInto` when it has stabilized.
///
/// For more information about path-related types, see the [module-level
/// documentation](index.html).
///
pub trait IntoDatabasePath {
    fn into_database_path(self) -> Result<DatabasePath, Error>;
}

impl IntoDatabasePath for DatabasePath {
    fn into_database_path(self) -> Result<DatabasePath, Error> {
        Ok(self)
    }
}

impl<'a> IntoDatabasePath for &'a str {
    fn into_database_path(self) -> Result<DatabasePath, Error> {
        DatabasePath::from_str(self)
    }
}

/// `DocumentId` specifies a document id.
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
    /// Returns whether this document id specifies a design document—i.e., the
    /// document begins with the `_design/` prefix.
    pub fn is_design(&self) -> bool {
        DocumentId::has_given_prefix(&self.0, DESIGN_PREFIX)
    }

    /// Convert this document id into a design document id, if possible.
    pub fn upgrade_into_design_document_id(self) -> Result<DesignDocumentId, DocumentId> {
        if self.is_design() { Ok(DesignDocumentId(self)) } else { Err(self) }
    }

    /// Returns whether this document id specifies a local document—i.e., the
    /// document begins with the `_local/` prefix.
    pub fn is_local(&self) -> bool {
        DocumentId::has_given_prefix(&self.0, LOCAL_PREFIX)
    }

    fn encode_to(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (prefix, base) = self.split_prefix();
        if let Some(prefix) = prefix {
            percent_encoding::percent_encode(prefix.as_bytes(), percent_encoding::PATH_SEGMENT_ENCODE_SET)
                .fmt(f)?;
            '/'.fmt(f)?;
        }
        percent_encoding::percent_encode(base.as_bytes(), percent_encoding::PATH_SEGMENT_ENCODE_SET).fmt(f)
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

/// `DocumentPath` specifies the full path of a document.
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
    /// Borrows this document path's database name.
    pub fn database_name(&self) -> &DatabaseName {
        &self.db_name
    }

    /// Borrows this document path's document id.
    pub fn document_id(&self) -> &DocumentId {
        &self.doc_id
    }

    /// Joins this document path with the given attachment name to form an
    /// attachment path.
    pub fn with_attachment_name(self, att_name: AttachmentName) -> AttachmentPath {
        AttachmentPath::from((self.db_name, self.doc_id, att_name))
    }
}

impl<T, U> From<(T, U)> for DocumentPath
where
    T: Into<DatabaseName>,
    U: Into<DocumentId>,
{
    fn from((db_name, doc_id): (T, U)) -> Self {
        DocumentPath {
            db_name: db_name.into(),
            doc_id: doc_id.into(),
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        '/'.fmt(f)?;
        self.db_name.encode_to(f)?;
        '/'.fmt(f)?;
        self.doc_id.encode_to(f)
    }
}

/// The `IntoDocumentPath` trait converts types into a `DocumentPath`.
///
/// **TODO:** Replace this trait with `TryInto` when it has stabilized.
///
/// For more information about path-related types, see the [module-level
/// documentation](index.html).
///
pub trait IntoDocumentPath {
    fn into_document_path(self) -> Result<DocumentPath, Error>;
}

impl IntoDocumentPath for DocumentPath {
    fn into_document_path(self) -> Result<DocumentPath, Error> {
        Ok(self)
    }
}

impl<'a> IntoDocumentPath for &'a str {
    fn into_document_path(self) -> Result<DocumentPath, Error> {
        DocumentPath::from_str(self)
    }
}

/// `DesignDocumentId` specifies a design document id—i.e., a document id that
/// begins with the `_design/` prefix.
///
/// For example, given the document path `/db/_design/doc`, the design document
/// id is `_design/doc`.
///
/// The `DesignDocumentId` type is a special form of the `DocumentId` type. All
/// design document ids are document ids, but not all document ids are design
/// document ids.
///
/// For more information about path-related types, see the [module-level
/// documentation](index.html).
///
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct DesignDocumentId(DocumentId);

impl DesignDocumentId {
    fn validate(s: &str) -> Result<(), Error> {
        if s.len() <= DESIGN_PREFIX.len() + '/'.len_utf8() || !s.starts_with(DESIGN_PREFIX) ||
            !s[DESIGN_PREFIX.len()..].starts_with('/')
        {
            return Err(Error::from(
                ("String does not begin with '_design/' prefix"),
            ));
        }
        Ok(())
    }

    fn encode_to(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.encode_to(f)
    }

    /// Converts this design document id into a general document id.
    pub fn into_document_id(self) -> DocumentId {
        self.0
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

impl<'a> Deserialize<'a> for DesignDocumentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        struct Visitor;

        impl<'b> serde::de::Visitor<'b> for Visitor {
            type Value = DesignDocumentId;

            fn expecting(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
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

/// `DesignDocumentPath` specifies the full path of a design document.
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
    /// Borrows this design document path's database name.
    pub fn database_name(&self) -> &DatabaseName {
        &self.db_name
    }

    /// Borrows this design document path's design document id.
    pub fn design_document_id(&self) -> &DesignDocumentId {
        &self.ddoc_id
    }

    /// Joins this design document path with the given attachment name to form an
    /// attachment path.
    pub fn with_attachment_name(self, att_name: AttachmentName) -> AttachmentPath {
        AttachmentPath::from((self.db_name, self.ddoc_id, att_name))
    }

    /// Joins this design document path with the given view name to form a view
    /// path.
    pub fn with_view_name(self, view_name: ViewName) -> ViewPath {
        ViewPath::from((self.db_name, self.ddoc_id, view_name))
    }
}

impl<T, U> From<(T, U)> for DesignDocumentPath
where
    T: Into<DatabaseName>,
    U: Into<DesignDocumentId>,
{
    fn from((db_name, ddoc_id): (T, U)) -> Self {
        DesignDocumentPath {
            db_name: db_name.into(),
            ddoc_id: ddoc_id.into(),
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        '/'.fmt(f)?;
        self.db_name.encode_to(f)?;
        '/'.fmt(f)?;
        self.ddoc_id.encode_to(f)
    }
}

/// The `IntoDesignDocumentPath` trait converts types into a
/// `DesignDocumentPath`.
///
/// **TODO:** Replace this trait with `TryInto` when it has stabilized.
///
/// For more information about path-related types, see the [module-level
/// documentation](index.html).
///
pub trait IntoDesignDocumentPath {
    fn into_design_document_path(self) -> Result<DesignDocumentPath, Error>;
}

impl IntoDesignDocumentPath for DesignDocumentPath {
    fn into_design_document_path(self) -> Result<DesignDocumentPath, Error> {
        Ok(self)
    }
}

impl<'a> IntoDesignDocumentPath for &'a str {
    fn into_design_document_path(self) -> Result<DesignDocumentPath, Error> {
        DesignDocumentPath::from_str(self)
    }
}

/// `AttachmentName` specifies the name of an attachment.
///
/// For example, given the attachment path `/db/doc/attachment`, the attachment
/// name is `attachment`.
///
/// For more information about path-related types, see the [module-level
/// documentation](index.html).
///
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct AttachmentName(String);

impl AttachmentName {
    fn encode_to(&self, f: &mut fmt::Formatter) -> fmt::Result {
        percent_encoding::percent_encode(self.0.as_bytes(), percent_encoding::PATH_SEGMENT_ENCODE_SET).fmt(f)
    }
}

impl AsRef<str> for AttachmentName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<AttachmentName> for String {
    fn from(att_name: AttachmentName) -> Self {
        String::from(att_name.0)
    }
}

impl Display for AttachmentName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a> From<&'a str> for AttachmentName {
    fn from(s: &'a str) -> Self {
        AttachmentName(String::from(s))
    }
}

impl From<String> for AttachmentName {
    fn from(s: String) -> Self {
        AttachmentName(s)
    }
}

/// `AttachmentPath` specifies the full path of an attachment.
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
    /// Borrows this attachment path's database name.
    pub fn database_name(&self) -> &DatabaseName {
        &self.db_name
    }

    /// Borrows this attachment path's document id.
    pub fn document_id(&self) -> &DocumentId {
        &self.doc_id
    }

    /// Borrows this attachment path's attachment name.
    pub fn attachment_name(&self) -> &AttachmentName {
        &self.att_name
    }
}

impl<T, U, V> From<(T, U, V)> for AttachmentPath
where
    T: Into<DatabaseName>,
    U: Into<DocumentId>,
    V: Into<AttachmentName>,
{
    fn from((db_name, doc_id, att_name): (T, U, V)) -> Self {
        AttachmentPath {
            db_name: db_name.into(),
            doc_id: doc_id.into(),
            att_name: att_name.into(),
        }
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        '/'.fmt(f)?;
        self.db_name.encode_to(f)?;
        '/'.fmt(f)?;
        self.doc_id.encode_to(f)?;
        '/'.fmt(f)?;
        self.att_name.encode_to(f)
    }
}

/// The `IntoAttachmentPath` trait converts types into an `AttachmentPath`.
///
/// **TODO:** Replace this trait with `TryInto` when it has stabilized.
///
/// For more information about path-related types, see the [module-level
/// documentation](index.html).
///
pub trait IntoAttachmentPath {
    fn into_attachment_path(self) -> Result<AttachmentPath, Error>;
}

impl IntoAttachmentPath for AttachmentPath {
    fn into_attachment_path(self) -> Result<AttachmentPath, Error> {
        Ok(self)
    }
}

impl<'a> IntoAttachmentPath for &'a str {
    fn into_attachment_path(self) -> Result<AttachmentPath, Error> {
        AttachmentPath::from_str(self)
    }
}

/// `ViewName` specifies the name of a view.
///
/// For example, given the view path `/db/_design/doc/_view/view`, the view name
/// is `view`.
///
/// For more information about path-related types, see the [module-level
/// documentation](index.html).
///
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ViewName(String);

impl ViewName {
    fn encode_to(&self, f: &mut fmt::Formatter) -> fmt::Result {
        percent_encoding::percent_encode(self.0.as_bytes(), percent_encoding::PATH_SEGMENT_ENCODE_SET).fmt(f)
    }
}

impl AsRef<str> for ViewName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<ViewName> for String {
    fn from(view_name: ViewName) -> Self {
        String::from(view_name.0)
    }
}

impl Display for ViewName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a> From<&'a str> for ViewName {
    fn from(s: &'a str) -> Self {
        ViewName(String::from(s))
    }
}

impl From<String> for ViewName {
    fn from(s: String) -> Self {
        ViewName(s)
    }
}

/// `ViewPath` specifies the full path of a view.
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
    /// Borrows this view path's database name.
    pub fn database_name(&self) -> &DatabaseName {
        &self.db_name
    }

    /// Borrows this view path's design document id.
    pub fn design_document_id(&self) -> &DesignDocumentId {
        &self.ddoc_id
    }

    /// Borrows this view path's view name.
    pub fn view_name(&self) -> &ViewName {
        &self.view_name
    }
}

impl<T, U, V> From<(T, U, V)> for ViewPath
where
    T: Into<DatabaseName>,
    U: Into<DesignDocumentId>,
    V: Into<ViewName>,
{
    fn from((db_name, ddoc_id, view_name): (T, U, V)) -> Self {
        ViewPath {
            db_name: db_name.into(),
            ddoc_id: ddoc_id.into(),
            view_name: view_name.into(),
        }
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        '/'.fmt(f)?;
        self.db_name.encode_to(f)?;
        '/'.fmt(f)?;
        self.ddoc_id.encode_to(f)?;
        '/'.fmt(f)?;
        VIEW_PREFIX.fmt(f)?;
        '/'.fmt(f)?;
        self.view_name.encode_to(f)
    }
}

/// The `IntoViewPath` trait converts types into a `ViewPath`.
///
/// **TODO:** Replace this trait with `TryInto` when it has stabilized.
///
/// For more information about path-related types, see the [module-level
/// documentation](index.html).
///
pub trait IntoViewPath {
    fn into_view_path(self) -> Result<ViewPath, Error>;
}

impl IntoViewPath for ViewPath {
    fn into_view_path(self) -> Result<ViewPath, Error> {
        Ok(self)
    }
}

impl<'a> IntoViewPath for &'a str {
    fn into_view_path(self) -> Result<ViewPath, Error> {
        ViewPath::from_str(self)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use super::{E_EMPTY_SEGMENT, E_NO_LEADING_SLASH, E_TOO_FEW_SEGMENTS, E_TOO_MANY_SEGMENTS, E_TRAILING_SLASH,
                E_UNEXPECTED_SEGMENT, PathDecoder};
    use serde_json;

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
    fn path_parsing_fails_on_an_unexpected_segment() {
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
    fn path_parsing_succeeds_on_a_prefix() {
        let mut p = PathDecoder::begin("/alpha/bravo/charlie").unwrap();
        assert_eq!(
            p.decode_with_prefix::<String>("alpha").unwrap(),
            "alpha/bravo"
        );
        assert_eq!(p.decode_segment::<String>().unwrap(), "charlie");
        p.end().unwrap();
    }

    #[test]
    fn path_parsing_succeeds_on_an_optional_prefix() {
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
    fn path_parsing_percent_decodes() {
        let mut p = PathDecoder::begin("/alpha%20bravo%2fcharlie").unwrap();
        assert_eq!(p.decode_segment::<String>().unwrap(), "alpha bravo/charlie");
        p.end().unwrap();
    }

    #[test]
    fn database_path_percent_encodes_itself() {
        let got = DatabasePath::from(("alpha bravo",)).to_string();
        let expected = "/alpha%20bravo";
        assert_eq!(got, expected);
    }

    #[test]
    fn database_path_decodes_str() {
        let got: DatabasePath = "/alpha bravo".parse().unwrap();
        let expected = DatabasePath::from(("alpha bravo",));
        assert_eq!(got, expected);
    }

    #[test]
    fn document_id_distinguishes_by_document_type() {
        let doc_id = DocumentId::from("alpha");
        assert!(!doc_id.is_design());
        assert!(!doc_id.is_local());

        let doc_id = DocumentId::from("_design/alpha");
        assert!(doc_id.is_design());
        assert!(!doc_id.is_local());

        let doc_id = DocumentId::from("_local/alpha");
        assert!(!doc_id.is_design());
        assert!(doc_id.is_local());
    }

    #[test]
    fn document_path_percent_encodes_itself() {
        let got = DocumentPath::from(("alpha bravo", "charlie delta")).to_string();
        let expected = "/alpha%20bravo/charlie%20delta";
        assert_eq!(got, expected);

        let got = DocumentPath::from(("alpha bravo", "_design/charlie delta")).to_string();
        let expected = "/alpha%20bravo/_design/charlie%20delta";
        assert_eq!(got, expected);

        let got = DocumentPath::from(("alpha bravo", "_local/charlie delta")).to_string();
        let expected = "/alpha%20bravo/_local/charlie%20delta";
        assert_eq!(got, expected);
    }

    #[test]
    fn document_path_from_str() {
        let got = "/alpha bravo/charlie delta".into_document_path().unwrap();
        let expected = DocumentPath::from(("alpha bravo", "charlie delta"));
        assert_eq!(got, expected);

        let got = "/alpha bravo/_design/charlie delta"
            .into_document_path()
            .unwrap();
        let expected = DocumentPath::from(("alpha bravo", "_design/charlie delta"));
        assert_eq!(got, expected);

        let got = "/alpha bravo/_local/charlie delta"
            .into_document_path()
            .unwrap();
        let expected = DocumentPath::from(("alpha bravo", "_local/charlie delta"));
        assert_eq!(got, expected);
    }

    #[test]
    fn design_document_id_deserialization_enforces_design_prefix() {

        let source = r#""_design/alpha""#;
        let got: DesignDocumentId = serde_json::from_str(&source).unwrap();
        let expected = DesignDocumentId(DocumentId::from("_design/alpha"));
        assert_eq!(got, expected);

        let source = r#""alpha""#;
        match serde_json::from_str::<DesignDocumentId>(&source) {
            Err(ref e) if e.is_data() => {}
            x => panic!("Got unexpected result {:?}", x),
        }
    }

    #[test]
    fn design_document_id_from_str_requires_design_prefix() {

        let got = DesignDocumentId::from_str("_design/alpha").unwrap();
        let expected = DesignDocumentId(DocumentId::from("_design/alpha"));
        assert_eq!(got, expected);

        DesignDocumentId::from_str("alpha").unwrap_err();
        DesignDocumentId::from_str("_local/alpha").unwrap_err();
    }

    #[test]
    fn design_document_path_percent_encodes_itself() {
        let got = DesignDocumentPath::from((
            "alpha bravo",
            DesignDocumentId::from_str("_design/charlie delta").unwrap(),
        )).to_string();
        let expected = "/alpha%20bravo/_design/charlie%20delta";
        assert_eq!(got, expected);
    }

    #[test]
    fn design_document_path_from_str() {
        let got = "/alpha bravo/_design/charlie delta"
            .into_design_document_path()
            .unwrap();
        let expected = DesignDocumentPath::from((
            "alpha bravo",
            DesignDocumentId::from_str("_design/charlie delta").unwrap(),
        ));
        assert_eq!(got, expected);

        "/alpha bravo/charlie delta"
            .into_design_document_path()
            .unwrap_err();
        "/alpha bravo/_local/charlie delta"
            .into_design_document_path()
            .unwrap_err();
    }

    #[test]
    fn attachment_path_percent_encodes_itself() {
        let got = AttachmentPath::from(("alpha bravo", "charlie delta", "echo foxtrot")).to_string();
        let expected = "/alpha%20bravo/charlie%20delta/echo%20foxtrot";
        assert_eq!(got, expected);
    }

    #[test]
    fn attachment_path_from_str() {
        let got = "/alpha bravo/charlie delta/echo foxtrot"
            .into_attachment_path()
            .unwrap();
        let expected = AttachmentPath::from(("alpha bravo", "charlie delta", "echo foxtrot"));
        assert_eq!(got, expected);
    }

    #[test]
    fn view_path_percent_encodes_itself() {
        let got = ViewPath::from((
            "alpha bravo",
            DesignDocumentId::from_str("_design/charlie delta").unwrap(),
            "echo foxtrot",
        )).to_string();
        let expected = "/alpha%20bravo/_design/charlie%20delta/_view/echo%20foxtrot";
        assert_eq!(got, expected);
    }

    #[test]
    fn view_path_from_str() {
        let got = "/alpha bravo/_design/charlie delta/_view/echo foxtrot"
            .into_view_path()
            .unwrap();
        let expected = ViewPath::from((
            "alpha bravo",
            DesignDocumentId::from_str("_design/charlie delta").unwrap(),
            "echo foxtrot",
        ));
        assert_eq!(got, expected);

        "/alpha/_design/bravo/_view/charlie"
            .into_view_path()
            .unwrap();
        "/alpha/_design/bravo/charlie".into_view_path().unwrap_err();
        "/alpha/bravo/_view/charlie".into_view_path().unwrap_err();
    }
}
