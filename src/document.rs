use serde;
use std;

/// Document revision.
#[derive(Debug)]
pub struct Revision(String);

impl Revision {
    pub fn as_str(&self) -> &str {
        let Revision(ref s) = *self;
        s
    }
}

pub fn new_revision_from_string(rev: String) -> Revision {
    Revision(rev)
}

impl Clone for Revision {
    fn clone(&self) -> Self {
        let Revision(ref s) = *self;
        new_revision_from_string(s.clone())
    }
}

impl std::fmt::Display for Revision {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let Revision(ref s) = *self;
        fmt.write_str(s)
    }
}

impl Ord for Revision {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let Revision(ref a) = *self;
        let Revision(ref b) = *other;
        a.cmp(b)
    }
}

impl Eq for Revision {}

impl PartialEq for Revision {
    fn eq(&self, other: &Self) -> bool {
        let Revision(ref a) = *self;
        let Revision(ref b) = *other;
        a.eq(b)
    }
}

impl PartialOrd for Revision {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let Revision(ref a) = *self;
        let Revision(ref b) = *other;
        a.partial_cmp(b)
    }
}

/// Document meta-information and content, as returned by the CouchDB server.
#[derive(Debug)]
pub struct Document<T: serde::Deserialize> {
    pub id: String,
    pub revision: Revision,
    pub content: T,
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_revision() {

        let r1 = super::new_revision_from_string("1-1234".to_string());

        let r2 = r1.clone();
        assert!(r1 == r2);
        assert!(!(r1 != r2));
        assert!(r1 <= r2);
        assert!(!(r1 < r2));
        assert!(r2 <= r1);
        assert!(!(r2 < r1));
        let r2 = super::new_revision_from_string("2-1234".to_string());
        assert!(!(r1 == r2));
        assert!(r1 != r2);
        assert!(r1 <= r2);
        assert!(r1 < r2);
        assert!(!(r2 <= r1));
        assert!(!(r2 < r1));
    }
}
