use {Error, IntoDatabasePath};
use action::E_ACTION_USED;
use error::ErrorCategory;
use futures::Future;
use transport::{ActionFuture, Method, Request, Response, ServerResponseFuture, StatusCode, Transport};

/// `HeadDatabase` is an action to test whether a database exists.
#[derive(Debug)]
pub struct HeadDatabase<'a, T: Transport + 'a> {
    transport: &'a T,
    inner: Option<Inner>,
}

#[derive(Debug)]
struct Inner {
    url_path: Result<String, Error>,
}

impl<'a, T: Transport> HeadDatabase<'a, T> {
    #[doc(hidden)]
    pub fn new<P: IntoDatabasePath>(transport: &'a T, db_path: P) -> Self {
        HeadDatabase {
            transport: transport,
            inner: Some(Inner {
                url_path: db_path.into_database_path().map(|x| x.to_string()),
            }),
        }
    }

    /// Sends the request and returns a future of the result.
    ///
    /// # Errors
    ///
    /// Some possible errors:
    ///
    /// * `Error::is_not_found`
    ///
    pub fn send(&mut self) -> ActionFuture<()> {

        let inner = self.inner.take().expect(E_ACTION_USED);

        ActionFuture::new(
            self.transport
                .request(Method::Head, inner.url_path)
                .and_then(|request| request.send_without_body())
                .and_then(|response| {
                    let maybe_category = match response.status_code() {
                        StatusCode::Ok => return ServerResponseFuture::ok(()),
                        StatusCode::NotFound => Some(ErrorCategory::NotFound),
                        _ => None,
                    };
                    ServerResponseFuture::err(response, maybe_category)
                })
                .map_err(|e| Error::chain("Failed to HEAD database", e)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::Future;
    use transport::MockTransport;

    #[test]
    fn head_database_succeeds_on_200_ok() {

        let transport = MockTransport::new();
        let action = HeadDatabase::new(&transport, "/foo").send();
        let result = transport.mock(action, |mock| {
            mock.and_then(|request| {
                let request = request.expect("Client did not send request");
                assert_eq!(request.method(), Method::Head);
                assert_eq!(request.url_path(), "/foo");
                let response = request.response(StatusCode::Ok);
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
    fn head_database_fails_on_404_not_found() {

        let transport = MockTransport::new();
        let action = HeadDatabase::new(&transport, "/foo").send();
        let result = transport.mock(action, |mock| {
            mock.and_then(|request| {
                let request = request.expect("Client did not send request");
                assert_eq!(request.method(), Method::Head);
                assert_eq!(request.url_path(), "/foo");
                let response = request.response(StatusCode::NotFound);
                response.finish()
            }).and_then(|request| {
                    assert!(request.is_none());
                    MockTransport::done()
                })
        });

        match result {
            Err(ref e) if e.is_not_found() => {}
            x => panic!("Got unexpected result {:?}", x),
        }
    }
}
