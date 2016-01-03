use hyper;

use Error;
use ErrorResponse;
use IntoDocumentPath;
use Revision;
use client::{self, ClientState};
use command::{self, Command, Request};
use json;

/// Command to delete a document.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this command:
///
/// * `Error::DocumentConflict`: The revision is not the latest for the
///   document.
/// * `Error::NotFound`: The document does not exist.
/// * `Error::Unauthorized`: The client is unauthorized.
///
pub struct DeleteDocument<'a, P: IntoDocumentPath> {
    client_state: &'a client::ClientState,
    path: P,
    rev: &'a Revision,
}

impl<'a, P: IntoDocumentPath> DeleteDocument<'a, P> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState, path: P, rev: &'a Revision) -> Self {
        DeleteDocument {
            client_state: client_state,
            path: path,
            rev: rev,
        }
    }

    impl_command_public_methods!(());
}

impl<'a, P: IntoDocumentPath> Command for DeleteDocument<'a, P> {
    type Output = ();
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let doc_path = try!(self.path.into_document_path());
        let uri = doc_path.into_uri(self.client_state.uri.clone());
        let req = try!(Request::new(hyper::Delete, uri))
                      .accept_application_json()
                      .if_match_revision(Some(self.rev));
        Ok((req, ()))
    }

    fn take_response(resp: hyper::client::Response,
                     _state: Self::State)
                     -> Result<Self::Output, Error> {
        match resp.status {
            hyper::status::StatusCode::Ok => {
                command::content_type_must_be_application_json(&resp.headers)
            }
            hyper::status::StatusCode::BadRequest => Err(make_couchdb_error!(BadRequest, resp)),
            hyper::status::StatusCode::Unauthorized => Err(make_couchdb_error!(Unauthorized, resp)),
            hyper::status::StatusCode::NotFound => Err(make_couchdb_error!(NotFound, resp)),
            hyper::status::StatusCode::Conflict => Err(make_couchdb_error!(DocumentConflict, resp)),
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status }),
        }
    }
}
