use serde;
use std;

/// Name of a view.
/// 
/// A view name wraps a string specifying a view—e.g., the `view-name` part of
/// the HTTP request to GET
/// `http://example.com:5984/db/_design/design-doc/_view/view-name`.
///
/// View names may be converted to and from strings. They are never
/// percent-encoded.
///
/// Although the `ViewName` type implements the `Ord` and `PartialOrd` traits,
/// it provides no guarantees how that ordering is defined and may change the
/// definition between any two releases of the couchdb crate. That is, for two
/// `ViewName` values `a` and `b`, the expression `a < b` may hold true now but
/// not in a subsequent release. Consequently, applications must not rely upon
/// any particular ordering definition.
///
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ViewName(String);
impl_name_type!(ViewName);

impl ViewName {
    /// Constructs an empty view name.
    pub fn new() -> Self {
        ViewName(String::new())
    }
}

#[cfg(test)]
mod tests {

    use serde_json;

    use ViewName;

    #[test]
    fn view_name_display() {
        let expected = "foo";
        let got = format!("{}", ViewName::from("foo"));
        assert_eq!(expected, got);
    }

    #[test]
    fn view_name_as_ref_str() {
        let expected = "foo";
        let d = ViewName::from("foo");
        let got: &str = d.as_ref();
        assert_eq!(expected, got);
    }

    #[test]
    fn view_name_as_ref_string() {
        let expected = "foo".to_string();
        let d = ViewName::from("foo");
        let got = d.as_ref();
        assert_eq!(expected, got);
    }

    #[test]
    fn string_from_view_name() {
        let expected = "foo".to_string();
        let got = String::from(ViewName::from("foo"));
        assert_eq!(expected, got);
    }

    #[test]
    fn view_name_serialization() {
        let expected = serde_json::Value::String("foo".to_string());
        let source = ViewName::from("foo");
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn view_name_deserialization() {
        let expected = ViewName::from("foo");
        let source = serde_json::Value::String("foo".to_string());
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn view_name_new() {
        let expected = ViewName::from(String::new());
        let got = ViewName::new();
        assert_eq!(expected, got);
    }
}
