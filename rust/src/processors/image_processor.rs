use crate::processors::processor_trait::{MediaMetadata, MediaProcessor, MediaType, ProcessingError};
use async_trait::async_trait;
use chrono::NaiveDateTime;
use std::path::Path;

/// EXIF Tag 枚举 - 基于实际日志分析
/// 用于文档化和扩展EXIF字段提取
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExifTag {
    // === 核心字段 (Core) ===
    // 时间相关
    DateTimeOriginal,   // 36867 - 拍摄时间
    DateTimeDigitized,  // 36868 - 数字化时间
    DateTime,           // 306 - 文件时间
    OffsetTimeOriginal, // 36881 - 拍摄时区
    OffsetTime,         // 36880 - 文件时区

    // 相机信息
    Make,               // 271 - 相机厂商
    Model,              // 272 - 相机型号
    LensModel,          // 34973 - 镜头型号

    // 曝光参数
    ExposureTime,       // 33434 - 快门速度
    FNumber,            // 33437 - 光圈值
    ISOSpeedRatings,    // 34855 - ISO感光度
    FocalLength,        // 37386 - 焦距

    // === 扩展字段 (Extended) - 考虑后续实现 ===
    ExposureProgram,    // 34850 - 曝光程序
    ExposureBiasValue,  // 37379 - 曝光补偿
    ExposureMode,       // 41986 - 曝光模式
    MeteringMode,       // 37383 - 测光模式
    WhiteBalance,       // 41987 - 白平衡
    Flash,              // 37385 - 闪光灯
    FocalLengthIn35mmFilm, // 37381 - 35mm等效焦距

    // === GPS 位置信息 ===
    GPSLatitudeRef,     // 1 - 纬度方向
    GPSLatitude,        // 2 - 纬度
    GPSLongitudeRef,    // 3 - 经度方向
    GPSLongitude,       // 4 - 经度
    GPSAltitude,        // 6 - 海拔
    GPSTimeStamp,       // 7 - GPS时间
    GPSDateStamp,       // 29 - GPS日期

    // === 厂商特定 (低优先级) ===
    Software,           // 305 - 软件版本
    SerialNumber,       // 37520 - 相机序列号
}

impl ExifTag {
    /// 从 (context, number) 转换为 ExifTag
    /// context: "Tiff", "Exif", "Gps", "Interop"
    pub fn from_raw(context: &str, number: u16) -> Option<Self> {
        match (context, number) {
            // 时间
            ("Exif", 36867) => Some(Self::DateTimeOriginal),
            ("Exif", 36868) => Some(Self::DateTimeDigitized),
            ("Tiff", 306) => Some(Self::DateTime),
            ("Exif", 36881) => Some(Self::OffsetTimeOriginal),
            ("Exif", 36880) => Some(Self::OffsetTime),

            // 相机
            ("Tiff", 271) => Some(Self::Make),
            ("Tiff", 272) => Some(Self::Model),
            ("Exif", 34973) => Some(Self::LensModel),

            // 曝光
            ("Exif", 33434) => Some(Self::ExposureTime),
            ("Exif", 33437) => Some(Self::FNumber),
            ("Exif", 34855) => Some(Self::ISOSpeedRatings),
            ("Exif", 37386) => Some(Self::FocalLength),

            // 扩展
            ("Exif", 34850) => Some(Self::ExposureProgram),
            ("Exif", 37379) => Some(Self::ExposureBiasValue),
            ("Exif", 41986) => Some(Self::ExposureMode),
            ("Exif", 37383) => Some(Self::MeteringMode),
            ("Exif", 41987) => Some(Self::WhiteBalance),
            ("Exif", 37385) => Some(Self::Flash),
            ("Exif", 37381) => Some(Self::FocalLengthIn35mmFilm),

            // GPS
            ("Gps", 1) => Some(Self::GPSLatitudeRef),
            ("Gps", 2) => Some(Self::GPSLatitude),
            ("Gps", 3) => Some(Self::GPSLongitudeRef),
            ("Gps", 4) => Some(Self::GPSLongitude),
            ("Gps", 6) => Some(Self::GPSAltitude),
            ("Gps", 7) => Some(Self::GPSTimeStamp),
            ("Gps", 29) => Some(Self::GPSDateStamp),

            // 厂商特定
            ("Tiff", 305) => Some(Self::Software),
            ("Exif", 37520) => Some(Self::SerialNumber),

            _ => None,
        }
    }

