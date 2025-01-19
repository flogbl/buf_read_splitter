# buf_read_splitter
BufReadSplitter can read a buffer, stopping before each separator encounter. This separator can be changed.

Example :


 ```rust
    use buf_read_splitter::buf_read_splitter::BufReadSplitter;
    use std::io::Read;

    // To simulate an input buffer (to serve the example purpose)
    // The aim is to split on each "<SEP>"
    let input = "First<SEP><SEP>X<SEP>Second<SEP>Third<SEP>The last!".to_string();
    let mut input_reader = input.as_bytes();

    // Buffer reader
    let mut reader = BufReadSplitter::new(10, &mut input_reader, "<SEP>".as_bytes());

    let mut i = 0;
    loop {
        i += 1;

        // Output buffer
        let mut buf = [0u8; 100];

        // The string to feed  (to serve the example purpose)
        let mut text = String::new();

        loop {
            // Read the input buffer
            let sz = reader.read(&mut buf).unwrap();

            if sz > 0 {
                // Push buffer content in `text`
                let str = String::from_utf8_lossy(&buf[..sz]);
                text.push_str(&str);
            } else {
                // End of buffer
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
                        assert_eq!(text, "Third", "Case 5");
                    }
                    6 => {
                        assert_eq!(text, "The last!", "Case 10");
                    }
                    _ => {
                        assert_eq!(false, true, "Overflow")
                    }
                }
                break;
            }
        }

        match reader.next() {
            Some(Ok(_)) => {}
            Some(Err(err)) => panic!("Error : {err}"),
            None => break,
        }
    }
 ```

 Or example with separator changing :

 ```rust
    use buf_read_splitter::buf_read_splitter::BufReadSplitter;
    use std::io::Read;

    let input = "First<SEP><SEP>X<SEP>Second<SEP2>Y<SEP2>Small<>0<>Bigger<SEPARATOR_03>Till the end...<end>The last!".to_string();
    //           123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789
    //                    10        20        30        40        50        60        70        80        90
    let mut input_reader = input.as_bytes();
    let mut reader = BufReadSplitter::new(20, &mut input_reader, "<SEP>".as_bytes());
    let mut i = 0;
    loop {
        i += 1;

        let mut buf = [0u8; 3];
        let mut text = String::new();

        loop {
            let sz = reader.read(&mut buf).unwrap();
            if sz > 0 {
                let str = String::from_utf8_lossy(&buf[..sz]);
                text.push_str(&str);
            } else {
                // End of buffer
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
                break;
            }
        }

        match reader.next_split_on({
            match i {
                3 => Some("<SEP2>".as_bytes()),
                5 => Some("<>".as_bytes()),
                7 => Some("<SEPARATOR_03>".as_bytes()),
                8 => Some("<end>".as_bytes()),
                _ => None,
            }
        }) {
            Some(Ok(_)) => {}
            Some(Err(err)) => panic!("Error : {err}"),
            None => break,
        }
    }

    assert_eq!(i, 10, "Not the count")
 ```

 Its functions are not thread safe as-is, a Mutex has to be explicitly used.

