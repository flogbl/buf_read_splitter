# buf_read_splitter

**buf_read_splitter** eases the way to read a buffer that has to stop on a defined pattern (like an array of [u8])

This could be a simple separator :
```rust
use std::io::Read;
use buf_read_splitter::{
       buf_read_splitter::BufReadSplitter,
       match_result::MatchResult,
       options::Options,
       simple_matcher::SimpleMatcher,
       };

// We simulate a stream with this content :
let input = "First<SEP>Second<SEP>Third<SEP>Fourth<SEP>Fifth".to_string();
let mut input_reader = input.as_bytes();

// We create a reader that will end at each "<SEP>" :
let mut reader = BufReadSplitter::new(
           &mut input_reader,
           SimpleMatcher::new(b"<SEP>"),
           Options::default(),
);

// List of separate String will be listed in a Vector :
let mut words = Vec::new();

// Working variables :
let mut word = String::new();
let mut buf = vec![0u8; 100];

while {
 // Read in buffer like any other buffer :
 match reader.read(&mut buf) {
   Err(err) => panic!("Error while reading : {err}"),
   Ok(sz) => {
     if sz > 0 {
       // === Treat the buffer ===
       let to_str = String::from_utf8_lossy(&buf[..sz]);
       word.push_str(&to_str);
       true
     } else {
       // === End of buffer part ===
       words.push(word.clone());
       word.clear();
       match reader.next_part() {  //Try to pass to the next part of the buffer
         Ok(Some(())) => true,     //There's a next part!
         Ok(None) => false,        //There's no next part, so go out of the loop
         Err(err) => panic!("Error in next_part() : {err}"),
       }
     }
   }
 }
} {}

assert_eq!(words.len(), 5);
assert_eq!(&words[0], "First");
assert_eq!(&words[1], "Second");
assert_eq!(&words[2], "Third");
assert_eq!(&words[3], "Fourth");
assert_eq!(&words[4], "Fifth");
```

This can be also a more complex pattern. It's done by implementing the trait `Matcher`.\
For example a Matcher able to split a stream at each Mac, Unix or Windows end of line :
```rust
use buf_read_splitter::{
       match_result::MatchResult,
       matcher::Matcher,
       };

struct AllEndOfLineMatcher {
   prev_char: u8,
}
impl AllEndOfLineMatcher {
   pub fn new() -> Self {
       Self { prev_char: 0 }
   }
}
impl Matcher for AllEndOfLineMatcher {
   // This function is called at each byte read
   //   `el_buf` contains the value of the byte
   //   `pos` contains the position matched
   fn sequel(&mut self, el_buf: u8, pos: usize) -> MatchResult {
       if pos == 0 {
           if el_buf == b'\r' || el_buf == b'\n' {
               self.prev_char = el_buf;
               MatchResult::NeedNext
           } else {
               MatchResult::Mismatch
           }
       } else if pos == 1 {
           if el_buf == b'\n' && self.prev_char == b'\r' {
               MatchResult::Match(0, 0) //We are on \r\n
           } else {
               MatchResult::Match(0, 1) //We have to ignore the last byte since it's not a part of the end of line pattern
           }
       } else {
           panic!("We can't reach this code since we just manage 2 positions")
       }
   }

   // This function is called at the end of the buffer, useful to manage partial cases
   fn sequel_eos(&mut self, pos: usize) -> MatchResult {
       if pos == 0 {
           MatchResult::Match(0, 0) //Here the last char is \r or \n, at position 0
       } else {
           panic!("We can't reach this code since we just manage 2 positions")
       }
   }
}
```
...so the reader can be created like this :\
`let mut reader = BufReadSplitter::new( &mut input_reader, AllEndOfLineMatcher::new(), Options::default() );`

The separator pattern can be changed on the fly by calling the function `matcher` :\
`reader.matcher(SimpleMatcher::new(b"<CHANGE SEP>"))`

The size of the buffer part can be limited.\
For example to limit the buffer part to read only 100 bytes :\
`reader.set_limit_read(Some(100));`\
...and to reinitialize it :\
`reader.set_limit_read(None);`\


A call to `.next_part()` pass to the next part, however the end was reached or not, so it skips what has not been readed.


For debug purpose, you can activate the "log" features in the Cargo.toml :\
`[dependencies]`\
`buf_read_splitter = { path = "../buf_read_splitter_v0.3/buf_read_splitter", features = ["log",] }`


For more information :\
- [https://docs.rs/buf_read_splitter/latest/buf_read_splitter/]\
- [https://crates.io/crates/buf_read_splitter]\

A suggestion or bug alert ? Feel free to fill an issue :\
- [https://github.com/flogbl/buf_read_splitter/issues]\

Thanks for your interest!


License: MIT
