use hyper;

use Document;
use Error;
use ErrorResponse;
use IntoDocumentPath;
use Revision;
use client::ClientState;
use action::{self, Action, Request, Response};

enum QueryIterator<'a> {
    Rev(&'a QueryParams<'a>),
    Done,
}

impl<'a> Iterator for QueryIterator<'a> {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self {
                &mut QueryIterator::Rev(params) => {
                    *self = QueryIterator::Done;
                    if let Some(ref rev) = params.rev {
                        return Some(("rev".to_string(), rev.to_string()));
                    }
                }
                &mut QueryIterator::Done => {
                    return None;
                }
            }
        }
    }
}

// The query parameters reside in a separate structure to facilitate iteration,
// which is useful when constructing the URI query string.
#[derive(Default)]
struct QueryParams<'a> {
    rev: Option<&'a Revision>,
}

impl<'a> QueryParams<'a> {
    fn is_default(&self) -> bool {
        self.rev.is_none()
    }

    fn iter(&self) -> QueryIterator {
        QueryIterator::Rev(self)
    }
}

/// Action to get document meta-information and application-defined content.
///
/// # Return
///
/// This action returns an `Option` type. The return value is `None` if the
/// action specifies a revision and the document hasn't been modified since
/// that revision. Otherwise, the return value is `Some` and contains the
/// document meta-information and application-defined content.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this action:
///
///
/// * `Error::NotFound`: The document does not exist.
/// * `Error::Unauthorized`: The client is unauthorized.
///
pub struct GetDocument<'a, P>
    where P: IntoDocumentPath
{
    client_state: &'a ClientState,
    path: P,
    if_none_match: Option<&'a Revision>,
    query: QueryParams<'a>,
}

impl<'a, P: IntoDocumentPath> GetDocument<'a, P> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState, path: P) -> Self {
        GetDocument {
            client_state: client_state,
            path: path,
            if_none_match: None,
            query: Default::default(),
        }
    }

    /// Sets the If-None-Match header.
    pub fn if_none_match(mut self, rev: &'a Revision) -> Self {
        self.if_none_match = Some(rev);
        self
    }

    /// Sets the `rev` query parameter to get the document at the given
    /// revision.
    pub fn rev(mut self, rev: &'a Revision) -> Self {
        self.query.rev = Some(rev);
        self
    }

    impl_action_public_methods!(Option<Document>);
}

