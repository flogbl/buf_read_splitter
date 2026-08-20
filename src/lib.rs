//!A stream reader with ability to read a stream until a defined pattern is reached (usually an array of [u8])
//!
//!I initially wrote this library because `read_until` accepts only one char as the separator.
//!Priority is given to low memory and CPU usage (try `cargo bench` for more details)
//!
//!Demonstation by an exemple, where stream is split by `<SEP>`, each content is loaded into `contents`
//!```rust
//!use std::io::Read;
//!use buf_read_splitter::{BufReadSplitter,MatchResult,Options,SimpleMatcher};
//!
//!// To simulate a stream
//!let input = "First<SEP>Second<SEP>Third<SEP>Fourth<SEP>Fifth".to_string();
//!let mut input_reader = input.as_bytes();
//!
//!// Create a reader that will be separated by "<SEP>"
//!let mut reader = BufReadSplitter::new(
//!    &mut input_reader,
//!    SimpleMatcher::new(b"<SEP>"),
//!    Options::default(),
//!);
//!
//!let mut contents = Vec::new();
//!
//!while reader.next().unwrap() {
//!    let mut buf = Vec::new();
//!    let _ = reader.read_to_end(&mut buf);
//!    let str = String::from_utf8(buf).unwrap();
//!    contents.push(str);
//!}
//!
//!assert_eq!(&words[0], "First");
//!assert_eq!(&words[1], "Second");
//!assert_eq!(&words[2], "Third");
//!assert_eq!(&words[3], "Fourth");
//!assert_eq!(&words[4], "Fifth");
//!assert_eq!(words.len(), 5);
//!```
//!\
//!To manage more complexe pattern, the trait `Matcher` has to be implemented.\
//!For example above a Matcher able to split a stream at each Mac, Unix or Windows end of line (note the use of the position in the separator determination function) :
//!```rust
//!use buf_read_splitter::{
//!       MatchResult,
//!       Matcher,
//!       };
//!
//!struct AllEndOfLineMatcher {
//!   prev_char: u8,
//!}
//!impl AllEndOfLineMatcher {
//!   pub fn new() -> Self {
//!       Self { prev_char: 0 }
//!   }
//!}
//!impl Matcher for AllEndOfLineMatcher {
//!   // This function is called at each byte read
//!   //   `el_buf` contains the value of the byte
//!   //   `pos` contains the position matched
//!   fn sequel(&mut self, el_buf: u8, pos: usize) -> MatchResult {
//!       if pos == 0 {
//!           if el_buf == b'\r' || el_buf == b'\n' {
//!               self.prev_char = el_buf;
//!               MatchResult::NeedNext
//!           } else {
//!               MatchResult::Mismatch
//!           }
//!       } else if pos == 1 {
//!           if el_buf == b'\n' && self.prev_char == b'\r' {
//!               //We are on \r\n
//!               MatchResult::Match(0, 0)
//!           } else {
//!               //Ignore the last byte (it's not a part of the end of line)
//!               MatchResult::Match(0, 1)
//!           }
//!       } else {
//!           //Unreachable
//!           panic!("We can't reach this code since we just manage 2 positions")
//!       }
//!   }
//!
//!   // This function is called at the end of the buffer, useful to manage partial cases
//!   fn sequel_eos(&mut self, pos: usize) -> MatchResult {
//!       if pos == 0 {
//!           MatchResult::Match(0, 0) //Here the last char is \r or \n, at position 0
//!       } else {
//!           panic!("We can't reach this code since we just manage 2 positions")
//!       }
//!   }
//!}
//!```
//!...so the reader can be created with this code :
//!```ignore
//!let mut reader = BufReadSplitter::new(
//!                            &mut input_reader,
//!                            AllEndOfLineMatcher::new(),
//!                            Options::default()
//!                            );
//!```
//!\
//!The separator pattern can be changed on the fly by calling the `matcher` function :
//!```ignore
//!reader.matcher(SimpleMatcher::new(b"<CHANGE SEP>"))
//!```
//!\
//!The buffer part can be limited in size readed.\
//!For example to limit to 100 bytes :
//!```ignore
//!let mut reader = BufReadSplitter::new(
//!    &mut input_reader,
//!    AllEndOfLineMatcher,
//!    Options::default.set_limit_read(100), //Avoid memory overload
//!);
//!```
//!...or on the fly :
//!```ignore
//!reader.set_limit_read(Some(100));
//!```
//!...and to reinitialize it to "no limit" :
//!```ignore
//!reader.set_limit_read(None);
//!```
//!
//!\
//!For debug purpose, you can activate the "log" features in the Cargo.toml (note that it slows down the processing) :
//!```ignore
//![dependencies]
//!buf_read_splitter = {"0.4", features = ["log"] }
//!```
//!
//!License: MIT
//!

mod all_end_of_line_matcher;
pub use all_end_of_line_matcher::AllEndOfLineMatcher;

mod buf_read_splitter;
pub use buf_read_splitter::BufReadSplitter;

mod match_result;
pub use match_result::MatchResult;

mod matcher;
pub use matcher::Matcher;

mod options;
pub use options::Options;

mod simple_matcher;
pub use simple_matcher::SimpleMatcher;

mod errors;
pub use errors::*;

// private
mod buf_ext;
use buf_ext::BufExt;

mod buf_growing_ext_iter;
use buf_growing_ext_iter::BufGrowingExtIter;

mod pos_size_helper;
use pos_size_helper::PosSizeHelper;
