use std::ops::Range;

pub(crate) fn alphabets(range: Range<usize>) -> impl Iterator<Item = Vec<&'static str>> {
    let alphabet = ["A", "B", "C", "D", "E", "F", "G", "H"];
    range
        .into_iter()
        .map(move |index| alphabet[0..index].to_vec())
}
