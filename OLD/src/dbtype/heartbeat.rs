use std;

/// A heartbeat specifies a duration and is used to continue retrieving database
/// changes indefinitely.
#[derive(Debug, Eq, PartialEq)]
pub enum Heartbeat {
    /// Use the server's default heartbeat period.
    Default,

    /// Use the given duration as the heartbeat period.
    Duration(std::time::Duration),
}

impl Default for Heartbeat {
    fn default() -> Self {
        Heartbeat::Default
    }
}

impl std::fmt::Display for Heartbeat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            &Heartbeat::Duration(duration) => {
                let sec = duration.as_secs();
                let nsec = duration.subsec_nanos() as u64;
                let ms = 1000 * sec + nsec / 1_000_000;
                ms.fmt(f)
            }
            &Heartbeat::Default => write!(f, "true"),
        }
    }
}

impl From<std::time::Duration> for Heartbeat {
    fn from(duration: std::time::Duration) -> Self {
        Heartbeat::Duration(duration)
    }
}

#[cfg(test)]
mod tests {

    use std;

    use super::Heartbeat;

    #[test]
    fn default() {
        let expected = Heartbeat::Default;
        let got = Default::default();
        assert_eq!(expected, got);
    }

    #[test]
    fn display() {
        assert_eq!("true", format!("{}", Heartbeat::Default));
        assert_eq!("0",
                   format!("{}", Heartbeat::Duration(std::time::Duration::new(0, 0))));
        assert_eq!("12345",
                   format!("{}",
                           Heartbeat::Duration(std::time::Duration::from_millis(12345))));
    }

    #[test]
    fn eq() {

        fn ms(milliseconds: u64) -> std::time::Duration {
            std::time::Duration::from_millis(milliseconds)
        }

        assert!(Heartbeat::Default == Heartbeat::Default);
        assert!(Heartbeat::Default != Heartbeat::Duration(ms(0)));
        assert!(Heartbeat::Default != Heartbeat::Duration(ms(60_000)));
        assert!(Heartbeat::Duration(ms(0)) == Heartbeat::Duration(ms(0)));
        assert!(Heartbeat::Duration(ms(1234)) == Heartbeat::Duration(ms(1234)));
        assert!(Heartbeat::Duration(ms(1234)) != Heartbeat::Duration(ms(5678)));
    }

    #[test]
    fn from_duration() {
        let expected = Heartbeat::Duration(std::time::Duration::from_millis(12345));
        let got = Heartbeat::from(std::time::Duration::from_millis(12345));
        assert_eq!(expected, got);
    }
}
