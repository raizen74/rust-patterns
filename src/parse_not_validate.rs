use std::num::NonZeroUsize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositiveU32(u32);

impl PositiveU32 {
    pub fn new(value: u32) -> Result<Self, ConfigError> {
        if value > 0 {
            Ok(Self(value))
        } else {
            Err(ConfigError::MustBePositive("value".to_string()))
        }
    }
    pub fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkPort(u16);
impl NetworkPort {
    pub fn new(port: u16) -> Result<Self, ConfigError> {
        if port > 1023 {
            // Ports 0-1023 are reserved
            Ok(Self(port))
        } else {
            Err(ConfigError::PortInReservedRange(port))
        }
    }
    pub fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    MustBePositive(String),
    PortInReservedRange(u16),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MustBePositive(field) => write!(
                f,
                "{} must be
positive",
                field
            ),
            ConfigError::PortInReservedRange(port) => write!(
                f,
                "Port {} is in
reserved range",
                port
            ),
        }
    }
}
impl std::error::Error for ConfigError {}

#[derive(Debug, Clone)]
pub struct BrokerConfig {
    max_connections: PositiveU32,
    buffer_size: NonZeroUsize,
    port: NetworkPort,
}
impl BrokerConfig {
    pub fn builder() -> BrokerConfigBuilder {
        BrokerConfigBuilder::new()
    }
    // Getters that never need validation
    pub fn max_connections(&self) -> u32 {
        self.max_connections.get()
    }
    pub fn buffer_size(&self) -> usize {
        self.buffer_size.get()
    }
    pub fn port(&self) -> u16 {
        self.port.get()
    }
}

// The builder pattern allows for more flexible construction of the BrokerConfig struct, with validation at each step.
pub struct BrokerConfigBuilder {
    max_connections: Option<PositiveU32>,
    buffer_size: Option<NonZeroUsize>,
    port: Option<NetworkPort>,
}
impl BrokerConfigBuilder {
    pub fn new() -> Self {
        Self {
            max_connections: None,
            buffer_size: None,
            port: None,
        }
    }
    pub fn max_connections(mut self, value: u32) -> Result<Self, ConfigError> {
        self.max_connections = Some(PositiveU32::new(value)?);
        Ok(self)
    }
    pub fn buffer_size(mut self, value: usize) -> Result<Self, ConfigError> {
        self.buffer_size = Some(
            NonZeroUsize::new(value)
                .ok_or_else(|| ConfigError::MustBePositive("buffer_size".to_string()))?,
        );
        Ok(self)
    }
    pub fn port(mut self, value: u16) -> Result<Self, ConfigError> {
        self.port = Some(NetworkPort::new(value)?);
        Ok(self)
    }
}

impl BrokerConfigBuilder {
    pub fn build(self) -> Result<BrokerConfig, ConfigError> {
        Ok(BrokerConfig {
            max_connections: self.max_connections.ok_or_else(|| {
                ConfigError::MustBePositive(
                    "max_connections not
set"
                    .to_string(),
                )
            })?,
            buffer_size: self.buffer_size.ok_or_else(|| {
                ConfigError::MustBePositive(
                    "buffer_size not
set"
                    .to_string(),
                )
            })?,
            port: self
                .port
                .ok_or_else(|| ConfigError::PortInReservedRange(0))?,
        })
    }
}
