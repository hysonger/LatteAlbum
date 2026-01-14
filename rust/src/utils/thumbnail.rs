use image::{DynamicImage, GenericImageView};
use std::io::Cursor;

/// Generate a thumbnail from an image
pub fn generate_thumbnail(
    image: &DynamicImage,
    target_width: u32,
    quality: f32,
) -> Result<Vec<u8>, String> {
    // Calculate dimensions maintaining aspect ratio
    let ratio = image.height() as f64 / image.width() as f64;
    let target_height = (target_width as f64 * ratio) as u32;

    // Resize image using Lanczos3 for high quality
    let thumbnail = image.resize(target_width, target_height, image::imageops::FilterType::Lanczos3);

    // Convert to RGB (JPEG doesn't support alpha)
    let rgb_thumbnail = thumbnail.to_rgb8();

    // Encode as JPEG
    let mut buffer = Cursor::new(Vec::new());
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
        &mut buffer,
        (quality * 100.0) as u8,
    );
    encoder
        .encode_image(&rgb_thumbnail)
        .map_err(|e| e.to_string())?;

    Ok(buffer.into_inner())
}

/// Resize an image to fit within the given dimensions
pub fn resize_to_fit(
    image: &DynamicImage,
    max_width: u32,
    max_height: u32,
) -> DynamicImage {
    let (width, height) = image.dimensions();

    if width <= max_width && height <= max_height {
        // Image is already small enough
        return image.clone();
    }

    // Calculate the scaling factor
    let scale_w = max_width as f64 / width as f64;
    let scale_h = max_height as f64 / height as f64;
    let scale = scale_w.min(scale_h);

    let new_width = (width as f64 * scale) as u32;
    let new_height = (height as f64 * scale) as u32;

    image.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
}

/// Crop an image to a square
pub fn crop_to_square(image: &DynamicImage) -> DynamicImage {
    let (width, height) = image.dimensions();
    let min_dim = width.min(height);

    let x = (width - min_dim) / 2;
    let y = (height - min_dim) / 2;

    image.crop_imm(x, y, min_dim, min_dim)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbImage;

    #[test]
    fn test_generate_thumbnail() {
        // Create a small test image (100x100 RGB)
        let mut img = RgbImage::new(100, 100);
        for x in 0..100 {
            for y in 0..100 {
                img.put_pixel(x, y, image::Rgb([255, 0, 0]));
            }
        }
        let dynamic = DynamicImage::ImageRgb8(img);

        let thumbnail = generate_thumbnail(&dynamic, 50, 0.8);

        assert!(thumbnail.is_ok());
        let bytes = thumbnail.unwrap();
        assert!(!bytes.is_empty());
        // JPEG header
        assert_eq!(bytes[0..2], [0xFF, 0xD8]);
    }

    #[test]
    fn test_resize_to_fit() {
        let large = RgbImage::new(1000, 800);
        let dynamic = DynamicImage::ImageRgb8(large);

        let resized = resize_to_fit(&dynamic, 500, 500);

        assert!(resized.width() <= 500);
        assert!(resized.height() <= 500);
    }

    #[test]
    fn test_crop_to_square() {
        let wide = RgbImage::new(200, 100);
        let dynamic = DynamicImage::ImageRgb8(wide);

        let cropped = crop_to_square(&dynamic);

        assert_eq!(cropped.width(), 100);
        assert_eq!(cropped.height(), 100);
    }
}