    /// 获取字段描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::DateTimeOriginal => "拍摄时间",
            Self::DateTimeDigitized => "数字化时间",
            Self::DateTime => "文件修改时间",
            Self::OffsetTimeOriginal => "拍摄时区",
            Self::OffsetTime => "文件时区",
            Self::Make => "相机厂商",
            Self::Model => "相机型号",
            Self::LensModel => "镜头型号",
            Self::ExposureTime => "快门速度",
            Self::FNumber => "光圈值",
            Self::ISOSpeedRatings => "ISO感光度",
            Self::FocalLength => "焦距",
            Self::ExposureProgram => "曝光程序",
            Self::ExposureBiasValue => "曝光补偿",
            Self::ExposureMode => "曝光模式",
            Self::MeteringMode => "测光模式",
            Self::WhiteBalance => "白平衡",
            Self::Flash => "闪光灯",
            Self::FocalLengthIn35mmFilm => "35mm等效焦距",
            Self::GPSLatitudeRef => "纬度方向",
            Self::GPSLatitude => "纬度",
            Self::GPSLongitudeRef => "经度方向",
            Self::GPSLongitude => "经度",
            Self::GPSAltitude => "海拔",
            Self::GPSTimeStamp => "GPS时间",
            Self::GPSDateStamp => "GPS日期",
            Self::Software => "软件版本",
            Self::SerialNumber => "相机序列号",
        }
    }
}

/// Standard image processor for JPEG, PNG, GIF, WebP, TIFF, BMP
pub struct StandardImageProcessor;

impl Default for StandardImageProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl StandardImageProcessor {
    pub fn new() -> Self {
        Self
    }

    const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff"];
}

#[async_trait]
impl MediaProcessor for StandardImageProcessor {
    fn supports(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            Self::SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str())
        } else {
            false
        }
    }

    fn priority(&self) -> i32 {
        10
    }

    fn media_type(&self) -> MediaType {
        MediaType::Image
    }

    async fn process(&self, path: &Path) -> Result<MediaMetadata, ProcessingError> {
        let mut metadata = MediaMetadata::default();

        // Get dimensions (format-specific for standard images)
        let (width, height) = get_image_dimensions(path)?;
        metadata.width = Some(width as i32);
        metadata.height = Some(height as i32);

        // Extract EXIF metadata for all supported image formats
        extract_exif(path, &mut metadata);

        // Set MIME type
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            metadata.mime_type = Some(match ext.to_lowercase().as_str() {
                "jpg" | "jpeg" => "image/jpeg".to_string(),
                "png" => "image/png".to_string(),
                "gif" => "image/gif".to_string(),
                "webp" => "image/webp".to_string(),
                "tiff" => "image/tiff".to_string(),
                "bmp" => "image/bmp".to_string(),
                _ => "image/jpeg".to_string(),
            });
        }

        Ok(metadata)
    }

    async fn generate_thumbnail(
        &self,
        path: &Path,
        target_width: u32,
        quality: f32,
    ) -> Result<Option<Vec<u8>>, ProcessingError> {
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            use image::ImageReader;

            let img = ImageReader::open(path)?.decode()?;

            // If target_width is 0, return full-size transcoded image (no resize)
            let result_img = if target_width == 0 {
                // Full size - just convert to RGB JPEG without resizing
                img.to_rgb8()
            } else {
                // Use thumbnail() method - fast integer algorithm, ~2x faster than resize(Triangle)
                // thumbnail() maintains aspect ratio and uses efficient downscaling
                let thumb = img.thumbnail(target_width, target_width);
                thumb.to_rgb8()
            };

            let mut bytes = Vec::new();
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                &mut bytes,
                (quality * 100.0) as u8,
            );
            encoder.encode_image(&result_img)?;

            Ok(Some(bytes))
        })
        .await
        .map_err(|e| ProcessingError::Processing(e.to_string()))?
    }
}

