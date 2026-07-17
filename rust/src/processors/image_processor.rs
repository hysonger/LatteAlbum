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
        target_size: u32,
        quality: f32,
        fit_to_height: bool,
    ) -> Result<Option<Vec<u8>>, ProcessingError> {
        let path = path.to_path_buf();
        let orientation = read_exif_orientation(&path);
        tokio::task::spawn_blocking(move || {
            use image::{DynamicImage, ImageReader};

            let mut img = ImageReader::open(path)?.decode()?;

            if let Some(orientation) = orientation {
                img.apply_orientation(orientation);
            }

            // If target_size is 0, return full-size transcoded image (no resize)
            let result_img = if target_size == 0 {
                // 先转为 RGBA8 保留 alpha，再用 ImageRgba8 包装后 to_rgb8()
                // 这样会对透明/半透明区域进行白色背景合成，避免颜色错误
                DynamicImage::ImageRgba8(img.to_rgba8()).to_rgb8()
            } else {
                // thumbnail(w, h) - 缩放到不超过 w×h 范围，保持宽高比
                let thumb = if fit_to_height {
                    // fit_to_height=true: 按固定高度缩放
                    // 目标高度 = target_size，需要计算对应的宽度
                    let ratio = img.width() as f64 / img.height() as f64;
                    let target_width = (target_size as f64 * ratio) as u32;
                    img.thumbnail(target_width, target_size)
                } else {
                    // fit_to_height=false: 按固定宽度缩放
                    // 目标宽度 = target_size，高度按比例计算
                    img.thumbnail(target_size, u32::MAX)
                };
                let thumb = thumb.to_rgba8();
                // 转为 RGBA8 保留 alpha，再用 ImageRgba8 包装后 to_rgb8() 进行白色背景合成
                DynamicImage::ImageRgba8(thumb).to_rgb8()
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

    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            tracing::debug!("[{}] Failed to open file for EXIF: {}", file_name, e);
            return;
        }
    };

    // Use Reader to parse EXIF data from the image file
    let exif = match Reader::new().read_from_container(&mut std::io::BufReader::new(file)) {
        Ok(e) => e,
        Err(e) => {
            // HEIC files may have EXIF in non-standard format
            // Log at debug level since this is expected for some formats
            tracing::debug!("[{}] No EXIF data available (may be HEIC format issue): {}", file_name, e);
            return;
        }
    };

    // GPS DMS 原始值暂存：Lat/Lon 与各自的 Ref 是独立 tag，出现顺序不可预测
    let mut lat_rational: Option<Vec<exif::Rational>> = None;
    let mut lon_rational: Option<Vec<exif::Rational>> = None;
    let mut lat_ref: Option<u8> = None;
    let mut lon_ref: Option<u8> = None;

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
            exif::Tag::FocalLength
                // Focal length (e.g., "50 mm")
                if !value_str.is_empty() => {
                    metadata.focal_length = Some(value_str);
                }

            // --- GPS Coordinates ---
            // 使用 Value::Rational / Value::Ascii 原始枚举匹配，避免依赖 display_as 的字符串格式。
            // GPSLatitude/GPSLongitude 是 3 个 Rational 数组：[度, 分, 秒]。
            exif::Tag::GPSLatitude => {
                if let exif::Value::Rational(ref v) = field.value {
                    if !v.is_empty() {
                        lat_rational = Some(v.clone());
                    }
                }
            }
            exif::Tag::GPSLatitudeRef => {
                if let exif::Value::Ascii(ref v) = field.value {
                    if let Some(first) = v.first() {
                        if let Some(&b) = first.first() {
                            lat_ref = Some(b);
                        }
                    }
                }
            }
            exif::Tag::GPSLongitude => {
                if let exif::Value::Rational(ref v) = field.value {
                    if !v.is_empty() {
                        lon_rational = Some(v.clone());
                    }
                }
            }
            exif::Tag::GPSLongitudeRef => {
                if let exif::Value::Ascii(ref v) = field.value {
                    if let Some(first) = v.first() {
                        if let Some(&b) = first.first() {
                            lon_ref = Some(b);
                        }
                    }
                }
            }

            _ => {}
        }
    }

    // 循环结束后换算 DMS → 十进制度数，写入 metadata
    if let (Some(lat_dms), Some(lon_dms), Some(lat_r), Some(lon_r)) =
        (lat_rational, lon_rational, lat_ref, lon_ref)
    {
        if let (Some(lat), Some(lon)) = (
            dms_to_decimal(&lat_dms, lat_r, true),
            dms_to_decimal(&lon_dms, lon_r, false),
        ) {
            // 过滤 (0.0, 0.0)：几内亚湾默认值，通常表示缺失数据
            if !(lat == 0.0 && lon == 0.0) {
                metadata.gps_latitude = Some(round6(lat));
                metadata.gps_longitude = Some(round6(lon));
            }
        }
    }
}

