use moka::future::Cache;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

/// Three-level cache service for thumbnails
pub struct CacheService {
    // L1: Memory cache
    memory_cache: Arc<Cache<String, Vec<u8>>>,
    // L2: Disk cache directory
    disk_cache_dir: PathBuf,
}

impl CacheService {
    /// Create a new cache service
    pub async fn new(cache_dir: &PathBuf) -> Result<Self, std::io::Error> {
        // Ensure cache directory exists
        fs::create_dir_all(cache_dir).await?;

        let memory_cache = Arc::new(Cache::builder()
            .max_capacity(1000)
            .time_to_live(std::time::Duration::from_secs(3600))
            .build());

        Ok(Self {
            memory_cache,
            disk_cache_dir: cache_dir.clone(),
        })
    }

    /// Get thumbnail from cache
    pub async fn get_thumbnail(&self, file_id: &str, size: &str) -> Option<Vec<u8>> {
        let cache_key = format!("{}_{}", file_id, size);

        // 1. Check memory cache
        if let Some(data) = self.memory_cache.get(&cache_key).await {
            tracing::debug!("L1 cache hit for {}", cache_key);
            return Some(data);
        }

        // 2. Check disk cache
        let disk_path = self.disk_cache_dir.join(&cache_key);
        if let Ok(data) = fs::read(&disk_path).await {
            tracing::debug!("L2 cache hit for {}", cache_key);
            // Populate memory cache
            self.memory_cache.insert(cache_key, data.clone()).await;
            return Some(data);
        }

        None
    }

    /// Store thumbnail in cache
    pub async fn put_thumbnail(&self, file_id: &str, size: &str, data: &[u8]) -> std::io::Result<()> {
        let cache_key = format!("{}_{}", file_id, size);

        // 1. Store in memory cache
        self.memory_cache.insert(cache_key.clone(), data.to_vec()).await;

        // 2. Store in disk cache
        let disk_path = self.disk_cache_dir.join(&cache_key);
        fs::write(&disk_path, data).await?;

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
