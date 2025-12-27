use rkyv::{Archive, Deserialize, Serialize};
use std::{
    num::NonZeroU64,
    ops::{RangeFrom, RangeFull},
};
use thiserror::Error;
use tonic::Status;

#[derive(Error, Debug)]
pub enum BlobRangeError {
    #[error("length is zero. input start {start:0} and length {length:?}")]
    ZeroLength { start: u64, length: u64 },
}

impl From<BlobRangeError> for Status {
    fn from(err: BlobRangeError) -> Self {
        Status::invalid_argument(format!("Failed to convert range: {:?}", err))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
pub struct BlobRange {
    start: u64,
    length: Option<NonZeroU64>,
}

impl AsRef<BlobRange> for BlobRange {
    fn as_ref(&self) -> &BlobRange {
        self
    }
}

impl Default for BlobRange {
    fn default() -> Self {
        Self::full()
    }
}

impl BlobRange {
    /// Create a new BlobRange,
    ///
    /// This usually returns `Some(BlobRange)` if the length is non-zero, otherwise returns `None`.
    ///
    /// In other terms, the `From<Range<u64>>` may panic if someone write `0..0` or `100..100`, so this is not supported.
    #[inline]
    pub fn new(start: u64, length: Option<u64>) -> Result<Self, BlobRangeError> {
        if let Some(length) = length {
            if length == 0 {
                return Err(BlobRangeError::ZeroLength { start, length });
            }
            Ok(Self {
                start,
                length: NonZeroU64::new(length),
            })
        } else {
            Ok(Self {
                start,
                length: None,
            })
        }
    }

    pub fn full() -> Self {
        Self {
            start: 0,
            length: None,
        }
    }

    pub fn start(&self) -> u64 {
        self.start
    }

    pub fn is_out_of_span_length(&self, span_length: u64) -> bool {
        if self.start() >= span_length {
            true
        } else if let Some(length) = self.length() {
            self.start() + length > span_length
        } else {
            false
        }
    }

    pub fn length(&self) -> Option<u64> {
        self.length.map(|l| l.get())
    }

    pub fn length_nonzero(&self) -> Option<NonZeroU64> {
        self.length
    }

    /// Get the end offset of the BlobRange.
    ///
    /// It will return `None` if the [BlobRange::length] is `None`.
    ///
    /// Or it will return [BlobRange::start] + [BlobRange::length].
    pub fn end(&self) -> Option<u64> {
        if let Some(length) = self.length {
            Some(self.start + length.get())
        } else {
            None
        }
    }
}

/// Support `100..`
impl From<RangeFrom<u64>> for BlobRange {
    fn from(r: RangeFrom<u64>) -> Self {
        Self {
            start: r.start,
            length: None,
        }
    }
}

/// Support `..` (full range)
impl From<RangeFull> for BlobRange {
    fn from(_: RangeFull) -> Self {
        Self::full()
    }
}

impl TryFrom<crate::protobuf::range::BlobRange> for BlobRange {
    type Error = BlobRangeError;

    fn try_from(value: crate::protobuf::range::BlobRange) -> Result<Self, Self::Error> {
        Self::new(value.start, value.length)
    }
}

impl Into<crate::protobuf::range::BlobRange> for BlobRange {
    fn into(self) -> crate::protobuf::range::BlobRange {
        crate::protobuf::range::BlobRange {
            start: self.start,
            length: self.length.map(|l| l.get()),
        }
    }
}
