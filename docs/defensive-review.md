# 防御性编程审查报告

全面代码审查，以防御性编程思路检查项目中逻辑不一致与潜在实现错误。

审查日期: 2026-06-07
修复日期: 2026-06-13（严重问题全部修复，高风险问题修复进行中）
审查范围: Rust 后端 (30 源文件)、Vue/TS 前端 (18 源文件)、测试代码 (15 文件)

---

## 一、严重问题（安全漏洞 / 崩溃风险 / 数据损坏）

### S1. 静态文件路径遍历漏洞 ✅ 已修复 (b595ab3)

**位置**: `rust/src/app.rs:155-175`

`serve_static` 直接拼接用户输入的 URL 路径到文件系统路径，缺少 `canonicalize()` + 路径前缀校验。攻击者可通过 `/assets/../../etc/passwd` 读取系统任意文件。

**风险**: 任意文件读取，可泄露数据库文件、环境变量、配置等敏感信息。

**修复**: 6 层防御 — 空路径检查 → null byte 检查 → `canonicalize()` → `starts_with()` 前缀校验 → 文件类型检查 → 服务文件。`AppState` 新增 `assets_base_path: Option<PathBuf>` 字段存储预 canonicalize 的基础路径。

### S2. `to_rgb8()` 对含 Alpha 通道图像可能 panic ✅ 已修复 (d551c67)

**位置**: `rust/src/processors/image_processor.rs:213, 227`

~~`DynamicImage::to_rgb8()` 对 `ImageRgba8` 等带 alpha 通道的变体可能直接 unwrap 失败。~~ 经深入分析 `image` crate 0.25.9 源码，`to_rgb8()` 对所有变体均有处理，实际不会 panic。**真正问题**：带透明通道的图像（PNG/WebP）在 `to_rgb8()` 时 alpha 通道被简单丢弃，未与白色背景做合成，导致透明/半透明区域颜色错误。严重程度降级为功能缺陷。

**风险**: 透明 PNG/WebP 缩略图颜色不正确。

**修复**: 先转为 RGBA8 再通过 `DynamicImage::ImageRgba8` 包装后 `to_rgb8()`，完成白色背景 alpha 合成。

### S3. DisplayMatrix 解析未对齐内存读取（Undefined Behavior） ✅ 已修复 (d551c67)

**位置**: `rust/src/processors/video_processor.rs:21-26`

将 `&[u8]` 指针直接 cast 为 `*const i32`，`u8` 仅有 1 字节对齐保证而 `i32` 要求 4 字节对齐。未对齐读取属于未定义行为。

**风险**: 在 ARM 平台可能导致 SIGBUS 或读取到错误值。

**修复**: 使用 `i32::from_le_bytes` 安全逐字节解析 DisplayMatrix（9 个 int32_t），零依赖。

### S4. 扫描服务并发竞态条件 ✅ 已修复 (d551c67)

**位置**: `rust/src/services/scan_service.rs:67-72`

`scan()` 方法中的 `is_scanning` 检查存在经典的 check-then-act 竞态条件：

```rust
if self.is_scanning.load(Ordering::SeqCst) { return; }
self.is_scanning.store(true, Ordering::SeqCst);
```

两个并发调用可同时通过检查，导致两个扫描任务同时运行。

**修复**: 使用 `AtomicBool::compare_exchange(false, true, ...)` 实现原子化的 check-and-set。

### S5. 扫描异常后状态卡死 ✅ 已修复 (d551c67)

**位置**: `rust/src/services/scan_service.rs:78-80`

若 `perform_scan()` 内部发生 panic，`is_scanning` 永远不会被设回 `false`。服务将永久认为"扫描中"，后续所有扫描请求都被拒绝。

**修复**: 新增 `ScanGuard` RAII 守护结构体，`Drop` 中调用 `store(false)`。`compare_exchange` 成功后立即创建 `_guard`，无论 `perform_scan()` 正常返回还是 panic，状态均被重置。

### S6. PhotoViewer ObjectURL 内存泄漏 ✅ 已修复 (d551c67)

**位置**: `frontend/src/components/PhotoViewer.vue:311-357`

