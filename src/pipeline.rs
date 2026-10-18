use crate::error::{CustomError, Result};

#[derive(Debug, Clone)]
struct RawMessage {
    data: Vec<u8>,
    headers: std::collections::HashMap<String, String>,
}

#[derive(Debug)]
pub struct ProcessedMessage {
    id: u64,
    topic: String,
    payload: Vec<u8>,
    timestamp: std::time::SystemTime,
}

impl RawMessage {
    fn validate(self) -> Result<ValidatedMessage> {
        if self.data.is_empty() {
            return Err(CustomError("Empty message".to_string()));
        }
        // We could also validate headers here
        Ok(ValidatedMessage(self))
    }
}
struct ValidatedMessage(RawMessage);

impl ValidatedMessage {
    fn transform(self) -> Result<TransformedMessage> {
        let topic = self
            .0
            .headers
            .get("topic")
            .ok_or_else(|| CustomError("Missing topic".to_string()))?
            .clone();
        Ok(TransformedMessage {
            topic,
            data: self.0.data,
            headers: self.0.headers,
        })
    }
}
struct TransformedMessage {
    topic: String,
    data: Vec<u8>,
    headers: std::collections::HashMap<String, String>,
}

impl TransformedMessage {
    fn enrich(self) -> Result<ProcessedMessage> {
        use std::time::SystemTime;
        let id = self
            .headers
            .get("message_id")
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| 1);
        Ok(ProcessedMessage {
            id,
            topic: self.topic,
            payload: self.data,
            timestamp: SystemTime::now(),
        })
    }
}
// Error composition, each stage can fail fast
fn process_message_batch(messages: Vec<RawMessage>) -> Result<Vec<ProcessedMessage>> {
    messages
        .into_iter()
        .map(|msg| -> Result<ProcessedMessage> {
            let validated = msg.validate()?;
            let transformed = validated.transform()?;
            Ok(transformed.enrich()?)
        })
        .collect()
}

fn process_message_batch_with_failure(
    messages: Vec<RawMessage>,
) -> (Vec<ProcessedMessage>, Vec<CustomError>) {
    let mut processed = Vec::new();
    let mut errors = Vec::new();

    for message in messages {
        let result = message
            .validate()
            .and_then(|validated| validated.transform())
            .and_then(|transformed| transformed.enrich());

        match result {
            Ok(message) => processed.push(message),
            Err(error) => errors.push(error),
        }
    }

    (processed, errors)
}

pub fn pipeline_processing_pattern() -> Result<Vec<ProcessedMessage>> {
    let messages = vec![
        RawMessage {
            data: b"Hello, World!".to_vec(),
            headers: [("topic".to_string(), "greetings".to_string())]
                .iter()
                .cloned()
                .collect(),
        },
        RawMessage {
            data: b"".to_vec(),
            headers: [("topic".to_string(), "errors".to_string())]
                .iter()
                .cloned()
                .collect(),
        },
    ];
    let processed_messages_with_failure = process_message_batch_with_failure(messages.clone());
    println!("Processed messages: {:#?}", processed_messages_with_failure);
    let processed_messages = process_message_batch(messages.clone())?;
    println!("Processed messages: {:#?}", processed_messages);
    Ok(processed_messages)
}
