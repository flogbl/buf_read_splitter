#[path = "common/mod.rs"]
mod common;

use buf_read_splitter::{BufReadSplitter, Options, SimpleMatcher};
use common::stream_generator::StreamGenerator;
use std::io::Read;

const BUF_SIZE: usize = 255;

#[divan::bench(args = [10, 100, 1_000, 10_000, 100_000])]
fn buf_read_splitter(content_len: usize) {
    let nbr_of_iterations = 10_000_000 / content_len;
    let separator = "<SEP>";
    let mut stream = StreamGenerator::new(content_len, separator, nbr_of_iterations);

    let mut reader = BufReadSplitter::new(
        &mut stream,
        SimpleMatcher::new(separator.as_bytes()),
        Options::default(),
    );

    let mut buf = vec![0u8; BUF_SIZE];

    let mut nb_part_found = 0usize;

    while {
        let sz = reader.read(&mut buf).unwrap();
        if sz > 0 {
            true
        } else {
            nb_part_found += 1;
            match reader.next_part().unwrap() {
                //Pass to the next part of the buffer
                Some(_) => true, //There's a next part
                None => false,   //End of the stream
            }
        }
    } {}
    assert!(
        nb_part_found == nbr_of_iterations,
        "nb_found different of nbr_of_iterations ( {nb_part_found} != {nbr_of_iterations} )"
    )
}
