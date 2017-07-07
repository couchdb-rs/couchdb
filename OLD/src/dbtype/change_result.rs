use serde;
use std;

use ChangeItem;
use ChangeItemBuilder;
use DocumentId;
use Revision;

/// Builder for constructing a change result.
#[derive(Debug)]
pub struct ChangeResultBuilder {
    target: ChangeResult,
}

impl ChangeResultBuilder {
    /// Constructs a new change result builder.
    ///
    /// The change result contained within the builder has the given sequence
    /// number and document id. The change result's `deleted` field is `false`,
    /// and the `changes` field is empty.
    ///
    pub fn new<I: Into<DocumentId>>(seq: u64, doc_id: I) -> Self {
        ChangeResultBuilder {
            target: ChangeResult {
                seq: seq,
                id: doc_id.into(),
                changes: Vec::new(),
                deleted: false,
                _dummy: std::marker::PhantomData,
            },
        }
    }

    /// Returns the change result contained within the builder.
    pub fn unwrap(self) -> ChangeResult {
        self.target
    }

    /// Sets the `deleted` field of the change result contained within the
    /// builder.
    pub fn deleted(mut self, deleted: bool) -> Self {
        self.target.deleted = deleted;
        self
    }

    /// Adds a change item to the change result contained within the builder.
    pub fn push_change(mut self, change: ChangeItem) -> Self {
        self.target.changes.push(change);
        self
    }

    /// Constructs and adds a change item to the change result contained within
    /// the builder.
    pub fn build_change<F>(mut self, rev: Revision, f: F) -> Self
        where F: FnOnce(ChangeItemBuilder) -> ChangeItemBuilder
    {
        let builder = ChangeItemBuilder::new(rev);
        self.target.changes.push(f(builder).unwrap());
        self
    }

    /// Constructs and adds a change item to the change result contained within
    /// the builder.
    ///
    /// # Panics
    ///
    /// Panics if the revision string is invalid.
    ///
    pub fn build_change_from_rev_str<R, F>(mut self, rev: R, f: F) -> Self
        where R: AsRef<str>,
              F: FnOnce(ChangeItemBuilder) -> ChangeItemBuilder
    {
        let builder = ChangeItemBuilder::new_from_rev_str(rev);
        self.target.changes.push(f(builder).unwrap());
        self
    }
}

/// Single element as returned in a change list.
///
/// A change result represents one document that has changed.
///
#[derive(Clone, Debug, PartialEq)]
pub struct ChangeResult {
    /// Sequence number for the change.
    pub seq: u64,

    /// Id of the changed document.
    pub id: DocumentId,

    /// Leafs of the changed document.
    pub changes: Vec<ChangeItem>,

    /// Whether the changed document has been deleted.
    pub deleted: bool,
    _dummy: std::marker::PhantomData<()>,
}

impl ChangeResult {
    #[doc(hidden)]
    pub fn new(seq: u64, id: DocumentId, changes: Vec<ChangeItem>, deleted: bool) -> Self {
        ChangeResult {
            seq: seq,
            id: id,
            changes: changes,
            deleted: deleted,
            _dummy: std::marker::PhantomData,
        }
    }
}

impl serde::Deserialize for ChangeResult {
    fn deserialize<D>(d: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        enum Field {
            Changes,
            Deleted,
            Id,
            Seq,
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
                            "changes" => Ok(Field::Changes),
                            "deleted" => Ok(Field::Deleted),
                            "id" => Ok(Field::Id),
                            "seq" => Ok(Field::Seq),
                            _ => Err(E::unknown_field(value)),
                        }
                    }
                }

                d.deserialize(Visitor)
            }
        }

        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = ChangeResult;

            fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
                where V: serde::de::MapVisitor
            {
                let mut changes = None;
                let mut deleted = None;
                let mut id = None;
                let mut seq = None;
                loop {
                    match try!(visitor.visit_key()) {
                        Some(Field::Changes) => {
                            changes = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Deleted) => {
                            deleted = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Id) => {
                            id = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Seq) => {
                            seq = Some(try!(visitor.visit_value()));
                        }
                        None => {
                            break;
                        }
                    }
                }

                try!(visitor.end());

                let changes = match changes {
                    Some(x) => x,
                    None => try!(visitor.missing_field("changes")),
                };

                let deleted = deleted.unwrap_or(false);

                let id = match id {
                    Some(x) => x,
                    None => try!(visitor.missing_field("id")),
                };

                let seq = match seq {
                    Some(x) => x,
                    None => try!(visitor.missing_field("seq")),
                };

                Ok(ChangeResult {
                    changes: changes,
                    deleted: deleted,
                    id: id,
                    seq: seq,
                    _dummy: std::marker::PhantomData,
                })
            }
        }

        static FIELDS: &'static [&'static str] = &["changes", "deleted", "id", "seq"];
        d.deserialize_struct("ChangeResult", FIELDS, Visitor)
    }
}

#[cfg(test)]
mod tests {

    use serde_json;
    use std;

    use ChangeItemBuilder;
    use Revision;
    use super::ChangeResult;
    use super::ChangeResultBuilder;

