pub type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub type GenericResult<T> = Result<T, GenericError>;