use std::{collections::HashMap, sync::Arc};

use futures::future::join_all;
use hub::hub_proto::{
    hub_service_server::{HubService, HubServiceServer},
    publish_request::MessageType,
    PublishReply, PublishRequest, SubscribeReply, SubscribeRequest, Topic,
};
use tokio::sync::mpsc::{self, Sender};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::{transport::Server, Request, Response, Status};

#[derive(Debug)]
struct HubServiceImpl {
    action_tx: Sender<Action>,
}

type TopicKey = String;

#[derive(Default)]
struct TopicSubscribers {
    subscribers: Vec<SubscriberSender>,
}

impl HubServiceImpl {
    fn new() -> HubServiceImpl {
        let (tx, mut rx) = mpsc::channel::<Action>(256);

        tokio::spawn(async move {
            let mut topics: HashMap<TopicKey, TopicSubscribers> = HashMap::new();

            while let Some(action) = rx.recv().await {
                match action {
                    Action::Subscribe { sender, topic } => {
                        let subscribers = topics
                            .entry(topic.name)
                            .or_insert_with(TopicSubscribers::default);

                        subscribers.subscribers.push(sender);
                    }
                    Action::Publish { topic, data } => {
                        let response = Ok(SubscribeReply { data });
                        if let Some(subs) = topics.get(&topic.name) {
                            let futures = subs
                                .subscribers
                                .iter()
                                .map(|sender| sender.send(response.clone()));

                            join_all(futures).await;
                        }
                    }
                }
            }
        });

        HubServiceImpl { action_tx: tx }
    }
}

#[tonic::async_trait]
impl HubService for HubServiceImpl {
    async fn publish(
        &self,
        request: Request<tonic::Streaming<PublishRequest>>,
    ) -> Result<Response<PublishReply>, Status> {
        let mut stream = request.into_inner();

        if let Some(msg) = stream.next().await {
            let msg = msg?;

            let init = match msg.message_type {
                Some(MessageType::Init(init)) => init,
                _ => return Err(Status::invalid_argument("publish must start with Init")),
            };

            let topic = init
                .topic
                .ok_or(Status::invalid_argument("topic missing"))?;

            let topic = Arc::new(topic);

            while let Some(msg) = stream.next().await {
                let msg = msg?;
                let data = match msg.message_type {
                    Some(MessageType::Data(data)) => data,
                    _ => return Err(Status::invalid_argument("publish tail must be Data")),
                };

                self.action_tx
                    .send(Action::Publish {
                        topic: Arc::clone(&topic),
                        data: data.data,
                    })
                    .await
                    .map_err(|err| Status::internal(err.to_string()))?;
            }
        }

        Ok(Response::new(PublishReply {}))
    }

    type SubscribeStream = ReceiverStream<Result<SubscribeReply, Status>>;

    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let (tx, rx) = mpsc::channel(256);
        let request = request.into_inner();

        let topic = request
            .topic
            .ok_or(Status::invalid_argument("topic missing"))?;

        self.action_tx
            .send(Action::Subscribe { sender: tx, topic })
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

type Bytes = Vec<u8>;
type SubscriberSender = Sender<Result<SubscribeReply, Status>>;
#[derive(Debug)]
enum Action {
    Subscribe {
        topic: Topic,
        sender: SubscriberSender,
    },
    Publish {
        topic: Arc<Topic>,
        data: Bytes,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = HubServiceImpl::new();

    Server::builder()
        .add_service(HubServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
