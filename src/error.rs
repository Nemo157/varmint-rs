use std::fmt;
use std::error::Error as StdError;

#[allow(missing_copy_implementations)] // TODO: Decide on copy or not
#[derive(Debug)]
/// Potential varint parsing errors
pub enum Error {
    /// While decoding a varint it had more continuation bytes than expected
    /// for the decoded type
    LengthExceeded,
}

/// A specialized `Result` type for parsing varints
pub type Result<T> = ::std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::LengthExceeded => "varint exceeded allowed length"
        }
    }
}
