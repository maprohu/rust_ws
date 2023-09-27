use std::collections::HashMap;

use hub::hub_proto::{
    hub_service_server::{HubService, HubServiceServer},
    PublishReply, PublishRequest, SubscribeReply, SubscribeRequest, Topic,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

struct Subscriber {

}
#[derive(Debug, Default)]
struct HubServiceImpl {
    topics: HashMap<Topic, >
}

#[tonic::async_trait]
impl HubService for HubServiceImpl {
    async fn publish(
        &self,
        _request: Request<tonic::Streaming<PublishRequest>>,
    ) -> Result<Response<PublishReply>, Status> {
        unimplemented!()
    }

    type SubscribeStream = ReceiverStream<Result<SubscribeReply, Status>>;

    async fn subscribe(
        &self,
        _request: Request<SubscribeRequest>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        unimplemented!()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = HubServiceImpl::default();

    Server::builder()
        .add_service(HubServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
