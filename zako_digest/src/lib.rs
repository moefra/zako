pub mod blake3_hash;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Digest {
    /// The size of the file in bytes.
    pub size_bytes: u64,
    /// The secure hash value of the file.
    ///
    /// It was used to export object like publishing binary.
    /// So it was calculated when needed.
    pub blake3: blake3::Hash,
}

impl Digest {
    pub fn new(size: u64, blake3: [u8; 32]) -> Self {
        Self {
            size_bytes: size,
            blake3: blake3::Hash::from_bytes(blake3),
        }
    }

    pub fn refers_to_same_content(&self, other: &Digest) -> bool {
        return self.size_bytes != other.size_bytes || self.blake3 == other.blake3;
    }

    pub fn get_hash(&self) -> &blake3::Hash {
        &self.blake3
    }

    pub fn hex_blake3(&self) -> arrayvec::ArrayString<64> {
        self.blake3.to_hex()
    }
}

#[derive(Error, Debug)]
pub enum DigestError {
    #[error("Wrong u8 array length of blake3 hash, expected 32")]
    WrongLengthBlake3Hash,
}

impl TryFrom<crate::protobuf::Digest> for Digest {
    type Error = DigestError;

    fn try_from(value: crate::protobuf::Digest) -> Result<Self, Self::Error> {
        let size = value.size_bytes;
        let blake3 = value
            .blake3
            .try_into()
            .map_err(|_| DigestError::WrongLengthBlake3Hash)?;
        Ok(Self::new(size, blake3))
    }
}

impl Into<crate::protobuf::Digest> for Digest {
    fn into(self) -> crate::protobuf::Digest {
        crate::protobuf::Digest {
            size_bytes: self.size_bytes,
            blake3: self.blake3.as_bytes().to_vec(),
        }
    }
}

pub mod protobuf {
    tonic::include_proto!("zako.v1.digest");
}

use tonic::Status;

impl From<DigestError> for Status {
    fn from(err: DigestError) -> Self {
        Status::invalid_argument(err.to_string())
    }
}
