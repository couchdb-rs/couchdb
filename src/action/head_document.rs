use hyper;

use Error;
use IntoDocumentPath;
use Revision;
use client::ClientState;
use action::{self, Action, Request, Response};

/// Action to check whether a document exists.
///
/// # Return
///
/// This action returns an `Option` type. The return value is `None` if the
/// action specifies a revision and the document hasn't been modified since
/// that revision. Otherwise, the return value is `Some`.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this action:
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

    /// Sets the If-None-Match header.
    pub fn if_none_match(mut self, rev: &'a Revision) -> Self {
        self.if_none_match = Some(rev);
        self
    }

    impl_action_public_methods!(Option<()>);
}

impl<'a, P: IntoDocumentPath> Action for HeadDocument<'a, P> {
    type Output = Option<()>;

    fn make_request(self) -> Result<Request, Error> {
        let doc_path = try!(self.path.into_document_path());
        let uri = doc_path.into_uri(self.client_state.uri.clone());
        let request = Request::new(hyper::Head, uri).set_if_none_match_revision(self.if_none_match);
        Ok(request)
    }

    fn take_response<R: Response>(response: R) -> Result<Self::Output, Error> {
        match response.status() {
            hyper::status::StatusCode::Ok => Ok(Some(())),
            hyper::status::StatusCode::NotModified => Ok(None),
            hyper::status::StatusCode::Unauthorized => Err(Error::Unauthorized(None)),
            hyper::status::StatusCode::NotFound => Err(Error::NotFound(None)),
            _ => Err(Error::UnexpectedHttpStatus { got: response.status() }),
        }
    }
}

#[cfg(test)]
mod tests {

    use hyper;

    use DocumentPath;
    use Revision;
    use client::ClientState;
    use action::{Action, NoContentResponse};
    use super::HeadDocument;

    #[test]
    fn make_request_default() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let action = HeadDocument::new(&client_state, "/foo/bar");
        let request = action.make_request().unwrap();
        expect_request_method!(request, hyper::method::Method::Head);
        expect_request_uri!(request, "http://example.com:1234/foo/bar");
    }

    #[test]
    fn make_request_if_none_match() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let rev = Revision::parse("42-1234567890abcdef1234567890abcdef").unwrap();
        let action = HeadDocument::new(&client_state, "/foo/bar").if_none_match(&rev);
        let request = action.make_request().unwrap();
        expect_request_method!(request, hyper::method::Method::Head);
        expect_request_uri!(request, "http://example.com:1234/foo/bar");
        expect_request_if_none_match_revision!(request, rev.to_string().as_ref());
    }

    #[test]
    fn take_response_ok() {
        let response = NoContentResponse::new(hyper::Ok);
        let got = HeadDocument::<DocumentPath>::take_response(response).unwrap();
        assert!(got.is_some());
    }

    #[test]
    fn take_response_not_modified() {
        let response = NoContentResponse::new(hyper::status::StatusCode::NotModified);
        let got = HeadDocument::<DocumentPath>::take_response(response).unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn take_response_not_found() {
        let response = NoContentResponse::new(hyper::NotFound);
        let got = HeadDocument::<DocumentPath>::take_response(response);
        expect_couchdb_error!(got, NotFound);
    }

    #[test]
    fn take_response_unauthorized() {
        let response = NoContentResponse::new(hyper::status::StatusCode::Unauthorized);
        let got = HeadDocument::<DocumentPath>::take_response(response);
        expect_couchdb_error!(got, Unauthorized);
    }
}
