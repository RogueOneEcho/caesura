use std::fs::File;
use std::io::BufWriter;

use fast_image_resize::images::Image;
use fast_image_resize::{FilterType, IntoImageView, ResizeAlg, ResizeOptions, Resizer};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{ColorType, DynamicImage, ExtendedColorType, ImageEncoder, ImageFormat, ImageReader};

use crate::prelude::*;

const RESIZE_ALGORITHM: ResizeAlg = ResizeAlg::Interpolation(FilterType::CatmullRom);

/// Information needed to resize an image.
pub(crate) struct Resize {
    /// Path to the input file.
    pub input: PathBuf,
    /// Path to the output file.
    pub output: PathBuf,
    /// Maximum size in pixels.
    pub max_pixel_size: u32,
    /// Quality percentage to apply for JPG compression.
    pub quality: u8,
}

impl Resize {
    /// Resize the image, preserving aspect ratio and only shrinking.
    pub(crate) fn execute(self) -> Result<(), Failure<ResizeAction>> {
        let source = decode_image(&self.input)?;
        let output_format = ImageFormat::from_path(&self.output)
            .map_err(Failure::wrap_with_path(ResizeAction::Encode, &self.output))?;
        let source = normalize_for_format(source, output_format);
        let color_type = ExtendedColorType::from(source.color());
        let resized = resize_image(&source, self.max_pixel_size)?;
        let (buffer, width, height) = match &resized {
            Some((image, w, h)) => (image.buffer(), *w, *h),
            None => (source.as_bytes(), source.width(), source.height()),
        };
        let encoder = Encoder {
            path: &self.output,
            format: output_format,
            buffer,
            width,
            height,
            color_type,
            quality: self.quality,
        };
        encoder.write()
    }
}

/// Decode an image from a file path.
fn decode_image(path: &Path) -> Result<DynamicImage, Failure<ResizeAction>> {
    ImageReader::open(path)
        .map_err(Failure::wrap_with_path(ResizeAction::Open, path))?
        .with_guessed_format()
        .map_err(Failure::wrap_with_path(ResizeAction::Open, path))?
        .decode()
        .map_err(Failure::wrap_with_path(ResizeAction::Decode, path))
}

/// Convert the image to RGB8 when the output format requires it.
///
/// JPEG only supports 8-bit RGB, so images with alpha channels or higher bit
/// depths must be converted before resizing and encoding.
fn normalize_for_format(source: DynamicImage, format: ImageFormat) -> DynamicImage {
    if format == ImageFormat::Jpeg && source.color() != ColorType::Rgb8 {
        DynamicImage::ImageRgb8(source.to_rgb8())
    } else {
        source
    }
}

/// Resize the image if either dimension exceeds `max_pixel_size`.
fn resize_image(
    source: &DynamicImage,
    max_pixel_size: u32,
) -> Result<Option<(Image<'_>, u32, u32)>, Failure<ResizeAction>> {
    let (src_w, src_h) = (source.width(), source.height());
    if src_w <= max_pixel_size && src_h <= max_pixel_size {
        return Ok(None);
    }
    let (dst_w, dst_h) = fit_dimensions(src_w, src_h, max_pixel_size);
    let pixel_type = source
        .pixel_type()
        .expect("source image should have a pixel type");
    let mut dst_image = Image::new(dst_w, dst_h, pixel_type);
    let options = ResizeOptions::default()
        .resize_alg(RESIZE_ALGORITHM)
        .fit_into_destination(None);
    Resizer::new()
        .resize(source, &mut dst_image, &options)
        .map_err(Failure::wrap(ResizeAction::Resize))?;
    Ok(Some((dst_image, dst_w, dst_h)))
}

/// Compute target dimensions that fit within `max_size` while preserving aspect ratio.
#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::integer_division,
    reason = "result is always <= max_size which fits in u32; integer division is intentional for pixel dimensions"
)]
fn fit_dimensions(width: u32, height: u32, max_size: u32) -> (u32, u32) {
    let width = u64::from(width);
    let height = u64::from(height);
    let max = u64::from(max_size);
    let longest = width.max(height);
    let w = (width * max / longest) as u32;
    let h = (height * max / longest) as u32;
    (w.max(1), h.max(1))
}

/// Image data ready to be encoded to a file.
struct Encoder<'a> {
    path: &'a Path,
    format: ImageFormat,
    buffer: &'a [u8],
    width: u32,
    height: u32,
    color_type: ExtendedColorType,
    quality: u8,
}

impl Encoder<'_> {
    /// Encode the image and write to disk.
    fn write(&self) -> Result<(), Failure<ResizeAction>> {
        let file = File::create(self.path)
            .map_err(Failure::wrap_with_path(ResizeAction::Write, self.path))?;
        let writer = BufWriter::new(file);
        match self.format {
            ImageFormat::Jpeg => {
                JpegEncoder::new_with_quality(writer, self.quality)
                    .write_image(self.buffer, self.width, self.height, self.color_type)
                    .map_err(Failure::wrap_with_path(ResizeAction::Encode, self.path))?;
            }
            ImageFormat::Png => {
                PngEncoder::new(writer)
                    .write_image(self.buffer, self.width, self.height, self.color_type)
                    .map_err(Failure::wrap_with_path(ResizeAction::Encode, self.path))?;
            }
            _ => {
                return Err(Failure::new(
                    ResizeAction::Encode,
                    ResizeError::UnsupportedFormat(self.format),
                )
                .with_path(self.path));
            }
        }
        Ok(())
    }
}

/// Leaf operations for image resizing.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub(crate) enum ResizeAction {
    #[error("open image")]
    Open,
    #[error("decode image")]
    Decode,
    #[error("resize image")]
    Resize,
    #[error("encode image")]
    Encode,
    #[error("write image")]
    Write,
}

/// Errors that can occur during image resizing.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub(crate) enum ResizeError {
    #[error("unsupported format: {0:?}")]
    UnsupportedFormat(ImageFormat),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn _fit_dimensions() {
        // Landscape
        assert_eq!(fit_dimensions(600, 450, 320), (320, 240));
        // Portrait
        assert_eq!(fit_dimensions(450, 600, 320), (240, 320));
        // Square
        assert_eq!(fit_dimensions(500, 500, 320), (320, 320));
        // Extreme aspect ratio: very wide
        assert_eq!(fit_dimensions(10000, 100, 500), (500, 5));
        // Extreme aspect ratio: very tall
        assert_eq!(fit_dimensions(100, 10000, 500), (5, 500));
        // Non-divisible dimensions truncate toward zero
        assert_eq!(fit_dimensions(700, 300, 320), (320, 137));
        // Narrow dimension clamps to 1
        assert_eq!(fit_dimensions(10000, 1, 320), (320, 1));
        // Large source dimensions (u64 intermediate prevents overflow)
        assert_eq!(fit_dimensions(60000, 40000, 1280), (1280, 853));
        // max_size of 1
        assert_eq!(fit_dimensions(1920, 1080, 1), (1, 1));
    }
}