fn get_image_dimensions(path: &Path) -> Result<(u32, u32), ProcessingError> {
    use image::{ImageReader, GenericImageView};

    let img = ImageReader::open(path)?.decode()?;
    Ok(img.dimensions())
}

/// Extract EXIF metadata from image files (JPEG, HEIC, etc.)
/// Uses kamadak-exif which supports multiple formats
pub(crate) fn extract_exif(path: &Path, metadata: &mut MediaMetadata) {
    use exif::Reader;

    let _file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => {
            return;
        }
    };

    // Use Reader to parse EXIF data from the image file
    let exif = match Reader::new().read_from_container(&mut std::io::BufReader::new(file)) {
        Ok(e) => e,
        Err(_) => {
            // HEIC files may have EXIF in non-standard format
            // This is expected for some HEIC files, so silently skip
            return;
        }
    };

    for field in exif.fields() {
        let tag = field.tag;
        let value_str = clean_exif_string(&field.value.display_as(tag).to_string());

        match tag {
            // --- Time & Timestamp ---
            exif::Tag::DateTimeOriginal | exif::Tag::DateTimeDigitized => {
                if let Ok(ts) = NaiveDateTime::parse_from_str(&value_str, "%Y-%m-%d %H:%M:%S") {
                    if metadata.exif_timestamp.is_none() || tag == exif::Tag::DateTimeOriginal {
                        metadata.exif_timestamp = Some(ts);
                    }
                }
            }
            exif::Tag::OffsetTimeOriginal => {
                // Timezone offset from DateTimeOriginal (e.g., "+08:00")
                if !value_str.is_empty() {
                    metadata.exif_timezone_offset = Some(value_str);
                }
            }
            exif::Tag::OffsetTime => {
                // Fallback: timezone offset from DateTime
                if metadata.exif_timezone_offset.is_none() && !value_str.is_empty() {
                    metadata.exif_timezone_offset = Some(value_str);
                }
            }

            // --- Camera Info ---
            exif::Tag::Make => {
                if !value_str.is_empty() {
                    metadata.camera_make = Some(value_str);
                }
            }
            exif::Tag::Model => {
                if !value_str.is_empty() {
                    metadata.camera_model = Some(value_str);
                }
            }

            // --- Lens Info ---
            exif::Tag::LensModel => {
                if !value_str.is_empty() {
                    metadata.lens_model = Some(value_str);
                }
            }

            // --- Exposure Settings ---
            exif::Tag::FNumber => {
                // Aperture value (e.g., "2.8")
                if !value_str.is_empty() {
                    metadata.aperture = Some(value_str);
                }
            }
            exif::Tag::ExposureTime => {
                // Shutter speed (e.g., "1/1000")
                if !value_str.is_empty() {
                    metadata.exposure_time = Some(value_str);
                }
            }
            exif::Tag::ISOSpeed | exif::Tag::PhotographicSensitivity => {
                // ISO value - may be a single value or multiple
                if metadata.iso.is_none() {
                    if let Ok(iso) = value_str.parse::<i32>() {
                        if iso > 0 {
                            metadata.iso = Some(iso);
                        }
                    }
                }
            }
            exif::Tag::FocalLength => {
                // Focal length (e.g., "50 mm")
                if !value_str.is_empty() {
                    metadata.focal_length = Some(value_str);
                }
            }

            _ => {}
        }
    }
}

/// Clean EXIF string value - remove leading/trailing quotes added by the library
pub(crate) fn clean_exif_string(s: &str) -> String {
    let s = s.trim();
    // Remove surrounding quotes if present (both " and ')
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len()-1].to_string()
    } else {
        s.to_string()
    }
}
