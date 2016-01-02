use hyper;
use serde;
use serde_json;

use DocumentId;
use Error;
use ErrorResponse;
use IntoDatabasePath;
use Revision;
use client::ClientState;
use command::{self, Command, Request};
use dbtype::PostToDatabaseResponse;
use error::EncodeErrorKind;
use json;

/// Command to create a document.
///
/// # Return
///
/// This command returns the new document's revision and id.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this command:
///
/// * `Error::NotFound`: The database does not exist.
/// * `Error::Unauthorized`: The client is unauthorized.
///
pub struct PostToDatabase<'a, P, T>
    where P: IntoDatabasePath,
          T: 'a + serde::Serialize
{
    client_state: &'a ClientState,
    path: P,
    doc_content: &'a T,
}

impl<'a, P: IntoDatabasePath, T: 'a + serde::Serialize> PostToDatabase<'a, P, T> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState, path: P, doc_content: &'a T) -> Self
        where T: serde::Serialize
    {
        PostToDatabase {
            client_state: client_state,
            path: path,
            doc_content: doc_content,
        }
    }

    impl_command_public_methods!((Revision, DocumentId));
}

impl<'a, P: IntoDatabasePath, T: 'a + serde::Serialize> Command for PostToDatabase<'a, P, T> {
    type Output = (Revision, DocumentId);
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let db_path = try!(self.path.into_database_path());
        let uri = db_path.into_uri(self.client_state.uri.clone());
        let body = try!(serde_json::to_vec(self.doc_content)
                            .map_err(|e| Error::Encode(EncodeErrorKind::Serde { cause: e })));
        let req = try!(Request::new(hyper::method::Method::Post, uri))
                      .accept_application_json()
                      .content_type_application_json()
                      .body(body);
        Ok((req, ()))
    }

    fn take_response(resp: hyper::client::Response,
                     _state: Self::State)
                     -> Result<Self::Output, Error> {
        match resp.status {
            hyper::status::StatusCode::Created => {
                try!(command::content_type_must_be_application_json(&resp.headers));
                let content = try!(json::decode_json::<_, PostToDatabaseResponse>(resp));
                let id = DocumentId::from(String::from(content.id));
                let rev: Revision = content.rev.into();
                Ok((rev, id))
            }
            hyper::status::StatusCode::BadRequest => {
                Err(Error::BadRequest(try!(json::decode_json::<_, ErrorResponse>(resp))))
            }
            hyper::status::StatusCode::Unauthorized => {
                Err(Error::Unauthorized(Some(try!(json::decode_json::<_, ErrorResponse>(resp)))))
            }
            hyper::status::StatusCode::NotFound => {
                Err(Error::NotFound(Some(try!(json::decode_json::<_, ErrorResponse>(resp)))))
            }
            hyper::status::StatusCode::Conflict => {
                // Need to include this error variant in the command's
                // documentation if we ever add support for an explicit id.
                Err(Error::DocumentConflict(try!(json::decode_json::<_, ErrorResponse>(resp))))
            }
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status }),
        }
    }
}
