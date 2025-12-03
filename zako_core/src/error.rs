use crate::digest::DigestError;
use tonic::Status;

impl From<DigestError> for Status {
    fn from(err: DigestError) -> Self {
        Status::invalid_argument(err.to_string())
    }
}
