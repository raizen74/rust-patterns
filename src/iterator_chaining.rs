use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SubscriptionEvent {
    pub user_id: u64,
    pub topic: String,
    pub timestamp: u64,
    pub subscription_type: SubscriptionType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubscriptionType {
    Subscribe,
    Unsubscribe,
    Invalid,
}

#[derive(Debug)]
pub struct SubscriptionStats {
    total_valid: usize,
    subscriptions_by_topic: HashMap<String, usize>,
}

impl SubscriptionEvent {
    fn is_valid(&self) -> bool {
        self.subscription_type != SubscriptionType::Invalid
            && !self.topic.is_empty()
            && self.user_id > 0
    }
    fn is_subscription(&self) -> bool {
        self.subscription_type == SubscriptionType::Subscribe
    }
}

pub fn process_subscription_events(events: Vec<SubscriptionEvent>) -> SubscriptionStats {
    let valid_events: Vec<_> = events
        .into_iter()
        .filter(|event| event.is_valid())
        .collect();
    let subscriptions_by_topic = valid_events
        .iter()
        .filter(|event| event.is_subscription())
        .map(|event| &event.topic)
        .fold(HashMap::new(), |mut acc, topic| {
            *acc.entry(topic.clone()).or_insert(0) += 1;
            acc
        });
    SubscriptionStats {
        total_valid: valid_events.len(),
        subscriptions_by_topic,
    }
}

// Domain-specific COMBINATORS
pub trait SubscriptionProcessing {
    fn valid_subscriptions(self) -> impl Iterator<Item = SubscriptionEvent>;
    fn count_by_topic(self) -> HashMap<String, usize>;
    fn recent_events(self, cutoff_timestamp: u64) -> impl Iterator<Item = SubscriptionEvent>;
}

// This adds our custom operations to any iterator producing SubscriptionEvent.
impl<I> SubscriptionProcessing for I
where
    I: Iterator<Item = SubscriptionEvent>,
{
    fn valid_subscriptions(self) -> impl Iterator<Item = SubscriptionEvent> {
        self.filter(|event| event.is_valid())
            .filter(|event| event.is_subscription())
    }
    fn count_by_topic(self) -> HashMap<String, usize> {
        self.fold(HashMap::new(), |mut acc, event| {
            *acc.entry(event.topic).or_insert(0) += 1;
            acc
        })
    }
    fn recent_events(self, cutoff_timestamp: u64) -> impl Iterator<Item = SubscriptionEvent> {
        self.filter(move |event| event.timestamp >= cutoff_timestamp)
    }
}

pub fn analyze_recent_subscriptions(
    events: Vec<SubscriptionEvent>,
    cutoff_timestamp: u64,
) -> HashMap<String, usize> {
    events
        .into_iter()
        .recent_events(cutoff_timestamp)
        .valid_subscriptions()
        .count_by_topic()
}
