use {Error, transport};
use error::ErrorCategory;
use futures::Future;
use transport::{ActionFuture, Method, StatusCode, Transport, header};
use url::Url;

#[derive(Debug)]
pub struct PutDatabase {
    transport: Transport,
    url: Url,
}

impl PutDatabase {
    #[doc(hidden)]
    pub fn new(transport: Transport, db_path: &str) -> Self {

        let mut url = transport.server_url().clone();
        url.set_path(db_path);

        PutDatabase {
            transport: transport,
            url: url,
        }
    }

    pub fn send(self) -> ActionFuture<()> {
        ActionFuture::new(
            self.transport
                .request(Method::Put, self.url, |request| {
                    request.header(header::Accept::json());
                })
                .and_then(|response| {
                    let maybe_category = match response.status() {
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
