
use image::{DynamicImage, ImageFormat};
use std::fs;
use std::path::Path;

pub struct ImageProcessor;

impl ImageProcessor {
    pub fn load_image(path: &str) -> Result<DynamicImage, String> {
        image::open(path).map_err(|e| format!("Failed to load image: {}", e))
    }

    pub fn resize_image(
        img: &DynamicImage,
        width: u32,
        height: u32,
    ) -> DynamicImage {
        img.resize_exact(width, height, image::imageops::FilterType::Lanczos3)
    }

    pub fn convert_format(
        img: &DynamicImage,
        format: ImageFormat,
        output_path: &str,
    ) -> Result<(), String> {
        let mut buffer = Vec::new();
        img.write_to(&mut buffer, format)
            .map_err(|e| format!("Failed to encode image: {}", e))?;

        fs::write(output_path, buffer)
            .map_err(|e| format!("Failed to write image: {}", e))
    }

    pub fn process_image(
        input_path: &str,
        output_path: &str,
        width: u32,
        height: u32,
        format: ImageFormat,
    ) -> Result<(), String> {
        let img = Self::load_image(input_path)?;
        let resized = Self::resize_image(&img, width, height);
        Self::convert_format(&resized, format, output_path)
    }

    pub fn batch_process(
        inputs: Vec<(&str, &str)>,
        width: u32,
        height: u32,
        format: ImageFormat,
    ) -> Vec<Result<(), String>> {
        inputs
            .into_iter()
            .map(|(input, output)| {
                Self::process_image(input, output, width, height, format)
            })
            .collect()
    }
}

pub fn validate_image_path(path: &str) -> bool {
    Path::new(path).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_image_loading() {
        let result = ImageProcessor::load_image("nonexistent.jpg");
        assert!(result.is_err());
    }

    #[test]
    fn test_path_validation() {
        assert!(!validate_image_path("fake_path.png"));
    }

    #[test]
    fn test_batch_processing() {
        let inputs = vec![];
        let results = ImageProcessor::batch_process(inputs, 100, 100, ImageFormat::Png);
        assert_eq!(results.len(), 0);
    }
}