use hub::hub_proto::{hub_service_client::HubServiceClient, SubscribeRequest, Topic};
use tonic::Request;
use std::str;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = HubServiceClient::connect("http://[::1]:50051").await?;

    let mut stream = client
        .subscribe(Request::new(SubscribeRequest {
            topic: Some(Topic {
                name: "sample_topic".into(),
            }),
        }))
        .await?
        .into_inner();

    while let Some(reply) = stream.message().await? {
        println!("{}", str::from_utf8(&reply.data)?);
    }

    Ok(())
}
