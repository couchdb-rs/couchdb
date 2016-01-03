use serde;
use std;

/// Name of a database.
/// 
/// A database name wraps a string specifying a database—e.g., the `db` part of
/// the HTTP request to GET `http://example.com:5984/db`.
///
/// Database names may be converted to and from strings. They are never
/// percent-encoded.
///
/// Although the `DatabaseName` type implements the `Ord` and `PartialOrd`
/// traits, it provides no guarantees how that ordering is defined and may
/// change the definition between any two releases of the couchdb crate. That
/// is, for two `DatabaseName` values `a` and `b`, the expression `a < b` may
/// hold true now but not in a subsequent release. Consequently, applications
/// must not rely upon any particular ordering definition.
///
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DatabaseName(String);
impl_name_type!(DatabaseName);

impl DatabaseName {
    /// Constructs an empty database name.
    pub fn new() -> Self {
        DatabaseName(String::new())
    }
}

#[cfg(test)]
mod tests {

    use serde_json;

    use DatabaseName;

    #[test]
    fn database_name_display() {
        let expected = "foo";
        let got = format!("{}", DatabaseName::from("foo"));
        assert_eq!(expected, got);
    }

    #[test]
    fn database_name_as_ref_str() {
        let expected = "foo";
        let d = DatabaseName::from("foo");
        let got: &str = d.as_ref();
        assert_eq!(expected, got);
    }

    #[test]
    fn database_name_as_ref_string() {
        let expected = "foo".to_string();
        let d = DatabaseName::from("foo");
        let got = d.as_ref();
        assert_eq!(expected, got);
    }

    #[test]
    fn database_name_from_str_ref() {
        let expected = DatabaseName("foo".to_string());
        let got = DatabaseName::from("foo");
        assert_eq!(expected, got);
    }

    #[test]
    fn database_name_from_string() {
        let expected = DatabaseName("foo".to_string());
        let got = DatabaseName::from("foo".to_string());
        assert_eq!(expected, got);
    }

    #[test]
    fn string_from_database_name() {
        let expected = "foo".to_string();
        let got = String::from(DatabaseName::from("foo"));
        assert_eq!(expected, got);
    }

    #[test]
    fn database_name_serialization() {
        let expected = serde_json::Value::String("foo".to_string());
        let source = DatabaseName::from("foo");
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn database_name_deserialization() {
        let expected = DatabaseName::from("foo");
        let source = serde_json::Value::String("foo".to_string());
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn database_name_new() {
        let expected = DatabaseName::from(String::new());
        let got = DatabaseName::new();
        assert_eq!(expected, got);
    }
}
