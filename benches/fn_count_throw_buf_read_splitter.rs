use buf_read_splitter::{BufReadSplitter, Options, SimpleMatcher};
use std::io::Read;

#[path = "common/mod.rs"]
mod common;
use crate::common::stream_generator::*;

pub fn count_throw_buf_read_splitter(total_length: usize, buf_size: usize, content_len: usize) {
    let nbr_of_iterations = total_length / content_len;
    let separator = "<SEP>";
    let mut stream = StreamGenerator::new(content_len, separator, nbr_of_iterations);

    let mut reader = BufReadSplitter::new(
        &mut stream,
        SimpleMatcher::new(separator.as_bytes()),
        Options::default(),
    );

    let mut buf = vec![0u8; buf_size];

    let mut nb_part_found = 0usize;
    let mut nb_datas = 0usize;

    while reader.next().unwrap() {
        nb_part_found += 1;
        let mut sz;
        while {
            sz = reader.read(&mut buf).unwrap();
            sz > 0
        } {
            nb_datas += 1;
        }
    }

    assert!(
        nb_part_found == nbr_of_iterations,
        "nb_found different of nbr_of_iterations ( {nb_part_found} != {nbr_of_iterations} )"
    );
    assert!(nb_datas > 0)
}
