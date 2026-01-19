use bytes::Bytes;
use moka::future::Cache;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

/// Three-level cache service for thumbnails
pub struct CacheService {
    // L1: Memory cache - using Bytes for efficient cloning
    memory_cache: Arc<Cache<String, Bytes>>,
    // L2: Disk cache directory
    disk_cache_dir: PathBuf,
}

impl CacheService {
    /// Create a new cache service with configurable parameters
    pub async fn new(
        cache_dir: &PathBuf,
        max_capacity: usize,
        ttl_seconds: u64,
    ) -> Result<Self, std::io::Error> {
        // Ensure cache directory exists
        fs::create_dir_all(cache_dir).await?;

        let memory_cache = Arc::new(Cache::builder()
            .max_capacity(max_capacity as u64)
            .time_to_live(std::time::Duration::from_secs(ttl_seconds))
            .build());

        Ok(Self {
            memory_cache,
            disk_cache_dir: cache_dir.clone(),
        })
    }

    /// Create a new cache service with default settings (for backward compatibility)
    pub async fn new_with_defaults(cache_dir: &PathBuf) -> Result<Self, std::io::Error> {
        Self::new(cache_dir, 1000, 3600).await
    }

    /// Get thumbnail from cache
    /// Returns Bytes for efficient cloning in downstream operations
    pub async fn get_thumbnail(&self, file_id: &str, size: &str) -> Option<Bytes> {
        let cache_key = format!("{}_{}", file_id, size);

        // 1. Check memory cache - Bytes supports cheap cloning
        if let Some(data) = self.memory_cache.get(&cache_key).await {
            return Some(data);
        }

        // 2. Check disk cache
        let disk_path = self.disk_cache_dir.join(&cache_key);
        if let Ok(data) = fs::read(&disk_path).await {
            // Convert to Bytes - cheap clone for memory cache insertion
            let bytes = Bytes::from(data);
            // Clone for memory cache (Bytes clone is O(1))
            self.memory_cache.insert(cache_key.clone(), bytes.clone()).await;
            return Some(bytes);
        }

        None
    }

    /// Get thumbnail disk cache path (for streaming)
    /// Returns None if not in disk cache
    pub fn get_thumbnail_disk_path(&self, file_id: &str, size: &str) -> Option<PathBuf> {
        let cache_key = format!("{}_{}", file_id, size);
        let disk_path = self.disk_cache_dir.join(&cache_key);
        if disk_path.exists() {
            Some(disk_path)
        } else {
            None
        }
    }

    /// Check if thumbnail exists in cache (memory or disk)
    pub async fn has_thumbnail(&self, file_id: &str, size: &str) -> bool {
        let cache_key = format!("{}_{}", file_id, size);

        // Check memory cache first
        if self.memory_cache.get(&cache_key).await.is_some() {
            return true;
        }

        // Check disk cache
        let disk_path = self.disk_cache_dir.join(&cache_key);
        disk_path.exists()
    }

    /// Store thumbnail in cache
    /// Accepts Bytes or Vec<u8> for flexibility
    pub async fn put_thumbnail(&self, file_id: &str, size: &str, data: &[u8]) -> std::io::Result<()> {
        let cache_key = format!("{}_{}", file_id, size);

        // Convert to Bytes for memory cache (efficient storage)
        let bytes = Bytes::from(data.to_vec());

        // Store in memory cache
        self.memory_cache.insert(cache_key.clone(), bytes).await;

        // Store in disk cache
        let disk_path = self.disk_cache_dir.join(&cache_key);
        fs::write(&disk_path, data).await?;

        Ok(())
    }

    /// Alternative put method that accepts Bytes directly
    /// Avoids reallocation if caller already has Bytes
    pub async fn put_thumbnail_bytes(&self, file_id: &str, size: &str, data: Bytes) -> std::io::Result<()> {
        let cache_key = format!("{}_{}", file_id, size);

        // Store in memory cache (Bytes is efficient)
        self.memory_cache.insert(cache_key.clone(), data.clone()).await;

        // Store in disk cache
        let disk_path = self.disk_cache_dir.join(&cache_key);
        fs::write(&disk_path, &data).await?;

        Ok(())
    }

    /// Delete thumbnail from cache
    pub async fn delete_thumbnail(&self, file_id: &str, size: Option<&str>) {
        let keys: Vec<String> = match size {
            Some(s) => vec![format!("{}_{}", file_id, s)],
            None => vec![
                format!("{}_small", file_id),
                format!("{}_medium", file_id),
                format!("{}_large", file_id),
            ],
        };

        for key in keys {
            self.memory_cache.invalidate(&key).await;

            let disk_path = self.disk_cache_dir.join(&key);
            let _ = fs::remove_file(&disk_path).await;
        }
    }

    /// Clear all cache
    pub async fn clear_all(&self) -> std::io::Result<()> {
        self.memory_cache.invalidate_all();

        let mut entries = tokio::fs::read_dir(&self.disk_cache_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_file() {
                fs::remove_file(entry.path()).await?;
            }
        }

        Ok(())
    }

    /// Get cache size in MB
    pub async fn get_cache_size_mb(&self) -> std::io::Result<f64> {
        let mut total_size = 0u64;

        let mut entries = tokio::fs::read_dir(&self.disk_cache_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_file() {
                total_size += entry.metadata().await?.len();
            }
        }

        Ok(total_size as f64 / (1024.0 * 1024.0))
    }

    /// Get disk cache directory
    pub fn get_disk_cache_dir(&self) -> &PathBuf {
        &self.disk_cache_dir
    }
}
