//! BufReadSplitter is a buffer reader that as the hability to stopping before each separator encounter.
//! This separator is able to be updated on the fly.
//!
//! Example :
//! ```
//! ```
//!
//! To debug purpose, add dependencies "log"
//!
pub mod buf_read_splitter;
pub mod match_result;
pub mod matcher;
pub mod options;
pub mod simple_matcher;

// private
mod buf_ext;
mod buf_ext_iter;
