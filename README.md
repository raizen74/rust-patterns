# Rust Patterns

This crate demonstrates Rust patterns for making invalid states harder to represent and for composing data processing: the NewType pattern, the TypeState pattern, validation when parsing or constructing configuration, and iterator chaining with domain-specific operations.

## NewType

NewType wraps a primitive or another type in a distinct struct. Although `TopicId` and `ConsumerId` each contain a `String`, Rust treats them as different types. A function that accepts a `TopicId` therefore cannot be called with an arbitrary `String` by accident. Their fields are private, so callers create them through the constructors, which validate that the value is non-empty; `TopicId` also limits the name to 128 bytes.

```rust
let topic = TopicId::new("events")?;
let consumer = ConsumerId::new("worker-1")?;
```

`MessageId` is another wrapper, around `u64`. It demonstrates type distinction, but its constructor currently accepts every `u64` without additional validation.

## TypeState

TypeState encodes an object's lifecycle state in its type. Here, `Consumer<State>` uses marker types such as `Disconnected`, `Connected`, `Subscribed`, and `Paused`. Each parameterized type by state implements the methods valid for that state:

```rust
let consumer = Consumer::new(consumer_id); // Disconnected
let connected = consumer.connect(connection_info)?;
let subscribed = connected.subscribe(topic)?;
let paused = subscribed.pause();
let subscribed = paused.resume();
let connected = subscribed.unsubscribe();
let disconnected = connected.disconnect();
```

For example, `subscribe` is implemented for `Consumer<Connected>`, not `Consumer<Disconnected>`. Trying to subscribe before connecting is consequently a compile-time error. `PhantomData<State>` carries the state in the type without storing a runtime state value. Some checks still happen at runtime: `connect` returns an error if the broker address is empty.

This approach keeps invalid transitions out of the API, but operations are attached to concrete state-specific `impl` blocks. When multiple states share a capability, code that uses that capability generically needs an abstraction describing it. The next pattern adds that abstraction with traits.

## Type Classes

Type Classes are a combination of a Trait and an Associated Type. In this example, “type classes” are modeled with Rust traits that describe capabilities a subscription state supports. `Activatable`, `Suspendable`, and `Cancellable` each define an operation, and their associated `Output` type records the state produced by a successful transition.

For example, both `Subscription<Pending>` and `Subscription<Suspended>` implement `Activatable`, and both transition to `Subscription<Active>`. Both `Subscription<Active>` and `Subscription<Suspended>` implement `Cancellable`, each producing `Subscription<Cancelled>`. Code can therefore be generic over the capability instead of being written separately for each input state:

```rust
fn cancel_subscription_with_audit<S>(
    subscription: Subscription<S>,
    reason: String,
) -> Subscription<Cancelled>
where
    Subscription<S>: Cancellable<Output = Subscription<Cancelled>>,
{
    println!("Auditing cancellation of subscription {}: {}", subscription.id, reason);
    subscription.cancel(reason)
}
```

Compared with the basic TypeState example, this makes shared operations explicit and lets **generic functions accept any state that implements the required capability** via trait bounds. The compiler still checks which transitions are supported, and the associated output keeps the resulting state in the type. It does not eliminate state-specific implementations: each supported state still needs an implementation of its capability, and a new state must opt into the capabilities it supports. The improvement is that shared behavior and generic algorithms can be expressed in terms of those capabilities rather than duplicating callers for each state.

## Validation at the Parsing Stage

Validate input as it enters the program, then keep validated values in dedicated types. In this example, `PositiveU32`, `NetworkPort`, and `NonZeroUsize` represent constraints used by `BrokerConfig`. The builder accepts raw values, validates them as each setting is supplied, and `build` rejects settings that were never provided.

```rust
let config = BrokerConfigBuilder::new()
    .max_connections(100)?
    .buffer_size(1024)?
    .port(8080)?
    .build()?;
```

The constructors enforce the rules in this example: maximum connections must be positive, buffer size must be non-zero, and the port must be greater than 1023. After construction, `BrokerConfig` stores the validated types rather than the original raw integers, so its getters can return values without repeating those checks. Failures are returned as `ConfigError`.

## Iterator Chaining

`SubscriptionProcessing` is an extension trait that gives iterators over `SubscriptionEvent` domain-specific operations: `recent_events` filters by timestamp, `valid_subscriptions` keeps valid subscribe events, and `count_by_topic` aggregates event counts into a `HashMap`.

The blanket implementation applies the trait to every type `I` that implements `Iterator<Item = SubscriptionEvent>`:

```rust
impl<I> SubscriptionProcessing for I
where
    I: Iterator<Item = SubscriptionEvent>,
{
    // Trait methods are implemented for all matching iterators.
}
```

This means callers can use the methods on any matching iterator without defining a wrapper type or writing a separate trait implementation for each iterator. The filtering methods return opaque `impl Iterator` values, so they compose lazily without boxing. `count_by_topic` is a terminal operation: it consumes the iterator and folds its events into a map.

```rust
let counts = events
    .into_iter()
    .recent_events(cutoff_timestamp)
    .valid_subscriptions()
    .count_by_topic();
```

The blanket implementation requires only the iterator item type; it does not require the iterator to be `'static`, so iterators borrowing data can also use the extension methods when their item type matches.

## Run

```sh
cargo run
```