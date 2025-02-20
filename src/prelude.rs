pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;
pub type IoResult<T> = core::result::Result<T, std::io::Error>;
