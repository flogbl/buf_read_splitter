# buf_read_splitter

A stream reader with ability to read a stream until a defined pattern is reached (usually an array of [u8])

This could be a simple separator :
```rust
use std::io::Read;
use buf_read_splitter::{BufReadSplitter,MatchResult,Options,SimpleMatcher};

// To simulate a stream of this content :
let input = "First<SEP>Second<SEP>Third<SEP>Fourth<SEP>Fifth".to_string();
let mut input_reader = input.as_bytes();

// Create a reader that will end at each "<SEP>" :
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
 let sz = reader.read(&mut buf).unwrap();
 if sz > 0 {
    let to_str = String::from_utf8_lossy(&buf[..sz]);
    word.push_str(&to_str);
    true
 } else {
    words.push(word.clone());
    word.clear();
    match reader.next_part().unwrap() {  //Pass to the next part of the buffer
       Some(_) => true,     //There's a next part
       None => false,       //End of the stream
    }
 }
} {}

assert_eq!(&words[0], "First");
assert_eq!(&words[1], "Second");
assert_eq!(&words[2], "Third");
assert_eq!(&words[3], "Fourth");
assert_eq!(&words[4], "Fifth");
assert_eq!(words.len(), 5);
```
\
For more complexe pattern, the trait `Matcher` has to be implementing.\
For example above a Matcher able to split a stream at each Mac, Unix or Windows end of line :
```rust
use buf_read_splitter::{
       MatchResult,
       Matcher,
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
               //We are on \r\n
               MatchResult::Match(0, 0)
           } else {
               //Ignore the last byte (it's not a part of the end of line)
               MatchResult::Match(0, 1)
           }
       } else {
           //Unreachable
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
...so the reader can be created like with this code :
```rust
let mut reader = BufReadSplitter::new(
                            &mut input_reader,
                            AllEndOfLineMatcher::new(),
                            Options::default()
                            );
```
\
The separator pattern can be changed on the fly by calling "`matcher`" function :
```rust
reader.matcher(SimpleMatcher::new(b"<CHANGE SEP>"))
```
\
The buffer part can be limited in size readed.\
For example to limit to 100 bytes :
```rust
reader.set_limit_read(Some(100));
```
...and to reinitialize it to "no limit" :
```rust
reader.set_limit_read(None);
```

\
A call to "`.next_part()`" pass to the next part, however the end was reached or not (skips what has not been readed)\

\
For debug purpose, you can activate the "log" features in the Cargo.toml (slow down processing) :
```rust
[dependencies]
buf_read_splitter = {"0.4", features = ["log"] }
```


License: MIT
