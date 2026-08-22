pub fn format_octet(o: usize) -> String {
    if o >= 1024usize.pow(3) {
        format!("{} Go", o / 1024usize.pow(3))
    } else if o >= 1024usize.pow(2) {
        format!("{} Mo", o / 1024usize.pow(2))
    } else if o >= 1024usize {
        format!("{} Ko", o / 1024usize)
    } else {
        format!("{}  o", o)
    }
}
