/// Thumbnail generation utilities
pub mod thumbnail;

/// General utility functions
pub fn format_file_size(size_bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size_bytes >= GB {
        format!("{:.2} GB", size_bytes as f64 / GB as f64)
    } else if size_bytes >= MB {
        format!("{:.2} MB", size_bytes as f64 / MB as f64)
    } else if size_bytes >= KB {
        format!("{:.2} KB", size_bytes as f64 / KB as f64)
    } else {
        format!("{} B", size_bytes)
    }
}

/// Parse file extension from path
pub fn get_file_extension(path: &str) -> Option<String> {
    // Check if there's a dot in the path
    if !path.contains('.') {
        return None;
    }
    path.rsplit('.')
        .next()
        .map(|s| s.to_lowercase())
}

/// Check if file is a video based on extension
pub fn is_video_file(path: &str) -> bool {
    if let Some(ext) = get_file_extension(path) {
        matches!(
            ext.as_str(),
            "mp4" | "avi" | "mov" | "mkv" | "wmv" | "flv" | "webm"
        )
    } else {
        false
    }
}

/// Check if file is an image based on extension
pub fn is_image_file(path: &str) -> bool {
    if let Some(ext) = get_file_extension(path) {
        matches!(
            ext.as_str(),
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "tiff" | "heic" | "heif"
        )
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(2048), "2.00 KB");
        assert_eq!(format_file_size(1048576), "1.00 MB");
        assert_eq!(format_file_size(1073741824), "1.00 GB");
    }

    #[test]
    fn test_format_file_size_edge_cases() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(1023), "1023 B");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(2048), "2.00 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_file_extension("photo.jpg"), Some("jpg".to_string()));
        assert_eq!(get_file_extension("/path/to/photo.JPG"), Some("jpg".to_string()));
        assert_eq!(get_file_extension("photo"), None);
        assert_eq!(get_file_extension("/path/to.noextension"), Some("noextension".to_string()));
        assert_eq!(get_file_extension("photo.tar.gz"), Some("gz".to_string()));
        assert_eq!(get_file_extension("C:\\Users\\photo.JPG"), Some("jpg".to_string()));
    }

    #[test]
    fn test_is_video_file() {
        assert!(is_video_file("video.mp4"));
        assert!(is_video_file("video.MOV"));
        assert!(is_video_file("video.AVI"));
        assert!(is_video_file("video.mkv"));
        assert!(is_video_file("video.wmv"));
        assert!(is_video_file("video.flv"));
        assert!(is_video_file("video.webm"));
        assert!(!is_video_file("photo.jpg"));
        assert!(!is_video_file("document.pdf"));
        assert!(!is_video_file("video"));
    }

    #[test]
    fn test_is_image_file() {
        assert!(is_image_file("photo.jpg"));
        assert!(is_image_file("photo.HEIC"));
        assert!(is_image_file("photo.JPEG"));
        assert!(is_image_file("photo.png"));
        assert!(is_image_file("photo.gif"));
        assert!(is_image_file("photo.bmp"));
        assert!(is_image_file("photo.webp"));
        assert!(is_image_file("photo.tiff"));
        assert!(!is_image_file("video.mp4"));
        assert!(!is_image_file("document.pdf"));
        assert!(!is_image_file("photo"));
    }

    #[test]
    fn test_combined_utils_usage() {
        let test_cases = vec![
            ("video.mp4", true, false),
            ("photo.jpg", false, true),
            ("document.pdf", false, false),
        ];

        for (path, expected_video, expected_image) in test_cases {
            assert_eq!(is_video_file(path), expected_video, "is_video_file({})", path);
            assert_eq!(is_image_file(path), expected_image, "is_image_file({})", path);
        }
    }
}
