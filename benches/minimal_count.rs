#[path = "common/mod.rs"]
mod common;

use common::stream_generator::StreamGenerator;
use std::io::Read;

#[divan::bench(args = [10, 100, 1_000, 10_000, 100_000])]
fn minimal_count(content_len: usize) {
    let nbr_of_iterations = 10_000_000 / content_len;
    let separator = "<SEP>";
    let mut stream = StreamGenerator::new(content_len, separator, nbr_of_iterations);

    let mut buf = vec![0u8; 255];
    let mut nb_sep_found = 0usize;

    let mut pos_found = 0usize;
    let separator_bytes = separator.as_bytes();

    while {
        let sz = stream.read(&mut buf).unwrap();
        if sz > 0 {
            for &b in &buf[..sz] {
                if b == separator_bytes[pos_found] {
                    pos_found += 1;
                    if pos_found == separator_bytes.len() {
                        nb_sep_found += 1;
                        pos_found = 0;
                    }
                } else if b == separator_bytes[0] {
                    pos_found = 1;
                    if pos_found == separator_bytes.len() {
                        nb_sep_found += 1;
                        pos_found = 0;
                    }
                } else {
                    if pos_found > 0 {
                        pos_found = 0;
                    }
                }
            }
            true
        } else {
            false
        }
    } {}

    assert!(
        nb_sep_found + 1 == nbr_of_iterations,
        "nb_found different of nbr_of_iterations ({nb_sep_found}!={nbr_of_iterations}) "
    )
}
