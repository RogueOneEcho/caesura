use std::fmt::{Display, Write};

/// Join multiple strings into with commas plus an "and" before the last element.
///
/// # Examples
/// `join_humanized(&["a", "b", "c"])` returns `a, b and c`
/// `join_humanized(&["a", "b"])` returns `a and b`
/// `join_humanized(&["a"])` returns `a`
pub fn join_humanized<I, T>(strings: I) -> String
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
            write!(output, " and {last}").expect("should be able to use a string as a buffer");
            output
        }
    }
}
