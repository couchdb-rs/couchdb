use std;

use Error;

// TODO: This function should take a String so that no changes occur in the
// (common) case that the string contains no percent-encodings.
pub fn percent_decode<T: AsRef<str>>(s: T) -> Result<String, Error> {
    use url::percent_encoding::from_hex;

    let s: &str = s.as_ref();
    let mut parts = s.split("%");
    let mut o = parts.next().unwrap().to_string();

    for p in parts {

        fn make_error(part: &str) -> Error {
            let mut encoding = "%".to_string();
            encoding.push_str(part);
            Error::InvalidPercentEncoding { encoding: encoding }
        }

        let mut v: u8 = 0;
        let mut iter = p.bytes();
        for _ in 0..2 {
            match iter.next() {
                None => {
                    return Err(make_error(p));
                }
                Some(x) => {
                    match from_hex(x) {
                        None => {
                            return Err(make_error(p));
                        }
                        Some(x) => {
                            v <<= 4;
                            v += x;
                        }
                    }
                }
            }
        }

        let s = &[v];
        let t = try!(std::str::from_utf8(s).map_err(|_| make_error(p)));
        o.push_str(t);
        o.push_str(&p[2..]);
    }

    Ok(o)
}

pub fn percent_encode_uri_path<T: AsRef<str>>(s: T) -> String {
    use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
    let s: &str = s.as_ref();
    let s = s.replace("%", "%25");
    let s = s.replace("/", "%2F");
    utf8_percent_encode(&s, DEFAULT_ENCODE_SET)
}

#[cfg(test)]
mod tests {

    macro_rules! expect_decode_error {
        ($result:ident) => {
            match $result {
                Ok(..) => { panic!("Got unexpected OK result"); },
                Err(ref e) => match *e {
                    Error::InvalidPercentEncoding{..} => (),
                    _ => {
                        panic!("Got unexpected error: {}", e);
                    }
                }
            }
        }
    }

    use Error;

    #[test]
    fn percent_decode_ok_basic() {
        use super::percent_decode;
        let exp = "foobar";
        let got = percent_decode("foobar").unwrap();
        assert_eq!(exp, got);
    }

    #[test]
    fn percent_decode_ok_first_and_middle_chars_encoded() {
        use super::percent_decode;
        let exp = "foobar";
        let got = percent_decode("%66oo%62ar").unwrap();
        assert_eq!(exp, got);
    }

    #[test]
    fn percent_decode_ok_middle_and_last_chars_encoded() {
        use super::percent_decode;
        let exp = "foobar";
        let got = percent_decode("fo%6Fba%72").unwrap();
        assert_eq!(exp, got);
    }

    #[test]
    fn percent_decode_ok_all_chars_encoded() {
        use super::percent_decode;
        let exp = "foobar";
        let got = percent_decode("%66%6F%6F%62%61%72").unwrap();
        assert_eq!(exp, got);
    }

    #[test]
    fn percent_decode_ok_lowercase() {
        use super::percent_decode;
        let exp = "foobar";
        let got = percent_decode("%66%6f%6f%62%61%72").unwrap();
        assert_eq!(exp, got);
    }

    #[test]
    fn percent_decode_nok_end_with_percent_sign() {
        use super::percent_decode;
        let got = percent_decode("foobar%");
        expect_decode_error!(got);
    }

    #[test]
    fn percent_decode_nok_end_with_percent_sign_and_one_nibble() {
        use super::percent_decode;
        let got = percent_decode("foobar%6");
        expect_decode_error!(got);
    }

    #[test]
    fn percent_decode_nok_end_with_a_bad_nibble() {
        use super::percent_decode;
        let got = percent_decode("foobar%6z");
        expect_decode_error!(got);
    }

    #[test]
    fn percent_decode_nok_bad_second_nibble() {
        use super::percent_decode;
        let got = percent_decode("foo%6zbar");
        expect_decode_error!(got);
    }

    #[test]
    fn percent_decode_nok_bad_first_nibble() {
        use super::percent_decode;
        let got = percent_decode("foo%z0bar");
        expect_decode_error!(got);
    }

    #[test]
    fn percent_decode_nok_bad_utf8() {
        use super::percent_decode;
        let got = percent_decode("foo%bar");
        expect_decode_error!(got);
    }

    #[test]
    fn percent_encode_uri_path_no_encodings() {
        use super::percent_encode_uri_path;
        let exp = "foobar";
        let got = percent_encode_uri_path("foobar");
        assert_eq!(exp, got);
    }

    #[test]
    fn percent_encode_uri_path_slash_char() {
        use super::percent_encode_uri_path;
        let exp = "foo%2Fbar";
        let got = percent_encode_uri_path("foo/bar");
        assert_eq!(exp, got);
    }

    #[test]
    fn percent_encode_uri_path_percent_char() {
        use super::percent_encode_uri_path;
        let exp = "foo%25bar";
        let got = percent_encode_uri_path("foo%bar");
        assert_eq!(exp, got);
    }

    #[test]
    fn percent_encode_uri_path_space_char() {
        use super::percent_encode_uri_path;
        let exp = "foo%20bar";
        let got = percent_encode_uri_path("foo bar");
        assert_eq!(exp, got);
    }

    #[test]
    fn percent_encode_uri_path_query_char() {
        use super::percent_encode_uri_path;
        let exp = "foo%3Fbar";
        let got = percent_encode_uri_path("foo?bar");
        assert_eq!(exp, got);
    }

    #[test]
    fn percent_encode_uri_path_fragment_char() {
        use super::percent_encode_uri_path;
        let exp = "foo%23bar";
        let got = percent_encode_uri_path("foo#bar");
        assert_eq!(exp, got);
    }

    #[test]
    fn percent_encode_uri_path_many_chars() {
        use super::percent_encode_uri_path;
        let exp = "%2Ffoo%2F%25%20bar%2F";
        let got = percent_encode_uri_path("/foo/% bar/");
        assert_eq!(exp, got);
    }
}