每次翻页/切换文件时，旧的 Blob URL 直接被覆盖为 `undefined`，未调用 `URL.revokeObjectURL()`。`onUnmounted` 只释放最终 URL，中间翻过的全部泄漏。

**风险**: 长时间浏览（翻几十到上百张）会导致浏览器内存耗尽、页面卡顿甚至崩溃。

**修复**: 封装 `revokeImageUrl()` 辅助函数，在每次设置新 URL 前自动 `revoke` 旧 URL。所有 5 处 `createObjectURL` 调用点均改为先 `revokeImageUrl()` 再设置新值。

### S7. Gallery 级联加载所有页面 ✅ 已修复 (b595ab3)

**位置**: `frontend/src/stores/gallery.ts:46-49`

`loadPage` 完成后自动调用 `loadNextPage`，而 `loadNextPage` 内部又调用 `loadPage`，形成递归，导致一次性加载所有剩余页面。

**风险**: 打开页面时一次性加载全部数据，完全破坏分页，造成大量 API 请求和内存消耗。

**修复**: 删除 `loadPage` 内对 `loadNextPage` 的递归调用 + 修复 `Gallery.vue` 中 IntersectionObserver 哨兵管理（`setColumnSentinel` 自管理模式）。

---

## 二、高风险问题（逻辑错误 / 功能缺陷）

### H1. HEIF 处理器忽略 EXIF Orientation ✅ 已修复 (c622d8c)

**位置**: `rust/src/processors/heif_processor.rs:98-183`

`transcoding_generate_heic_thumbnail` 解码后直接缩放编码，完全未检查或应用 EXIF Orientation 标签。

**风险**: 竖拍照片缩略图方向错误。

**修复**: 复用 `image_processor::read_exif_orientation()` 读取 Orientation；90°/270° 旋转时交换宽高计算缩放尺寸；RGBA→RGB 前调用 `apply_orientation()` 校正方向。

### H2. 标准图片处理器同样忽略 EXIF Orientation ✅ 已修复 (c622d8c)

**位置**: `rust/src/processors/image_processor.rs:198-241`

`generate_thumbnail` 直接解码并缩放，`image` crate 的 `ImageReader::decode()` 默认不自动应用 Orientation。

**风险**: 与 H1 相同。

**修复**: 新增 `read_exif_orientation()` 函数（`pub(crate)`，HEIF 处理器共享）；解码后 `apply_orientation()`，在缩放前校正图像方向。

### H3. HEIF RGBA→JPEG 缺少 alpha 合成步骤 ✅ 已修复 (03f18af)

**位置**: `rust/src/processors/heif_processor.rs:170-181`

将 `RgbaImage` 包装为 `DynamicImage::ImageRgba8` 后直接编码为 JPEG，未做 alpha 合成。JPEG 不支持 alpha 通道。

**风险**: 含半透明区域的图片颜色不正确。

**修复**: 编码前执行 `DynamicImage::ImageRgba8(rgba_image).to_rgb8()` 完成白色背景 alpha 合成。

### H4. `INSERT OR REPLACE` 导致 `id` 不稳定 ✅ 已修复 (b9358f6)

**位置**: `rust/src/db/repository.rs:145, 348`

`INSERT OR REPLACE` 在 `file_path` UNIQUE 约束冲突时先 DELETE 旧行再 INSERT 新行。由于每次扫描生成新 UUID，旧 id 被替换，导致缩略图缓存全部失效（key 含 id），API URL 不稳定。

**风险**: 每次 rescan 后缩略图缓存 miss、分享链接失效。

**修复**: 改为 `INSERT INTO ... ON CONFLICT(file_path) DO UPDATE SET ...`，冲突时仅更新数据列，保留旧行的 `id` 不变。

### H5. `find_neighbors` 遗漏仅有 modify_time 的记录 ✅ 已修复 (03f18af)

**位置**: `rust/src/db/repository.rs:105-110`

WHERE 子句仅覆盖 `exif_timestamp` 和 `create_time` 两种情况，遗漏了 `exif_timestamp IS NULL AND create_time IS NULL AND modify_time < ?` 的第三种情况。

**风险**: 在只有修改时间的文件之间导航时会跳过这些文件。

**修复**: 新增第三种 WHERE 条件分支处理仅含 modify_time 的记录。

