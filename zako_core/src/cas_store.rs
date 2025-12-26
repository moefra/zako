use std::pin::Pin;
use std::time::Duration;

use crate::blob_handle::BlobHandle;
use crate::blob_range::BlobRange;
use crate::cas::{Cas, CasError};
use crate::local_cas::LocalCas;
use moka::future::Cache;
use tokio::io::AsyncRead;
use tracing::instrument;
use zako_digest::Digest;

pub type CasCache = crate::FastCache<::zako_digest::blake3_hash::Hash, ::std::vec::Vec<u8>>;

#[derive(Debug, thiserror::Error)]
pub enum CasStoreError {
    #[error("Get an error from cas")]
    CasError(#[from] CasError),
}

#[derive(Debug)]
pub struct CasStore {
    local: Box<LocalCas>,
    remote: Option<Box<dyn Cas>>,
    memory: CasCache,
}

#[derive(Debug, Clone)]
pub struct CasStoreOptions {
    pub max_cache_capacity: u64,
    pub max_cache_ttl: Duration,
    pub max_cache_tti: Duration,
}

impl CasStore {
    pub fn new(
        local: Box<LocalCas>,
        remote: Option<Box<dyn Cas>>,
        options: CasStoreOptions,
    ) -> Self {
        Self {
            local,
            remote,
            memory: Cache::builder()
                // Max capacity
                .max_capacity(options.max_cache_capacity)
                // Weigher
                .weigher(|_key, value: &Vec<u8>| -> u32 {
                    value.len().try_into().unwrap_or(u32::MAX)
                })
                // Time to live (TTL)
                .time_to_live(options.max_cache_ttl)
                // Time to idle (TTI)
                .time_to_idle(options.max_cache_tti)
                .build_with_hasher(::ahash::RandomState::default()),
        }
    }

    #[instrument]
    pub async fn open(
        &self,
        digest: &Digest,
        range: &BlobRange,
    ) -> Result<Pin<Box<dyn AsyncRead + Send>>, CasStoreError> {
        if let Some(cached) = self.memory.get(&digest.blake3).await {
            let cached_len = cached.len() as u64;
            if cached_len < range.start() {
                return Err(CasStoreError::CasError(
                    CasError::RequestedIndexOutOfRange {
                        requested_range: range.clone(),
                        blob_digest: digest.clone(),
                        blob_length: cached_len as u64,
                    },
                ));
            }
            let length = if let Some(length) = range.length() {
                if cached_len < range.start() + length {
                    return Err(CasStoreError::CasError(
                        CasError::RequestedIndexOutOfRange {
                            requested_range: range.clone(),
                            blob_digest: digest.clone(),
                            blob_length: cached_len as u64,
                        },
                    ));
                } else {
                    length
                }
            } else {
                cached_len - range.start()
            };

            return Ok(Box::pin(std::io::Cursor::new(
                cached[range.start() as usize..(range.start() + length) as usize].to_vec(),
            )));
        }

        if let Ok(local) = self.local.fetch(digest, range).await {
            return Ok(local);
        }

        if let Some(remote) = self.remote.as_ref() {
            if let Ok(remote) = remote.fetch(digest, range).await {
                return Ok(remote);
            }
        }

        return Err(CasStoreError::CasError(CasError::NotFound(digest.clone())));
    }

    pub async fn read(&self, digest: &Digest, range: &BlobRange) -> Result<Vec<u8>, CasStoreError> {
        let mut data = self.open(digest, range).await?;
        let mut bytes = Vec::with_capacity(range.length().unwrap_or(1024 * 64) as usize);
        tokio::io::copy(&mut data, &mut bytes)
            .await
            .map_err(|err| CasStoreError::CasError(CasError::Io(err.into())))?;
        Ok(bytes)
    }

    pub fn get_local_cas(&self) -> &LocalCas {
        &self.local
    }

    pub async fn put_bytes(&self, bytes: Vec<u8>) -> Result<BlobHandle, CasStoreError> {
        let blake3 = ::blake3::hash(&bytes);
        let blake3_bytes = blake3.try_into().unwrap();

        let digest = Digest::new(bytes.len() as u64, blake3_bytes);

        // check if the bytes can be cached
        if bytes.len() <= 1024 * 64
        /* 64kb maybe good */
        {
            self.memory.insert(digest.blake3, bytes.clone()).await;
        }

        // put the bytes to the local cas
        // TODO: clone data maybe too slow, find a new way!
        // Issue URL: https://github.com/moefra/zako/issues/16
        self.local
            .store(&digest, Box::new(std::io::Cursor::new(bytes.clone())))
            .await?;

        // put the bytes to the remote cas if exists
        // TODO: add to remote maybe too slow, maybe we should add a individual task to do this
        // Issue URL: https://github.com/moefra/zako/issues/15
        if let Some(remote) = self.remote.as_ref() {
            remote
                .store(&digest, Box::new(std::io::Cursor::new(bytes.clone())))
                .await?;
        }

        // TODO: maybe we should add a memory inlined mode
        Ok(BlobHandle::new_referenced(digest))
    }
}
