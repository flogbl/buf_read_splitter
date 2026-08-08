///
/// This test aims to verify there's no performance problem with
/// huge buffers on small chunks (10 octets)
///
#[path = "common/mod.rs"]
mod common;

#[cfg(test)]
mod tests {

    use crate::common::stream_generator::StreamGenerator;
    use buf_read_splitter::{BufReadSplitter, Options, SimpleMatcher};
    use std::io::Read;

    const BUF_SIZE: usize = 100_000;

    #[test]
    fn count_buf_read_splitter() {
        let content_len = 100;
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
}
