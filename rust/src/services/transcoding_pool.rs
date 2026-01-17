//! 图片转码专用线程池服务
//! 使用 rayon 为 CPU 密集型的图片转码任务创建独立线程池

use rayon::ThreadPool;
use std::sync::Arc;

/// 图片转码专用线程池（CPU 密集型任务）
///
/// 该线程池与 Tokio 的阻塞线程池分离，专门用于：
/// - 图片解码（JPEG/PNG/HEIC 等）
/// - 图片缩放/缩略图生成
/// - JPEG/HEIC 编码
#[derive(Clone)]
pub struct TranscodingPool {
    inner: Arc<ThreadPool>,
}

impl TranscodingPool {
    /// 创建新的转码线程池
    ///
    /// # Arguments
    ///
    /// * `num_threads` - 线程数量，默认使用 CPU 核心数
    pub fn new(num_threads: usize) -> Self {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .expect("Failed to build transcoding thread pool");

        Self {
            inner: Arc::new(pool),
        }
    }

    /// 在转码线程池中执行任务并等待结果
    ///
    /// # Arguments
    ///
    /// * `f` - 要在线程池中执行的闭包
    ///
    /// # Returns
    ///
    /// 闭包的返回值
    pub fn scope<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&rayon::Scope<'_>) -> R + Send,
        R: Send,
    {
        self.inner.scope(f)
    }

    /// 在转码线程池中异步执行任务（不等待结果）
    ///
    /// 注意：由于 rayon 的 spawn 不返回 JoinHandle，
    /// 如果需要等待结果，请使用 `scope` 方法
    pub fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.inner.spawn(f);
    }
}

impl Default for TranscodingPool {
    fn default() -> Self {
        Self::new(4)
    }
}
