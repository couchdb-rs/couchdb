use serde;

use docpath::DocumentId;
use revision::Revision;

/// Document, including meta-information and content.
#[derive(Debug)]
pub struct Document<T: serde::Deserialize> {
    pub id: DocumentId, // FIXME: This should be DocumentPathBuf.
    pub revision: Revision,
    pub content: T,
}
