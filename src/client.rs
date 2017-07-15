use {Error, IntoDatabasePath, action, tokio_core};
use transport::NetTransport;
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
pub struct Client {
    transport: NetTransport,
}

impl Client {
    pub fn new<U: IntoUrl>(
        server_url: U,
        _options: ClientOptions,
        reactor_handle: &tokio_core::reactor::Handle,
    ) -> Result<Self, Error> {

        let server_url = server_url.into_url()?;
        let transport = NetTransport::new_with_external_executor(server_url, reactor_handle)?;

        Ok(Client { transport: transport })
    }

    /// Constructs an action to check whether a database exists.
    pub fn head_database<P: IntoDatabasePath>(&self, db_path: P) -> action::HeadDatabase<NetTransport> {
        action::HeadDatabase::new(&self.transport, db_path)
    }

    /// Constructs an action to create a database.
    pub fn put_database<P: IntoDatabasePath>(&self, db_path: P) -> action::PutDatabase<NetTransport> {
        action::PutDatabase::new(&self.transport, db_path)
    }

    /// Constructs an action to delete a database.
    pub fn delete_database<P: IntoDatabasePath>(&self, db_path: P) -> action::DeleteDatabase<NetTransport> {
        action::DeleteDatabase::new(&self.transport, db_path)
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
}
