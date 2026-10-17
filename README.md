# Rust Patterns

This crate demonstrates three Rust patterns for making invalid states harder to represent: the NewType pattern, the TypeState pattern, and validation when parsing or constructing configuration.

## NewType

NewType wraps a primitive or another type in a distinct struct. Although `TopicId` and `ConsumerId` each contain a `String`, Rust treats them as different types. A function that accepts a `TopicId` therefore cannot be called with an arbitrary `String` by accident. Their fields are private, so callers create them through the constructors, which validate that the value is non-empty; `TopicId` also limits the name to 128 bytes.

```rust
let topic = TopicId::new("events")?;
let consumer = ConsumerId::new("worker-1")?;
```

`MessageId` is another wrapper, around `u64`. It demonstrates type distinction, but its constructor currently accepts every `u64` without additional validation.

## TypeState

TypeState encodes an object's lifecycle state in its type. Here, `Consumer<State>` uses marker types such as `Disconnected`, `Connected`, `Subscribed`, and `Paused`. Each state has only the methods valid for that state:

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

## Run

```sh
cargo run
```