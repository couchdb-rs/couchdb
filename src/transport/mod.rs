mod mock;
#[cfg(test)]
mod net;

#[cfg(test)]
pub use self::mock::Transport as MockTransport;
pub use self::net::Transport as NetTransport;
use Error;
use futures::BoxFuture;
pub use reqwest::{Method, StatusCode};
use serde::Deserialize;
use url::Url;

pub trait Transport {
    type Request: Request;
    fn request(&self, method: Method, url: Url) -> Self::Request;
}

pub trait Request {
    fn accept_application_json(self) -> Self;
    fn send<H: ResponseHandler + 'static>(self, handler: H) -> BoxFuture<H::Item, Error>;
}

pub trait ResponseFuture<T> {}

pub trait Response {
    fn status_code(&self) -> StatusCode;
    fn decode_json_body<T>(&mut self) -> Result<T, Error>
    where
        for<'de> T: Deserialize<'de>;
}

pub trait ResponseHandler: Send {
    type Item: Send;
    fn handle_response<R: Response>(self, response: R) -> Result<Self::Item, Error>;
}
