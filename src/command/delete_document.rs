use hyper;

use client::{self, ClientState};
use docpath::DocumentPath;
use error::{Error, ErrorResponse};
use revision::Revision;
use transport::{self, Command, Request};

/// Command to delete a document.
pub struct DeleteDocument<'a> {
    client_state: &'a client::ClientState,
    path: DocumentPath,
    rev: &'a Revision,
}

impl<'a> DeleteDocument<'a> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState, path: DocumentPath, rev: &'a Revision) -> Self {
        DeleteDocument {
            client_state: client_state,
            path: path,
            rev: rev,
        }
    }

    /// Send the command request and wait for the response.
    ///
    /// # Errors
    ///
    /// Note: Other errors may occur.
    ///
    /// * `Error::DocumentConflict`: The revision is not the latest for the
    ///   document.
    /// * `Error::NotFound`: The document does not exist.
    /// * `Error::Unauthorized`: The client is unauthorized.
    ///
    pub fn run(self) -> Result<(), Error> {
        transport::run_command(self)
    }
}

impl<'a> Command for DeleteDocument<'a> {
    type Output = ();
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let uri = self.path.into_uri(self.client_state.uri.clone());
        let req = try!(Request::new(hyper::Delete, uri))
                      .accept_application_json()
                      .if_match_revision(Some(self.rev));
        Ok((req, ()))
    }

    fn take_response(resp: hyper::client::Response, _state: Self::State) -> Result<Self::Output, Error> {
        match resp.status {
            hyper::status::StatusCode::Ok => transport::content_type_must_be_application_json(&resp.headers),
            hyper::status::StatusCode::BadRequest => Err(Error::BadRequest(try!(ErrorResponse::from_reader(resp)))),
            hyper::status::StatusCode::Unauthorized => Err(Error::Unauthorized(try!(ErrorResponse::from_reader(resp)))),
            hyper::status::StatusCode::NotFound => Err(Error::NotFound(Some(try!(ErrorResponse::from_reader(resp))))),
            hyper::status::StatusCode::Conflict => Err(Error::DocumentConflict(try!(ErrorResponse::from_reader(resp)))),
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status }),
        }
    }
}
