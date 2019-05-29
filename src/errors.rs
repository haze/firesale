use reqwest::Error as ReqwestError;
use serde_json::Error as SerdeError;

/// General purpose error describing multiple fault points
/// in either firestore or processing of firestore responses
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Network Error: {}", source))]
    Network { source: ReqwestError },

    #[snafu(display("JSON Encode/Decode Error: {}", source))]
    JSON { source: ReqwestError },

    #[snafu(display("Unknown Error from reqwest: {}", source))]
    UnknownReqwest { source: ReqwestError },
}

impl From<ReqwestError> for Error {
    fn from(source: ReqwestError) -> Self {
        if source.is_serialization() {
            return Error::JSON { source };
        } else if source.is_server_error()
            || source.is_client_error()
            || source.is_http()
            || source.is_redirect()
        {
            return Error::Network { source };
        }
        Error::UnknownReqwest { source }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
