use hyper;

use client::ClientState;
use docpath::DocumentPath;
use error::{self, Error};
use revision::Revision;
use transport::{self, Command, Request};

/// Command to get document meta-information.
pub struct HeadDocument<'a>
{
    client_state: &'a ClientState,
    path: DocumentPath,
    if_none_match: Option<&'a Revision>,
}

impl<'a> HeadDocument<'a>
{
    #[doc(hidden)]
    pub fn new_head_document(client_state: &'a ClientState, path: DocumentPath)
        -> Self
    {
        HeadDocument {
            client_state: client_state,
            path: path,
            if_none_match: None,
        }
    }

    /// Set the If-None-Match header.
    pub fn if_none_match(mut self, rev: &'a Revision) -> Self
    {
        self.if_none_match = Some(rev);
        self
    }

    /// Send the command request and wait for the response.
    ///
    /// # Return
    ///
    /// Return `None` if an If-None-Match revision is given and the document
    /// hasn't been modified since that revision. Otherwise, return `Some`.
    ///
    /// # Errors
    ///
    /// Note: Other errors may occur.
    ///
    /// * `Error::NotFound`: The document does not exist.
    /// * `Error::Unauthorized`: The client is unauthorized.
    ///
    pub fn run(self) -> Result<Option<()>, Error> {
        transport::run_command(self)
    }
}

impl<'a> Command for HeadDocument<'a>
{
    type Output = Option<()>;
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let uri = self.path.into_uri(self.client_state.uri.clone());
        let req = try!(Request::new(hyper::Head, uri))
            .if_none_match_revision(self.if_none_match);
        Ok((req, ()))
    }

    fn take_response(mut resp: hyper::client::Response, _state: Self::State)
        -> Result<Self::Output, Error>
    {
        match resp.status {
            hyper::status::StatusCode::Ok => Ok(Some(())),
            hyper::status::StatusCode::NotModified => Ok(None),
            hyper::status::StatusCode::Unauthorized =>
                Err(error::new_because_unauthorized(&mut resp)),
            hyper::status::StatusCode::NotFound =>
                Err(Error::NotFound { response: None } ),
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status } ),
        }
    }
}
