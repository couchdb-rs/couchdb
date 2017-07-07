use {Error, request};
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
    server_url: Url,
    transport: NetTransport,
}

impl Client {
    pub fn new<U: IntoUrl>(server_url: U) -> Result<Self, Error> {
        Ok(Client {
            server_url: server_url.into_url()?,
            transport: NetTransport::new()?,
        })
    }

    pub fn url(&self) -> &Url {
        &self.server_url
    }

    pub fn put_database(&self, db_path: &str) -> request::PutDatabase<NetTransport> {
        request::PutDatabase::new(&self.transport, &self.server_url, db_path)
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
