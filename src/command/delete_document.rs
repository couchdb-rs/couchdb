use hyper;

use client;
use document;
use error::{self, Error};

/// Command to delete a document.
pub struct DeleteDocument<'a> {
    client_state: &'a client::ClientState,
    uri: hyper::Url,
    rev: &'a document::Revision,
}

impl<'a> DeleteDocument<'a> {

    pub fn new_db_document(
        client_state: &'a client::ClientState,
        db_name: &str,
        doc_id: &str,
        rev: &'a document::Revision)
        -> DeleteDocument<'a>
    {
        let mut u = client_state.uri.clone();
        u.path_mut().unwrap()[0] = db_name.to_string();
        u.path_mut().unwrap().push(doc_id.to_string());
        DeleteDocument {
            client_state: client_state,
            uri: u,
            rev: rev,
        }
    }

    pub fn new_design_document(
        client_state: &'a client::ClientState,
        db_name: &str,
        ddoc_id: &str,
        rev: &'a document::Revision)
        -> DeleteDocument<'a>
    {
        let mut u = client_state.uri.clone();
        u.path_mut().unwrap()[0] = db_name.to_string();
        u.path_mut().unwrap().push("_design".to_string());
        u.path_mut().unwrap().push(ddoc_id.to_string());
        DeleteDocument {
            client_state: client_state,
            uri: u,
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

        let mut resp = {
            use hyper::mime::{Mime, TopLevel, SubLevel};
            let mut req = self.client_state.http_client.delete(self.uri)
                    .header(hyper::header::Accept(vec![
                        hyper::header::qitem(
                            Mime(TopLevel::Application, SubLevel::Json, vec![]))]));
            req = req.header(
                hyper::header::IfMatch::Items(
                    vec![
                        hyper::header::EntityTag::new(
                            false,
                            self.rev.as_str().to_string())
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

