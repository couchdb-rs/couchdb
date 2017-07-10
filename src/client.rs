use {Error, action, tokio_core};
use transport::{AsyncTransport, SyncTransport, Transport};
use url::Url;

/// `IntoUrl` is a trait for converting a type into a type into a URL.
///
/// The `IntoUrl` trait is like the `Into` trait with the difference that its
/// conversion may fail, such as when parsing a string containing an invalid
/// URL.
///
pub trait IntoUrl {
    fn into_url(self) -> Result<Url, Error>;
}

impl IntoUrl for Url {
    fn into_url(self) -> Result<Url, Error> {
        Ok(self)
    }
}

impl<'a> IntoUrl for &'a str {
    fn into_url(self) -> Result<Url, Error> {
        let u = Url::parse(self).map_err(|e| {
            ((format!("Failed to parse URL (url: {})", self), e))
        })?;
        Ok(u)
    }
}

impl<'a> IntoUrl for &'a String {
    fn into_url(self) -> Result<Url, Error> {
        self.as_str().into_url()
    }
}

#[derive(Debug)]
pub struct Client<T: Transport> {
    transport: T,
}

pub type AsyncClient = Client<AsyncTransport>;
pub type SyncClient = Client<SyncTransport>;

impl Client<AsyncTransport> {
    pub fn new<U: IntoUrl>(
        server_url: U,
        reactor_handle: &tokio_core::reactor::Handle,
        _options: ClientOptions,
    ) -> Result<Self, Error> {
        Ok(Client {
            transport: AsyncTransport::new(reactor_handle, server_url.into_url()?)?,
        })
    }
}

impl Client<SyncTransport> {
    pub fn new<U: IntoUrl>(server_url: U, _options: ClientOptions) -> Result<Self, Error> {
        Ok(Client {
            transport: SyncTransport::new(server_url.into_url()?)?,
        })
    }
}

impl<T: Transport> Client<T> {
    pub fn put_database(&self, db_path: &str) -> action::PutDatabase<T> {
        action::PutDatabase::new(&self.transport, db_path)
    }
}

#[derive(Debug, Default)]
pub struct ClientOptions {}

impl ClientOptions {
    pub fn new() -> Self {
        ClientOptions {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_url_converts_string_ok() {
        let u = "http://example.com:5984/foo/bar".into_url().unwrap();
        assert_eq!(u.path(), "/foo/bar");
    }

    #[test]
    fn into_url_fails_for_invalid_string() {
        "not_a_valid_url".into_url().unwrap_err();
    }

    #[test]
    fn sync_client_implements_send() {
        fn requires_send<T: Send>() {}
        requires_send::<SyncClient>();
    }
}
