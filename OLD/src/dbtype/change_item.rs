use serde;
use std;

use Revision;

/// Builder for constructing a change item.
#[derive(Debug)]
pub struct ChangeItemBuilder {
    target: ChangeItem,
}

impl ChangeItemBuilder {
    /// Constructs a new change item builder.
    ///
    /// The change item contained within the builder has the given revision.
    ///
    pub fn new(rev: Revision) -> Self {
        ChangeItemBuilder {
            target: ChangeItem {
                rev: rev,
                _dummy: std::marker::PhantomData,
            },
        }
    }

    /// Constructs a new change item builder.
    ///
    /// The change item contained within the builder is assigned a revision
    /// equivalent to the given revision string.
    ///
    /// # Panics
    ///
    /// Panics if the revision string is invalid.
    ///
    pub fn new_from_rev_str<R: AsRef<str>>(rev: R) -> Self {
        ChangeItemBuilder {
            target: ChangeItem {
                rev: Revision::parse(rev.as_ref())
                         .expect("Cannot build ChangeItem from invalid revision string"),
                _dummy: std::marker::PhantomData,
            },
        }
    }

    /// Returns the change item contained within the builder.
    pub fn unwrap(self) -> ChangeItem {
        self.target
    }
}

/// Document leaf as returned in a change result.
#[derive(Clone, Debug, PartialEq)]
pub struct ChangeItem {
    /// Revision of the document leaf.
    pub rev: Revision,

    _dummy: std::marker::PhantomData<()>,
}

impl ChangeItem {
    #[doc(hidden)]
    pub fn new(rev: Revision) -> Self {
        ChangeItem {
            rev: rev,
            _dummy: std::marker::PhantomData,
        }
    }
}

impl serde::Deserialize for ChangeItem {
    fn deserialize<D>(d: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        enum Field {
            Rev,
        }

        impl serde::Deserialize for Field {
            fn deserialize<D>(d: &mut D) -> Result<Field, D::Error>
                where D: serde::Deserializer
            {
                struct Visitor;

                impl serde::de::Visitor for Visitor {
                    type Value = Field;

                    fn visit_str<E>(&mut self, value: &str) -> Result<Field, E>
                        where E: serde::de::Error
                    {
                        match value {
                            "rev" => Ok(Field::Rev),
                            _ => Err(E::unknown_field(value)),
                        }
                    }
                }

                d.deserialize(Visitor)
            }
        }

        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = ChangeItem;

            fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
                where V: serde::de::MapVisitor
            {
                let mut rev = None;
                loop {
                    match try!(visitor.visit_key()) {
                        Some(Field::Rev) => {
                            rev = Some(try!(visitor.visit_value()));
                        }
                        None => {
                            break;
                        }
                    }
                }

                try!(visitor.end());

                let rev = match rev {
                    Some(x) => x,
                    None => try!(visitor.missing_field("rev")),
                };

                Ok(ChangeItem {
                    rev: rev,
                    _dummy: std::marker::PhantomData,
                })
            }
        }

        static FIELDS: &'static [&'static str] = &["rev"];
        d.deserialize_struct("ChangeItem", FIELDS, Visitor)
    }
}

#[cfg(test)]
mod tests {

    use serde_json;
    use std;

    use Revision;
    use super::ChangeItem;
    use super::ChangeItemBuilder;

    #[test]
    fn builder_new() {
        let expected = ChangeItem {
            rev: Revision::parse("42-1234567890abcdef1234567890abcdef").unwrap(),
            _dummy: std::marker::PhantomData,
        };
        let rev = Revision::parse("42-1234567890abcdef1234567890abcdef").unwrap();
        let got = ChangeItemBuilder::new(rev).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn builder_new_from_rev_str() {
        let expected = ChangeItem {
            rev: Revision::parse("42-1234567890abcdef1234567890abcdef").unwrap(),
            _dummy: std::marker::PhantomData,
        };
        let rev = "42-1234567890abcdef1234567890abcdef";
        let got = ChangeItemBuilder::new_from_rev_str(rev).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn deserialization_ok_with_all_fields() {
        let rev = "42-1234567890abcdef1234567890abcdef";
        let expected = ChangeItemBuilder::new_from_rev_str(rev).unwrap();
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("rev", rev)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn deserialization_nok_with_no_rev_field() {
        let source = serde_json::builder::ObjectBuilder::new().unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<ChangeItem>(&s);
        expect_json_error_missing_field!(got, "rev");
    }
}
