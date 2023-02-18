use std::error::Error;

#[derive(Debug)]
pub enum TelnetErrorType {
    Codec,
    Io,
}

#[derive(Debug)]
pub struct TelnetError {
    pub kind: TelnetErrorType,
    pub message: String,
}

impl std::fmt::Display for TelnetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<String> for TelnetError {
    fn from(err: String) -> Self {
        Self { kind: TelnetErrorType::Codec, message: err }
    }
}

impl From<std::io::Error> for TelnetError {
    fn from(err: std::io::Error) -> Self {
        Self { kind: TelnetErrorType::Io, message: err.to_string() }
    }
}

impl Error for TelnetError {}
