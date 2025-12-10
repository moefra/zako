use crate::cas::Cas;
use crate::protobuf::cas::{
    GetTransportDetailsRequest, NegotiateBlobsRequest, NegotiateBlobsResponse, TransportDetails,
    content_addressable_storage_server::ContentAddressableStorage,
};
use dashmap::DashMap;
use futures::StreamExt; // 引入 Stream 扩展方法
use std::pin::Pin;
use std::sync::Arc;
use tonic::{Request, Response, Status, Streaming};
use uuid::Uuid;
use zako_digest::Digest;
use zako_digest::DigestError;

#[derive(Debug, Clone)]
pub struct CasServerOptions {
    pub cas: Arc<dyn Cas + Send + Sync + 'static>,
    pub server_address: crate::protobuf::net::SocketAddress,
    pub buffered_io_count: usize,
    pub recommended_concurrency: usize,
}

impl CasServerOptions {
    pub fn new_default(
        cas: Arc<dyn Cas + Send + Sync + 'static>,
        server_address: crate::protobuf::net::SocketAddress,
    ) -> Self {
        Self {
            cas,
            server_address,
            buffered_io_count: num_cpus::get(),
            recommended_concurrency: num_cpus::get(),
        }
    }
}

#[derive(Debug)]
pub struct CasServer {
    cas: Arc<dyn Cas + Send + Sync + 'static>,
    // TODO: TTL
    tokens: Arc<DashMap<String, ()>>,
    server_address: crate::protobuf::net::SocketAddress,
    buffered_io_count: usize,
    recommended_concurrency: usize,
}

impl CasServer {
    pub fn new(options: CasServerOptions) -> Self {
        Self {
            cas: options.cas,
            tokens: Arc::new(DashMap::new()),
            server_address: options.server_address.into(),
            buffered_io_count: options.buffered_io_count,
            recommended_concurrency: options.recommended_concurrency,
        }
    }
}

#[tonic::async_trait]
impl ContentAddressableStorage for CasServer {
    type NegotiateBlobsStream =
        Pin<Box<dyn tokio_stream::Stream<Item = Result<NegotiateBlobsResponse, Status>> + Send>>;

    async fn negotiate_blobs(
        &self,
        request: Request<Streaming<NegotiateBlobsRequest>>,
    ) -> Result<Response<Self::NegotiateBlobsStream>, Status> {
        let cas = self.cas.clone();
        let mut in_stream = request.into_inner();

        let io_buffered_count = self.buffered_io_count;

        let output_stream = async_stream::try_stream! {
            while let Some(req) = in_stream.message().await? {

                let mut missing_blobs = Vec::new();

                let checks =
                futures::stream::iter(req.blob_digests.into_iter())
                    .map(|proto_digest| {
                        let cas_ref = cas.clone();
                        return (async move || {
                            let digest: Digest = proto_digest.try_into()
                                .map_err(|e: DigestError| Status::from(e))?;

                            return if !cas_ref.contains(&digest).await {
                                Ok(Some(digest))
                            } else {
                                Ok(None)
                            };
                        })();
                    })
                .buffer_unordered(io_buffered_count);

                let results: Vec<Result<Option<Digest>, Status>> = checks.collect().await;

                for res in results {
                    if let Some(d) = res? {
                        missing_blobs.push(d.into());
                    }
                }

                if !missing_blobs.is_empty() {
                    yield NegotiateBlobsResponse {
                        missing_blob_digests: missing_blobs,
                    };
                }
            }
        };

        Ok(Response::new(Box::pin(output_stream)))
    }

    async fn get_transport_details(
        &self,
        request: Request<GetTransportDetailsRequest>,
    ) -> Result<Response<TransportDetails>, Status> {
        let inner = request.into_inner();

        let mut used_protocol: Option<crate::protobuf::net::Protocol> = None;

        for protocol in inner.supported_protocols {
            if protocol == crate::protobuf::net::Protocol::Grpc as i32 {
                // Only GRpc is supported
                used_protocol = Some(crate::protobuf::net::Protocol::Grpc);
            } else {
                return Err(Status::failed_precondition(
                    "No supported transport protocol found",
                ));
            }
        }

        if used_protocol.is_none() {
            return Err(Status::failed_precondition(
                "No supported transport protocol found",
            ));
        }

        let token = Uuid::new_v4().to_string();
        self.tokens.insert(token.clone(), ());

        Ok(Response::new(TransportDetails {
            server_addr: Some(self.server_address.clone()),
            auth_token: token,
            recommended_concurrency: self.recommended_concurrency as u32,
        }))
    }
}