/// Convert EXIF GPS DMS (deg/min/sec as Rational array) + N/S/E/W ref to decimal degrees.
/// Returns None if the data is malformed (denom=0, missing components, out-of-range).
/// `is_latitude`: true → range [-90, 90]; false → range [-180, 180].
fn dms_to_decimal(dms: &[exif::Rational], ref_byte: u8, is_latitude: bool) -> Option<f64> {
    // EXIF 规范要求 GPS 坐标包含完整的度/分/秒三个 Rational 分量
    if dms.len() < 3 {
        return None;
    }

    let to_f64 = |r: &exif::Rational| -> Option<f64> {
        if r.denom == 0 {
            None
        } else {
            Some(r.num as f64 / r.denom as f64)
        }
    };

    let deg = to_f64(&dms[0])?;
    let min = to_f64(&dms[1])?;
    let sec = to_f64(&dms[2])?;

    if deg < 0.0 || min < 0.0 || sec < 0.0 || min >= 60.0 || sec >= 60.0 {
        return None;
    }

    let mut decimal = deg + min / 60.0 + sec / 3600.0;

    let limit = if is_latitude { 90.0 } else { 180.0 };
    if decimal > limit {
        return None;
    }

    // S/W 为负
    let negative = matches!(ref_byte, b'S' | b's' | b'W' | b'w');
    if negative {
        decimal = -decimal;
    }

    Some(decimal)
}

/// Round to 6 decimal places (~0.11m precision) to remove floating-point noise.
fn round6(v: f64) -> f64 {
    (v * 1_000_000.0).round() / 1_000_000.0
}

/// 从文件读取 EXIF Orientation 值（tag 274），用于缩略图方向校正
pub(crate) fn read_exif_orientation(path: &Path) -> Option<image::metadata::Orientation> {
    let file = std::fs::File::open(path).ok()?;
    let exif = exif::Reader::new()
        .read_from_container(&mut std::io::BufReader::new(file))
        .ok()?;
    let orientation_field = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)?;
    let value = orientation_field.value.get_uint(0)?;
    image::metadata::Orientation::from_exif(value as u8)
}

