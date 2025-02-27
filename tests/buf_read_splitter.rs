#[cfg(test)]
mod tests {
    use std::io::Read;

    use buf_read_splitter::{
        buf_read_splitter::BufReadSplitter, match_result::MatchResult, matcher::Matcher,
        options::Options, simple_matcher::SimpleMatcher,
    };

    #[test]
    fn test_none_to_match() {
        let input = "one to three four five six seven height nine ten".to_string();
        let mut input_reader = input.as_bytes();
        let mut reader = BufReadSplitter::new(
            &mut input_reader,
            SimpleMatcher::new(b"<SEP NOT IN>"),
            Options::default(),
        );
        let mut buf = vec![0u8; 10];
        let mut text = String::new();
        loop {
            let sz = reader.read(&mut buf).unwrap();
            let str = String::from_utf8_lossy(&buf[..sz]);

            text.push_str(&str);

            if sz == 0 {
                if reader.next_part().unwrap() == None {
                    break;
                }
            }
        }
        assert_eq!(text, input, "Case 1");
    }

    #[test]
    fn test_common() {
        for i in 1..1000 {
            sub_test_common(i);
        }
    }
    fn sub_test_common(buf_ext: usize) {
        let input = "First<SEP><SEP>X<SEP>Second<SEP2>Y<SEP2>Small<>0<>Bigger<SEPARATOR_03>Till the end...<end>The last!".to_string();
        //           123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789
        //                    10        20        30        40        50        60        70        80        90

        let mut input_reader = input.as_bytes();
        let mut reader = BufReadSplitter::new(
            &mut input_reader,
            SimpleMatcher::new(b"<SEP>"),
            Options::default()
                .set_reserve_sz_to_match(2)
                .set_extend_buffer_additionnal_sz(1)
                .clone(),
        );
        let mut i = 0;
        let mut buf = vec![0u8; buf_ext];
        let mut text = String::new();
        loop {
            let sz = reader.read(&mut buf).unwrap();
            let str = String::from_utf8_lossy(&buf[..sz]);

            text.push_str(&str);

            // At end of the buffer part ?
            if sz == 0 {
                i += 1;

                match i {
                    1 => assert_eq!(text.as_str(), "First", "Case 1"),
                    2 => assert_eq!(text.as_str(), "", "Case 2"),
                    3 => assert_eq!(text.as_str(), "X", "Case 3"),
                    4 => assert_eq!(text.as_str(), "Second", "Case 4"),
                    5 => assert_eq!(text.as_str(), "Y", "Case 5"),
                    6 => assert_eq!(text.as_str(), "Small", "Case 6"),
                    7 => assert_eq!(text.as_str(), "0", "Case 7"),
                    8 => assert_eq!(text.as_str(), "Bigger", "Case 8"),
                    9 => assert_eq!(text.as_str(), "Till the end...", "Case 9"),
                    10 => assert_eq!(text.as_str(), "The last!", "Case 10"),
                    _ => assert_eq!(false, true, "Overflow"),
                }
                text.clear();

                if reader.next_part().unwrap() == None {
                    break;
                }
            }

            match i {
                3 => reader.matcher(SimpleMatcher::new(b"<SEP2>")),
                5 => reader.matcher(SimpleMatcher::new(b"<>")),
                7 => reader.matcher(SimpleMatcher::new(b"<SEPARATOR_03>")),
                8 => reader.matcher(SimpleMatcher::new(b"<end>")),
                _ => {}
            }
        }
        assert_eq!(i, 10, "Missing iterations for {buf_ext}")
    }

