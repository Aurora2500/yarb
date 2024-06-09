use std::error::Error;

pub type Result<T = (), E = Box<dyn Error + Send + Sync>> = std::result::Result<T, E>;
