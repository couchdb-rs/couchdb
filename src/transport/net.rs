use {Error, futures, reqwest, std, transport};
use futures::{BoxFuture, Future};
use serde::Deserialize;
use transport::{Method, StatusCode};
use url::Url;

#[derive(Debug)]
pub struct Transport {
    http_client: reqwest::Client,
}

#[derive(Debug)]
pub struct Request {
    http_request_builder: reqwest::RequestBuilder,
}

#[derive(Debug)]
pub struct Response {
    http_response: reqwest::Response,
}

impl Transport {
    pub fn new() -> Result<Self, Error> {
        Ok(Transport {
            http_client: reqwest::Client::new().map_err(|e| {
                (("Failed to construct HTTP client", e))
            })?,
        })
    }
}
impl transport::Transport for Transport {
    type Request = Request;
    fn request(&self, method: Method, url: Url) -> Self::Request {
        Request { http_request_builder: self.http_client.request(method, url) }
    }
}

impl Request {
    fn thread_main<H: transport::ResponseHandler>(
        self,
        handler: H,
        return_channel: futures::sync::oneshot::Sender<Result<H::Item, Error>>,
    ) {
        let response = match self.http_request_builder.send() {
            Ok(x) => Response { http_response: x },
            Err(e) => {
                return_channel
                    .send(Err(Error::from(("Failed to transact HTTP request", e))))
                    .unwrap_or(()); // ignore application hangup
                return;
            }
        };

        return_channel
            .send(handler.handle_response(response))
            .unwrap_or(());
    }
}

impl transport::Request for Request {
    fn accept_application_json(mut self) -> Self {
        self.http_request_builder = self.http_request_builder.header(
            reqwest::header::Accept::json(),
        );
        self
    }

    fn send<H: transport::ResponseHandler + 'static>(self, handler: H) -> BoxFuture<H::Item, Error> {

        let (tx, rx) = futures::sync::oneshot::channel();

        std::thread::spawn(move || self.thread_main(handler, tx));

        rx.map_err(|_canceled| {
            Error::from("HTTP client thread canceled the request")
        }).and_then(|result: Result<H::Item, Error>| result)
            .boxed()
    }
}

impl transport::Response for Response {
    fn status_code(&self) -> StatusCode {
        *self.http_response.status()
    }

    fn decode_json_body<T>(&mut self) -> Result<T, Error>
    where
        for<'de> T: Deserialize<'de>,
    {
        self.http_response.json().map_err(|e| {
            Error::from(("Failed to decode HTTP response body as JSON", e))
        })
    }
}
