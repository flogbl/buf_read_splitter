#[path = "common/mod.rs"]
mod common;
use std::io::Read;

use crate::common::stream_generator::*;

pub fn all_in_memory(total_length: usize, buf_size: usize, content_len: usize) {
    let nbr_of_iterations = total_length / content_len;
    let separator = "<SEP>";
    let mut stream = StreamGenerator::new(content_len, separator, nbr_of_iterations);

    let mut buf = vec![0u8; buf_size];
    let _ = stream.read_to_end(&mut buf);
    let text = String::from_utf8(buf).unwrap();

    let mut nb_sep_found = 0usize;
    let mut nb_datas = 0usize;

    for s in text.split(separator) {
        nb_sep_found += 1;
        nb_datas += s.len();
    }

    assert!(
        nb_sep_found == nbr_of_iterations,
        "nb_found different of nbr_of_iterations ({nb_sep_found}!={nbr_of_iterations}) "
    );

    assert!(nb_datas > 0)
}
