use {Error, IntoDatabasePath};
use action::E_ACTION_USED;
use error::ErrorCategory;
use futures::Future;
use transport::{ActionFuture, Method, Request, Response, ServerResponseFuture, StatusCode, Transport};

#[derive(Debug)]
pub struct DeleteDatabase<'a, T: Transport + 'a> {
    transport: &'a T,
    inner: Option<Inner>,
}

#[derive(Debug)]
struct Inner {
    url_path: Result<String, Error>,
}

impl<'a, T: Transport> DeleteDatabase<'a, T> {
    #[doc(hidden)]
    pub fn new<P: IntoDatabasePath>(transport: &'a T, db_path: P) -> Self {
        DeleteDatabase {
            transport: transport,
            inner: Some(Inner {
                url_path: db_path.into_database_path().map(|x| x.to_string()),
            }),
        }
    }

    pub fn send(&mut self) -> ActionFuture<()> {

        let inner = self.inner.take().expect(E_ACTION_USED);

        ActionFuture::new(
            self.transport
                .request(Method::Delete, inner.url_path)
                .and_then(|mut request| {
                    request.accept_application_json();
                    request.send_without_body()
                })
                .and_then(|response| {
                    let maybe_category = match response.status_code() {
                        StatusCode::Ok => return ServerResponseFuture::ok(()),
                        StatusCode::NotFound => Some(ErrorCategory::DatabaseDoesNotExist),
                        StatusCode::Unauthorized => Some(ErrorCategory::Unauthorized),
                        _ => None,
                    };
                    ServerResponseFuture::err(response, maybe_category)
                })
                .map_err(|e| Error::chain("Failed to DELETE database", e)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::Future;
    use transport::MockTransport;

    #[test]
    fn delete_database_succeeds_on_200_ok() {

        let transport = MockTransport::new();
        let action = DeleteDatabase::new(&transport, "/foo").send();
        let result = transport.mock(action, |mock| {
            mock.and_then(|request| {
                let request = request.expect("Client did not send request");
                assert_eq!(request.method(), Method::Delete);
                assert_eq!(request.url_path(), "/foo");
                assert!(request.is_accept_application_json());
                let mut response = request.response(StatusCode::Ok);
                response.set_json_body(&json!({"ok": true}));
                response.finish()
            }).and_then(|request| {
                    assert!(request.is_none());
                    MockTransport::done()
                })
        });

        match result {
            Ok(()) => {}
            x => panic!("Got unexpected result {:?}", x),
        }
    }

    #[test]
    fn delete_database_fails_on_404_not_found() {

        let transport = MockTransport::new();
        let action = DeleteDatabase::new(&transport, "/foo").send();
        let result = transport.mock(action, |mock| {
            mock.and_then(|request| {
                let request = request.expect("Client did not send request");
                assert_eq!(request.method(), Method::Delete);
                assert_eq!(request.url_path(), "/foo");
                assert!(request.is_accept_application_json());
                let mut response = request.response(StatusCode::NotFound);
                response.set_json_body(&json!({
                    "error": "not_found",
                    "reason": "missing"
                }));
                response.finish()
            }).and_then(|request| {
                    assert!(request.is_none());
                    MockTransport::done()
                })
        });

        match result {
            Err(ref e) if e.is_database_does_not_exist() => {}
            x => panic!("Got unexpected result {:?}", x),
        }
    }

    // FIXME: Test Unauthorized.
}
