use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Message {
    pub topic: String,
    pub key: Option<String>,
    pub value: Vec<u8>,
    pub timestamp: Instant,
}

impl Message {
    pub fn new(topic: impl Into<String>, key: Option<String>, value: impl Into<Vec<u8>>) -> Self {
        Self {
            topic: topic.into(),
            key,
            value: value.into(),
            timestamp: Instant::now(),
        }
    }
}

struct MessageFilter<F> {
    predicate: F,
    name: String,
}
impl<F> MessageFilter<F>
where
    F: Fn(&Message) -> bool,
{
    fn new(name: String, predicate: F) -> Self {
        Self { predicate, name }
    }
    fn matches(&self, message: &Message) -> bool {
        (self.predicate)(message)
    }
    fn and<G>(self, other: MessageFilter<G>) -> MessageFilter<impl Fn(&Message) -> bool>
    where
        G: Fn(&Message) -> bool,
    {
        let combined_name = format!("{} AND {}", self.name, other.name);
        MessageFilter::new(combined_name, move |msg| {
            self.matches(msg) && other.matches(msg)
        })
    }
}

/// Returns a tuple of three filters with complex impl Trait types.
/// The type complexity is intentional to demonstrate returning closures with
///different predicates.
#[allow(clippy::type_complexity)]
pub fn create_message_filters() -> (
    MessageFilter<impl Fn(&Message) -> bool>,
    MessageFilter<impl Fn(&Message) -> bool>,
    // MessageFilter<impl Fn(&Message) -> bool>,
) {
    let large_filter = MessageFilter::new("Large Messages".to_string(), |msg: &Message| {
        msg.value.len() > 1000
    });
    let system_filter = MessageFilter::new("System Topics".to_string(), |msg: &Message| {
        msg.topic.starts_with("system.")
    });
    // let recent_filter = MessageFilter::new("Recent Messages".to_string(), move |msg: &Message| {
    //     let now = Instant::now();
    //     now.saturating_sub(msg.timestamp) <= 3600
    // });
    (large_filter, system_filter)
}

pub fn closures_pattern() {
    let filter1 = MessageFilter::new("Topic is 'events'".to_string(), |msg: &Message| {
        msg.topic == "events"
    });
    // true when msg.key dereferenced is equal to "user123"
    let filter2 = MessageFilter::new("Key is 'user123'".to_string(), |msg: &Message| {
        msg.key.as_deref() == Some("user123")
    });

    let combined_filter = filter1.and(filter2);

    let test_message = Message::new("events", Some("user123".to_string()), vec![1, 2, 3]);

    if combined_filter.matches(&test_message) {
        println!("Message matches combined filter: {}", combined_filter.name);
    } else {
        println!(
            "Message does NOT match combined filter: {}",
            combined_filter.name
        );
    }
}
