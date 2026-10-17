use crate::error::{CustomError, Result as CustomResult};
use std::marker::PhantomData;

#[derive(Debug)]
struct Pending;
#[derive(Debug)]
struct Active;
#[derive(Debug)]
struct Suspended;
#[derive(Debug)]
struct Cancelled;

#[derive(Debug)]
struct Subscription<S> {
    id: u64,
    user_id: u64,
    topic: String,
    created_at: u64,
    _state: PhantomData<S>, // PhantomData<S> field tells the compiler about the state parameter without storing anything at runtime
}

// Type class for subscriptions that can be activated
trait Activatable {
    type Output;
    fn activate(self) -> Result<Self::Output, ActivationError>;
}
// Type class for subscriptions that can be suspended
trait Suspendable {
    type Output;
    fn suspend(self, reason: String) -> Self::Output;
}
// Type class for subscriptions that can be cancelled
trait Cancellable {
    type Output;
    fn cancel(self, reason: String) -> Self::Output;
}
#[derive(Debug)]
enum ActivationError {
    InvalidUser,
    TopicNotFound,
    QuotaExceeded,
}

impl Subscription<Pending> {
    fn new(id: u64, user_id: u64, topic: String) -> Self {
        Self {
            id,
            user_id,
            topic,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            _state: PhantomData,
        }
    }
}

impl Activatable for Subscription<Pending> {
    type Output = Subscription<Active>;
    fn activate(self) -> Result<Self::Output, ActivationError> {
        if self.user_id == 0 {
            return Err(ActivationError::InvalidUser);
        }
        if self.topic.is_empty() {
            return Err(ActivationError::TopicNotFound);
        }
        Ok(Subscription {
            id: self.id,
            user_id: self.user_id,
            topic: self.topic,
            created_at: self.created_at,
            _state: PhantomData,
        })
    }
}

impl Activatable for Subscription<Suspended> {
    type Output = Subscription<Active>;
    fn activate(self) -> Result<Self::Output, ActivationError> {
        if self.user_id == 0 {
            return Err(ActivationError::InvalidUser);
        }
        if self.topic.is_empty() {
            return Err(ActivationError::TopicNotFound);
        }
        Ok(Subscription {
            id: self.id,
            user_id: self.user_id,
            topic: self.topic,
            created_at: self.created_at,
            _state: PhantomData,
        })
    }
}

impl Cancellable for Subscription<Suspended> {
    type Output = Subscription<Cancelled>;
    fn cancel(self, reason: String) -> Self::Output {
        println!("Cancelling Subscription<Suspended>, reason: {reason}");
        Subscription {
            id: self.id,
            user_id: self.user_id,
            topic: self.topic,
            created_at: self.created_at,
            _state: PhantomData,
        }
    }
}

// An active subscription can be suspended and cancelled
impl Suspendable for Subscription<Active> {
    type Output = Subscription<Suspended>;
    fn suspend(self, reason: String) -> Self::Output {
        println!("Suspending Subscription<Active>, reason: {reason}");
        Subscription {
            id: self.id,
            user_id: self.user_id,
            topic: self.topic,
            created_at: self.created_at,
            _state: PhantomData,
        }
    }
}

impl Cancellable for Subscription<Active> {
    type Output = Subscription<Cancelled>;
    fn cancel(self, reason: String) -> Self::Output {
        println!("Cancelling Subscription<Active>, reason: {reason}");
        Subscription {
            id: self.id,
            user_id: self.user_id,
            topic: self.topic,
            created_at: self.created_at,
            _state: PhantomData,
        }
    }
}

// generic functions that work with any type satisfying certain constraints
fn cancel_subscription_with_audit<S>(
    subscription: Subscription<S>,
    reason: String,
) -> Subscription<Cancelled>
where
    Subscription<S>: Cancellable<Output = Subscription<Cancelled>>,
{
    println!(
        "Auditing cancellation of subscription {}: {}",
        subscription.id, reason
    );
    subscription.cancel(reason)
}

#[derive(Debug)]
struct SubscriptionManager {
    active_subscriptions: Vec<Subscription<Active>>,
    suspended_subscriptions: Vec<Subscription<Suspended>>,
    cancelled_subscriptions: Vec<Subscription<Cancelled>>,
}

impl SubscriptionManager {
    fn new() -> Self {
        Self {
            active_subscriptions: Vec::new(),
            suspended_subscriptions: Vec::new(),
            cancelled_subscriptions: Vec::new(),
        }
    }
    fn create_subscription(&mut self, user_id: u64, topic: String) -> Result<u64, ActivationError> {
        let pending = Subscription::new(1, user_id, topic);
        let active = pending.activate()?;
        self.active_subscriptions.push(active);
        println!("{self:#?}");
        Ok(1)
    }

    fn suspend_subscription(&mut self, id: u64, reason: String) -> CustomResult<()> {
        if let Some(pos) = self.active_subscriptions.iter().position(|s| s.id == id) {
            let subscription = self.active_subscriptions.remove(pos);
            let suspended = subscription.suspend(reason);
            self.suspended_subscriptions.push(suspended);
            println!("{self:#?}");
            Ok(())
        } else {
            Err(CustomError(
                "Subscription not found or not active".to_string(),
            ))
        }
    }
    fn cancel_subscription(&mut self, id: u64, reason: String) -> CustomResult<()> {
        // Try to cancel from active subscriptions
        if let Some(pos) = self.active_subscriptions.iter().position(|s| s.id == id) {
            let subscription = self.active_subscriptions.remove(pos);
            let cancelled = subscription.cancel(reason);
            self.cancelled_subscriptions.push(cancelled);
            println!("{self:#?}");
            return Ok(());
        }
        // Try to cancel from suspended subscriptions
        if let Some(pos) = self.suspended_subscriptions.iter().position(|s| s.id == id) {
            let subscription = self.suspended_subscriptions.remove(pos);
            let cancelled = subscription.cancel(reason);
            self.cancelled_subscriptions.push(cancelled);
            println!("{self:#?}");
            return Ok(());
        }
        Err(CustomError("Subscription not found".to_string()))
    }
}

pub fn type_classes_pattern() {
    let mut manager = SubscriptionManager::new();
    let subs_id = manager.create_subscription(99, "foo".to_string()).unwrap();
    manager
        .suspend_subscription(subs_id, "Suspending subs_id".to_string())
        .unwrap();
    manager
        .cancel_subscription(subs_id, "Cancelling subs_id".to_string())
        .unwrap();
}
