mod error;
mod new_type;
mod parse_not_validate;

use new_type::{MessageId, TopicId};
use parse_not_validate::{BrokerConfigBuilder, ConfigError};

use crate::error::CustomError;

// message does not accept u64, topic only accepts TopicId
fn new_type_pattern(message: MessageId, topic: TopicId) -> Result<(), CustomError> {
    println!("Pushing message {} to {}", message, topic);
    Ok(())
}

// BrokerConfigBuilder struct holds ows types already validated: PositiveU32, NonZeroUsize, NetworkPort
fn parse_not_validate_pattern(max_connections: u32, buffer_size: usize, port: u16) -> Result<(), ConfigError> {
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

fn main() {
    let topic_id = TopicId::new("my_topic").unwrap();
    new_type_pattern(MessageId::new(42), topic_id).unwrap();
    parse_not_validate_pattern(100, 1024, 8080).unwrap();
}
