use crate::error::{self, CustomError};
use crate::new_type::{ConsumerId, TopicId};
use std::marker::PhantomData;

// State marker types organized in a module
pub mod states {
    #[derive(Debug)]
    pub struct Disconnected;
    #[derive(Debug)]
    pub struct Connected;
    #[derive(Debug)]
    pub struct Subscribed;
    #[derive(Debug)]
    pub struct Paused;
}

// Consumer parameterized by state
#[derive(Debug, Clone)]
pub struct Consumer<State> {
    topic: Option<TopicId>,
    consumer_id: ConsumerId,
    connection_info: Option<ConnectionInfo>,
    state: PhantomData<State>,
}

/// Type aliases for consumer states
pub type DisconnectedConsumer = Consumer<states::Disconnected>;
pub type ConnectedConsumer = Consumer<states::Connected>;
pub type SubscribedConsumer = Consumer<states::Subscribed>;
pub type PausedConsumer = Consumer<states::Paused>;

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub broker_address: String,
    pub consumer_group: Option<String>,
}

impl ConnectionInfo {
    pub fn new(broker_address: String, consumer_group: Option<String>) -> Self {
        Self {
            broker_address,
            consumer_group,
        }
    }
}

impl Consumer<states::Disconnected> {
    pub fn new(consumer_id: ConsumerId) -> Self {
        Self {
            topic: None,
            consumer_id,
            connection_info: None,
            state: PhantomData,
        }
    }
    // A disconnected consumer only implements connect
    pub fn connect(self, connection_info: ConnectionInfo) -> error::Result<ConnectedConsumer> {
        if connection_info.broker_address.is_empty() {
            return Err(CustomError("Empty broker address".to_string()));
        }
        Ok(Consumer {
            topic: self.topic,
            consumer_id: self.consumer_id,
            connection_info: Some(connection_info),
            state: PhantomData,
        })
    }
}

// A connected consumer can transition to 2 states: Subscribed or Disconnected
impl Consumer<states::Connected> {
    pub fn subscribe(mut self, topic: TopicId) -> error::Result<SubscribedConsumer> {
        self.topic = Some(topic);
        Ok(Consumer {
            topic: self.topic,
            consumer_id: self.consumer_id,
            connection_info: self.connection_info,
            state: PhantomData,
        })
    }
    pub fn disconnect(self) -> DisconnectedConsumer {
        Consumer {
            topic: None,
            consumer_id: self.consumer_id,
            connection_info: None,
            state: PhantomData,
        }
    }
}

// A subscribed consumer can transition to 2 states: Paused or Connected
impl Consumer<states::Subscribed> {
    pub fn pause(self) -> PausedConsumer {
        Consumer {
            topic: self.topic,
            consumer_id: self.consumer_id,
            connection_info: self.connection_info,
            state: PhantomData,
        }
    }
    pub fn unsubscribe(mut self) -> ConnectedConsumer {
        self.topic = None;
        Consumer {
            topic: self.topic,
            consumer_id: self.consumer_id,
            connection_info: self.connection_info,
            state: PhantomData,
        }
    }
}

// A paused consumer can only transition back to Subscribed
impl Consumer<states::Paused> {
    pub fn resume(self) -> SubscribedConsumer {
        Consumer {
            topic: self.topic,
            consumer_id: self.consumer_id,
            connection_info: self.connection_info,
            state: PhantomData,
        }
    }
}

pub fn type_state_pattern() -> Result<(), CustomError> {
    let consumer_id = ConsumerId::new("test-consumer")?;
    let topic = TopicId::new("events")?;
    // Create disconnected consumer
    let consumer = Consumer::new(consumer_id);
    println!("DisconnectedConsumer: {:#?}", consumer);
    // consumer.subscribe(topic); // Error: not available on Disconnected
    // Connect with connection info
    let connection_info = ConnectionInfo::new("localhost:9092".to_string(), None);
    let connected = consumer.connect(connection_info)?;
    println!("ConnectedConsumer: {:#?}", connected);
    // Subscribe to topic
    let subscribed = connected.subscribe(topic)?;
    // Can pause and resume
    let paused = subscribed.pause();
    let subscribed = paused.resume();
    // Unsubscribe and disconnect
    let connected = subscribed.unsubscribe();
    let _disconnected = connected.disconnect();
    Ok(())
}
