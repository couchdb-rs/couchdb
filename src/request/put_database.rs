use Error;
use error::ServerResponseCategory;
use futures::{BoxFuture, Future};
use transport::{Method, Request, Response, ResponseHandler, StatusCode, Transport};
use url::Url;

#[derive(Debug)]
pub struct PutDatabase<'a, T: Transport + 'a> {
    transport: &'a T,
    url: Url,
}

impl<'a, T: Transport> PutDatabase<'a, T> {
    pub fn new(transport: &'a T, server_url: &Url, db_path: &str) -> Self {

        let mut u = server_url.clone();
        u.set_path(db_path);

        PutDatabase {
            transport: transport,
            url: u,
        }
    }

    // FIXME: Don't use trait object for return type.
    // FIXME: Make the return type opaque to the application.
    pub fn send(self) -> BoxFuture<(), Error> {
        self.transport
            .request(Method::Put, self.url)
            .accept_application_json()
            .send(Handler)
            .boxed()
    }
}

#[derive(Debug)]
struct Handler;

impl ResponseHandler for Handler {
    type Item = ();
    fn handle_response<R: Response>(self, mut response: R) -> Result<Self::Item, Error> {

        let category = match response.status_code() {
            StatusCode::Created => return Ok(()),
            StatusCode::PreconditionFailed => Some(ServerResponseCategory::DatabaseExists),
            StatusCode::Unauthorized => Some(ServerResponseCategory::Unauthorized),
            _ => None,
        };

        Err(Error::new_server_response_error(
            category,
            "put database",
            &mut response,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use transport::MockTransport;

    #[test]
    fn put_database_succeeds_on_201_created() {

        let transport = MockTransport::new();
        let response = PutDatabase::new(&transport, transport.mock_url(), "/alpha").send();

        transport.handle_request(|request| {
            assert!(request.is_accept_application_json());
            transport.new_response(StatusCode::Created)
        });

        match response.wait() {
            Ok(()) => {}
            x => panic!("Got unexpected result {:?}", x),
        }
    }

    #[test]
    fn put_database_already_exists_on_412_precondition_failed() {

        let transport = MockTransport::new();
        let response = PutDatabase::new(&transport, transport.mock_url(), "/alpha").send();

        transport.handle_request(|request| {
            assert!(request.is_accept_application_json());
            transport.new_response(StatusCode::PreconditionFailed)
        });

        match response.wait() {
            Err(ref e) if e.is_database_exists() => {}
            x => panic!("Got unexpected result {:?}", x),
        }
    }
}
