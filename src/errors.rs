use failure::Fail;

// Errors defined here are used when the library fails
pub mod processing {
    #[derive(Debug, Fail)]
    enum Error {
        #[fail(display = "Dummy Error")]
        Dummy, //TODO(hazebooth): remove
    }
}

// Errors defined here are to be used for when some sort of
// parsing fails on on our
pub mod parsing {
    use reqwest::Error as ReqwestError;
    use serde_json::error::Error as SerdeError;
    use std::error::Error as StdError;

    type StandardError = StdError + Send + Sync;

    #[derive(Debug, Fail)]
    pub enum Error {
        #[fail(display = "JSON Encoding/Decoding failed: {}", inner)]
        Serde { inner: SerdeError },
        #[fail(display = "Unknown error: {}", inner)]
        Unknown { inner: ReqwestError },
    }

    impl From<ReqwestError> for Error {
        fn from(inner: ReqwestError) -> Self {
            let is_serde = inner.is_serialization();
            match inner.get_ref() {
                None => return Error::Unknown { inner },
                Some(err) => {
                    if is_serde {
                        return Error::Serde {
                            inner: SerdeError::from(err),
                        };
                    }
                    return Error::Unknown { inner };
                }
            }
        }
    }

    impl From<SerdeError> for Error {
        fn from(inner: SerdeError) -> Self {
            Error::Serde { inner }
        }
    }
}

// External errors defined here are to represent remote
// errors
pub mod api {
    use reqwest::Error as ReqwestError;
    #[derive(Debug, Fail)]
    pub enum Error {
        #[fail(display = "Network request failed: {}", inner)]
        NetworkError { inner: ReqwestError },
    }

    impl From<ReqwestError> for Error {
        fn from(inner: ReqwestError) -> Self {
            Error::NetworkError { inner }
        }
    }
}
