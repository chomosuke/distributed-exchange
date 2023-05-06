pub mod read_writer;
pub mod interfaces;

pub type GResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
