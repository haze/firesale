use reqwest::Error as ReqwestError;

/// General purpose error enum
pub enum Error {
    Reqwest(ReqwestError),
}

impl From<ReqwestError> for Error {
    fn from(source: ReqwestError) -> Self {
        Error::Reqwest(source)
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
