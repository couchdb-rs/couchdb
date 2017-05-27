use serde;
use std;

/// Name of a design document.
/// 
/// A design document name wraps a string specifying a design document—e.g., the
/// `design-doc` part of the HTTP request to GET
/// `http://example.com:5984/db/_design/design-doc`. The `DesignDocumentName`
/// type is a specialization of the `DocumentName` type. All design document
/// names are document names, but not all document names are design document
/// names.
///
/// Design document names may be converted to and from strings. They are never
/// percent-encoded.
///
/// Although the `DesignDocumentName` type implements the `Ord` and `PartialOrd`
/// traits, it provides no guarantees how that ordering is defined and may
/// change the definition between any two releases of the couchdb crate. That
/// is, for two `DesignDocumentName` values `a` and `b`, the expression `a < b`
/// may hold true now but not in a subsequent release. Consequently,
/// applications must not rely upon any particular ordering definition.
///
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DesignDocumentName(String);
impl_name_type!(DesignDocumentName);

impl DesignDocumentName {
    /// Constructs an empty design document name.
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
