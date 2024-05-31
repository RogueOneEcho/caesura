use crate::source::Metadata;
use regex::Regex;

pub struct Shortener;

impl Shortener {
    #[must_use]
    pub fn shorten_album(metadata: &Metadata) -> Option<Metadata> {
        let result = remove_parenthetical_suffix(&metadata.album);
        match result {
            None => None,
            Some(album) => {
                let mut metadata = metadata.clone();
                metadata.album = album;
                Some(metadata)
            }
        }
    }
}

fn remove_parenthetical_suffix(input: &str) -> Option<String> {
    let result = Regex::new(r"^(.*)(\(.*\))$")
        .expect("Regex should compile")
        .captures(input);
    match result {
        None => None,
        Some(captures) => {
            let shortened = captures.get(1).expect("Should have captures").as_str();
            let shortened = shortened.trim();
            if shortened.len() > 4 {
                Some(shortened.to_owned())
            } else {
                None
            }
        }
    }
}
