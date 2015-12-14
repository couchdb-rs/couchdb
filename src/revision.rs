use serde;
use std;

/// Document revision.
///
/// `Revision` wraps a `String`, providing CouchDB revision semantics. Revisions
/// are case-insensitive.
///
#[derive(Clone, Debug, Hash)]
pub struct Revision(String);

impl AsRef<str> for Revision {
    fn as_ref(&self) -> &str {
        let Revision(ref s) = *self;
        s
    }
}

impl std::fmt::Display for Revision {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(self.as_ref())
    }
}

impl From<String> for Revision {
    fn from(rev: String) -> Self {
        Revision(rev)
    }
}

impl<'a> From<&'a str> for Revision {
    fn from(rev: &str) -> Self {
        Revision(rev.to_string())
    }
}

impl From<Revision> for String {
    fn from(rev: Revision) -> Self {
        let Revision(rev) = rev;
        rev
    }
}

impl Eq for Revision {}

impl PartialEq for Revision {
    fn eq(&self, other: &Self) -> bool {
        let a = self.as_ref().to_lowercase();
        let b = other.as_ref().to_lowercase();
        a.eq(&b)
    }
}

impl Ord for Revision {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let a = self.as_ref().to_lowercase();
        let b = other.as_ref().to_lowercase();
        a.cmp(&b)
    }
}

impl PartialOrd for Revision {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let a = self.as_ref().to_lowercase();
        let b = other.as_ref().to_lowercase();
        a.partial_cmp(&b)
    }
}

impl serde::Serialize for Revision {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        let Revision(ref s) = *self;
        serializer.visit_str(s)
    }
}

impl serde::Deserialize for Revision {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = Revision;

            fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E>
                where E: serde::de::Error
            {
                Ok(Revision::from(v))
            }
        }

        deserializer.visit(Visitor)
    }
}

#[cfg(test)]
mod tests {

    use serde_json;

    use super::*;

    #[test]
    fn test_clone() {
        let r1 = Revision::from("1-1234");
        let r2 = r1.clone();
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_eq() {

        let r1 = Revision::from("1-1234");
        assert!(r1 == r1);

        let r1 = Revision::from("1-1234");
        let r2 = Revision::from("1-1234");
        assert!(r1 == r2);

        let r1 = Revision::from("1-1234");
        let r2 = Revision::from("2-5678");
        assert!(r1 != r2);

        // Verify: Equality comparison is case-insensitive.
        let r1 = Revision::from("1-abcd");
        let r2 = Revision::from("1-ABCD");
        assert!(r1 == r2);
    }

    #[test]
    fn test_ord() {

        let r1 = Revision::from("1-1234");
        let r2 = Revision::from("2-5678");
        assert!(r1 < r2);

        // Verify: Order is case-insensitive.

        let r1 = Revision::from("1-abcd");
        let r2 = Revision::from("1-ABCD");
        assert!(r1 <= r2);
        assert!(r2 <= r1);

        let r1 = Revision::from("1-aaaa");
        let r2 = Revision::from("1-BBBB");
        assert!(r1 < r2);

        let r1 = Revision::from("1-AAAA");
        let r2 = Revision::from("1-bbbb");
        assert!(r1 < r2);
    }

    #[test]
    fn test_serialize() {

        // VERIFY: Serialization outputs a string value.

        let exp = serde_json::Value::String("1-1234abcd".to_string());

        let rev = Revision::from("1-1234abcd");

        let s = serde_json::to_string(&rev).unwrap();
        let got = serde_json::from_str::<serde_json::Value>(&s).unwrap();

        assert_eq!(got, exp);
    }

    #[test]
    fn test_deserialize() {

        // VERIFY: Deserialization succeeds when input is a string value.

        let exp = Revision::from("1-1234abcd");
        let s = r#""1-1234abcd""#;

        let got = serde_json::from_str::<Revision>(&s).unwrap();

        assert_eq!(got, exp);
    }
}
