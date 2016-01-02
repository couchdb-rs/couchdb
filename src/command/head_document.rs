use hyper;

use Error;
use IntoDocumentPath;
use Revision;
use client::ClientState;
use command::{self, Command, Request};

/// Command to get document meta-information.
///
/// # Return
///
/// This command returns an `Option` type. The return value is `None` if the
/// command specifies a revision and the document hasn't been modified since
/// that revision. Otherwise, the return value is `Some`.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this command:
///
/// * `Error::NotFound`: The document does not exist.
/// * `Error::Unauthorized`: The client is unauthorized.
///
pub struct HeadDocument<'a, P>
    where P: IntoDocumentPath
{
    client_state: &'a ClientState,
    path: P,
    if_none_match: Option<&'a Revision>,
}

impl<'a, P: IntoDocumentPath> HeadDocument<'a, P> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState, path: P) -> Self {
        HeadDocument {
            client_state: client_state,
            path: path,
            if_none_match: None,
        }
    }

    /// Set the If-None-Match header.
    pub fn if_none_match(mut self, rev: &'a Revision) -> Self {
        self.if_none_match = Some(rev);
        self
    }

    impl_command_public_methods!(Option<()>);
}

impl<'a, P: IntoDocumentPath> Command for HeadDocument<'a, P> {
    type Output = Option<()>;
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let doc_path = try!(self.path.into_document_path());
        let uri = doc_path.into_uri(self.client_state.uri.clone());
        let req = try!(Request::new(hyper::Head, uri)).if_none_match_revision(self.if_none_match);
        Ok((req, ()))
    }

    fn take_response(resp: hyper::client::Response,
                     _state: Self::State)
                     -> Result<Self::Output, Error> {
        match resp.status {
            hyper::status::StatusCode::Ok => Ok(Some(())),
            hyper::status::StatusCode::NotModified => Ok(None),
            hyper::status::StatusCode::Unauthorized => Err(Error::Unauthorized(None)),
            hyper::status::StatusCode::NotFound => Err(Error::NotFound(None)),
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status }),
        }
    }
}
