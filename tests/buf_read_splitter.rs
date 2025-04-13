#[cfg(test)]
mod tests {
    use std::io::Read;

    use buf_read_splitter::{BufReadSplitter, MatchResult, Matcher, Options, SimpleMatcher};

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
                MatchResult::Match(0, 0)
            } else {
                MatchResult::Mismatch
            }
        }
    }

    #[test]
    fn test_take() {
        for left in 0..5 {
            for right in 0..(5 - left) {
                for sz in 1..50 {
                    #[cfg(feature = "log")]
                    log::debug!("sz={sz} lef={left} right={right}");

                    subtest_take(sz, left, right);
                }
            }
        }
    }
    fn subtest_take(sz_buf: usize, take_left: usize, take_right: usize) {
        let input = "First<SEP>Second<SEP>Third<SEP>Fourth<SEP>Fifth".to_string();
        let mut input_reader = input.as_bytes();

        let sep = "<SEP>";
        let left_sep = &sep[0..take_left];
        let right_sep = &sep[sep.len() - take_right..];

        let matcher = IncludeMatcher::new(sep, take_left, take_right);
        let mut reader = BufReadSplitter::new(&mut input_reader, matcher, Options::default());

        let mut buf = vec![0u8; sz_buf];
        let mut words = Vec::new();
        let mut word = String::new();

        while {
            match reader.read(&mut buf) {
                Ok(sz) => {
                    #[cfg(feature = "log")]
                    log::debug!("sz={sz}");

                    if sz == 0 {
                        words.push(word.clone());
                        word.clear();
                        match reader.next_part() {
                            Ok(Some(())) => true,
                            Ok(None) => false,
                            Err(err) => panic!("Error in next_part() : {err}"),
                        }
                    } else {
                        let to_str = String::from_utf8_lossy(&buf[..sz]);
                        word.push_str(&to_str);

                        #[cfg(feature = "log")]
                        log::debug!("word={word}");
                        true
                    }
                }
                Err(err) => panic!("Error while reading : {err}"),
            }
        } {}
        assert_eq!(
            words.len(),
            5,
            "Case 1a --> sz_buf:{sz_buf}, take_left:{take_left}, take_right:{take_right}"
        );
        assert_eq!(
            &words[0],
            &format!("First{left_sep}"),
            "Case 2a --> sz_buf:{sz_buf}, take_left:{take_left}, take_right:{take_right}"
        );
        assert_eq!(
            &words[1],
            &format!("{right_sep}Second{left_sep}"),
            "Case 2b --> sz_buf:{sz_buf}, take_left:{take_left}, take_right:{take_right}"
        );
        assert_eq!(
            &words[2],
            &format!("{right_sep}Third{left_sep}"),
            "Case 2c --> sz_buf:{sz_buf}, take_left:{take_left}, take_right:{take_right}"
        );
        assert_eq!(
            &words[3],
            &format!("{right_sep}Fourth{left_sep}"),
            "Case 2d --> sz_buf:{sz_buf}, take_left:{take_left}, take_right:{take_right}"
        );
        assert_eq!(
            &words[4],
            &format!("{right_sep}Fifth"),
            "Case 2e --> sz_buf:{sz_buf}, take_left:{take_left}, take_right:{take_right}"
        );
    }
    struct IncludeMatcher {
        sep: Vec<u8>,
        take_left: usize,
        take_right: usize,
    }
    impl IncludeMatcher {
        pub fn new(str_sep: &str, take_left: usize, take_right: usize) -> Self {
            Self {
                sep: Vec::from(str_sep.as_bytes()),
                take_left,
                take_right,
            }
        }
    }
    impl Matcher for IncludeMatcher {
        fn sequel(&mut self, el_buf: u8, pos: usize) -> MatchResult {
            if pos >= self.sep.len() || self.sep[pos] != el_buf {
                MatchResult::Mismatch
            } else {
                if self.sep.len() == pos + 1 {
                    MatchResult::Match(self.take_left, self.take_right)
                } else {
                    MatchResult::NeedNext
                }
            }
        }
    }

    #[test]
    fn test_end_of_stream() {
        let lst_inputs = vec![
            "First\rSecond\nTh1rd\r\nFourth\n\rFifth".to_string(),
            "\rFirst\rSecond\nTh2rd\r\nFourth\n\rFifth".to_string(),
            "\r\nFirst\rSecond\nTh3rd\r\nFourth\n\rFifth".to_string(),
            "First\rSecond\nTh4rd\r\nFourth\n\rFifth\r".to_string(),
            "First\rSecond\nTh5rd\r\nFourth\n\rFifth\r\n".to_string(),
        ];
        let lst_outputs = vec![
            "First.Second.Th1rd.Fourth..Fifth.".to_string(),
            ".First.Second.Th2rd.Fourth..Fifth.".to_string(),
            ".First.Second.Th3rd.Fourth..Fifth.".to_string(),
            "First.Second.Th4rd.Fourth..Fifth..".to_string(),
            "First.Second.Th5rd.Fourth..Fifth..".to_string(),
        ];

        for (i, o) in std::iter::zip(lst_inputs, lst_outputs) {
            for sz in 1..50 {
                subtest_end_of_stream(sz, &i, &o);
            }
        }
    }
    fn subtest_end_of_stream(sz_buf: usize, i: &str, o: &str) {
        let mut input_reader = i.as_bytes();

        let mut reader = BufReadSplitter::new(
            &mut input_reader,
            AllEndOfLineMatcher::new(),
            Options::default(),
        );

        let mut buf = vec![0u8; sz_buf];
        let mut text = String::new();

        while {
            match reader.read(&mut buf) {
                Ok(sz) => {
                    #[cfg(feature = "log")]
                    log::debug!("sz={sz}");

                    if sz == 0 {
                        text.push('.');
                        match reader.next_part() {
                            Ok(Some(())) => true,
                            Ok(None) => false,
                            Err(err) => panic!("Error in next_part() : {err}"),
                        }
                    } else {
                        let to_str = String::from_utf8_lossy(&buf[..sz]);
                        text.push_str(&to_str);
                        true
                    }
                }
                Err(err) => panic!("Error while reading : {err}"),
            }
        } {}
        assert_eq!(&text, o, "Case :  sz_buf:{sz_buf}");
    }
    struct AllEndOfLineMatcher {
        prev_char: u8,
    }
    impl AllEndOfLineMatcher {
        pub fn new() -> Self {
            Self { prev_char: 0 }
        }
    }
    impl Matcher for AllEndOfLineMatcher {
        /// Words can be \r, \n or \r\n
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
        fn sequel_eos(&mut self, pos: usize) -> MatchResult {
            if pos == 0 {
                MatchResult::Match(0, 0) //Here the last char is \r or \n, at position 0
            } else {
                panic!("We can't reach this code since we just manage 2 positions")
            }
        }
    }
}
