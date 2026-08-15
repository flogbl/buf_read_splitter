/*TODO: Change to this in a major version :
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BufReadSplitterError {
    #[error("IO error: `{0}`")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, BufReadSplitterError>;
*/
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