/// Clean EXIF string value - remove leading/trailing quotes added by the library
pub(crate) fn clean_exif_string(s: &str) -> String {
    let s = s.trim();
    if s.len() >= 2
        && ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\''))) {
            return s[1..s.len()-1].to_string();
        }
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_exif_tag_from_raw() {
        assert_eq!(ExifTag::from_raw("Exif", 36867), Some(ExifTag::DateTimeOriginal));
        assert_eq!(ExifTag::from_raw("Exif", 36868), Some(ExifTag::DateTimeDigitized));
        assert_eq!(ExifTag::from_raw("Tiff", 306), Some(ExifTag::DateTime));
        assert_eq!(ExifTag::from_raw("Tiff", 271), Some(ExifTag::Make));
        assert_eq!(ExifTag::from_raw("Tiff", 272), Some(ExifTag::Model));
        assert_eq!(ExifTag::from_raw("Exif", 33434), Some(ExifTag::ExposureTime));
        assert_eq!(ExifTag::from_raw("Exif", 33437), Some(ExifTag::FNumber));
        assert_eq!(ExifTag::from_raw("Exif", 34855), Some(ExifTag::ISOSpeedRatings));
        assert_eq!(ExifTag::from_raw("Exif", 37386), Some(ExifTag::FocalLength));
        assert_eq!(ExifTag::from_raw("Exif", 34973), Some(ExifTag::LensModel));
    }

    #[test]
    fn test_exif_tag_from_raw_invalid() {
        assert_eq!(ExifTag::from_raw("Exif", 65535), None);
        assert_eq!(ExifTag::from_raw("Unknown", 306), None);
        assert_eq!(ExifTag::from_raw("Tiff", 0), None);
    }

    #[test]
    fn test_exif_tag_description() {
        assert_eq!(ExifTag::DateTimeOriginal.description(), "拍摄时间");
        assert_eq!(ExifTag::DateTimeDigitized.description(), "数字化时间");
        assert_eq!(ExifTag::DateTime.description(), "文件修改时间");
        assert_eq!(ExifTag::Make.description(), "相机厂商");
        assert_eq!(ExifTag::Model.description(), "相机型号");
        assert_eq!(ExifTag::ExposureTime.description(), "快门速度");
        assert_eq!(ExifTag::FNumber.description(), "光圈值");
        assert_eq!(ExifTag::ISOSpeedRatings.description(), "ISO感光度");
        assert_eq!(ExifTag::FocalLength.description(), "焦距");
    }

    #[test]
    fn test_exif_tag_all_variants() {
        let tags = [
            ExifTag::DateTimeOriginal,
            ExifTag::DateTimeDigitized,
            ExifTag::DateTime,
            ExifTag::OffsetTimeOriginal,
            ExifTag::OffsetTime,
            ExifTag::Make,
            ExifTag::Model,
            ExifTag::LensModel,
            ExifTag::ExposureTime,
            ExifTag::FNumber,
            ExifTag::ISOSpeedRatings,
            ExifTag::FocalLength,
        ];

        for tag in &tags {
            let desc = tag.description();
            assert!(!desc.is_empty());
        }
    }

    #[test]
    fn test_standard_image_processor_new() {
        let processor = StandardImageProcessor::new();
        assert!(processor.supports(Path::new("test.jpg")));
        assert!(processor.supports(Path::new("test.png")));
        assert!(!processor.supports(Path::new("test.mp4")));
    }

    #[test]
    fn test_standard_image_processor_priority() {
        let processor = StandardImageProcessor::new();
        assert_eq!(processor.priority(), 10);
    }

    #[test]
    fn test_standard_image_processor_media_type() {
        let processor = StandardImageProcessor::new();
        assert_eq!(processor.media_type(), MediaType::Image);
    }

    #[test]
    fn test_standard_image_processor_default() {
        let processor = StandardImageProcessor;
        assert!(processor.supports(Path::new("test.jpg")));
    }

    #[test]
    fn test_clean_exif_string() {
        assert_eq!(clean_exif_string("\"value\""), "value");
        assert_eq!(clean_exif_string("'value'"), "value");
        assert_eq!(clean_exif_string("value"), "value");
        assert_eq!(clean_exif_string("  \"value\"  "), "value");
        assert_eq!(clean_exif_string("no quotes"), "no quotes");
        assert_eq!(clean_exif_string(""), "");
    }

    #[test]
    fn test_clean_exif_string_edge_cases() {
        assert_eq!(clean_exif_string("\""), "\"");
        assert_eq!(clean_exif_string("'"), "'");
        assert_eq!(clean_exif_string("\"\""), "");
        assert_eq!(clean_exif_string("''"), "");
        assert_eq!(clean_exif_string("\"incomplete"), "\"incomplete");
    }

    #[test]
    fn test_exif_tag_gps() {
        assert_eq!(ExifTag::from_raw("Gps", 1), Some(ExifTag::GPSLatitudeRef));
        assert_eq!(ExifTag::from_raw("Gps", 2), Some(ExifTag::GPSLatitude));
        assert_eq!(ExifTag::from_raw("Gps", 3), Some(ExifTag::GPSLongitudeRef));
        assert_eq!(ExifTag::from_raw("Gps", 4), Some(ExifTag::GPSLongitude));
        assert_eq!(ExifTag::from_raw("Gps", 6), Some(ExifTag::GPSAltitude));
    }

    #[test]
    fn test_exif_tag_vendor_specific() {
        assert_eq!(ExifTag::from_raw("Tiff", 305), Some(ExifTag::Software));
        assert_eq!(ExifTag::from_raw("Exif", 37520), Some(ExifTag::SerialNumber));
    }

    #[test]
    fn test_exif_tag_extended() {
        assert_eq!(ExifTag::from_raw("Exif", 34850), Some(ExifTag::ExposureProgram));
        assert_eq!(ExifTag::from_raw("Exif", 37379), Some(ExifTag::ExposureBiasValue));
        assert_eq!(ExifTag::from_raw("Exif", 41986), Some(ExifTag::ExposureMode));
        assert_eq!(ExifTag::from_raw("Exif", 37383), Some(ExifTag::MeteringMode));
    }

    // ---- GPS DMS → decimal tests ----

    fn r(num: u32, denom: u32) -> exif::Rational {
        exif::Rational { num, denom }
    }

    #[test]
    fn test_dms_to_decimal_beijing() {
        // 39°54'12"N → 39.903333
        let lat = dms_to_decimal(&[r(39, 1), r(54, 1), r(12, 1)], b'N', true).unwrap();
        assert!((lat - 39.903333).abs() < 1e-5);
        // 116°23'30"E → 116.391667
        let lon = dms_to_decimal(&[r(116, 1), r(23, 1), r(30, 1)], b'E', false).unwrap();
        assert!((lon - 116.391667).abs() < 1e-5);
    }

    #[test]
    fn test_dms_to_decimal_south_west() {
        // 33°51'54"S → -33.865
        let lat = dms_to_decimal(&[r(33, 1), r(51, 1), r(54, 1)], b'S', true).unwrap();
        assert!((lat - (-33.865)).abs() < 1e-5);
        // 151°12'34"W → -151.209444
        let lon = dms_to_decimal(&[r(151, 1), r(12, 1), r(34, 1)], b'W', false).unwrap();
        assert!((lon - (-151.209444)).abs() < 1e-5);
    }

    #[test]
    fn test_dms_to_decimal_equator_prime_meridian() {
        let lat = dms_to_decimal(&[r(0, 1), r(0, 1), r(0, 1)], b'N', true).unwrap();
        assert_eq!(lat, 0.0);
        let lon = dms_to_decimal(&[r(0, 1), r(0, 1), r(0, 1)], b'E', false).unwrap();
        assert_eq!(lon, 0.0);
    }

    #[test]
    fn test_dms_to_decimal_missing_seconds() {
        // EXIF 规范要求完整的度/分/秒：只有度/分视为数据不完整 → None
        assert!(dms_to_decimal(&[r(39, 1), r(54, 1)], b'N', true).is_none());
    }

    #[test]
    fn test_dms_to_decimal_fractional_seconds() {
        // 39°54'12.5" → 39.9034722...
        let lat = dms_to_decimal(&[r(39, 1), r(54, 1), r(125, 10)], b'N', true).unwrap();
        assert!((lat - 39.903472).abs() < 1e-5);
    }

    #[test]
    fn test_dms_to_decimal_denom_zero() {
        // 分母为 0 → 返回 None
        assert!(dms_to_decimal(&[r(39, 0), r(54, 1), r(12, 1)], b'N', true).is_none());
        assert!(dms_to_decimal(&[r(39, 1), r(54, 0), r(12, 1)], b'N', true).is_none());
        assert!(dms_to_decimal(&[r(39, 1), r(54, 1), r(12, 0)], b'N', true).is_none());
    }

    #[test]
    fn test_dms_to_decimal_out_of_range() {
        // 纬度 > 90 → None
        assert!(dms_to_decimal(&[r(91, 1), r(0, 1), r(0, 1)], b'N', true).is_none());
        // 经度 > 180 → None
        assert!(dms_to_decimal(&[r(181, 1), r(0, 1), r(0, 1)], b'E', false).is_none());
        // 分/秒超 60 → None
        assert!(dms_to_decimal(&[r(39, 1), r(60, 1), r(0, 1)], b'N', true).is_none());
        assert!(dms_to_decimal(&[r(39, 1), r(0, 1), r(60, 1)], b'N', true).is_none());
    }

    #[test]
    fn test_dms_to_decimal_too_short() {
        // 只有度：拒绝
        assert!(dms_to_decimal(&[r(39, 1)], b'N', true).is_none());
        assert!(dms_to_decimal(&[], b'N', true).is_none());
    }

    #[test]
    fn test_dms_to_decimal_boundary_values() {
        // 北极 90°0'0"N
        let lat = dms_to_decimal(&[r(90, 1), r(0, 1), r(0, 1)], b'N', true).unwrap();
        assert_eq!(lat, 90.0);
        // 南极 -90°
        let lat = dms_to_decimal(&[r(90, 1), r(0, 1), r(0, 1)], b'S', true).unwrap();
        assert_eq!(lat, -90.0);
        // 反子午线 180°E
        let lon = dms_to_decimal(&[r(180, 1), r(0, 1), r(0, 1)], b'E', false).unwrap();
        assert_eq!(lon, 180.0);
    }

    #[test]
    fn test_round6() {
        assert_eq!(round6(39.903333333333335), 39.903333);
        assert_eq!(round6(116.39166666666667), 116.391667);
        assert_eq!(round6(0.0), 0.0);
        assert_eq!(round6(-33.86500000000001), -33.865);
    }
}
