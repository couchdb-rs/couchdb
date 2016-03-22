macro_rules! unexpected_result {
    ($result:expr) => {
        match $result {
            Err(e) => panic!("Got unexpected error result {:?}", e),
            Ok(x) => panic!("Got unexpected OK result {:?}", x),
        }
    }
}

// Panics if the given result, $result, is not a serde_json 'missing field'
// error.
macro_rules! expect_json_error_missing_field {
    ($result:ident, $expected_missing_field_name:expr) => {
        match $result {
            Err(serde_json::Error::Syntax(
                    serde_json::ErrorCode::MissingField($expected_missing_field_name),
                    ref _line,
                    ref _column)) => (),
            x @ _ => unexpected_result!(x),
        }
    }
}

// Panics if the given result, $result, is not a serde_json 'unknown field'
// error.
macro_rules! expect_json_error_unknown_field {
    ($result:expr, $expected_unknown_field_name:expr) => {
        {
            use serde_json;
            match $result {
                Err(serde_json::Error::Syntax(serde_json::ErrorCode::UnknownField(ref field), _, _)) => {
                    if $expected_unknown_field_name != field {
                        panic!("Got unexpected unknown field: {}", field);
                    }
                }
                x @ _ => unexpected_result!(x),
            }
        }
    }
}

macro_rules! expect_json_error_invalid_value {
    ($result:ident) => {
        match $result {
            Err(serde_json::Error::Syntax(
                    serde_json::ErrorCode::InvalidValue(..),
                    ref _line,
                    ref _column)) => (),
            x @ _ => unexpected_result!(x),
        }
    }
}
