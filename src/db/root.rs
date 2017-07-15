use uuid::Uuid;

#[derive(Clone, Debug, Deserialize)]
pub struct Vendor {
    name: String,
    version: String,
}

/// `Root` contains the content of a CouchDB server's root resource (`/`).
#[derive(Clone, Debug, Deserialize)]
pub struct Root {
    couchdb: String,
    uuid: Uuid,
    vendor: Vendor,
    version: String,
}

impl Root {
    /// Returns the server's welcome message.
    pub fn welcome(&self) -> &str {
        &self.couchdb
    }

    /// Returns the server's UUID.
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    /// Returns the server's version.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Tries to parse the server's version into major, minor, and patch
    /// numbers.
    pub fn version_triple(&self) -> Option<(u64, u64, u64)> {
        parse_version(&self.version)
    }
}

fn parse_version(s: &str) -> Option<(u64, u64, u64)> {

    const BASE: u32 = 10;

    let parts = s.split(|c: char| !c.is_digit(BASE))
        .map(|s| {
            u64::from_str_radix(s, BASE).map(|x| Some(x)).unwrap_or(
                None,
            )
        })
        .take(3)
        .collect::<Vec<_>>();

    if parts.len() < 3 || parts.iter().any(|&x| x.is_none()) {
        return None;
    }

    Some((parts[0].unwrap(), parts[1].unwrap(), parts[2].unwrap()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn we_can_parse_couchdb_server_version() {
        use super::parse_version;
        assert_eq!(parse_version("1.6.1"), Some((1, 6, 1)));
        assert_eq!(parse_version("1.6.1_1"), Some((1, 6, 1))); // seen in Homebrew
        assert_eq!(parse_version("obviously_bad"), None);
    }
}
