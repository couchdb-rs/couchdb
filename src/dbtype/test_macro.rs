// Panics if the given result, $result, is not a serde_json 'missing field'
// error.
macro_rules! expect_json_error_missing_field {
    ($result:ident, $expected_missing_field_name:expr) => {
        match $result {
            Ok(..) => { panic!("Got unexpected OK result"); },
            Err(ref e) => {
                use serde_json;
                if let serde_json::Error::SyntaxError(ref code, ref _line, ref _column) = *e {

                    // There's a error-reporting bug in serde_json that makes
                    // this check impossible. See here:
                    // https://github.com/serde-rs/json/issues/22.
                    //
                    // When this bug is resolved, the commented-out block of
                    // code should work. Until then, we use the workaround
                    // below.
                    //
                    // if let serde_json::ErrorCode::MissingField(ref field_name) = *code {
                    //     if *field_name != $expected_missing_field_name {
                    //     }
                    //     panic!("Got unexpected missing field: {}", field_name);
                    // }
                    // panic!("Got unexpected error code variant: {}", e);

                    if let serde_json::ErrorCode::ExpectedSomeValue = *code {
                        // Okay
                    } else {
                        panic!("Got unexpected error code: {}", e);
                    }
                } else {
                    panic!("Got unexpected error: {}", e);
                }
            }
        }
    }
}

macro_rules! expect_json_error_invalid_value {
    ($result:ident) => {
        match $result {
            Ok(..) => { panic!("Got unexpected OK result"); },
            Err(ref e) => {
                use serde_json;
                if let serde_json::Error::SyntaxError(ref code, ref _line, ref _column) = *e {
                    if let serde_json::ErrorCode::ExpectedSomeValue = *code {
                        // Okay
                    } else {
                        panic!("Got unexpected error code: {}", e);
                    }
                } else {
                    panic!("Got unexpected error: {}", e);
                }
            }
        }
    }
}
