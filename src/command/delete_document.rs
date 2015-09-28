use hyper;
use std;

use client::{self, ClientState};
use document::{self, DocumentType, Revision};
use error::{self, Error};

#[doc(hidden)]
pub fn new_delete_document<'a, D>(
    client_state: &'a ClientState,
    db_name: &'a str,
    doc_id: &'a str,
    rev: &'a Revision)
    -> DeleteDocument<'a, D>
    where D: DocumentType
{
    DeleteDocument::<'a, D> {
        client_state: client_state,
        doc_type: std::marker::PhantomData,
        db_name: db_name,
        doc_id: doc_id,
        rev: rev,
    }
}

/// Command to delete a document.
pub struct DeleteDocument<'a, D>
    where D: DocumentType
{
    client_state: &'a client::ClientState,
    doc_type: std::marker::PhantomData<D>,
    db_name: &'a str,
    doc_id: &'a str,
    rev: &'a Revision,
}

impl<'a, D> DeleteDocument<'a, D> where D: DocumentType {

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

        let mut resp = {
            use hyper::mime::{Mime, TopLevel, SubLevel};
            let uri = document::new_uri::<D>(
                &self.client_state.uri,
                self.db_name,
                self.doc_id);
            let mut req = self.client_state.http_client.delete(uri)
                    .header(hyper::header::Accept(vec![
                        hyper::header::qitem(
                            Mime(TopLevel::Application, SubLevel::Json, vec![]))]));
            req = req.header(
                hyper::header::IfMatch::Items(
                    vec![
                        hyper::header::EntityTag::new(
                            false,
                            self.rev.to_string())
                    ]
                )
            );
            try!(
                req.send()
                .or_else(|e| {
                    Err(Error::Transport { cause: error::TransportCause::Hyper(e) })
                })
            )
        };

        match resp.status {
            hyper::status::StatusCode::Ok =>
                Ok(try!(client::require_content_type_application_json(&resp.headers))),
            hyper::status::StatusCode::BadRequest =>
                Err(error::new_because_invalid_request(&mut resp)),
            hyper::status::StatusCode::Unauthorized =>
                Err(error::new_because_unauthorized(&mut resp)),
            hyper::status::StatusCode::NotFound =>
                Err(error::new_because_not_found(&mut resp)),
            hyper::status::StatusCode::Conflict =>
                Err(error::new_because_document_conflict(&mut resp)),
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status } ),
        }
    }
}

