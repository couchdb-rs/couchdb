use hyper;
use serde;
use serde_json;

use Error;
use ErrorResponse;
use IntoDocumentPath;
use Revision;
use client::ClientState;
use action::{self, Action, Request, Response};
use dbtype::PutDocumentResponse;
use error::EncodeErrorKind;

/// Action to create or update a document.
///
/// # Return
///
/// This action returns the document's new revision.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this action:
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

    impl_action_public_methods!(Revision);
}

impl<'a, P: IntoDocumentPath, T: 'a + serde::Serialize> Action for PutDocument<'a, P, T> {
    type Output = Revision;
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let doc_path = try!(self.path.into_document_path());
        let uri = doc_path.into_uri(self.client_state.uri.clone());
        let body = try!(serde_json::to_vec(self.doc_content)
                            .map_err(|e| Error::Encode(EncodeErrorKind::Serde { cause: e })));
        let request = Request::new(hyper::method::Method::Put, uri)
                          .set_accept_application_json()
                          .set_content_type_application_json()
                          .set_if_match_revision(self.if_match)
                          .set_body(body);
        Ok((request, ()))
    }

    fn take_response<R>(mut response: R, _state: Self::State) -> Result<Self::Output, Error>
        where R: Response
    {
        match response.status() {
            hyper::status::StatusCode::Created => {
                try!(response.content_type_must_be_application_json());
                let content = try!(response.decode_json_all::<PutDocumentResponse>());
                let rev: Revision = content.rev.into();
                Ok(rev)
            }
            hyper::status::StatusCode::BadRequest => Err(make_couchdb_error!(BadRequest, response)),
            hyper::status::StatusCode::Unauthorized => {
                Err(make_couchdb_error!(Unauthorized, response))
            }
            hyper::status::StatusCode::NotFound => Err(make_couchdb_error!(NotFound, response)),
            hyper::status::StatusCode::Conflict => {
                Err(make_couchdb_error!(DocumentConflict, response))
            }
            _ => Err(Error::UnexpectedHttpStatus { got: response.status() }),
        }
    }
}

#[cfg(test)]
mod tests {

    use hyper;
    use serde_json;

    use DocumentPath;
    use Revision;
    use client::ClientState;
    use action::{Action, JsonResponse};
    use super::PutDocument;

    #[test]
    fn make_request_default() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let content = serde_json::builder::ObjectBuilder::new()
                          .insert("foo", 17)
                          .insert("bar", "hello")
                          .unwrap();
        let action = PutDocument::new(&client_state, "/db/doc-id", &content);
        let (request, _) = action.make_request().unwrap();
        expect_request_method!(request, hyper::method::Method::Put);
        expect_request_uri!(request, "http://example.com:1234/db/doc-id");
        expect_request_accept_application_json!(request);
        expect_request_content_type_application_json!(request);
        let expected_body = serde_json::to_vec(&content).unwrap();
        expect_request_body!(request, expected_body);
    }

    #[test]
    fn make_request_if_match() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let content = serde_json::builder::ObjectBuilder::new()
                          .insert("foo", 17)
                          .insert("bar", "hello")
                          .unwrap();
        let rev = Revision::parse("42-1234567890abcdef1234567890abcdef").unwrap();
        let action = PutDocument::new(&client_state, "/db/doc-id", &content).if_match(&rev);
        let (request, _) = action.make_request().unwrap();
        expect_request_method!(request, hyper::method::Method::Put);
        expect_request_uri!(request, "http://example.com:1234/db/doc-id");
        expect_request_accept_application_json!(request);
        expect_request_content_type_application_json!(request);
        expect_request_if_match_revision!(request, rev.to_string().as_ref());
        let expected_body = serde_json::to_vec(&content).unwrap();
        expect_request_body!(request, expected_body);
    }

    #[test]
    fn take_response_created() {
        let source_rev = Revision::parse("42-1234567890abcdef1234567890abcdef").unwrap();
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("id", "doc-id")
                         .insert("ok", true)
                         .insert("rev", source_rev.to_string())
                         .unwrap();
        let response = JsonResponse::new(hyper::status::StatusCode::Created, &source);
        let rev = PutDocument::<DocumentPath, serde_json::Value>::take_response(response, ())
                      .unwrap();
        assert_eq!(rev, source_rev);
    }

    #[test]
    fn take_response_bad_request() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "bad_request")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::BadRequest, &source);
        let got = PutDocument::<DocumentPath, serde_json::Value>::take_response(response, ());
        expect_couchdb_error!(got, BadRequest);
    }

    #[test]
    fn take_response_conflict() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "conflict")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::status::StatusCode::Conflict, &source);
        let got = PutDocument::<DocumentPath, serde_json::Value>::take_response(response, ());
        expect_couchdb_error!(got, DocumentConflict);
    }

    #[test]
    fn take_response_not_found() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "not_found")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::NotFound, &source);
        let got = PutDocument::<DocumentPath, serde_json::Value>::take_response(response, ());
        expect_couchdb_error!(got, NotFound);
    }

    #[test]
    fn take_response_unauthorized() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "unauthorized")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::status::StatusCode::Unauthorized, &source);
        let got = PutDocument::<DocumentPath, serde_json::Value>::take_response(response, ());
        expect_couchdb_error!(got, Unauthorized);
    }
}
