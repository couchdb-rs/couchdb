use {Error, transport};
use error::ErrorCategory;
use futures::Future;
use transport::{Action, ActionFuture, AsyncTransport, Method, Request, RequestMaker, Response, StatusCode,
                SyncTransport, Transport};

#[derive(Debug)]
pub struct PutDatabase<'a, T: Transport + 'a> {
    transport: &'a T,
    url_path: String,
}

impl<'a, T: Transport> PutDatabase<'a, T> {
    #[doc(hidden)]
    pub fn new(transport: &'a T, db_path: &str) -> Self {
        PutDatabase {
            transport: transport,
            url_path: String::from(db_path), // FIXME: Use stronger type for database path.
        }
    }
}

impl<'a> PutDatabase<'a, AsyncTransport> {
    pub fn send(&self) -> ActionFuture<()> {
        self.transport.transport_async(self)
    }
}

impl<'a> PutDatabase<'a, SyncTransport> {
    pub fn run(&self) -> Result<(), Error> {
        self.transport.transport_sync(self)
    }
}

impl<'a, T: Transport> Action for PutDatabase<'a, T> {
    type Item = ();
    fn act<R: RequestMaker>(&self, request_maker: R) -> ActionFuture<Self::Item> {
        ActionFuture::new(
            request_maker
                .make_request(Method::Put, &self.url_path)
                .and_then(|mut request| {
                    request.set_accept_application_json();
                    request.send_without_body()
                })
                .and_then(|response| {
                    let maybe_category = match response.status_code() {
                        StatusCode::Created => return transport::ServerResponseFuture::ok(()),
                        StatusCode::PreconditionFailed => Some(ErrorCategory::DatabaseExists),
                        StatusCode::Unauthorized => Some(ErrorCategory::Unauthorized),
                        _ => None,
                    };
                    transport::ServerResponseFuture::err(response, maybe_category)
                })
                .map_err(|e| Error::chain("Failed to PUT database", e)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use transport::MockTransport;

    #[test]
    fn put_database_succeeds_on_201_created() {
        let transport = MockTransport::new();
        let action = PutDatabase::new(&transport, "/foo");
        match transport.mock(action, |request| {
            assert_eq!(request.method(), Method::Put);
            assert_eq!(request.url_path(), "/foo");
            assert!(request.is_accept_application_json());
            let mut response = MockTransport::new_response(StatusCode::Created);
            response.set_json_body(&json!({"ok": true}));
            response
        }) {
            Ok(_) => {}
            x => panic!("Got unexpected result {:?}", x),
        }
    }

    // FIXME: Test PreconditionFailed.

    // FIXME: Test Unauthorized.
}
