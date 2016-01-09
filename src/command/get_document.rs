use hyper;

use Document;
use Error;
use ErrorResponse;
use IntoDocumentPath;
use Revision;
use client::ClientState;
use command::{self, Command, Request};
use json;

/// Command to get document meta-information and application-defined content.
///
/// # Return
///
/// This command returns an `Option` type. The return value is `None` if the
/// command specifies a revision and the document hasn't been modified since
/// that revision. Otherwise, the return value is `Some` and contains the
/// document meta-information and application-defined content.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this command:
///
///
/// * `Error::NotFound`: The document does not exist.
/// * `Error::Unauthorized`: The client is unauthorized.
///
pub struct GetDocument<'a, P>
    where P: IntoDocumentPath
{
    client_state: &'a ClientState,
    path: P,
    if_none_match: Option<&'a Revision>,
}

impl<'a, P: IntoDocumentPath> GetDocument<'a, P> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState, path: P) -> Self {
        GetDocument {
            client_state: client_state,
            path: path,
            if_none_match: None,
        }
    }

    /// Sets the If-None-Match header.
    pub fn if_none_match(mut self, rev: &'a Revision) -> Self {
        self.if_none_match = Some(rev);
        self
    }

    impl_command_public_methods!(Option<Document>);
}

impl<'a, P: IntoDocumentPath> Command for GetDocument<'a, P> {
    type Output = Option<Document>;
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let doc_path = try!(self.path.into_document_path());
        let uri = doc_path.into_uri(self.client_state.uri.clone());
        let req = try!(Request::new(hyper::Get, uri))
                      .accept_application_json()
                      .if_none_match_revision(self.if_none_match);
        Ok((req, ()))
    }

    fn take_response(resp: hyper::client::Response,
                     _state: Self::State)
                     -> Result<Self::Output, Error> {
        match resp.status {
            hyper::status::StatusCode::Ok => {
                try!(command::content_type_must_be_application_json(&resp.headers));
                let doc = try!(json::decode_json::<_, Document>(resp));
                Ok(Some(doc))
            }
            hyper::status::StatusCode::NotModified => Ok(None),
            hyper::status::StatusCode::BadRequest => Err(make_couchdb_error!(BadRequest, resp)),
            hyper::status::StatusCode::Unauthorized => Err(make_couchdb_error!(Unauthorized, resp)),
            hyper::status::StatusCode::NotFound => Err(make_couchdb_error!(NotFound, resp)),
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status }),
        }
    }
}
