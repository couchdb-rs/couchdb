use std;

/// A “since” value specifies an update sequence number of a database.
///
/// Applications may use `Since` to limit which change events the CouchDB server
/// returns to the application when retrieving database changes.
///
/// # Examples
///
/// Applications may construct a `Since` directly from a number and convert a
/// `Since` to a string.
///
/// ```
/// use couchdb::Since;
/// let x: Since = 42.into();
/// assert_eq!("42", x.to_string());
/// ```
///
#[derive(Debug, Eq, PartialEq)]
pub enum Since {
    /// A literal sequence number.
    SequenceNumber(u64),

    /// The `now` value.
    Now,
}

impl std::fmt::Display for Since {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            &Since::SequenceNumber(x) => x.fmt(f),
            &Since::Now => write!(f, "now"),
        }
    }
}

impl From<u64> for Since {
    fn from(seq: u64) -> Self {
        Since::SequenceNumber(seq)
    }
}

#[cfg(test)]
mod tests {

    use super::Since;

    #[test]
    fn display() {
        assert_eq!("42", format!("{}", Since::SequenceNumber(42)));
        assert_eq!("now", format!("{}", Since::Now));
    }

    #[test]
    fn eq() {
        let a = Since::SequenceNumber(42);
        let b = Since::SequenceNumber(42);
        assert!(a == b);

        let a = Since::SequenceNumber(17);
        let b = Since::SequenceNumber(42);
        assert!(a != b);

        let a = Since::Now;
        let b = Since::SequenceNumber(42);
        assert!(a != b);

        let a = Since::Now;
        let b = Since::Now;
        assert!(a == b);
    }

    #[test]
    fn from_number() {
        let expected = Since::SequenceNumber(42);
        let got = Since::from(42);
        assert_eq!(expected, got);
    }
}
