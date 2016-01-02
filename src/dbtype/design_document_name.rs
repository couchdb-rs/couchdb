use serde;
use std;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// FIXME: Document this. Include note about Ord guarantees.
pub struct DesignDocumentName(String);
impl_name_type!(DesignDocumentName);

impl DesignDocumentName {
    /// Construct an empty design document name.
    pub fn new() -> Self {
        DesignDocumentName(String::new())
    }
}

#[cfg(test)]
mod tests {

    use serde_json;

    use DesignDocumentName;

    #[test]
    fn design_document_name_display() {
        let expected = "foo";
        let got = format!("{}", DesignDocumentName::from("foo"));
        assert_eq!(expected, got);
    }

    #[test]
    fn design_document_name_as_ref_str() {
        let expected = "foo";
        let d = DesignDocumentName::from("foo");
        let got: &str = d.as_ref();
        assert_eq!(expected, got);
    }

    #[test]
    fn design_document_name_as_ref_string() {
        let expected = "foo".to_string();
        let d = DesignDocumentName::from("foo");
        let got = d.as_ref();
        assert_eq!(expected, got);
    }

    #[test]
    fn design_document_name_from_str_ref() {
        let expected = DesignDocumentName("foo".to_string());
        let got = DesignDocumentName::from("foo");
        assert_eq!(expected, got);
    }

    #[test]
    fn design_document_name_from_string() {
        let expected = DesignDocumentName("foo".to_string());
        let got = DesignDocumentName::from("foo".to_string());
        assert_eq!(expected, got);
    }

    #[test]
    fn string_from_design_document_name() {
        let expected = "foo".to_string();
        let got = String::from(DesignDocumentName::from("foo"));
        assert_eq!(expected, got);
    }

    #[test]
    fn design_document_name_serialization() {
        let expected = serde_json::Value::String("foo".to_string());
        let source = DesignDocumentName::from("foo");
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn design_document_name_deserialization() {
        let expected = DesignDocumentName::from("foo");
        let source = serde_json::Value::String("foo".to_string());
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn design_document_name_new() {
        let expected = DesignDocumentName::from(String::new());
        let got = DesignDocumentName::new();
        assert_eq!(expected, got);
    }
}
