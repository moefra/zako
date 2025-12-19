use async_trait::async_trait;
use std::{path::PathBuf, pin::Pin};
use thiserror::Error;
use tokio::io::AsyncRead;
use zako_digest::Digest;

/// A Content Addressable Storage (CAS) is a storage system that stores data by its content rather than by its location.
#[async_trait]
pub trait Cas: Send + Sync + 'static + std::fmt::Debug {
    /// Store the data in the CAS.
    async fn store(
        &self,
        digest: &Digest,
        data: Box<dyn AsyncRead + Send + Unpin + 'static>,
    ) -> Result<(), CasError>;
    /// Check if the data is in the CAS.
    ///
    /// Returns the length of the data if exists, otherwise returns None.
    ///
    /// It will check both the data exists and application has permission to access the data.
    async fn check(&self, digest: &Digest) -> Option<u64>;
    /// Check if the data is in the CAS, return bool.
    ///
    /// This may be cheaper than `check`, but less informative.
    ///
    /// It will check both the data exists and application has permission to access the data.
    async fn contains(&self, digest: &Digest) -> bool;
    /// Fetch the data from the CAS.
    ///
    /// If not found, it will return a `CasError::NotFound` error.
    async fn fetch(
        &self,
        digest: &Digest,
        offset: u64,
        length: Option<u64>,
    ) -> Result<Pin<Box<dyn AsyncRead + Send>>, CasError>;
    /// Get the local path of the data in the CAS.
    ///
    /// If the data is not in the CAS, or the CAS is online(like S3),it will return None.
    ///
    /// This is helpful for API like `send_file`.
    async fn get_local_path(&self, digest: &Digest) -> Option<PathBuf>;
}

#[derive(Error, Debug)]
pub enum CasError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Blob `{0:?}` not found")]
    NotFound(Digest),
    #[error("Internal storage error: {0}")]
    Internal(String),
    #[error(
        "Requested index is out of range: blob digest: {blob_digest:?}, blob length: {blob_length}, requested offset: {requested_offset}, requested length: {requested_length:?}"
    )]
    RequestedIndexOutOfRange {
        requested_offset: u64,
        requested_length: Option<u64>,
        blob_digest: Digest,
        blob_length: u64,
    },
}
