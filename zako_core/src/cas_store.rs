use std::pin::Pin;
use std::time::Duration;

use crate::FastMap;
use crate::cas::{Cas, CasError};
use futures::Stream;
use moka::future::Cache;
use sysinfo::System;
use tokio::io::AsyncRead;
use tracing::{instrument, trace_span};
use zako_digest::Digest;

#[derive(Debug, thiserror::Error)]
pub enum CasStoreError {
    #[error("Get an error from cas")]
    CasError(#[from] CasError),
}

#[derive(Debug)]
pub struct CasStore {
    local: Box<dyn Cas>,
    remote: Option<Box<dyn Cas>>,
    memory: moka::future::Cache<u128, Vec<u8>, ::ahash::RandomState>,
}

impl CasStore {
    pub fn new(
        local: Box<dyn Cas>,
        remote: Option<Box<dyn Cas>>,
        memory_capacity: u64,
        memory_ttl: Duration,
        memory_tti: Duration,
    ) -> Self {
        Self {
            local,
            remote,
            memory: Cache::builder()
                // Max capacity
                .max_capacity(memory_capacity)
                // Weigher
                .weigher(|_key, value: &Vec<u8>| -> u32 {
                    value.len().try_into().unwrap_or(u32::MAX)
                })
                // Time to live (TTL)
                .time_to_live(memory_ttl)
                // Time to idle (TTI)
                .time_to_idle(memory_tti)
                .build_with_hasher(::ahash::RandomState::default()),
        }
    }

    #[instrument]
    pub async fn read(
        &self,
        digest: &Digest,
        offset: u64,
        length: Option<u64>,
    ) -> Result<Pin<Box<dyn AsyncRead + Send>>, CasStoreError> {
        if let Some(cached) = self.memory.get(&digest.fast_xxhash3_128).await {
            let cached_len = cached.len() as u64;
            if cached_len < offset {
                return Err(CasStoreError::CasError(
                    CasError::RequestedIndexOutOfRange {
                        requested_offset: offset,
                        requested_length: length,
                        blob_digest: digest.clone(),
                        blob_length: cached_len as u64,
                    },
                ));
            }
            let length = if let Some(length) = length {
                if cached_len < offset + length {
                    return Err(CasStoreError::CasError(
                        CasError::RequestedIndexOutOfRange {
                            requested_offset: offset,
                            requested_length: Some(length),
                            blob_digest: digest.clone(),
                            blob_length: cached_len as u64,
                        },
                    ));
                } else {
                    length
                }
            } else {
                cached_len - offset
            };

            return Ok(Box::pin(std::io::Cursor::new(
                cached[offset as usize..(offset + length) as usize].to_vec(),
            )));
        }

        if let Ok(local) = self.local.fetch(digest, offset, length).await {
            return Ok(local);
        }

        if let Some(remote) = self.remote.as_ref() {
            if let Ok(remote) = remote.fetch(digest, offset, length).await {
                return Ok(remote);
            }
        }

        return Err(CasStoreError::CasError(CasError::NotFound(digest.clone())));
    }
}
