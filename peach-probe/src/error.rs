#[derive(Debug)]
pub enum ProbeError {
    GetServiceVersionRegexError(regex::Error),
    GetServiceVersionRegexMatchError,
    GetServiceVersionParseError(core::str::Utf8Error),
    GetServiceLogParseError(std::string::FromUtf8Error),
    GetServiceVersionAptError(std::io::Error),
}

impl From<regex::Error> for ProbeError {
    fn from(err: regex::Error) -> ProbeError {
        ProbeError::GetServiceVersionRegexError(err)
    }
}

impl From<core::str::Utf8Error> for ProbeError {
    fn from(err: core::str::Utf8Error) -> ProbeError {
        ProbeError::GetServiceVersionParseError(err)
    }
}

impl From<std::string::FromUtf8Error> for ProbeError {
    fn from(err: std::string::FromUtf8Error) -> ProbeError {
        ProbeError::GetServiceLogParseError(err)
    }
}

impl From<std::io::Error> for ProbeError {
    fn from(err: std::io::Error) -> ProbeError {
        ProbeError::GetServiceVersionAptError(err)
    }
}
