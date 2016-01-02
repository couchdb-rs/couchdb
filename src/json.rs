use serde;
use serde_json;
use std;

use Error;
use error::{DecodeErrorKind, TransportKind};

// Decodes JSON from a reader, returning the appropriate error variant in case
// of error.
pub fn decode_json<R, T>(r: R) -> Result<T, Error>
    where R: std::io::Read,
          T: serde::Deserialize
{
    serde_json::from_reader::<_, T>(r).map_err(|e| {
        match e {
            serde_json::Error::IoError(e) => Error::Transport(TransportKind::Io(e)),
            _ => Error::Decode(DecodeErrorKind::Serde { cause: e }),
        }
    })
}
