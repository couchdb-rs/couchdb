use hyper;
use serde;
use std;

use Document;
use Error;
use ErrorResponse;
use IntoDocumentPath;
use Revision;
use client::ClientState;
use command::{self, Command, Request};
use json;

/// Command to get a document.
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
pub struct GetDocument<'a, P, T>
    where P: IntoDocumentPath,
          T: serde::Deserialize
{
    client_state: &'a ClientState,
    path: P,
    if_none_match: Option<&'a Revision>,
    _content_type: std::marker::PhantomData<T>,
}

impl<'a, P: IntoDocumentPath, T: serde::Deserialize> GetDocument<'a, P, T> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState, path: P) -> Self {
        GetDocument {
            client_state: client_state,
            path: path,
            if_none_match: None,
            _content_type: std::marker::PhantomData,
        }
    }

    /// Set the If-None-Match header.
    pub fn if_none_match(mut self, rev: &'a Revision) -> Self {
        self.if_none_match = Some(rev);
        self
    }

    impl_command_public_methods!(Option<Document<T>>);
}

impl<'a, P: IntoDocumentPath, T: serde::Deserialize> Command for GetDocument<'a, P, T> {
    type Output = Option<Document<T>>;
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
                let doc = try!(Document::from_reader(resp));
                Ok(Some(doc))
            }
            hyper::status::StatusCode::NotModified => Ok(None),
            hyper::status::StatusCode::BadRequest => {
                Err(Error::BadRequest(try!(json::decode_json::<_, ErrorResponse>(resp))))
            }
            hyper::status::StatusCode::Unauthorized => {
                Err(Error::Unauthorized(Some(try!(json::decode_json::<_, ErrorResponse>(resp)))))
            }
            hyper::status::StatusCode::NotFound => {
                Err(Error::NotFound(Some(try!(json::decode_json::<_, ErrorResponse>(resp)))))
            }
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status }),
        }
    }
}