impl<'a, P: IntoDocumentPath> Action for GetDocument<'a, P> {
    type Output = Option<Document>;
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let doc_path = try!(self.path.into_document_path());
        let uri = {
            let mut uri = doc_path.into_uri(self.client_state.uri.clone());
            if !self.query.is_default() {
                uri.set_query_from_pairs(self.query.iter());
            }
            uri
        };
        let request = Request::new(hyper::Get, uri)
                          .set_accept_application_json()
                          .set_if_none_match_revision(self.if_none_match);
        Ok((request, ()))
    }

    fn take_response<R>(mut response: R, _state: Self::State) -> Result<Self::Output, Error>
        where R: Response
    {
        match response.status() {
            hyper::status::StatusCode::Ok => {
                try!(response.content_type_must_be_application_json());
                let doc = try!(response.decode_json_all::<Document>());
                Ok(Some(doc))
            }
            hyper::status::StatusCode::NotModified => Ok(None),
            hyper::status::StatusCode::BadRequest => Err(make_couchdb_error!(BadRequest, response)),
            hyper::status::StatusCode::Unauthorized => {
                Err(make_couchdb_error!(Unauthorized, response))
            }
            hyper::status::StatusCode::NotFound => Err(make_couchdb_error!(NotFound, response)),
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
    use action::{Action, JsonResponse, NoContentResponse};
    use super::{GetDocument, QueryParams};

    #[test]
    fn query_iterator() {
        let rev = Revision::parse("42-1234567890abcdef1234567890abcdef").unwrap();
        let query = QueryParams { rev: Some(&rev) };
        let expected = vec![("rev".to_string(), rev.to_string())];
        let got = query.iter().collect::<Vec<_>>();
        assert_eq!(expected, got);
    }

    #[test]
    fn make_request_default() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let action = GetDocument::new(&client_state, "/foo/bar");
        let (request, ()) = action.make_request().unwrap();
        expect_request_method!(request, hyper::Get);
        expect_request_uri!(request, "http://example.com:1234/foo/bar");
        expect_request_accept_application_json!(request);
    }

    #[test]
    fn make_request_if_none_match() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let rev = Revision::parse("42-1234567890abcdef1234567890abcdef").unwrap();
        let action = GetDocument::new(&client_state, "/foo/bar").if_none_match(&rev);
        let (request, ()) = action.make_request().unwrap();
        expect_request_method!(request, hyper::Get);
        expect_request_uri!(request, "http://example.com:1234/foo/bar");
        expect_request_accept_application_json!(request);
        expect_request_if_none_match_revision!(request, "42-1234567890abcdef1234567890abcdef");
    }

    #[test]
    fn make_request_rev() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let rev = "42-1234567890abcdef1234567890abcdef".parse().unwrap();
        let action = GetDocument::new(&client_state, "/foo/bar").rev(&rev);
        let (request, ()) = action.make_request().unwrap();
        expect_request_method!(request, hyper::Get);
        expect_request_uri!(request,
                            "http://example.com:\
                             1234/foo/bar?rev=42-1234567890abcdef1234567890abcdef");
        expect_request_accept_application_json!(request);
    }

    #[test]
    fn take_response_ok() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("_id", "foo")
                         .insert("_rev", "42-1234567890abcdef1234567890abcdef")
                         .insert("bar", 17)
                         .unwrap();
        let response = JsonResponse::new(hyper::Ok, &source);
        let got = GetDocument::<DocumentPath>::take_response(response, ()).unwrap();
        let got = got.unwrap();
        assert_eq!(got.id, "foo".into());
        assert_eq!(got.rev,
                   "42-1234567890abcdef1234567890abcdef".parse().unwrap());
        let expected = serde_json::builder::ObjectBuilder::new()
                           .insert("bar", 17)
                           .unwrap();
        let got = got.into_content::<serde_json::Value>().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn take_response_ok_deleted() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("_id", "foo")
                         .insert("_rev", "42-1234567890abcdef1234567890abcdef")
                         .insert("_deleted", true)
                         .unwrap();
        let response = JsonResponse::new(hyper::Ok, &source);
        let got = GetDocument::<DocumentPath>::take_response(response, ()).unwrap();
        let got = got.unwrap();
        assert_eq!(got.id, "foo".into());
        assert_eq!(got.rev,
                   "42-1234567890abcdef1234567890abcdef".parse().unwrap());
        assert!(got.deleted);
        let expected = serde_json::builder::ObjectBuilder::new().unwrap();
        let got = got.into_content::<serde_json::Value>().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn take_response_not_modified() {
        let response = NoContentResponse::new(hyper::status::StatusCode::NotModified);
        let got = GetDocument::<DocumentPath>::take_response(response, ()).unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn take_response_bad_request() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "bad_request")
                         .insert("reason", "Invalid rev format")
                         .unwrap();
        let response = JsonResponse::new(hyper::BadRequest, &source);
        let got = GetDocument::<DocumentPath>::take_response(response, ());
        expect_couchdb_error!(got, BadRequest);
    }

    #[test]
    fn take_response_not_found() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "not_found")
                         .insert("reason", "missing")
                         .unwrap();
        let response = JsonResponse::new(hyper::NotFound, &source);
        let got = GetDocument::<DocumentPath>::take_response(response, ());
        expect_couchdb_error!(got, NotFound);
    }

    #[test]
    fn take_response_unauthorized() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "unauthorized")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::status::StatusCode::Unauthorized, &source);
        let got = GetDocument::<DocumentPath>::take_response(response, ());
        expect_couchdb_error!(got, Unauthorized);
    }
}
