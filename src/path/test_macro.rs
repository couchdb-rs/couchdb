macro_rules! expect_path_parse_error {
    ($result:ident, $expected_error:ident, $expected_kind:ident) => {
        match $result {
            Ok(..) => { panic!("Got unexpected OK result"); },
            Err(ref e) => match *e {
                Error::$expected_error(ref kind) => match *kind {
                    BadPathKind::$expected_kind => (),
                    _ => { panic!("Got unexpected kind: {}", e); }
                },
                _ => { panic!("Got unexpected error: {}", e); }
            }
        }
    }
}
