macro_rules! impl_hex16_base_type {
    ($type_name:ident, $argument_name:ident, $error_variant:ident) => {

        impl std::fmt::Display for $type_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                use dbtype::hex16::to_hex_string;
                let &$type_name(ref value) = self;
                // TODO: Refactor this to avoid dynamic memory allocation.
                let s = to_hex_string(value);
                s.fmt(f)
            }
        }

        impl std::str::FromStr for $type_name {
            type Err = Error;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                use dbtype::hex16::from_str;
                let x = try!(from_str(s).ok_or(Error::$error_variant));
                Ok($type_name(x))
            }
        }

        impl From<$type_name> for String {
            fn from($argument_name: $type_name) -> Self {
                format!("{}", $argument_name)
            }
        }
    }
}

pub fn to_hex_string(x: &[u8; 16]) -> String {
    let mut o = String::new();
    o.reserve(32);
    for i in 0..16 {
        o.push(nibble_to_hex(x[i] >> 4));
        o.push(nibble_to_hex(x[i] & 0xf));
    }
    o
}

pub fn from_str(s: &str) -> Option<[u8; 16]> {

    if s.len() != 32 {
        return None;
    }

    let mut h = [0 as u8; 32];
    let mut i = 0;
    for c in s.chars() {
        h[i] = match c.to_digit(16) {
            None => {
                return None;
            }
            Some(x) => x as u8,
        };
        i += 1;
    }

    let mut o = [0 as u8; 16];
    for i in 0..16 {
        o[i] = (h[2 * i] << 4) + h[2 * i + 1];
    }

    Some(o)
}

fn nibble_to_hex(x: u8) -> char {
    match x {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
        4 => '4',
        5 => '5',
        6 => '6',
        7 => '7',
        8 => '8',
        9 => '9',
        10 => 'a',
        11 => 'b',
        12 => 'c',
        13 => 'd',
        14 => 'e',
        15 => 'f',
        x @ _ => {
            panic!("Invalid nibble value {}", x);
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn nibble_to_hex() {
        use super::nibble_to_hex;
        assert_eq!('0', nibble_to_hex(0));
        assert_eq!('1', nibble_to_hex(1));
        assert_eq!('2', nibble_to_hex(2));
        assert_eq!('3', nibble_to_hex(3));
        assert_eq!('4', nibble_to_hex(4));
        assert_eq!('5', nibble_to_hex(5));
        assert_eq!('6', nibble_to_hex(6));
        assert_eq!('7', nibble_to_hex(7));
        assert_eq!('8', nibble_to_hex(8));
        assert_eq!('9', nibble_to_hex(9));
        assert_eq!('a', nibble_to_hex(10));
        assert_eq!('b', nibble_to_hex(11));
        assert_eq!('c', nibble_to_hex(12));
        assert_eq!('d', nibble_to_hex(13));
        assert_eq!('e', nibble_to_hex(14));
        assert_eq!('f', nibble_to_hex(15));
    }

    #[test]
    fn to_hex_string() {
        use super::to_hex_string;
        let source: [u8; 16] = [0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56,
                                0x78, 0x90, 0xab, 0xcd, 0xef];
        let expected = "1234567890abcdef1234567890abcdef".to_string();
        let got = to_hex_string(&source);
        assert_eq!(expected, got);
    }

    #[test]
    fn from_str_ok() {
        use super::from_str;
        let expected: [u8; 16] = [0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x10, 0x32,
                                  0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe];
        let got = from_str("1234567890abcdef1032547698BADCFE").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn from_str_nok() {
        macro_rules! expect_nok {
            ($input: expr) => {
                {
                    use super::from_str;
                    let x = from_str($input);
                    assert!(x.is_none());
                }
            }
        }

        expect_nok!("");
        expect_nok!("bad_revision");
        expect_nok!("12345678");
        expect_nok!("1234567812345678123456781234567");
        expect_nok!("12345678123456781234567812345678a");
        expect_nok!("1234567812345678123456781234567z");
        expect_nok!("z2345678123456781234567812345678");
        expect_nok!("12345678123456z81234567812345678");
        expect_nok!("12345678123456g81234567812345678");
        expect_nok!("12345678123456_81234567812345678");
        expect_nok!("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz");
    }
}
