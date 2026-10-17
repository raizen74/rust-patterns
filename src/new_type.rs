// src/new_type.rs
use crate::error::{CustomError, Result};

use std::fmt;
/// Topic identifier for tracking individual topics
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TopicId(String);

/// Message identifier for tracking individual messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MessageId(u64);

impl TopicId {
    pub fn new(topic: impl AsRef<str>) -> Result<Self> {
        let topic = topic.as_ref();
        if topic.is_empty() {
            return Err(CustomError::Broker(
                "Topic name cannot be empty".to_string(),
            ));
        }
        if topic.len() > 128 {
            return Err(CustomError::Broker(format!(
                "Topic name too long: {} chars (max 128)",
                topic.len()
            )));
        }
        Ok(TopicId(topic.to_string()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl MessageId {
    pub fn new(id: u64) -> Self {
        MessageId(id)
    }
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for TopicId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
