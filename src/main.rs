mod closures;
mod error;
mod iterator_chaining;
mod new_type;
mod parse_not_validate;
mod pipeline;
mod smart_constructors;
mod type_classes;
mod typestate;

use closures::closures_pattern;
use iterator_chaining::{
    SubscriptionEvent, SubscriptionType, analyze_recent_subscriptions, process_subscription_events,
};
use new_type::{ConsumerId, MessageId, TopicId};
use parse_not_validate::{BrokerConfigBuilder, ConfigError};
use pipeline::pipeline_processing_pattern;
use smart_constructors::smart_constructors_pattern;
use type_classes::type_classes_pattern;
use typestate::{
    ConnectedConsumer, ConnectionInfo, Consumer, DisconnectedConsumer, PausedConsumer,
    SubscribedConsumer,
};

use crate::error::CustomError;

// message does not accept u64, topic only accepts TopicId
fn new_type_pattern(message: MessageId, topic: TopicId) -> Result<(), CustomError> {
    println!("Pushing message {} to {}", message, topic);
    Ok(())
}

// BrokerConfigBuilder struct holds ows types already validated: PositiveU32, NonZeroUsize, NetworkPort
fn parse_not_validate_pattern(
    max_connections: u32,
    buffer_size: usize,
    port: u16,
) -> Result<(), ConfigError> {
    let config = BrokerConfigBuilder::new()
        .max_connections(max_connections)?
        .buffer_size(buffer_size)?
        .port(port)?
        .build()?;
    println!(
        "BrokerConfig: max_connections={}, buffer_size={}, port={}",
        config.max_connections(),
        config.buffer_size(),
        config.port()
    );
    Ok(())
}

fn type_state_pattern() -> Result<(), CustomError> {
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

fn main() {
    let topic_id = TopicId::new("my_topic").unwrap();
    new_type_pattern(MessageId::new(42), topic_id).unwrap();
    parse_not_validate_pattern(100, 1024, 8080).unwrap();
    type_state_pattern().unwrap();
    let events = vec![
        SubscriptionEvent {
            subscription_type: SubscriptionType::Subscribe,
            topic: "topic1".to_string(),
            user_id: 1,
            timestamp: 1000,
        },
        SubscriptionEvent {
            subscription_type: SubscriptionType::Subscribe,
            topic: "topic2".to_string(),
            user_id: 2,
            timestamp: 2000,
        },
        SubscriptionEvent {
            subscription_type: SubscriptionType::Unsubscribe,
            topic: "topic1".to_string(),
            user_id: 1,
            timestamp: 3000,
        },
        SubscriptionEvent {
            subscription_type: SubscriptionType::Invalid,
            topic: "".to_string(),
            user_id: 0,
            timestamp: 4000,
        },
    ];
    println!(
        "Subscription stats: {:?}",
        process_subscription_events(events.clone())
    );
    println!(
        "Recent subscriptions: {:?}",
        analyze_recent_subscriptions(events, 1500)
    );
    type_classes_pattern();
    closures_pattern();
    match pipeline_processing_pattern() {
        Ok(processed_messages) => {
            println!("Pipeline processed messages: {:#?}", processed_messages);
        }
        Err(e) => {
            eprintln!("Error processing pipeline: {:?}", e);
        }
    };
    smart_constructors_pattern(100, "topic".to_string(), 1);
}
