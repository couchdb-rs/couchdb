use hyper;
use serde;
use serde_json;

use Error;
use ErrorResponse;
use IntoDocumentPath;
use Revision;
use client::ClientState;
use command::{self, Command, Request};
use dbtype::PutDocumentResponse;
use error::EncodeErrorKind;
use json;

/// Command to create or update a document.
///
/// # Return
///
/// This command returns the document's new revision.
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
pub struct PutDocument<'a, P: IntoDocumentPath, T: 'a + serde::Serialize> {
    client_state: &'a ClientState,
    path: P,
    doc_content: &'a T,
    if_match: Option<&'a Revision>,
}

impl<'a, P: IntoDocumentPath, T: 'a + serde::Serialize> PutDocument<'a, P, T> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState, path: P, doc_content: &'a T) -> Self {
        PutDocument {
            client_state: client_state,
            path: path,
            doc_content: doc_content,
            if_match: None,
        }
    }

    /// Sets the If-Match header.
    pub fn if_match(mut self, rev: &'a Revision) -> Self {
        self.if_match = Some(rev);
        self
    }

    impl_command_public_methods!(Revision);
}

impl<'a, P: IntoDocumentPath, T: 'a + serde::Serialize> Command for PutDocument<'a, P, T> {
    type Output = Revision;
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let doc_path = try!(self.path.into_document_path());
        let uri = doc_path.into_uri(self.client_state.uri.clone());
        let body = try!(serde_json::to_vec(self.doc_content)
                            .map_err(|e| Error::Encode(EncodeErrorKind::Serde { cause: e })));
        let req = try!(Request::new(hyper::method::Method::Put, uri))
                      .accept_application_json()
                      .content_type_application_json()
                      .if_match_revision(self.if_match)
                      .body(body);
        Ok((req, ()))
    }

    fn take_response(resp: hyper::client::Response,
                     _state: Self::State)
                     -> Result<Self::Output, Error> {
        match resp.status {
            hyper::status::StatusCode::Created => {
                try!(command::content_type_must_be_application_json(&resp.headers));
                let content = try!(json::decode_json::<_, PutDocumentResponse>(resp));
                let rev: Revision = content.rev.into();
                Ok(rev)
            }
            hyper::status::StatusCode::BadRequest => Err(make_couchdb_error!(BadRequest, resp)),
            hyper::status::StatusCode::Unauthorized => Err(make_couchdb_error!(Unauthorized, resp)),
            hyper::status::StatusCode::NotFound => Err(make_couchdb_error!(NotFound, resp)),
            hyper::status::StatusCode::Conflict => Err(make_couchdb_error!(DocumentConflict, resp)),
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status }),
        }
    }
}
