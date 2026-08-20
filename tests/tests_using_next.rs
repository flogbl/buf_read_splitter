#[cfg(test)]
mod tests_using_next {
    use std::io::Read;

    use buf_read_splitter::{BufReadSplitter, MatchResult, Matcher, Options, SimpleMatcher};

    #[test]
    fn test_empty() {
        let input = String::new();
        let mut input_reader = input.as_bytes();
        let mut reader = BufReadSplitter::new(
            &mut input_reader,
            SimpleMatcher::new(b"<SEP NOT IN>"),
            Options::default(),
        );
        let mut count_part = 0;
        while reader.next().unwrap() {
            count_part += 1;
        }
        assert_eq!(count_part, 0);
    }

    #[test]
    fn test_one_part() {
        let input = "one".to_string();
        let mut input_reader = input.as_bytes();
        let mut reader = BufReadSplitter::new(
            &mut input_reader,
            SimpleMatcher::new(b"<SEP>"),
            Options::default(),
        );
        let mut count_part = 0;

        while reader.next().unwrap() {
            count_part += 1;
        }
        assert_eq!(count_part, 1);
    }

    #[test]
    fn test_two_part() {
        let input = "one<SEP>two".to_string();
        let mut input_reader = input.as_bytes();
        let mut reader = BufReadSplitter::new(
            &mut input_reader,
            SimpleMatcher::new(b"<SEP>"),
            Options::default(),
        );
        let mut count_part = 0;

        while reader.next().unwrap() {
            count_part += 1;
        }
        assert_eq!(count_part, 2);
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
        while reader.next().unwrap() {
            let mut sz;
            while {
                sz = reader.read(&mut buf).unwrap();
                sz > 0
            } {
                let str = String::from_utf8_lossy(&buf[..sz]);

                text.push_str(&str);
            }

            // At end of the buffer part
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
        while reader.next().unwrap() {
            let mut sz;
            while {
                sz = reader.read(&mut buf).unwrap();
                sz > 0
            } {
                let str = String::from_utf8_lossy(&buf[..sz]);

                text.push_str(&str);
            }
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
        while reader.next().unwrap() {
            let mut sz;
            while {
                sz = reader.read(&mut buf).unwrap();
                sz > 0
            } {
                let str = String::from_utf8_lossy(&buf[..sz]);

                text.push_str(&str);
            }

            i += 1;

            match i {
                1 => assert_eq!(text.as_str(), "", "Case 1"),
                2 => assert_eq!(text.as_str(), "First<S", "Case 2"),
                _ => assert_eq!(false, true, "Overflow"),
            }
            text.clear();
        }

        assert_eq!(i, 2, "Missing iterations for {buf_sz}")
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

        while reader.next().unwrap() {
            let mut sz;
            while {
                sz = reader.read(&mut buf).unwrap();
                sz > 0
            } {
                let to_str = String::from_utf8_lossy(&buf[..sz]);
                text.push_str(&to_str);
            }
            text.push('.');
        }
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
