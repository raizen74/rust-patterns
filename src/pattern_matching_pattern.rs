#[derive(Debug, Clone)]
enum MessageContent {
    Text(String),
    Binary(Vec<u8>),
    Empty,
}
#[derive(Debug, Clone)]
enum Priority {
    Low,
    Normal,
    High,
    Critical,
}
#[derive(Debug, Clone)]
struct RichMessage {
    id: u64,
    topic: String,
    content: MessageContent,
    priority: Priority,
    timestamp: u64,
}
#[derive(Debug)]
enum ProcessingResult {
    Processed,
    Queued,
    Rejected(String),
    RequiresAuth(String),
}

impl RichMessage {
    fn route_message(&self) -> ProcessingResult {
        match (&self.priority, &self.content, self.topic.as_str()) {
            // Critical messages always get immediate processing
            (Priority::Critical, _, _) => {
                println!("Processing critical message immediately");
                ProcessingResult::Processed
            }
            // High priority text messages to system topics
            (Priority::High, MessageContent::Text(text), topic) if topic.starts_with("system.") => {
                if text.len() > 1000 {
                    ProcessingResult::Queued
                } else {
                    ProcessingResult::Processed
                }
            }
            // Binary messages over 10KB get queued
            (_, MessageContent::Binary(data), _) if data.len() > 10240 => ProcessingResult::Queued,
            // Empty messages are rejected
            (_, MessageContent::Empty, _) => ProcessingResult::Rejected(
                "Empty content not
allowed"
                    .to_string(),
            ),
            // Default case
            _ => ProcessingResult::Processed,
        }
    }
}

pub fn pattern_matching_pattern() {
    let rich_message = RichMessage {
        id: 1,
        topic: "Hello World".to_string(),
        content: MessageContent::Binary(vec![8]),
        priority: Priority::Critical,
        timestamp:22
    };
    let res = rich_message.route_message();
    println!("Pattern matching pattern result: {res:#?}")
}