### H6. 分页参数无范围校验 ✅ 已修复 (03f18af)

**位置**: `rust/src/api/files.rs:84-85`

未对 `page` 和 `size` 做范围限制。客户端可传 `size=999999999` 导致大量内存分配和数据库压力，也可传负数。

**修复**: 添加 `page.max(0)` 和 `size.clamp(1, 200)` 范围限制。

### H7. `try_send` 静默丢弃关键状态消息 ✅ 已修复 (6d4dd55)

**位置**: `rust/src/websocket/scan_state.rs:197-232`

所有 `ScanStateManager` 方法使用 `try_send` 并忽略结果。channel（容量 1000）在大量 increment 消息下被填满，`Completed`/`Error`/`Cancelled` 消息被丢弃，导致前端永远认为扫描在进行中。

**修复**: `completed()` / `error()` / `cancelled()` 改为 `async fn`，内部使用 `send().await` 保证送达；`increment_success` / `increment_failure` 保持 `try_send`（允许丢弃个别增量，仅影响进度百分比精度）。

### H8. WebSocket 未在组件销毁时断开 ✅ 已修复 (03f18af)

**位置**: `frontend/src/views/HomeView.vue:413-416`

`onUnmounted` 只移除了回调，未调用 `scanProgressWs.disconnect()`。WebSocket 的 `onclose` 处理器会自动重连，形成无用的重连风暴。

**修复**: `onUnmounted` 中添加 `scanProgressWs.disconnect()` 调用。

### H9. WebSocket 重连无退避策略

**位置**: `frontend/src/services/websocket.ts:73-85`

固定 5 秒间隔无限重试，无指数退避，无最大重试次数限制。

**建议**: 实现指数退避 + 最大重试次数（如 5 次后停止）。

### H10. 首次扫描使用嵌套 Tokio Runtime ✅ 已修复 (03f18af)

**位置**: `rust/src/app.rs:198-204`

`spawn_blocking` 内部创建新的 `tokio::runtime::Runtime`，与主 runtime 的连接池、channel 等共享资源行为不可预期。且含 `unwrap()` 可能 panic。

**修复**: 直接在主 runtime 上使用 `tokio::spawn` 替代嵌套 Runtime。

### H11. setTimeout 未清理导致扫描状态竞态

**位置**: `frontend/src/views/HomeView.vue:247-252, 273-278`

`setTimeout` 返回值未保存，无法在组件卸载时清除。快速连续扫描时旧的 setTimeout 会错误重置 UI 状态。

**建议**: 保存 setTimeout ID，在 `onUnmounted` 中 `clearTimeout`。

---

## 三、中风险问题（数据不一致 / 防御不足）

### M1. 环境变量解析错误静默回退默认值

**位置**: `rust/src/config.rs:186-238`

`get_env_u16`/`get_env_u32`/`get_env_f32` 在 `parse()` 失败时静默返回 `Ok(default)`，不报错。用户配置 `LATTE_PORT=abc` 会静默使用默认值 8080。

**建议**: 解析失败时返回 `ConfigError::InvalidValue`。

### M2. 注释默认值与实际默认值不一致

**位置**: `rust/src/config.rs:40` vs `rust/src/config.rs:101`

注释说 `thumbnail_medium` 默认 450，实际代码默认 600。

### M3. 时间戳反序列化错误被静默吞掉

**位置**: `rust/src/db/models.rs:27`

`.ok()` 将解析失败静默转为 `None`。带时区（`2024-06-15T10:30:00Z`）或带毫秒的时间字符串会静默丢失。

**建议**: 记录 warn 日志，至少保留可观测性。

### M4. SQLite 缺少 WAL 模式和忙等待配置

**位置**: `rust/src/db/pool.rs`

连接池未设置 `journal_mode=WAL`、`busy_timeout` 等 pragma。默认 DELETE journal 模式并发性能差，`busy_timeout` 默认 0 会立即返回 `SQLITE_BUSY`。

**风险**: 扫描期间同时处理 API 请求时可能频繁遇到 `database is locked`。

### M5. `find_dates_with_files` 过滤参数被忽略

**位置**: `rust/src/db/repository.rs:121-125`

