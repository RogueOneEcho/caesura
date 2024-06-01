use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum Category {
    Music = 0,
    Applications = 1,
    EBooks = 2,
    Audiobooks = 3,
    ELearningVideos = 4,
    Comedy = 5,
    Comics = 6,
    Unknown,
}

impl Category {
    #[must_use]
    pub fn as_int(self) -> u8 {
        self as u8
    }
}

impl From<&str> for Category {
    fn from(value: &str) -> Self {
        match value {
            "Music" => Category::Music,
            "Applications" => Category::Applications,
            "E-Books" => Category::EBooks,
            "Audiobooks" => Category::Audiobooks,
            "E-Learning Videos" => Category::ELearningVideos,
            "Comedy" => Category::Comedy,
            "Comics" => Category::Comics,
            _ => Category::Unknown,
        }
    }
}
