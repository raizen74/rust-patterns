mod error;
mod new_type;

use new_type::{MessageId, TopicId};

use crate::error::CustomError;

// message does not accept u64, topic only accepts TopicId
fn new_type_pattern(message: MessageId, topic: TopicId) -> Result<(), CustomError> {
    println!("Pushing message {} to {}", message, topic);
    Ok(())
}

fn main() {
    let topic_id = TopicId::new("my_topic").unwrap();
    println!("topic_id: {topic_id:?}");
    new_type_pattern(MessageId::new(42), topic_id).unwrap();
}
