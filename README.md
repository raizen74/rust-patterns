

# Rust Patterns <!-- omit from toc -->

This crate demonstrates Rust patterns for making invalid states harder to represent and for composing data processing: the NewType pattern, the TypeState pattern, smart constructors, validation when parsing or constructing configuration, pattern matching, iterator chaining, closure composition, and staged message processing.

- [NewType](#newtype)
- [TypeState](#typestate)
- [Type Classes](#type-classes)
- [Validation at the Parsing Stage](#validation-at-the-parsing-stage)
- [Smart Constructors](#smart-constructors)
- [Iterator Chaining](#iterator-chaining)
- [Closure Composition](#closure-composition)
- [Message Processing Pipeline](#message-processing-pipeline)
- [Pattern Matching and Exhaustiveness](#pattern-matching-and-exhaustiveness)
- [Run](#run)

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

Type Classes are a combination of a Trait and an Associated Type. In this example, “type classes” are modeled with Rust traits that describe capabilities a generic `Subscription<S>` supports. `Activatable`, `Suspendable`, and `Cancellable` each define an operation, and their **Associated Type** `Output` records the state produced by a successful transition.

For example, both `Subscription<Pending>` and `Subscription<Suspended>` implement `Activatable`, and both transition to `Subscription<Active>`. Both `Subscription<Active>` and `Subscription<Suspended>` implement `Cancellable`, each producing `Subscription<Cancelled>`. Code can therefore be **generic** over the capability `Activatable`, `Suspendable`, and `Cancellable` instead of being written separately for each input state:

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

Compared with the basic TypeState example, this makes shared operations explicit and lets **generic functions accept any state that implements the required capability** via trait bounds. The compiler still checks which transitions are supported, and the associated `Output` type keeps the resulting state in the type. It does not eliminate state-specific implementations: each supported state still needs an implementation of its capability, and a new state must opt into the capabilities it supports. The improvement is that shared behavior and **generic algorithms can be expressed in terms of those capabilities** rather than duplicating callers for each state.

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

## Smart Constructors

The smart-constructor pattern hides the representation of a valid value and exposes a constructor that validates raw input before producing it. In `smart_constructors.rs`, `SubscriptionRequest::validate` consumes a request and returns `Result<ValidatedSubscriptionRequest, CustomError>`. The validated wrapper can then be converted into an `ActiveSubscription` with `TryFrom`. This makes validation a required step before creating the active value.

The request-processing function combines iterator adapters with `Option` and `Result`:

```rust
requests
    .into_iter()
    .filter_map(|request| {
        if request.topic.contains("..") || request.topic.starts_with('/') {
            None
        } else {
            Some(request)
        }
    })
    .map(|request| request.validate())
    .filter_map(|result| result.ok())
    .map(ActiveSubscription::try_from)
    // fold the conversion results into successes and errors
```

The first `filter_map` uses `Option` to exclude topics that fail the quick pre-filter. Validation returns a `Result`; `Result::ok` converts it to an `Option`, allowing the next iterator stage to keep only successful validations. `TryFrom` also returns a `Result`, and the final `fold` accumulates its successful subscriptions and conversion errors into separate vectors.

There is an important consequence in the current implementation: `.filter_map(|result| result.ok())` discards validation errors, and the pre-filter also drops rejected requests without recording why. Thus the returned error vector contains conversion errors only; validation failures do not reach the `fold`. To preserve validation errors while composing the `Result` steps, chain validation and conversion with `and_then` and fold those results directly:

```rust
.map(|request| {
    request
        .validate()
        .and_then(ActiveSubscription::try_from)
})
.fold((Vec::new(), Vec::new()), |(mut successes, mut errors), result| {
    match result {
        Ok(subscription) => successes.push(subscription),
        Err(error) => errors.push(error),
    }
    (successes, errors)
})
```

This keeps validation and conversion failures as `Err` values through the iterator chain, so the fold can collect them rather than silently dropping them. The `Option` pre-filter can still intentionally skip requests, or it can be changed to produce an explicit error if those rejections should also be reported.

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

## Closure Composition

The closures pattern wraps predicates in `MessageFilter<F>`, where `F` is the predicate's concrete closure type and implements `Fn(&Message) -> bool`. The `and` method composes two filters by capturing them in a new closure. The resulting filter matches only when both predicates match, and its descriptive name combines the names of the original filters.

```rust
let topic_filter = MessageFilter::new("Topic is events".to_string(), |message: &Message| {
    message.topic == "events"
});
let key_filter = MessageFilter::new("Key is user123".to_string(), |message: &Message| {
    message.key.as_deref() == Some("user123")
});
let combined = topic_filter.and(key_filter);

if combined.matches(&message) {
    // Both predicates matched.
}
```

Because each closure has its own compiler-generated type, the generic `MessageFilter<F>` retains the concrete type without boxing. `and` returns another filter with an opaque `impl Fn` predicate, hiding the nested closure type while keeping static dispatch. Its use of `&&` also short-circuits: the second predicate runs only if the first matches.

## Message Processing Pipeline

The pipeline models each processing stage with a different type: `RawMessage` is validated into `ValidatedMessage`, transformed into `TransformedMessage`, then enriched into `ProcessedMessage`. Each stage consumes its input and returns `Result<NextStage, CustomError>`, so invalid or incomplete messages stop progressing through the pipeline.

This is a form of **monadic composition**: `Result::and_then` feeds a successful value into the next fallible function, while passing an `Err` through without invoking later stages. In this code, all stages use the same error type (`CustomError`), so they compose directly:

```rust
let processed = message
    .validate()
    .and_then(ValidatedMessage::transform)
    .and_then(TransformedMessage::enrich);
```

The `?` operator used in `process_message_batch` is the more ergonomic spelling for this kind of sequential composition. Each `?` unwraps an `Ok` value for the next stage or returns the error from the current closure immediately. If pipeline stages use different error types, convert them to a common error type (for example with `map_err` or `From`) before composing them.

For a batch, `process_message_batch` maps each raw message through those stages and collects the results:

```rust
messages
    .into_iter()
    .map(|message| -> Result<ProcessedMessage> {
        let validated = message.validate()?;
        let transformed = validated.transform()?;
        transformed.enrich()
    })
    .collect()
```

Here, `Result<Vec<ProcessedMessage>>` means all messages must succeed to produce the vector. Collecting an iterator of `Result` values short-circuits at the first error; it does not gather every message error. Thus the `Result` composition handles failure within each message, and `collect()` propagates the first failed message for the batch. This keeps error propagation simple, but if a batch should report all failures, the processing loop must instead collect successes and errors separately (or return a dedicated aggregate error type).

The example also provides `process_message_batch_with_failure`, which continues after an individual message fails and returns `(Vec<ProcessedMessage>, Vec<CustomError>)`. It still composes that message's fallible stages with `and_then`; the difference is that the outer loop handles each final `Result` instead of propagating it out of the whole batch:

```rust
let result = message
    .validate()
    .and_then(|validated| validated.transform())
    .and_then(|transformed| transformed.enrich());

match result {
    Ok(message) => processed.push(message),
    Err(error) => errors.push(error),
}
```

This is **error composition** within each pipeline, paired with error accumulation across the batch. In the demo, the valid message is retained and the empty message contributes an `Empty message` error; the separate fail-fast call still returns an error for the batch.

## Pattern Matching and Exhaustiveness

`pattern_matching_pattern.rs` routes a `RichMessage` by matching several values together: its priority, content variant, and topic. Tuple patterns make combinations clear, while guards handle conditions such as long text on system topics or binary content larger than 10 KB.

```rust
match (&self.priority, &self.content, self.topic.as_str()) {
    (Priority::Critical, _, _) => ProcessingResult::Processed,
    (Priority::High, MessageContent::Text(text), topic)
        if topic.starts_with("system.") =>
    {
        if text.len() > 1000 {
            ProcessingResult::Queued
        } else {
            ProcessingResult::Processed
        }
    }
    (_, MessageContent::Binary(data), _) if data.len() > 10240 => {
        ProcessingResult::Queued
    }
    (_, MessageContent::Empty, _) => {
        ProcessingResult::Rejected("Empty content not allowed".to_string())
    }
    _ => ProcessingResult::Processed,
}
```

Rust checks that a `match` covers every possible input, preventing an unhandled case from being reached at runtime. In this implementation, the final `_` arm handles all remaining combinations, so the match is exhaustive. That catch-all is convenient, but it also means adding a new `Priority` or `MessageContent` variant will not necessarily produce a compiler error here. For stronger change detection, handle the remaining enum variants explicitly (using a fallback only for combinations that genuinely share behavior), so new variants require the routing logic to be revisited. Match arm order also matters: the critical-priority arm comes first and wins before the other rules are considered.

## Run

```sh
cargo run
```