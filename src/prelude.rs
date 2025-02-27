pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;
pub type IoResult<T> = core::result::Result<T, std::io::Error>;


pub fn map_ioerr<E: ToString>(err: E) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
}
