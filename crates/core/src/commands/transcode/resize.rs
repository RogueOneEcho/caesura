use crate::prelude::*;

/// Information needed to resize an image
pub(crate) struct Resize {
    /// Path to the input file
    pub input: PathBuf,
    /// Path to the output file
    pub output: PathBuf,
    /// Maximum size in pixels
    pub max_pixel_size: u32,
    /// Quality percentage to apply for jpg compression.
    pub quality: u8,
}

impl Resize {
    /// Create a new convert command.
    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_info(self) -> CommandInfo {
        CommandInfo {
            program: CONVERT.to_owned(),
            args: vec![
                self.input.to_string_lossy().to_string(),
                "-resize".to_owned(),
                format!("{}x{}>", self.max_pixel_size, self.max_pixel_size),
                "-quality".to_owned(),
                format!("{}%", self.quality),
                self.output.to_string_lossy().to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_info_standard_resize() {
        // Arrange
        let resize = Resize {
            input: PathBuf::from("/source/cover.jpg"),
            output: PathBuf::from("/output/cover.jpg"),
            max_pixel_size: 1280,
            quality: 80,
        };

        // Act
        let info = resize.to_info();

        // Assert
        assert_eq!(info.program, CONVERT);
        assert_eq!(
            info.args,
            vec![
                "/source/cover.jpg",
                "-resize",
                "1280x1280>",
                "-quality",
                "80%",
                "/output/cover.jpg",
            ]
        );
    }

    #[test]
    fn to_info_custom_dimensions_and_quality() {
        // Arrange
        let resize = Resize {
            input: PathBuf::from("/source/art.png"),
            output: PathBuf::from("/output/art.jpg"),
            max_pixel_size: 500,
            quality: 95,
        };

        // Act
        let info = resize.to_info();

        // Assert
        assert_eq!(
            info.args,
            vec![
                "/source/art.png",
                "-resize",
                "500x500>",
                "-quality",
                "95%",
                "/output/art.jpg",
            ]
        );
    }
}
