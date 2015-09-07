use hyper;
use serde;
use serde_json;
use std;

use client;
use design::DesignDocument;
use document::Revision;
use error::{self, Error};

/// Command to create a document.
pub struct PutDocument<'a, 'b, T: 'b + serde::Serialize> {
    client_state: &'a client::ClientState,
    uri: hyper::Url,
    doc_content: &'b T,
    if_match: Revision,
}

impl<'a, 'b, T: 'b + serde::Serialize> PutDocument<'a, 'b, T> {

    pub fn new_db_document(client_state: &'a client::ClientState,
                           db_name: &str,
                           doc_id: &str,
                           doc_content: &'b T)
                           -> PutDocument<'a, 'b, T> {
        let mut u = client_state.uri.clone();
        u.path_mut().unwrap()[0] = db_name.to_string();
        u.path_mut().unwrap().push(doc_id.to_string());
        PutDocument {
            client_state: client_state,
            uri: u,
            doc_content: doc_content,
            if_match: Revision::new(),
        }
    }

    /// Set the If-Match header.
    pub fn if_match(mut self, rev: Revision) -> PutDocument<'a, 'b, T> {
        self.if_match = rev;
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

        let req_body = try!(
            serde_json::to_string(&self.doc_content)
            .or_else(|e| {
                Err(Error::Encode { cause: e } )
            })
        );

        let mut resp = {
            use hyper::mime::{Mime, TopLevel, SubLevel};
            let mut req = self.client_state.http_client.put(self.uri)
                .header(hyper::header::Accept(
                        vec![hyper::header::qitem(
                            Mime(TopLevel::Application, SubLevel::Json, vec![]))]))
                .header(hyper::header::ContentType(
                        hyper::mime::Mime(TopLevel::Application, SubLevel::Json, vec![])));
            if !self.if_match.is_empty() {
                req = req.header(hyper::header::IfMatch::Items(
                        vec![hyper::header::EntityTag::new(
                            false, self.if_match.as_str().to_string())]));
            }
            try!(
                req.body(&req_body)
                .send()
                .or_else(|e| {
                    Err(Error::Transport { cause: error::TransportCause::Hyper(e) })
                })
            )
        };

        match resp.status {
            hyper::status::StatusCode::Created => {
                let s = try!(client::read_json_response(&mut resp));
                let mut resp_body = try!(client::decode_json::<serde_json::Value>(&s));
                (|| {
                    let dot = match resp_body.as_object_mut() {
                        None => { return None; },
                        Some(x) => x,
                    };
                    let rev = match dot.get_mut("rev") {
                        None => { return None; },
                        Some(x) => x,
                    };
                    let rev = match *rev {
                        serde_json::Value::String(ref mut x) => std::mem::replace(
                            x, String::new()),
                        _ => { return None; },
                    };
                    let rev = Revision::from_string(rev);
                    Some(rev)
                })()
                .ok_or(Error::UnexpectedContent { got: s } )
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

impl<'a, 'b> PutDocument<'a, 'b, DesignDocument> {
    pub fn new_design_document(client_state: &'a client::ClientState,
                               db_name: &str,
                               ddoc_id: &str,
                               ddoc_content: &'b DesignDocument)
                               -> PutDocument<'a, 'b, DesignDocument> {
        let mut u = client_state.uri.clone();
        u.path_mut().unwrap()[0] = db_name.to_string();
        u.path_mut().unwrap().push("_design".to_string());
        u.path_mut().unwrap().push(ddoc_id.to_string());
        PutDocument {
            client_state: client_state,
            uri: u,
            doc_content: ddoc_content,
            if_match: Revision::new(),
        }
    }
}