    #[test]
    fn builder_default() {
        let expected = ChangeResult {
            seq: 42,
            id: "foo".into(),
            deleted: false,
            changes: Vec::new(),
            _dummy: std::marker::PhantomData,
        };
        let got = ChangeResultBuilder::new(42, "foo").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn builder_deleted() {
        let expected = ChangeResult {
            seq: 42,
            id: "foo".into(),
            deleted: true,
            changes: Vec::new(),
            _dummy: std::marker::PhantomData,
        };
        let got = ChangeResultBuilder::new(42, "foo").deleted(true).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn builder_push_change() {
        let c1 = ChangeItemBuilder::new_from_rev_str("2-7051cbe5c8faecd085a3fa619e6e6337").unwrap();
        let c2 = ChangeItemBuilder::new_from_rev_str("3-7379b9e515b161226c6559d90c4dc49f").unwrap();
        let expected = ChangeResult {
            seq: 42,
            id: "foo".into(),
            deleted: false,
            changes: vec![
                c1.clone(),
                c2.clone(),
            ],
            _dummy: std::marker::PhantomData,
        };
        let got = ChangeResultBuilder::new(42, "foo")
                      .push_change(c1)
                      .push_change(c2)
                      .unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn builder_build_change() {
        let expected = ChangeResult {
            seq: 42,
            id: "foo".into(),
            deleted: false,
            changes: vec![
                ChangeItemBuilder::new_from_rev_str("2-7051cbe5c8faecd085a3fa619e6e6337").unwrap(),
                ChangeItemBuilder::new_from_rev_str("3-7379b9e515b161226c6559d90c4dc49f").unwrap(),
            ],
            _dummy: std::marker::PhantomData,
        };
        let got = ChangeResultBuilder::new(42, "foo")
                      .build_change(Revision::parse("2-7051cbe5c8faecd085a3fa619e6e6337").unwrap(),
                                    |x| x)
                      .build_change(Revision::parse("3-7379b9e515b161226c6559d90c4dc49f").unwrap(),
                                    |x| x)
                      .unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn builder_build_change_from_str_ref() {
        let expected = ChangeResult {
            seq: 42,
            id: "foo".into(),
            deleted: false,
            changes: vec![
                ChangeItemBuilder::new_from_rev_str("2-7051cbe5c8faecd085a3fa619e6e6337").unwrap(),
                ChangeItemBuilder::new_from_rev_str("3-7379b9e515b161226c6559d90c4dc49f").unwrap(),
            ],
            _dummy: std::marker::PhantomData,
        };
        let got = ChangeResultBuilder::new(42, "foo")
                      .build_change_from_rev_str("2-7051cbe5c8faecd085a3fa619e6e6337", |x| x)
                      .build_change_from_rev_str("3-7379b9e515b161226c6559d90c4dc49f", |x| x)
                      .unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn deserialization_ok_with_deleted_field() {
        let expected = ChangeResultBuilder::new(9, "5bbc9ca465f1b0fcd62362168a7c8831")
                           .build_change_from_rev_str("3-7379b9e515b161226c6559d90c4dc49f", |x| x)
                           .deleted(true)
                           .unwrap();
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert_array("changes", |x| {
                             x.push_object(|x| {
                                 x.insert("rev", "3-7379b9e515b161226c6559d90c4dc49f")
                             })
                         })
                         .insert("deleted", true)
                         .insert("id", "5bbc9ca465f1b0fcd62362168a7c8831")
                         .insert("seq", 9)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn deserialization_ok_without_deleted_field() {
        let expected = ChangeResultBuilder::new(6, "6478c2ae800dfc387396d14e1fc39626")
                           .build_change_from_rev_str("2-7051cbe5c8faecd085a3fa619e6e6337", |x| x)
                           .unwrap();
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert_array("changes", |x| {
                             x.push_object(|x| {
                                 x.insert("rev", "2-7051cbe5c8faecd085a3fa619e6e6337")
                             })
                         })
                         .insert("id", "6478c2ae800dfc387396d14e1fc39626")
                         .insert("seq", 6)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn deserialization_nok_without_changes_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("deleted", true)
                         .insert("id", "5bbc9ca465f1b0fcd62362168a7c8831")
                         .insert("seq", 9)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<ChangeResult>(&s);
        expect_json_error_missing_field!(got, "changes");
    }

    #[test]
    fn deserialization_nok_without_id_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert_array("changes", |x| {
                             x.push_object(|x| {
                                 x.insert("rev", "3-7379b9e515b161226c6559d90c4dc49f")
                             })
                         })
                         .insert("deleted", true)
                         .insert("seq", 9)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<ChangeResult>(&s);
        expect_json_error_missing_field!(got, "id");
    }

    #[test]
    fn deserialization_nok_without_seq_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert_array("changes", |x| {
                             x.push_object(|x| {
                                 x.insert("rev", "3-7379b9e515b161226c6559d90c4dc49f")
                             })
                         })
                         .insert("deleted", true)
                         .insert("id", "5bbc9ca465f1b0fcd62362168a7c8831")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<ChangeResult>(&s);
        expect_json_error_missing_field!(got, "seq");
    }
}
