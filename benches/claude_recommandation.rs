#[path = "common/mod.rs"]
mod common;

use common::stream_generator::StreamGenerator;
use std::io::{BufReader, Read};

const PATTERN: &[u8] = b"--sep--";
const BUF_SIZE: usize = 255;

#[divan::bench(args = [10, 100, 1_000, 10_000, 100_000])]
fn claude_recommandation(content_len: usize) {
    let nbr_of_iterations = 10_000_000 / content_len;
    let stream = StreamGenerator::new(content_len, "--sep--", nbr_of_iterations);
    let reader = BufReader::new(stream);
    let count = count_pattern(reader, PATTERN).unwrap();

    assert!(
        count + 1 == nbr_of_iterations,
        "nb_found different of nbr_of_iterations ({count}!={nbr_of_iterations}) "
    );
}

/// Counts non-overlapping occurrences of `pattern` in the given reader,
/// correctly handling matches that straddle buffer boundaries.
fn count_pattern<R: Read>(mut reader: R, pattern: &[u8]) -> std::io::Result<usize> {
    let mut buf = vec![0u8; BUF_SIZE];
    let mut count = 0usize;

    // Holds the tail of the previous chunk that might be the start
    // of a match continuing into the next chunk.
    let mut carry: Vec<u8> = Vec::with_capacity(pattern.len() * 2);

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break; // EOF
        }

        // Combine leftover carry with the new chunk for searching.
        carry.extend_from_slice(&buf[..n]);

        // Search for all non-overlapping matches within `carry`.
        let mut search_start = 0;
        while let Some(pos) = find_subslice(&carry[search_start..], pattern) {
            count += 1;
            search_start += pos + pattern.len();
        }

        // Keep only the trailing bytes that could still be a partial
        // match at the start of the next chunk.
        let keep_from = carry.len().saturating_sub(pattern.len() - 1);
        // But don't cut into bytes already consumed by a match.
        let keep_from = keep_from.max(search_start);
        carry.drain(0..keep_from);
    }

    Ok(count)
}

/// Naive substring search (fine for short patterns like "--sep--").
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
