use crate::prelude::*;
use colored::Color;
use similar::ChangeTag;

pub struct DiffRenderer {
    styles: HashMap<ChangeTag, DiffStyle>,
}

/// Render a grapheme diff as ANSI-colored terminal text.
#[derive(Default)]
struct DiffStyle {
    color: Option<Color>,
    prefix: Option<String>,
    suffix: Option<String>,
}

impl DiffRenderer {
    /// Render a diff with ANSI colors.
    ///
    /// - Equal: Dimmed
    /// - Deletion: Red
    /// - Insert: Green
    pub(crate) fn colored() -> Self {
        let mut styles = HashMap::new();
        styles.insert(ChangeTag::Equal, DiffStyle::from(Color::Black));
        styles.insert(ChangeTag::Delete, DiffStyle::from(Color::Red));
        styles.insert(ChangeTag::Insert, DiffStyle::from(Color::Green));
        Self { styles }
    }

    /// Render a diff with ANSI colors.
    ///
    /// - Equal: Dimmed
    /// - Deletion: Red
    /// - Insert: Green
    pub(crate) fn colored_bb_code() -> Self {
        let mut styles = HashMap::new();
        styles.insert(ChangeTag::Equal, DiffStyle::from(Color::Black));
        styles.insert(
            ChangeTag::Delete,
            DiffStyle {
                color: Some(Color::Red),
                prefix: Some("[color=red]".to_owned()),
                suffix: Some("[/color]".to_owned()),
            },
        );
        styles.insert(
            ChangeTag::Insert,
            DiffStyle {
                color: Some(Color::Green),
                prefix: Some("[color=green]".to_owned()),
                suffix: Some("[/color]".to_owned()),
            },
        );
        Self { styles }
    }

    /// Render the diff
    pub fn render(&self, diff: Vec<DiffSpan>) -> String {
        diff.iter().fold(String::new(), |mut acc, diff| {
            let Some(style) = self.styles.get(&diff.tag) else {
                acc.push_str(&diff.value);
                return acc;
            };
            let mut value = String::new();
            if let Some(prefix) = &style.prefix {
                value.push_str(prefix);
            }
            value.push_str(&diff.value);
            if let Some(suffix) = &style.suffix {
                value.push_str(suffix);
            }
            match style.color {
                None => {
                    acc.push_str(&value);
                }
                Some(Color::Black) => write!(acc, "{}", value.dimmed()).expect("should write"),
                Some(color) => write!(acc, "{}", value.color(color)).expect("should write"),
            }
            acc
        })
    }
}

impl From<Color> for DiffStyle {
    fn from(color: Color) -> Self {
        Self {
            color: Some(color),
            ..DiffStyle::default()
        }
    }
}
