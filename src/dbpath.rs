use hyper;
use serde;

/// Database path&mdash;i.e., database name.
///
/// A database path comprises a single URI path component specifying a database
/// name&mdash;e.g., the `db` part in the HTTP request `GET
/// http://example.com:5984/db`.
///
/// `DatabasePath` provides additional type-safety over working with a raw
/// string. Nevertheless, `DatabasePath` may be converted to and from a string.
///
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DatabasePath(String);

impl DatabasePath {
    /// Convert the `DatabasePath` into a URI.
    pub fn into_uri(self, base_uri: hyper::Url) -> hyper::Url {
        let mut uri = base_uri;

        {
            let mut p = uri.path_mut().unwrap();
            if p.len() == 1 && p[0].is_empty() {
                p.clear();
            }
            let DatabasePath(db_name) = self;
            p.push(db_name);
        }

        uri
    }

    /// Return the database name part of the database path.
    pub fn database_name(&self) -> &String {
        let DatabasePath(ref db_name) = *self;
        db_name
    }
}

impl AsRef<String> for DatabasePath {
    fn as_ref(&self) -> &String {
        let DatabasePath(ref db_name) = *self;
        db_name
    }
}

impl<'a> From<&'a str> for DatabasePath {
    fn from(db_name: &str) -> Self {
        DatabasePath(db_name.to_string())
    }
}

impl From<String> for DatabasePath {
    fn from(db_name: String) -> Self {
        DatabasePath(db_name)
    }
}

impl From<DatabasePath> for String {
    fn from(db_name: DatabasePath) -> String {
        let DatabasePath(db_name) = db_name;
        db_name
    }
}

impl serde::Serialize for DatabasePath {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        let DatabasePath(ref db_name) = *self;
        serializer.visit_str(db_name)
    }
}

impl serde::Deserialize for DatabasePath {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = DatabasePath;

            fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E>
                where E: serde::de::Error
            {
                Ok(DatabasePath(v.to_string()))
            }

            fn visit_string<E>(&mut self, v: String) -> Result<Self::Value, E>
                where E: serde::de::Error
            {
                Ok(DatabasePath(v))
            }
        }

        deserializer.visit(Visitor)
    }
}

#[cfg(test)]
mod tests {
    use hyper;
    use serde_json;

    use super::*;

    #[test]
    fn test_database_path_clone() {
        let a = DatabasePath::from("foo");
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_database_path_eq() {
        let a = DatabasePath::from("foo");
        let b = DatabasePath::from("foo");
        assert!(a == b);
        assert!(a == a);

        let a = DatabasePath::from("foo");
        let b = DatabasePath::from("bar");
        assert!(a != b);

        let a = DatabasePath::from("foo");
        let b = DatabasePath::from("FOO");
        assert!(a != b);
    }

    #[test]
    fn test_database_path_ord() {
        let a = DatabasePath::from("foo");
        let b = DatabasePath::from("bar");
        assert!(b < a);
        assert!(a <= a);

        let a = DatabasePath::from("foo");
        let b = DatabasePath::from("foo");
        assert!(a <= b);
    }

    #[test]
    fn test_database_path_serialization() {
        let pre = DatabasePath::from("foo");
        let j = serde_json::to_string(&pre).unwrap();
        let post = serde_json::from_str::<DatabasePath>(&j).unwrap();
        assert_eq!(pre, post);
    }

    #[test]
    fn test_database_path_into_uri() {

        // Verify: A normal URI base yields a normal database URI path.
        let base = hyper::Url::parse("http://example.com:1234").unwrap();
        let uri = DatabasePath::from("foo").into_uri(base);
        let exp = hyper::Url::parse("http://example.com:1234/foo").unwrap();
        assert_eq!(uri, exp);

        // Verify: A URI base with a nonempty path yields a URI with the full
        // path.
        let base = hyper::Url::parse("http://example.com:1234/bar").unwrap();
        let uri = DatabasePath::from("foo").into_uri(base);
        let exp = hyper::Url::parse("http://example.com:1234/bar/foo").unwrap();
        assert_eq!(uri, exp);
    }

    #[test]
    fn test_database_path_accessors() {
        let db_path = DatabasePath::from("foo");
        assert_eq!(*db_path.database_name(), "foo".to_string());
    }
}
