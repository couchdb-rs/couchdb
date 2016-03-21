use serde;
use std;

use DesignDocumentName;
use DocumentName;

/// Document identifier.
///
/// A document id specifies a document's type and name. For example, given the
/// HTTP request to `GET http://example.com:5984/db/_design/design-doc`, the
/// document id comprises `_design/design-doc` and specifies a design document
/// with the name `design-doc`.
///
/// There are three types of documents: *normal*, *design* (i.e., `_design`),
/// and *local* (i.e., `_local`). Each type is expressed as an enum variant that
/// owns the underlying document name.
///
/// Although the `DocumentId` type implements the `Ord` and `PartialOrd` traits,
/// it provides no guarantees how that ordering is defined and may change the
/// definition between any two releases of the couchdb crate. That is, for two
/// `DocumentId` values `a` and `b`, the expression `a < b` may hold true now
/// but not in a subsequent release. Consequently, applications must not rely
/// upon any particular ordering definition.
///
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DocumentId {
    /// Normal document—i.e., neither a design document nor a local document.
    Normal(DocumentName),
    /// Design document (i.e., `_design`).
    Design(DesignDocumentName),

    /// Local document (i.e., `_local`).
    Local(DocumentName),
}

impl std::fmt::Display for DocumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            DocumentId::Normal(ref name) => write!(f, "{}", name),
            DocumentId::Design(ref name) => write!(f, "_design/{}", name),
            DocumentId::Local(ref name) => write!(f, "_local/{}", name),
        }
    }
}

impl<'a> From<&'a str> for DocumentId {
    fn from(name: &str) -> Self {
        DocumentId::from(name.to_string())
    }
}

impl From<String> for DocumentId {
    fn from(name: String) -> Self {
        let design = "_design/";
        let local = "_local/";
        if name.starts_with(design) {
            let name = name[design.len()..].to_string();
            DocumentId::Design(name.into())
        } else if name.starts_with(local) {
            let name = name[local.len()..].to_string();
            DocumentId::Local(name.into())
        } else {
            DocumentId::Normal(name.into())
        }
    }
}

impl From<DocumentId> for String {
    fn from(doc_id: DocumentId) -> Self {
        match doc_id {
            DocumentId::Normal(name) => name.into(),
            DocumentId::Design(name) => format!("_design/{}", name),
            DocumentId::Local(name) => format!("_local/{}", name),
        }
    }
}

impl serde::Serialize for DocumentId {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        match *self {
            DocumentId::Normal(ref name) => serializer.serialize_str(name.as_ref()),
            _ => serializer.serialize_str(self.to_string().as_ref()),
        }
    }
}

impl serde::Deserialize for DocumentId {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = DocumentId;

            fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E>
                where E: serde::de::Error
            {
                Ok(DocumentId::from(v))
            }

            fn visit_string<E>(&mut self, v: String) -> Result<Self::Value, E>
                where E: serde::de::Error
            {
                Ok(DocumentId::from(v))
            }
        }

        deserializer.deserialize(Visitor)
    }
}
#[cfg(test)]
mod tests {

    use serde_json;

    use DocumentId;

    #[test]
    fn document_id_display_normal() {
        let expected = "foo";
        let got = format!("{}", DocumentId::Normal("foo".into()));
        assert_eq!(expected, got);
    }

    #[test]
    fn document_id_display_design() {
        let expected = "_design/foo";
        let got = format!("{}", DocumentId::Design("foo".into()));
        assert_eq!(expected, got);
    }

    #[test]
    fn document_id_display_local() {
        let expected = "_local/foo";
        let got = format!("{}", DocumentId::Local("foo".into()));
        assert_eq!(expected, got);
    }

    #[test]
    fn document_id_from_str_ref_normal() {
        let expected = DocumentId::Normal("foo".into());
        let got = DocumentId::from("foo");
        assert_eq!(expected, got);
    }

    #[test]
    fn document_id_from_str_ref_design() {
        let expected = DocumentId::Design("foo".into());
        let got = DocumentId::from("_design/foo");
        assert_eq!(expected, got);
    }

    #[test]
    fn document_id_from_str_ref_local() {
        let expected = DocumentId::Local("foo".into());
        let got = DocumentId::from("_local/foo");
        assert_eq!(expected, got);
    }

    #[test]
    fn document_id_from_string_normal() {
        let expected = DocumentId::Normal("foo".into());
        let got = DocumentId::from("foo".to_string());
        assert_eq!(expected, got);
    }

    #[test]
    fn document_id_from_string_design() {
        let expected = DocumentId::Design("foo".into());
        let got = DocumentId::from("_design/foo".to_string());
        assert_eq!(expected, got);
    }

    #[test]
    fn document_id_from_string_local() {
        let expected = DocumentId::Local("foo".into());
        let got = DocumentId::from("_local/foo".to_string());
        assert_eq!(expected, got);
    }

    #[test]
    fn string_from_document_id_normal() {
        let expected = "foo".to_string();
        let got = String::from(DocumentId::Normal("foo".into()));
        assert_eq!(expected, got);
    }

    #[test]
    fn string_from_document_id_design() {
        let expected = "_design/foo".to_string();
        let got = String::from(DocumentId::Design("foo".into()));
        assert_eq!(expected, got);
    }

    #[test]
    fn string_from_document_id_local() {
        let expected = "_local/foo".to_string();
        let got = String::from(DocumentId::Local("foo".into()));
        assert_eq!(expected, got);
    }

    #[test]
    fn document_id_serialization_normal() {
        let expected = serde_json::Value::String("foo".to_string());
        let source = DocumentId::Normal("foo".into());
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn document_id_serialization_design() {
        let expected = serde_json::Value::String("_design/foo".to_string());
        let source = DocumentId::Design("foo".into());
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn document_id_serialization_local() {
        let expected = serde_json::Value::String("_local/foo".to_string());
        let source = DocumentId::Local("foo".into());
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn document_id_deserialization_normal() {
        let expected = DocumentId::Normal("foo".into());
        let source = serde_json::Value::String("foo".to_string());
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn document_id_deserialization_design() {
        let expected = DocumentId::Design("foo".into());
        let source = serde_json::Value::String("_design/foo".to_string());
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn document_id_deserialization_local() {
        let expected = DocumentId::Local("foo".into());
        let source = serde_json::Value::String("_local/foo".to_string());
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }
}
