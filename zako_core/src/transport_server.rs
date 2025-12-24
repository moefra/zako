use crate::blob_range::BlobRange;
use crate::cas::{Cas, CasError};
use crate::protobuf::transport::upload_request::Payload::Metadata;
use crate::protobuf::transport::{
    DownloadRequest, DownloadResponse, UploadRequest, UploadResponse,
};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use tokio_stream::{Stream, StreamExt};
use tonic::async_trait;
use tonic::{Request, Response, Status, Streaming};
use zako_digest::Digest;

#[derive(Debug)]
pub struct TransportServer {
    cas: Arc<dyn Cas + 'static>,
}

impl TransportServer {
    pub fn new(cas: Arc<dyn Cas + 'static>) -> Self {
        Self { cas }
    }
}

#[async_trait]
impl crate::protobuf::transport::transport_server::Transport for TransportServer {
    type DownloadStream = Pin<Box<dyn Stream<Item = Result<DownloadResponse, Status>> + Send>>;

    async fn download(
        &self,
        request: Request<DownloadRequest>,
    ) -> Result<Response<Self::DownloadStream>, Status> {
        let inner = request.into_inner();
        let digest = inner.digest;
        let offset = inner.offset;

        let data = self
            .cas
            .fetch(
                &Digest::try_from(digest.ok_or(Status::invalid_argument("digest is required"))?)
                    .map_err(|err| Status::from(err))?,
                &BlobRange::new(offset, None),
            )
            .await
            .map_err(|err| match err {
                CasError::NotFound(digest) => Status::not_found(digest.hex_blake3()),
                CasError::Io(err) => Status::internal(err.to_string()),
                CasError::Internal(err) => Status::internal(err),
                CasError::RequestedIndexOutOfRange { .. } => {
                    Status::invalid_argument(format!("requested index out of range: {:?}", err))
                }
            })?;

        Ok(Response::new(Box::pin(
            tokio_util::io::ReaderStream::new(data).map(|chunk| match chunk {
                Ok(data) => Ok(DownloadResponse {
                    data: data.to_vec(),
                }),
                Err(err) => Err(Status::internal(err.to_string())),
            }),
        )))
    }

    async fn upload(
        &self,
        request: Request<Streaming<UploadRequest>>,
    ) -> Result<Response<UploadResponse>, Status> {
        let mut request = request.into_inner();

        let digest: Option<UploadRequest> = request.message().await?;

        let digest = zako_digest::Digest::try_from(match digest {
            Some(request) => match request.payload {
                Some(metadata) => match metadata {
                    Metadata(meta) => meta,
                    _ => {
                        return Err(Status::failed_precondition(
                            "First message must be Metadata",
                        ));
                    }
                },
                None => return Err(Status::failed_precondition("No message received")),
            },
            None => return Err(Status::failed_precondition("No message received")),
        })
        .map_err(|err| Status::from(err))?;

        if (*self.cas).contains(&digest).await {
            return Err(Status::already_exists("Blob already exists in CAS"));
        }

        let length = Arc::from(AtomicU64::new(0));
        let cloned_length = length.clone();

        let stream = tokio_util::io::StreamReader::new(request.map(move |x| match x {
            Ok(upload_request) => match upload_request.payload {
                Some(crate::protobuf::transport::upload_request::Payload::Chunk(data)) => {
                    cloned_length.fetch_add(data.len() as u64, std::sync::atomic::Ordering::SeqCst);
                    Ok(bytes::Bytes::from(data))
                }
                _ => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Expect Chunk data not metadata",
                )),
            },
            Err(err) => Err(std::io::Error::new(std::io::ErrorKind::Other, err)),
        }));

        (*self.cas)
            .store(&digest, Box::new(stream))
            .await
            .map_err(|err| match err {
                CasError::Io(err) => Status::internal(err.to_string()),
                CasError::Internal(err) => Status::internal(err),
                _ => Status::internal("Unexpected error during store initialization"),
            })?;

        Ok(Response::new(UploadResponse {
            committed_size: length.load(std::sync::atomic::Ordering::SeqCst),
        }))
    }
}
