#[derive(Debug, Clone)]
pub enum CustomError {
    Broker(String),
}

pub type Result<T> = std::result::Result<T, CustomError>;