    #[test]
    fn test_sep_first_pos() {
        for i in 1..1000 {
            sub_test_sep_first_pos(i);
        }
    }
    fn sub_test_sep_first_pos(buf_sz: usize) {
        let input = "<SEP>First<SEP>".to_string();

        let mut input_reader = input.as_bytes();
        let mut reader = BufReadSplitter::new(
            &mut input_reader,
            SimpleMatcher::new(b"<SEP>"),
            Options::default(),
        );
        let mut i = 0;

        let mut buf = vec![0u8; buf_sz];
        let mut text = String::new();
        loop {
            let sz = reader.read(&mut buf).unwrap();
            let str = String::from_utf8_lossy(&buf[..sz]);

            text.push_str(&str);

            if sz == 0 {
                i += 1;

                match i {
                    1 => assert_eq!(text.as_str(), "", "Case 1"),
                    2 => {
                        assert_eq!(text.as_str(), "First", "Case 2")
                    }
                    3 => assert_eq!(text.as_str(), "", "Case 3"),
                    _ => {
                        assert_eq!(false, true, "Overflow")
                    }
                }
                text.clear();

                if reader.next_part().unwrap() == None {
                    break;
                }
            }
        }
        assert_eq!(i, 3, "Missing iterations for {buf_sz}")
    }

    #[test]
    fn test_sep_partial() {
        for i in 1..1000 {
            sub_test_sep_partial(i);
        }
    }
    fn sub_test_sep_partial(buf_sz: usize) {
        let input = "<SEP>First<S".to_string();

        let mut input_reader = input.as_bytes();
        let mut reader = BufReadSplitter::new(
            &mut input_reader,
            SimpleMatcher::new(b"<SEP>"),
            Options::default(),
        );
        let mut i = 0;

        let mut buf = vec![0u8; buf_sz];
        let mut text = String::new();
        loop {
            let sz = reader.read(&mut buf).unwrap();
            let str = String::from_utf8_lossy(&buf[..sz]);

            text.push_str(&str);

            if sz == 0 {
                i += 1;

                match i {
                    1 => assert_eq!(text.as_str(), "", "Case 1"),
                    2 => assert_eq!(text.as_str(), "First<S", "Case 2"),
                    _ => assert_eq!(false, true, "Overflow"),
                }
                text.clear();

                if reader.next_part().unwrap() == None {
                    // We enter here because of `sz=0` condition, so it's the end of the buffer
                    break;
                }
            }
        }
        assert_eq!(i, 2, "Missing iterations for {buf_sz}")
    }

    #[test]
    fn test_next_but_not_at_end() {
        let input = "First<SEP>A longue test ending without <SEP".to_string();
        //           123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789
        //                    10        20        30        40        50        60        70        80        90
        let mut input_reader = input.as_bytes();

        let mut reader = BufReadSplitter::new(
            &mut input_reader,
            SimpleMatcher::new(b"<SEP>"),
            Options::default(),
        );

        let mut i = 0;
        loop {
            i += 1;

            let mut buf = vec![0u8; 3];

            let sz = reader.read(&mut buf).unwrap();
            let text = String::from_utf8_lossy(&buf[..sz]);

            if i == 1 {
                assert_eq!(text, "Fir", "Case 1a");

                let has_next = reader.next_part().unwrap();
                assert_eq!(has_next, Some(()), "Case 1c");
            } else if i == 2 {
                assert_eq!(text, "A l", "Case 2a");

                let has_next = reader.next_part().unwrap();
                assert_eq!(has_next, None, "Case 2c");

                // A read after end simply return 0, no error
                let last_sz_read = reader.read(&mut buf).unwrap();
                assert_eq!(last_sz_read, 0, "Case 3");

                break;
            }
        }
    }

