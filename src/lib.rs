//! BufReadSplitter is a buffer reader that as the hability to stopping before each separator encounter.
//! This separator is able to be updated on the fly.
//!
//! Example :
//! ```
//! use buf_read_splitter::buf_read_splitter::{BufReadSplitter,Options};
//! use std::io::Read;
//!
//! // We simulate a buffer from a String for illustration
//! let input = "First<SEP>Second<SEP>Third<SEP>The last one !".to_string();
//! let mut input_reader = input.as_bytes();
//!
//! // Create a splitter on this buffer
//! let mut reader = BufReadSplitter::new(&mut input_reader, Options::default().clone());
//!
//! // Set the separator
//! reader.stop_on("<SEP>".as_bytes());
//!
//! // Declaring a buffer (indeed, a small one to test truncations)
//! let mut buf = vec![0u8; 5];
//!
//! // The worker
//! let mut text = String::new();
//!
//! while {
//!     // Read a chunk of the input buffer
//!     let sz = reader.read(&mut buf).unwrap();
//!     // ... append the worker
//!     text.push_str(&String::from_utf8_lossy(&buf[..sz]));
//!     // At the end of the current buffer ?
//!     if sz == 0 {
//!         // Do something with the part
//!         println!("{text} *end*");
//!     }
//!
//!     // Go to next part if there's one and if there's no error
//!     sz == 0 && reader.next_part().unwrap() == Some(())
//! } {}
//! ```
//! Output :
//! ```
//! First *end*
//! Second *end*
//! Third *end*
//! The last one ! *end*
//! ```

pub mod buf_read_splitter;
