use crate::cas::{Cas, CasError};
use crate::digest::Digest;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncRead, AsyncWriteExt};

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
}

#[async_trait]
impl Cas for LocalCas {
    async fn store(
        &self,
        digest: &Digest,
        mut data: Box<dyn AsyncRead + Send + Unpin + 'static>,
    ) -> Result<(), CasError> {
        let hex = digest.hex_fast_xxhash3_128();

        let first_prefix = &hex[0..2];
        let second_prefix = &hex[2..4];
        let suffix = &hex[4..];

        let first_dir = self.root.join(first_prefix);
        let second_dir = first_dir.join(second_prefix);
        let target_path = second_dir.join(suffix);

        if target_path.exists() {
            return Ok(());
        }

        fs::create_dir_all(&target_path)
            .await
            .map_err(|err| CasError::Io(err.into()))?;

        let temp_name = format!("tmp_{}", uuid::Uuid::new_v4());
        let temp_path = target_path.join(temp_name);

        let mut file = fs::File::create(&temp_path)
            .await
            .map_err(|err| CasError::Io(err.into()))?;

        tokio::io::copy(&mut data, &mut file).await?;

        file.sync_all()
            .await
            .map_err(|err| CasError::Io(err.into()))?;

        // 在 POSIX 系统上，rename 是原子的。
        // 如果重命名成功，说明文件完整地出现在了位置上。
        fs::rename(&temp_path, &target_path)
            .await
            .map_err(|err| CasError::Io(err.into()))?;

        Ok(())
    }

    async fn check(&self, digest: &Digest) -> bool {
        let hex = digest.hex_fast_xxhash3_128();

        let path = self.root.join(&hex[0..2]).join(&hex[2..4]).join(&hex[4..]);

        fs::metadata(path).await.is_ok()
    }

    async fn fetch(&self, digest: &Digest) -> Result<Box<dyn AsyncRead + Send + Unpin>, CasError> {
        let hex = digest.hex_fast_xxhash3_128();

        let path = self.root.join(&hex[0..2]).join(&hex[2..4]).join(&hex[4..]);

        let file = tokio::fs::File::open(path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CasError::NotFound(digest.hex_fast_xxhash3_128())
            } else {
                CasError::Io(e)
            }
        })?;

        Ok(Box::from(file))
    }

    async fn get_local_path(&self, digest: &Digest) -> Option<PathBuf> {
        let hex = digest.hex_fast_xxhash3_128();

        let path = self.root.join(&hex[0..2]).join(&hex[2..4]).join(&hex[4..]);

        if self.check(digest).await {
            Some(path)
        } else {
            None
        }
    }
}
