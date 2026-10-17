#[derive(Debug)]
pub struct CustomError(pub String);

pub type Result<T> = std::result::Result<T, CustomError>;
