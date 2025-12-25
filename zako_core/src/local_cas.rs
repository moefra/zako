use crate::blob_range::BlobRange;
use crate::cas::{Cas, CasError};
use async_trait::async_trait;
use camino::Utf8Path;
use memmap2::MmapOptions;
use std::path::PathBuf;
use std::pin::Pin;
use std::time::SystemTime;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeekExt};
use zako_digest::Digest;
use zako_digest::DigestError;

#[derive(Debug)]
pub struct LocalCas {
    root: PathBuf,
}

impl LocalCas {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn get_root(&self) -> &PathBuf {
        &self.root
    }

    pub fn get_path_for_digest(&self, digest: &Digest) -> PathBuf {
        let hex = digest.get_hash().to_hex();
        self.root.join(&hex[0..2]).join(&hex[2..])
    }

    pub async fn digest(file: &Utf8Path, metadata: &std::fs::Metadata) -> std::io::Result<Digest> {
        // if the file is bigger than 64kb,use mmap. Otherwise, use read.

        let file_size = metadata.len();

        let mut file = std::fs::File::open(file)?;

        if file_size <= 64 * 1024 {
            let mut hasher = blake3::Hasher::new();
            std::io::copy(&mut file as &mut dyn std::io::Read, &mut hasher)?;
            let hash = hasher.finalize();
            return Ok(Digest::new(file_size, hash.as_bytes().clone()));
        }

        unsafe {
            let mmap = MmapOptions::new().map(&file)?;

            let hash = blake3::hash(&mmap);
            Ok(Digest::new(file_size, hash.into()))
        }
    }

    pub async fn input_file(&self, source_path: &Utf8Path) -> std::io::Result<Digest> {
        let metadata = std::fs::metadata(source_path)?;

        let digest = Self::digest(source_path, &metadata).await?;

        let target_path = self.get_path_for_digest(&digest);

        if target_path.exists() {
            let file = std::fs::File::open(source_path)?;
            file.set_modified(SystemTime::now())?;
            return Ok(digest);
        }

        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        crate::link::ref_or_hard_link_file(
            source_path,
            &Utf8Path::from_path(target_path.as_path()).ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidFilename,
                "target path is invalid: contains non-utf8 characters",
            ))?,
        )?;

        Ok(digest)
    }
}

#[async_trait]
impl Cas for LocalCas {
    async fn store(
        &self,
        digest: &Digest,
        mut data: Box<dyn AsyncRead + Send + Unpin + 'static>,
    ) -> Result<(), CasError> {
        let hex = digest.blake3.to_hex();

        let first_prefix = &hex[0..2];
        let second_prefix = &hex[2..4];
        let suffix = &hex[4..];

        let first_dir = self.root.join(first_prefix);
        let second_dir = first_dir.join(second_prefix);
        let target_path = second_dir.join(suffix);

        if target_path.exists() {
            return Ok(());
        }

        tokio::fs::create_dir_all(&target_path)
            .await
            .map_err(|err| CasError::Io(err.into()))?;

        let temp_name = format!("tmp_{}", uuid::Uuid::new_v4());
        let temp_path = target_path.join(temp_name);

        let mut file = tokio::fs::File::create(&temp_path)
            .await
            .map_err(|err| CasError::Io(err.into()))?;

        tokio::io::copy(&mut data, &mut file).await?;

        file.sync_all()
            .await
            .map_err(|err| CasError::Io(err.into()))?;

        // rename应该是原子的。
        tokio::fs::rename(&temp_path, &target_path)
            .await
            .map_err(|err| CasError::Io(err.into()))?;

        Ok(())
    }

    async fn check(&self, digest: &Digest) -> Option<u64> {
        let hex = digest.blake3.to_hex();

        let path = self.root.join(&hex[0..2]).join(&hex[2..4]).join(&hex[4..]);

        tokio::fs::metadata(path).await.ok().map(|meta| meta.len())
    }

    async fn contains(&self, digest: &Digest) -> bool {
        self.check(digest).await.is_some()
    }

    async fn fetch(
        &self,
        digest: &Digest,
        range: &BlobRange,
    ) -> Result<Pin<Box<dyn AsyncRead + Send>>, CasError> {
        let hex = digest.blake3.to_hex();

        let path = self.root.join(&hex[0..2]).join(&hex[2..4]).join(&hex[4..]);

        let mut file = tokio::fs::File::open(path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CasError::NotFound(digest.clone())
            } else {
                CasError::Io(e)
            }
        })?;

        let file_size = file.metadata().await.map_err(CasError::Io)?.len();

        if range.is_out_of(file_size) {
            return Err(CasError::RequestedIndexOutOfRange {
                requested_range: range.clone(),
                blob_digest: digest.clone(),
                blob_length: file_size,
            });
        }

        let length = if let Some(length) = range.length() {
            length
        } else {
            file_size - range.start()
        };

        file.seek(std::io::SeekFrom::Start(range.start()))
            .await
            .map_err(CasError::Io)?;

        Ok(Box::pin(file.take(length)))
    }

    async fn get_local_path(&self, digest: &Digest) -> Option<PathBuf> {
        let path = self.get_path_for_digest(digest);

        if self.contains(digest).await {
            Some(path)
        } else {
            None
        }
    }
}