`_path_filter` 和 `_file_type` 参数以下划线前缀声明，函数体中完全未使用。SQL 查询硬编码了全量数据。

**风险**: 调用者以为传入了过滤条件，实际未生效。

### M6. `count_missing` 将全部路径加载到内存

**位置**: `rust/src/db/repository.rs:451-467`

对大型照片库（几十万张），将所有文件路径加载到 `Vec<String>` 后构建 `HashSet`，内存消耗与数据库大小成正比。

**建议**: 使用 SQL `NOT IN` 子查询在数据库端完成过滤。

### M7. 目录遍历可能因符号链接无限循环

**位置**: `rust/src/services/scan_service.rs:253-259`

`path.is_dir()` 默认跟随符号链接。循环符号链接（A -> B -> A）会导致无限递归。

**建议**: 使用 `entry.file_type()` 获取文件类型（不跟随链接），添加最大深度限制。

### M8. 权限错误中断整个扫描批次

**位置**: `rust/src/services/scan_service.rs:250`

`entries.next_entry().await?` 使用 `?` 操作符，单个目录权限不足会导致整个扫描中止。

**建议**: 替换为 `match`，遇到单条读取失败时记录日志并继续。

### M9. 缓存文件名存在路径穿越风险

**位置**: `rust/src/services/cache_service.rs:39, 47, 74, 80`

缓存键 `format!("{}_{}", file_id, size)` 直接作为文件名，若含路径分隔符可穿越目录。

**建议**: 对 `file_id` 和 `size` 进行 sanitize 或使用 hash 作为文件名。

### M10. 内存缓存与磁盘缓存不一致

**位置**: `rust/src/services/cache_service.rs:73-83`

先写内存再写磁盘。磁盘写入失败时内存已有数据，重启后丢失。反向情况：从磁盘读取成功后不验证完整性即提升到内存缓存。

### M11. 磁盘缓存无淘汰/清理机制

**位置**: `rust/src/services/cache_service.rs`

内存缓存有 TTL 和 max_capacity，但磁盘缓存无任何清理机制。长时间运行后无限增长。文件删除后对应缓存永不清理。

### M12. target_size=0 时可能产生巨大内存分配

**位置**: `rust/src/processors/heif_processor.rs:121`; `rust/src/processors/image_processor.rs:212`

当 `target_size` 为 0 时使用原始全尺寸图像。1 亿像素 HEIC（约 400MB RGBA）可能导致 OOM。

**建议**: 即使 `target_size == 0` 也设最大尺寸上限（如 4096px）。

### M13. `clean_exif_string` UTF-8 字节索引可能 panic

**位置**: `rust/src/processors/image_processor.rs:357-364`

`s[1..s.len()-1]` 使用字节索引切片。若字符串以多字节 UTF-8 字符（如中文引号 `\u{201C}`）开头/结尾，切片可能在字符中间，导致 panic。

**建议**: 改用字符级操作：`s.chars().collect()` 后按字符索引切片。

### M14. 极端竖图可能生成 0 宽度缩略图

**位置**: `rust/src/processors/image_processor.rs:219-221`

`target_width = (target_size as f64 * ratio) as u32` 对极端宽高比（如 10000x10）截断为 0。

**建议**: 添加 `target_width.max(1)` 下限。

### M15. 视频流识别方式不可靠

**位置**: `rust/src/processors/video_processor.rs:172-174`

使用 `stream.frames() > 0` 判断是否为视频流，音频流也可能满足此条件，无索引视频流可能帧数为 0。

**建议**: 使用 `stream.codecpar().codec_type() == Type::Video` 判断。

### M16. 视频 seek 使用绝对微秒未考虑 time_base

**位置**: `rust/src/processors/video_processor.rs:284-288`

`ictx.seek()` 的时间戳单位取决于容器格式时间基，`1_000_000` 对某些格式可能不对应 1 秒。

**风险**: 特定格式视频缩略图可能取到视频开头的黑帧。

### M17. `pool.scope` 阻塞 tokio 运行时线程

**位置**: `rust/src/processors/heif_processor.rs:80-93`

`pool.scope()` 是同步阻塞的，在 async 上下文中调用会阻塞 tokio 运行时线程，可能导致死锁。

