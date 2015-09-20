use client;
use hyper;

use document::Revision;
use error::{self, Error};

/// Command to get document meta-information.
pub struct HeadDocument<'a> {
    client_state: &'a client::ClientState,
    uri: hyper::Url,
    if_none_match: Option<&'a Revision>,
}

impl<'a> HeadDocument<'a> {

    pub fn new_db_document(
        client_state: &'a client::ClientState,
        db_name: &str,
        doc_id: &str)
        -> HeadDocument<'a>
    {
        let mut u = client_state.uri.clone();
        u.path_mut().unwrap()[0] = db_name.to_string();
        u.path_mut().unwrap().push(doc_id.to_string());
        HeadDocument {
            client_state: client_state,
            uri: u,
            if_none_match: None,
        }
    }

    pub fn new_design_document(
        client_state: &'a client::ClientState,
        db_name: &str,
        ddoc_id: &str)
        -> HeadDocument<'a>
    {
        let mut u = client_state.uri.clone();
        u.path_mut().unwrap()[0] = db_name.to_string();
        u.path_mut().unwrap().push("_design".to_string());
        u.path_mut().unwrap().push(ddoc_id.to_string());
        HeadDocument {
            client_state: client_state,
            uri: u,
            if_none_match: None,
        }
    }

    /// Set the If-None-Match header.
    pub fn if_none_match(mut self, rev: &'a Revision) -> HeadDocument<'a> {
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

        let mut resp = {
            let mut req = self.client_state.http_client.head(self.uri);
            if self.if_none_match.is_some() {
                req = req.header(
                    hyper::header::IfNoneMatch::Items(
                        vec![
                            hyper::header::EntityTag::new(
                                false,
                                self.if_none_match.unwrap().to_string())
                        ]
                    )
                );
            }
            try!(
                req.send()
                .or_else(|e| {
                    Err(Error::Transport { cause: error::TransportCause::Hyper(e) } )
                })
            )
        };

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
