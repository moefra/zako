use crate::blob_range::BlobRange;
use crate::cas::{Cas, CasError};
use async_trait::async_trait;
use camino::Utf8Path;
use eyre::Context;
use memmap2::MmapOptions;
use std::path::PathBuf;
use std::pin::Pin;
use std::time::SystemTime;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeekExt};
use zako_digest::Digest;

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

    pub async fn digest(
        file: &Utf8Path,
        metadata: &std::fs::Metadata,
        is_symlink: bool,
    ) -> std::io::Result<Digest> {
        // if the file is bigger than 64kb,use mmap. Otherwise, use read.

        let file_size = metadata.len();

        if is_symlink {
            let link = std::fs::read_link(file.as_std_path())?;
            match link.to_str() {
                Some(link) => {
                    let hash = blake3::hash(link.as_bytes());
                    return Ok(Digest::new(link.len() as u64, hash.into()));
                }
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "link is not a valid utf-8 string",
                    ));
                }
            };
        }

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

    pub async fn input_file(
        &self,
        source_path: &Utf8Path,
        is_symlink: bool,
    ) -> std::io::Result<Digest> {
        let metadata = std::fs::metadata(source_path)?;

        let digest = Self::digest(source_path, &metadata, is_symlink).await?;

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
                format!(
                    "target path {:?} is invalid: contains non-utf8 characters",
                    &source_path
                ),
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
        let target_path = self.get_path_for_digest(digest);

        if target_path.exists() {
            return Ok(());
        }

        if let Some(parent) = target_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|err| CasError::Io(err.into(), Some(parent.to_path_buf())))?;
        }

        let temp_name = format!("tmp_{}", uuid::Uuid::new_v4());
        let temp_path = target_path.join(temp_name);

        let mut file = tokio::fs::File::create(&temp_path)
            .await
            .map_err(|err| CasError::Io(err.into(), Some(temp_path.clone())))?;

        tokio::io::copy(&mut data, &mut file)
            .await
            .map_err(|err| CasError::Io(err, Some(temp_path.clone())))?;

        file.sync_all()
            .await
            .map_err(|err| CasError::Io(err.into(), Some(temp_path.clone())))?;

        // rename应该是原子的。
        tokio::fs::rename(&temp_path, &target_path)
            .await
            .map_err(|err| CasError::Io(err.into(), Some(target_path.clone())))?;

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

        let path = self.get_path_for_digest(digest);

        let mut file = tokio::fs::File::open(path.clone()).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CasError::NotFound(digest.clone(), path.clone())
            } else {
                CasError::Io(e, Some(path.clone()))
            }
        })?;

        let file_size = file
            .metadata()
            .await
            .map_err(|e| CasError::Io(e, Some(path.clone())))?
            .len();

        if range.is_out_of_span_length(file_size) {
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
            .map_err(|err| CasError::Io(err, Some(path.clone())))?;

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