**建议**: `pool.scope` 路径也应通过 `spawn_blocking` 包装。

### M18. CORS 允许所有来源

**位置**: `rust/src/app.rs:117-120`

`allow_origin(Any)` + `allow_headers(Any)` 对所有来源开放跨域。暴露在公网时有 CSRF 风险。

### M19. 无 graceful shutdown

**位置**: `rust/src/main.rs`, `rust/src/app.rs`

未捕获 Ctrl+C 信号，未使用 `with_graceful_shutdown`。进程被 kill 时扫描中断、WebSocket 断开、数据库可能未 flush WAL。

### M20. `RwLock::unwrap()` 在 poisoned lock 时级联 panic

**位置**: `rust/src/websocket/scan_state.rs:85, 237, 241`

若持有写锁的线程 panic，后续所有 `unwrap()` 也会 panic。

**建议**: 使用 `.unwrap_or_else(|e| e.into_inner())`。

### M21. WebSocket forward_task 不会被正确取消

**位置**: `rust/src/websocket/handler.rs:68-71`

`select!` 中一个 task 完成后另一 task 成为孤儿继续运行。应使用 `abort()` 取消。

### M22. `get_current_message` 回退方法永远返回 idle

**位置**: `rust/src/websocket/broadcast.rs:81-93`

新 `subscribe` 的 receiver 无法获取历史消息，`try_recv()` 对刚创建的 receiver 始终返回 `Err`。

### M23. Range 头解析不健壮

**位置**: `rust/src/api/files.rs:305-311`

后缀范围 `bytes=-500`（最后 500 字节）和开放式 `bytes=0-` 处理不正确。未校验 `start` 是否超出 `file_size`。

### M24. rescan API 始终返回 success

**位置**: `rust/src/api/system.rs:47-60`

即使扫描被拒绝（已在扫描中），API 仍返回 `success: true`。

**建议**: 先检查 `is_scanning`，若已在扫描中返回 409 Conflict。

### M25. sortBy/sortOrder/filterType ref 与 store 不同步

**位置**: `frontend/src/views/HomeView.vue:106-110`

三个 ref 在组件创建时从 store 拷贝初始值，但 store 被其他途径修改时不会同步更新。

**建议**: 使用 `storeToRefs(galleryStore)` 或 `computed`。

### M26. 扫描完成弹窗立即关闭

**位置**: `frontend/src/views/HomeView.vue:228-254`

`case 'completed'` 时弹窗立即关闭，用户来不及看到完成状态和最终统计。

**建议**: 延迟关闭或显示完成状态后再关闭。

### M27. DateNavigator 硬编码 size: 1000

**位置**: `frontend/src/components/DateNavigator.vue:84-88`

单次请求最多 1000 条，超过时数据静默丢失，无分页或"加载更多"。

### M28. useScreenSize 全局监听器多组件竞态

**位置**: `frontend/src/composables/useScreenSize.ts:24-31`

多个组件同时使用时，先卸载的组件会移除全局 resize 监听器，影响仍在使用的组件。

**建议**: 使用引用计数模式。

### M29. IntersectionObserver 初始哨兵元素为空

**位置**: `frontend/src/components/Gallery.vue:176-203`

`onMounted` 时 `columnSentinels` Map 为空，哨兵元素未被观察。依赖后续 `reobserveSentinels` 补救，但在某些时序下无限滚动可能不触发。

---

## 四、低风险问题（代码质量 / 改进建议）

### L1. ExifTag 枚举约 130 行死代码

**位置**: `rust/src/processors/image_processor.rs:9-134`

定义了完整的 `ExifTag` 枚举但从未使用，`extract_exif()` 直接使用 `exif::Tag`。

### L2. `u64` -> `i64` 文件大小转换理论上可溢出

**位置**: `rust/src/processors/file_metadata.rs:13`

### L3. 亚秒精度丢失

**位置**: `rust/src/processors/file_metadata.rs:32`

`system_time_to_naive_datetime` 中 `subsec_nanos()` 被忽略，硬编码为 0。

### L4. quality 参数缺少范围校验

**位置**: `rust/src/processors/heif_processor.rs:178`; `rust/src/processors/image_processor.rs:233`

`(quality * 100.0) as u8` 未校验 quality 范围。

