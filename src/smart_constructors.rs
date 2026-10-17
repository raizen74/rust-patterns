use crate::error::CustomError;

#[derive(Debug)]
struct SubscriptionRequest {
    user_id: u64,
    topic: String,
    qos_level: u8,
}

#[derive(Debug)]
pub struct ActiveSubscription {
    id: u64,
    user_id: u64,
    topic: String,
    qos_level: u8,
    created_at: std::time::SystemTime,
}

impl SubscriptionRequest {
    fn validate(self) -> Result<ValidatedSubscriptionRequest, CustomError> {
        if self.user_id == 0 {
            return Err(CustomError("Invalid user ID".to_string()));
        }
        if self.topic.is_empty() || self.topic.len() > 255 {
            return Err(CustomError("Invalid topic".to_string()));
        }
        if self.qos_level > 2 {
            return Err(CustomError("Invalid QoS level".to_string()));
        }
        Ok(ValidatedSubscriptionRequest(self))
    }
}
struct ValidatedSubscriptionRequest(SubscriptionRequest);

impl TryFrom<ValidatedSubscriptionRequest> for ActiveSubscription {
    type Error = CustomError;
    fn try_from(validated: ValidatedSubscriptionRequest) -> Result<Self, Self::Error> {
        let req = validated.0;
        Ok(ActiveSubscription {
            id: 1,
            user_id: req.user_id,
            topic: req.topic,
            qos_level: req.qos_level,
            created_at: std::time::SystemTime::now(),
        })
    }
}

fn process_subscription_requests(
    requests: Vec<SubscriptionRequest>,
) -> (Vec<ActiveSubscription>, Vec<CustomError>) {
    requests
        .into_iter()
        .filter_map(|req| {
            // Pre-filter obviously invalid requests
            if req.topic.contains("..") || req.topic.starts_with('/') {
                None
            } else {
                Some(req)
            }
        })
        .map(|req| req.validate())
        .filter_map(|result| result.ok())
        // Keep only valid requests
        .map(|validated| validated.try_into())
        .fold(
            (Vec::new(), Vec::new()),
            |(mut success, mut errors), result| {
                match result {
                    Ok(subscription) => success.push(subscription),
                    Err(error) => errors.push(error),
                }
                (success, errors)
            },
        )
}

pub fn smart_constructors_pattern(
    user_id: u64,
    topic: String,
    qos_level: u8,
) -> (Vec<ActiveSubscription>, Vec<CustomError>) {
    let requests = vec![SubscriptionRequest {
        user_id,
        topic,
        qos_level,
    }];
    let results = process_subscription_requests(requests);
    println!("{results:#?}");
    results
}
