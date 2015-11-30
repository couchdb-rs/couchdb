use serde;

use docid::DocumentId;
use revision::Revision;

/// Document, including meta-information and content.
#[derive(Debug)]
pub struct Document<T: serde::Deserialize> {
    pub id: DocumentId,
    pub revision: Revision,
    pub content: T,
}