**建议**: 添加 `quality.clamp(0.0, 1.0)`。

### L5. 缓存写入错误被静默忽略

**位置**: `rust/src/services/file_service.rs:80, 92`

`let _ = self.cache.put_thumbnail_bytes(...)` 静默忽略所有错误。

**建议**: 使用 `if let Err(e) = ... { warn!(...) }`。

### L6. 全尺寸文件不必要的内存拷贝

**位置**: `rust/src/services/file_service.rs:79`

`Bytes::from(data.clone())` 对大文件（几 MB 到几十 MB）做不必要的完整拷贝。

### L7. fallback 缩略图返回原始大文件

**位置**: `rust/src/services/file_service.rs:124-150`

缩略图生成失败时返回原始文件内容（可能 5-10MB），浪费带宽。

### L8. skip_list 过滤 O(N*M) 效率低

**位置**: `rust/src/services/scan_service.rs:147`

**建议**: 将 `skip_list` 转为 `HashSet`，查找降到 O(1)。

### L9. Scheduler 完全未实现

**位置**: `rust/src/services/scheduler.rs`

`start()` 和 `stop()` 都是 no-op。

### L10. TranscodingPool `expect` 可能 panic

**位置**: `rust/src/services/transcoding_pool.rs:28`

`num_threads=0` 时可能 panic。

### L11. 未知文件类型静默归类为 Image

**位置**: `rust/src/db/models.rs:43-48`

### L12. `now` 时间戳在大批次循环中不够精确

**位置**: `rust/src/db/repository.rs:414`

### L13. 多处 `.unwrap()` 可能在数据污染时 panic

**位置**: `rust/src/api/files.rs:182, 205, 343`

### L14. `file_path` 来自数据库无路径前缀校验

**位置**: `rust/src/api/files.rs:270`

### L15. `get_status` 无认证暴露内部状态

**位置**: `rust/src/api/system.rs:95`

### L16. `broadcast_interval` 运行时修改不生效

**位置**: `rust/src/websocket/scan_state.rs:81`

worker 循环开始时读取一次后不再更新。

### L17. 键盘事件在 setup 阶段注册

**位置**: `frontend/src/components/PhotoViewer.vue:481`

应在 `onMounted` 中注册。`defineExpose` 中的 `cleanup` 与 `onUnmounted` 形成重复清理。

### L18. formatDuration(0) 返回空字符串

**位置**: `frontend/src/utils/format.ts:10-15`

`!seconds` 对 0 也返回 true。应改为 `seconds == null || seconds < 0`。

### L19. formatDate 对无效日期返回 "Invalid Date"

**位置**: `frontend/src/utils/format.ts:35-56`

应先校验 `isNaN(date.getTime())`。

### L20. formatExposureTime 格式不自然

**位置**: `frontend/src/utils/format.ts:63-72`

`denominator.toFixed(3)` 将 "1/125" 格式化为 "1/125.000s"，整数分母应保持整数显示。

### L21. 下载大文件 30 秒超时不够

**位置**: `frontend/src/services/api.ts:8`

**建议**: 为 `getOriginalFile` 单独设置 `timeout: 0`。

### L22. WebSocket connect() 无防并发保护

**位置**: `frontend/src/services/websocket.ts:36`

未防止两个 `connect()` 同时 pending。

### L23. watch deep: true 对基本类型数组无意义

**位置**: `frontend/src/components/DateNavigator.vue:128`

---

## 五、测试代码问题

### T1. `test_scan_idempotent` 在空目录上测试幂等性——假阳性

**位置**: `rust/tests/services/scan_service_test.rs:92-121`

对空目录扫描两次结果都是 0，无论扫描逻辑是否幂等。应使用含文件的目录测试。

### T2. API 测试仅覆盖空状态

**位置**: `rust/tests/api/files_api_test.rs`, `directories_api_test.rs`

所有测试使用空数据库，仅检查 200 状态码，不验证业务逻辑。

### T3. processor 测试不验证处理器类型

**位置**: `rust/tests/processors/processor_test.rs:18-44`

仅检查 `is_some()`，不验证返回的确实是正确的处理器实现。

### T4. 扫描测试依赖 sleep 而非确定性等待

