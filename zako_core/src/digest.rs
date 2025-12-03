use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Digest {
    /// The fast hash value of the file.
    ///
    /// It was used in the internal system.So it is must.
    pub fast_xxhash3_128: u128,
    /// The secure hash value of the file.
    ///
    /// It was used to export object like publishing binary.
    /// So it was calculated when needed.
    pub secure_sha256: Option<[u8; 32]>,
    /// The size of the file in bytes.
    pub size_bytes: u64,
}

impl Digest {
    pub fn new(xxhash: u128, size: u64) -> Self {
        Self {
            fast_xxhash3_128: xxhash,
            secure_sha256: None,
            size_bytes: size,
        }
    }

    pub fn new_with_secure(xxhash: u128, size: u64, sha256: [u8; 32]) -> Self {
        Self {
            fast_xxhash3_128: xxhash,
            secure_sha256: Some(sha256),
            size_bytes: size,
        }
    }

    /// 检查是否匹配。
    ///
    /// 逻辑：如果双方都有 secure hash，优先比对 secure hash（更安全）。
    ///
    /// 否则比对 fast hash（更快）。
    pub fn refers_to_same_content(&self, other: &Digest) -> bool {
        if self.size_bytes != other.size_bytes {
            return false;
        }
        if let (Some(s1), Some(s2)) = (self.secure_sha256, other.secure_sha256) {
            return s1 == s2;
        }
        self.fast_xxhash3_128 == other.fast_xxhash3_128
    }

    pub fn hex_secure_sha256(&self) -> Option<String> {
        if let Some(sha256) = &self.secure_sha256 {
            return Some(hex::encode(sha256));
        }
        None
    }

    pub fn hex_fast_xxhash3_128(&self) -> String {
        hex::encode(self.fast_xxhash3_128.to_be_bytes())
    }

    /// 业务逻辑：是否不仅内容相同，而且当前对象包含了对方所有的信息？
    ///
    /// 用于判断是否需要更新缓存条目。
    pub fn is_superset_of(&self, other: &Digest) -> bool {
        if !self.refers_to_same_content(other) {
            return false;
        }
        // 如果对方有 sha256，我也必须有，才算 superset
        if other.secure_sha256.is_some() && self.secure_sha256.is_none() {
            return false;
        }
        true
    }
}

#[derive(Error, Debug)]
pub enum DigestError {
    #[error("the length of the fast hash is not 16")]
    WrongLengthFastHash,
    #[error("the length of the secure hash is not 32")]
    WrongLengthSecureHash,
}

impl TryFrom<crate::proto::digest::Digest> for Digest {
    type Error = DigestError;

    fn try_from(value: crate::proto::digest::Digest) -> Result<Self, Self::Error> {
        let fast = value.fast_xxhash3_128;

        let fast: u128 = u128::from_be_bytes(
            fast.try_into()
                .map_err(|_| DigestError::WrongLengthFastHash)?,
        );

        let slow = value.secure_sha256;

        return if let Some(slow) = slow {
            let slow: [u8; 32] = slow
                .try_into()
                .map_err(|_| DigestError::WrongLengthSecureHash)?;
            Ok(Self::new_with_secure(fast, value.size_bytes, slow))
        } else {
            Ok(Self::new(fast, value.size_bytes))
        };
    }
}

impl Into<crate::proto::digest::Digest> for Digest {
    fn into(self) -> crate::proto::digest::Digest {
        crate::proto::digest::Digest {
            fast_xxhash3_128: self.fast_xxhash3_128.to_be_bytes().to_vec(),
            secure_sha256: self.secure_sha256.map(|sha256| sha256.to_vec()),
            size_bytes: self.size_bytes,
        }
    }
}
