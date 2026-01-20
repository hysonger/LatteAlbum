//! Processor integration tests

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Arc;
    use latte_album::processors::{ProcessorRegistry, image_processor::StandardImageProcessor, heif_processor::HeifImageProcessor, video_processor::VideoProcessor};

    /// Create a fully initialized processor registry with all processors registered
    fn create_test_processor_registry() -> ProcessorRegistry {
        let mut registry = ProcessorRegistry::new(None);
        registry.register(Arc::new(StandardImageProcessor::new()));
        registry.register(Arc::new(HeifImageProcessor::new(None)));
        registry.register(Arc::new(VideoProcessor::new(None)));
        registry
    }

    #[tokio::test]
    async fn test_processor_registry_lookup_jpeg() {
        let registry = create_test_processor_registry();
        let processor = registry.find_processor(Path::new("test.jpg"));
        assert!(processor.is_some());
    }

    #[tokio::test]
    async fn test_processor_registry_lookup_png() {
        let registry = create_test_processor_registry();
        let processor = registry.find_processor(Path::new("test.png"));
        assert!(processor.is_some());
    }

    #[tokio::test]
    async fn test_processor_registry_lookup_heic() {
        let registry = create_test_processor_registry();
        let processor = registry.find_processor(Path::new("test.heic"));
        assert!(processor.is_some());
    }

    #[tokio::test]
    async fn test_processor_registry_lookup_unsupported() {
        let registry = create_test_processor_registry();
        let processor = registry.find_processor(Path::new("test.xyz"));
        assert!(processor.is_none());
    }
}
