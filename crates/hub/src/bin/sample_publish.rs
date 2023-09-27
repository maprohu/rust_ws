use std::time::Duration;

use hub::hub_proto::{
    hub_service_client::HubServiceClient,
    publish_request::{self, Data},
    PublishRequest, Topic,
};
use tokio::time;
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = HubServiceClient::connect("http://[::1]:50051").await?;

    let start = time::Instant::now();

    let outbound = async_stream::stream! {
        yield PublishRequest {
            message_type: Some(
                publish_request::MessageType::Init(publish_request::Init{
                    topic: Some(Topic {
                        name: "sample_topic".into(),
                    }),
                }),
            ),
        };

        let mut interval = time::interval(Duration::from_secs(1));

        loop {
            let time = interval.tick().await;

            let elapsed = time.duration_since(start);
            let message = format!("at {:?}", elapsed);

            yield PublishRequest {
                message_type: Some(
                    publish_request::MessageType::Data(
                        Data {
                            data: message.as_bytes().into(),
                        }
                    ))
             };
        }
    };

    let response = client.publish(Request::new(outbound)).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