    #[test]
    fn test_limit() {
        let input = "First<SEP>Second<SEP>Third<SEP>Fourth<SEP>Fifth".to_string();
        let mut input_reader = input.as_bytes();

        let mut reader = BufReadSplitter::new(
            &mut input_reader,
            SimpleMatcher::new(b"<SEP>"),
            Options::default(),
        );

        // Must take the separator into account, even if the limit fall in the middle of it
        {
            let mut buf = [0u8; 3];
            reader.set_limit_read(Some(7));
            let sz = reader.read(&mut buf).unwrap();
            assert_eq!(&buf[0..sz], "Fir".as_bytes(), "Case 1a");

            let sz = reader.read(&mut buf).unwrap();
            assert_eq!(&buf[0..sz], "st".as_bytes(), "Case 1b");

            let sz = reader.read(&mut buf).unwrap();
            assert_eq!(&buf[0..sz], "".as_bytes(), "Case 1c");
        }

        reader.next_part().unwrap();

        // Limit basic case
        {
            let mut buf = [0u8; 3];
            reader.set_limit_read(Some(5));
            let sz = reader.read(&mut buf).unwrap();
            assert_eq!(&buf[0..sz], "Sec".as_bytes(), "Case 2a");

            let sz = reader.read(&mut buf).unwrap();
            assert_eq!(&buf[0..sz], "on".as_bytes(), "Case 2b");

            let sz = reader.read(&mut buf).unwrap();
            assert_eq!(&buf[0..sz], "".as_bytes(), "Case 2c");
        }

        reader.next_part().unwrap();

        // Size of buffer larger than the limit
        {
            let mut buf = [0u8; 10];
            reader.set_limit_read(Some(2));
            let sz = reader.read(&mut buf).unwrap();
            assert_eq!(&buf[0..sz], "Th".as_bytes(), "Case 3a");

            let sz = reader.read(&mut buf).unwrap();
            assert_eq!(&buf[0..sz], "".as_bytes(), "Case 3b");
        }

        reader.next_part().unwrap();

        // Size of buffer = the limit
        {
            let mut buf = [0u8; 6];
            reader.set_limit_read(Some(6));
            let sz = reader.read(&mut buf).unwrap();
            assert_eq!(&buf[0..sz], "Fourth".as_bytes(), "Case 4a");

            let sz = reader.read(&mut buf).unwrap();
            assert_eq!(&buf[0..sz], "".as_bytes(), "Case 4b");
        }

        reader.next_part().unwrap();

        // Limit < End
        {
            let mut buf = [0u8; 3];
            reader.set_limit_read(Some(10));
            let sz = reader.read(&mut buf).unwrap();
            assert_eq!(&buf[0..sz], "Fif".as_bytes(), "Case 5a");

            let sz = reader.read(&mut buf).unwrap();
            assert_eq!(&buf[0..sz], "th".as_bytes(), "Case 5b");

            let sz = reader.read(&mut buf).unwrap();
            assert_eq!(&buf[0..sz], "".as_bytes(), "Case 5b");
        }

        let opt = reader.next_part().unwrap();
        assert_eq!(opt, None, "Case end");
    }

    #[test]
    fn test_linefeed() {
        let input = "First\nA longue test\r\nending".to_string();
        //           123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789
        //                    10        20        30        40        50        60        70        80        90
        let mut input_reader = input.as_bytes();

        let mut reader = BufReadSplitter::new(&mut input_reader, LFMatcher {}, Options::default());

        // The size of this buffer can contain all string parts, it's just for the test purpose
        let mut buf = vec![0u8; 30];

        let sz = reader.read(&mut buf).unwrap();
        let text = String::from_utf8_lossy(&buf[..sz]);
        {
            assert_eq!(text, "First", "Case 1a");

            let has_next = reader.next_part().unwrap();
            assert_eq!(has_next, Some(()), "Case 1c");
        }
        let sz = reader.read(&mut buf).unwrap();
        let text = String::from_utf8_lossy(&buf[..sz]);
        {
            assert_eq!(text, "A longue test", "Case 2a");

            let has_next = reader.next_part().unwrap();
            assert_eq!(has_next, Some(()), "Case 2c");
        }
        let sz = reader.read(&mut buf).unwrap();
        let text = String::from_utf8_lossy(&buf[..sz]);
        {
            assert_eq!(text, "ending", "Case 2a");

            let has_next = reader.next_part().unwrap();
            assert_eq!(has_next, None, "Case 2c");
        }
    }
    struct LFMatcher {}
    impl Matcher for LFMatcher {
        fn sequel(&mut self, el_buf: u8, _pos: usize) -> MatchResult {
            if el_buf == b'\r' {
                MatchResult::NeedNext
            } else if el_buf == b'\n' {
                MatchResult::Match
            } else {
                MatchResult::Mismatch
            }
        }
    }
}
