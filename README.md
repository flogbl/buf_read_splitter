# buf_read_splitter

BufReadSplitter can read a buffer, stopping before each separator encounter. This separator can be updated on the fly.

Example :

```rust
    use buf_read_splitter::buf_read_splitter::{BufReadSplitter,Options};
    use std::io::Read;

    let input = "First<SEP><SEP>X<SEP>Second<SEP>Third<SEP>The last!".to_string();
    let mut input_reader = input.as_bytes();
    let mut reader = BufReadSplitter::new(&mut input_reader, Options::default());
    reader.set_array_to_match("<SEP>".as_bytes());
    let mut i = 0;
    let mut buf = [0u8; 5];
    let mut text = String::new();
    loop {
        let sz = reader.read(&mut buf).unwrap();
        let str = String::from_utf8_lossy(&buf[..sz]);
        text.push_str(&str);
        if reader.matched() || sz == 0 {
            i += 1;
            // Matching the separator or  end of the buffer
            match i {
                1 => assert_eq!(text, "First", "Case 1"),
                2 => assert_eq!(text, "", "Case 2"),
                3 => assert_eq!(text, "X", "Case 3"),
                4 => assert_eq!(text, "Second", "Case 4"),
                5 => assert_eq!(text, "Third", "Case 5"),
                6 => assert_eq!(text, "The last!", "Case 10"),
                _ => assert_eq!(false, true, "Overflow"),
            }
            // End of the buffer case
            if reader.matched() == false {
                break;
            }
            text.clear();
        }
    }
    assert_eq!(i, 6, "Missing iterations")
```

Or example with separator changing on the fly :

```rust
    use buf_read_splitter::buf_read_splitter::{BufReadSplitter,Options};
    use std::io::Read;

    let input = "First<SEP><SEP>X<SEP>Second<SEP2>Y<SEP2>Small<>0<>Bigger<SEPARATOR_03>Till the end...<end>The last!".to_string();
    //           123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789
    //                    10        20        30        40        50        60        70        80        90

    let mut input_reader = input.as_bytes();
    let mut reader = BufReadSplitter::new(
        &mut input_reader,
        Options::default()
            .set_initiale_sz_to_match(2)
            .set_chunk_sz(5)
            .clone(),
    );
    reader.set_array_to_match("<SEP>".as_bytes());
    let mut i = 0;
    let mut buf = vec![0u8; 10];
    let mut text = String::new();
    loop {
        let sz = reader.read(&mut buf).unwrap();
        let str = String::from_utf8_lossy(&buf[..sz]);

        text.push_str(&str);

        if reader.matched() || sz == 0 {
            i += 1;

            match i {
                1 => {
                    assert_eq!(text, "First", "Case 1");
                }
                2 => {
                    assert_eq!(text, "", "Case 2");
                }
                3 => {
                    assert_eq!(text, "X", "Case 3");
                }
                4 => {
                    assert_eq!(text, "Second", "Case 4");
                }
                5 => {
                    assert_eq!(text, "Y", "Case 5");
                }
                6 => {
                    assert_eq!(text, "Small", "Case 6");
                }
                7 => {
                    assert_eq!(text, "0", "Case 7");
                }
                8 => {
                    assert_eq!(text, "Bigger", "Case 8");
                }
                9 => {
                    assert_eq!(text, "Till the end...", "Case 9");
                }
                10 => {
                    assert_eq!(text, "The last!", "Case 10");
                }
                _ => {
                    assert_eq!(false, true, "Overflow")
                }
            }
            text.clear();

            if reader.matched() == false {
                // We enter here because of `sz=0` condition, so it's the end of the buffer
                break;
            }
        }

        match i {
            3 => reader.set_array_to_match("<SEP2>".as_bytes()),
            5 => reader.set_array_to_match("<>".as_bytes()),
            7 => reader.set_array_to_match("<SEPARATOR_03>".as_bytes()),
            8 => reader.set_array_to_match("<end>".as_bytes()),
            _ => {}
        }
    }
    assert_eq!(i, 10, "Missing iterations")
```

Its functions are not thread safe as-is.