**位置**: `rust/tests/services/scan_service_test.rs`

使用 `sleep(500ms)` 等待扫描完成，CI 环境下可能不够。

**建议**: 使用 `wait_for_condition` 轮询状态。

### T5. 测试依赖相对路径

**位置**: `rust/tests/db/repository_test.rs:26`; `scan_service_test.rs:38`

`"./src/db/migrations"` 依赖运行目录。

**建议**: 使用 `env!("CARGO_MANIFEST_DIR")` 构建绝对路径。

### T6. src/fixtures 与 tests/fixtures 代码重复

**位置**: `rust/src/fixtures/mod.rs` vs `rust/tests/fixtures/mod.rs`

两份完全相同的 fixture 代码，修改其一忘另一会导致不同步。

### T7. src/helpers 与 tests/helpers 代码重复

**位置**: `rust/src/helpers/mod.rs` vs `rust/tests/helpers/mod.rs`

同 T6。

### T8. WebSocket 测试仅验证握手

**位置**: `rust/tests/api/websocket_test.rs:26-55`

从未发送/接收消息，不验证连接关闭行为。

### T9. 环境变量测试并行竞态

**位置**: `rust/src/config.rs:276-370`

`std::env::set_var` / `remove_var` 在多线程中不安全。`clear_env_vars()` 遗漏了多个变量。

---

## 汇总

| 严重程度 | 数量 | 已修复 | 关键项 |
|---------|------|--------|--------|
| **严重** | 7 | **7** ✅ | 路径遍历、to_rgb8 alpha合成、未对齐 UB、并发竞态、状态卡死、ObjectURL 泄漏、级联加载 |
| **高风险** | 11 | **8** 🔧 | EXIF Orientation (H1/H2)、alpha 合成 (H3)、INSERT OR REPLACE (H4)、find_neighbors 遗漏 (H5)、分页无校验 (H6)、消息丢弃 (H7)、WebSocket 断开 (H8)、嵌套 Runtime (H10) |
| **中风险** | 29 | 0 | 配置解析、WAL 缺失、符号链接、权限容错、缓存问题、OOM 风险、UTF-8 panic、视频流识别、CORS、shutdown、Range 解析、前端状态同步 |
| **低风险** | 23 | 0 | 死代码、精度丢失、参数校验、错误静默忽略、性能优化、格式化问题 |
| **测试** | 9 | 0 | 假阳性测试、空状态测试、代码重复、sleep 竞态、相对路径 |
| **合计** | **79** | **15** | |

### 修复优先级建议

**已完成**（严重 + 高风险问题，2026-06-13 ~ 2026-06-14）:
1. ✅ S1 静态文件路径遍历 (b595ab3)
2. ✅ S2 `to_rgb8` alpha 合成 (d551c67)
3. ✅ S3 未对齐内存读取 UB (d551c67)
4. ✅ S4/S5 扫描并发竞态 + 状态卡死 (d551c67)
5. ✅ S6 ObjectURL 内存泄漏 (d551c67)
6. ✅ S7 级联加载所有页面 (b595ab3)
7. ✅ H1/H2 EXIF Orientation 未处理 (c622d8c)
8. ✅ H3 HEIF RGBA→JPEG 缺少 alpha 合成 (03f18af)
9. ✅ H5 find_neighbors 遗漏仅 modify_time 记录 (03f18af)
10. ✅ H6 分页参数无范围校验 (03f18af)
11. ✅ H8 WebSocket 未在组件销毁时断开 (03f18af)
12. ✅ H10 首次扫描使用嵌套 Tokio Runtime (03f18af)
13. ✅ H7 try_send 丢弃关键状态消息 (6d4dd55)
14. ✅ H4 INSERT OR REPLACE -> ON CONFLICT DO UPDATE (b9358f6)

**待修复**（高风险剩余）:
14. H4 `INSERT OR REPLACE` -> `ON CONFLICT DO UPDATE`
15. H9 WebSocket 重连无退避策略
16. H11 setTimeout 未清理导致扫描状态竞态

**计划修复**（防御加固）:
17. M4 WAL + busy_timeout 配置
18. M7 符号链接循环保护
19. M8 权限错误容错
20. M1 配置解析错误提示
