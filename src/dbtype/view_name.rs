use serde;
use std;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// FIXME: Document this. Include note about Ord guarantees.
pub struct ViewName(String);
impl_name_type!(ViewName);

impl ViewName {
    /// Construct an empty view name.
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
    fn view_name_from_str_ref() {
        let expected = ViewName("foo".to_string());
        let got = ViewName::from("foo");
        assert_eq!(expected, got);
    }

    #[test]
    fn view_name_from_string() {
        let expected = ViewName("foo".to_string());
        let got = ViewName::from("foo".to_string());
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
