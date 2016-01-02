use serde;
use std;

use DesignDocumentName;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// FIXME: Document this. Include note about Ord guarantees.
pub struct DocumentName(String);
impl_name_type!(DocumentName);

impl DocumentName {
    /// Construct an empty document name.
    pub fn new() -> Self {
        DocumentName(String::new())
    }
}

impl From<DesignDocumentName> for DocumentName {
    fn from(ddoc_name: DesignDocumentName) -> Self {
        let s = String::from(ddoc_name);
        DocumentName::from(s)
    }
}

#[cfg(test)]
mod tests {

    use serde_json;

    use DesignDocumentName;
    use DocumentName;

    #[test]
    fn document_name_display() {
        let expected = "foo";
        let got = format!("{}", DocumentName::from("foo"));
        assert_eq!(expected, got);
    }

    #[test]
    fn document_name_as_ref_str() {
        let expected = "foo";
        let d = DocumentName::from("foo");
        let got: &str = d.as_ref();
        assert_eq!(expected, got);
    }

    #[test]
    fn document_name_as_ref_string() {
        let expected = "foo".to_string();
        let d = DocumentName::from("foo");
        let got = d.as_ref();
        assert_eq!(expected, got);
    }

    #[test]
    fn document_name_from_str_ref() {
        let expected = DocumentName("foo".to_string());
        let got = DocumentName::from("foo");
        assert_eq!(expected, got);
    }

    #[test]
    fn document_name_from_string() {
        let expected = DocumentName("foo".to_string());
        let got = DocumentName::from("foo".to_string());
        assert_eq!(expected, got);
    }

    #[test]
    fn document_name_from_design_document_name() {
        let expected = DocumentName("foo".to_string());
        let got = DocumentName::from(DesignDocumentName::from("foo"));
        assert_eq!(expected, got);
    }

    #[test]
    fn string_from_document_name() {
        let expected = "foo".to_string();
        let got = String::from(DocumentName::from("foo"));
        assert_eq!(expected, got);
    }

    #[test]
    fn document_name_serialization() {
        let expected = serde_json::Value::String("foo".to_string());
        let source = DocumentName::from("foo");
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn document_name_deserialization() {
        let expected = DocumentName::from("foo");
        let source = serde_json::Value::String("foo".to_string());
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn document_name_new() {
        let expected = DocumentName::from(String::new());
        let got = DocumentName::new();
        assert_eq!(expected, got);
    }
}
