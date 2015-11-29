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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_revision_clone() {
        let r1 = Revision::from("1-1234");
        let r2 = r1.clone();
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_revision_eq() {

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
    fn test_revision_ord() {

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
}
