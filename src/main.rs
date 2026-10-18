mod closures;
mod error;
mod iterator_chaining;
mod new_type;
mod parse_not_validate;
mod pipeline;
mod smart_constructors;
mod type_classes;
mod typestate;
mod pattern_matching_pattern;

use pattern_matching_pattern::pattern_matching_pattern;
use closures::closures_pattern;
use iterator_chaining::iterator_chaining_pattern;
use new_type::{MessageId, TopicId, new_type_pattern};
use parse_not_validate::parse_not_validate_pattern;
use pipeline::pipeline_processing_pattern;
use smart_constructors::smart_constructors_pattern;
use type_classes::type_classes_pattern;
use typestate::type_state_pattern;

fn main() {
    new_type_pattern(MessageId::new(42), TopicId::new("my_topic").unwrap()).unwrap();
    parse_not_validate_pattern(100, 1024, 8080).unwrap();
    type_state_pattern().unwrap();
    type_classes_pattern();
    pattern_matching_pattern();
    iterator_chaining_pattern();
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
