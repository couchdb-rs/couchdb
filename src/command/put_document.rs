use hyper;
use serde;
use serde_json;

use client::{self, ClientState};
use dbtype::PutDocumentResponse;
use docpath::DocumentPath;
use error::{self, Error};
use revision::Revision;
use transport::{self, Command, Request};

/// Command to create a document.
pub struct PutDocument<'a, T> where T: 'a + serde::Serialize
{
    client_state: &'a ClientState,
    path: DocumentPath,
    doc_content: &'a T,
    if_match: Option<&'a Revision>,
}

impl<'a, T> PutDocument<'a, T> where T: 'a + serde::Serialize
{
    #[doc(hidden)]
    pub fn new_put_document(
        client_state: &'a ClientState,
        path: DocumentPath,
        doc_content: &'a T)
        -> Self
        where T: serde::Serialize
    {
        PutDocument {
            client_state: client_state,
            path: path,
            doc_content: doc_content,
            if_match: None,
        }
    }

    /// Set the If-Match header.
    pub fn if_match(mut self, rev: &'a Revision) -> Self {
        self.if_match = Some(rev);
        self
    }

    /// Send the command request and wait for the response.
    ///
    /// # Return
    ///
    /// Return the new revision for the document.
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
    pub fn run(self) -> Result<Revision, Error> {
        transport::run_command(self)
    }
}

impl<'a, T> Command for PutDocument<'a, T> where T: 'a + serde::Serialize
{
    type Output = Revision;
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let uri = self.path.into_uri(self.client_state.uri.clone());
        let body = try!(
            serde_json::to_vec(self.doc_content)
                .map_err(|e| {
                    Error::Encode { cause: e }
                })
        );
        let req = try!(Request::new(hyper::method::Method::Put, uri))
            .accept_application_json()
            .content_type_application_json()
            .if_match_revision(self.if_match)
            .body(body);
        Ok((req, ()))
    }

    fn take_response(mut resp: hyper::client::Response, _state: Self::State)
        -> Result<Self::Output, Error>
    {
        match resp.status {
            hyper::status::StatusCode::Created => {
                let s = try!(client::read_json_response(&mut resp));
                let resp = try!(client::decode_json::<PutDocumentResponse>(&s));
                let rev: Revision = resp.rev.into();
                Ok(rev)
            },
            hyper::status::StatusCode::BadRequest =>
                Err(error::new_because_invalid_request(&mut resp)),
            hyper::status::StatusCode::Unauthorized =>
                Err(error::new_because_unauthorized(&mut resp)),
            hyper::status::StatusCode::NotFound =>
                Err(error::new_because_not_found(&mut resp)),
            hyper::status::StatusCode::Conflict =>
                Err(error::new_because_document_conflict(&mut resp)),
            _ => Err(Error::UnexpectedHttpStatus{ got: resp.status } ),
        }
    }
}
