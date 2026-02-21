use crate::prelude::*;

/// Join multiple strings with commas and ampersand.
///
/// # Examples
/// `and_join(&["a", "b", "c"])` returns `a, b & c`
/// `and_join(&["a", "b"])` returns `a & b`
/// `and_join(&["a"])` returns `a`
pub fn and_join<I, T>(strings: I) -> String
where
    I: IntoIterator<Item = T>,
    I::IntoIter: DoubleEndedIterator,
    T: Display,
{
    let mut iter = strings.into_iter();
    let first = iter.next();
    let last = iter.next_back();
    match (first, last) {
        (None, _) => String::new(),
        (Some(first), None) => first.to_string(),
        (Some(first), Some(last)) => {
            let mut output = iter.fold(first.to_string(), |mut output, x| {
                write!(output, ", {x}").expect("should be able to use a string as a buffer");
                output
            });
            write!(output, " & {last}").expect("should be able to use a string as a buffer");
            output
        }
    }
}
