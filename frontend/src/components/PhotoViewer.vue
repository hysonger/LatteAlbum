<template>
  <div class="photo-viewer" @click.self="close">
    <div class="viewer-content">
      <button class="nav-btn prev" @click="prev" :disabled="!hasPrev">‹</button>
      
      <div class="media-container" :style="containerStyle">
        <!-- 图片 -->
        <template v-if="isImage">
          <!-- 图片加载占位符 -->
          <div v-if="showImagePlaceholder" class="image-placeholder">
            <div class="spinner"></div>
            <span v-if="currentFile" class="placeholder-filename">{{ currentFile.fileName }}</span>
          </div>
          <img
            v-show="isImageLoaded"
            :src="currentImageUrl ?? undefined"
            :alt="currentFile?.fileName"
            @load="handleImageLoad"
            @error="handleError"
          />
        </template>
        <!-- 视频 -->
        <div v-else-if="isVideo" class="video-placeholder">
          <div class="video-wrapper">
            <video
              ref="videoRef"
              :src="currentVideoUrl ?? undefined"
              controls
              :poster="thumbnailUrl ?? undefined"
              @loadedmetadata="onVideoMetadataLoaded"
              @error="onVideoError"
              @play="onVideoPlay"
              @pause="onVideoPause"
              @timeupdate="onVideoTimeUpdate"
            />
            <div v-if="videoError" class="video-error">
              <div class="error-content">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10"></circle>
                  <line x1="12" y1="8" x2="12" y2="12"></line>
                  <line x1="12" y1="16" x2="12.01" y2="16"></line>
                </svg>
                <p>视频播放失败</p>
                <p class="error-message">{{ videoError }}</p>
                <button class="download-btn" @click="downloadVideo">下载视频</button>
              </div>
            </div>
          </div>
        </div>
        <!-- 加载中 -->
        <div v-else class="loading">加载中...</div>
      </div>
      
      <button class="nav-btn next" @click="next" :disabled="!hasNext">›</button>
      
      <div class="file-info file-info--bottom" v-if="currentFile">
        <div class="info-header">
          <div class="info-basic">
            <h3>{{ currentFile.fileName }}</h3>
            <p class="time-display">
              <span v-if="currentFile.exifTimestamp">{{ formatDate(currentFile.exifTimestamp, currentFile.exifTimezoneOffset) }}</span>
              <span v-else-if="currentFile.createTime">{{ formatDate(currentFile.createTime) }}</span>
            </p>
          </div>
          <div class="info-actions">
            <button class="download-btn" @click="downloadOriginal" :title="'下载原图'">
              <i class="fas fa-download"></i>
            </button>
            <button class="info-toggle-btn" @click="toggleInfo" :title="showDetailInfo ? '收起信息' : '显示详细信息'">
              <svg v-if="!showDetailInfo" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="12" y1="16" x2="12" y2="12"></line>
                <line x1="12" y1="8" x2="12.01" y2="8"></line>
              </svg>
              <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </button>
          </div>
        </div>
        <div class="meta-info" v-show="showDetailInfo">
          <!-- 时间信息 -->
          <div class="meta-group">
            <div class="meta-item" v-if="currentFile.exifTimestamp">
              <span class="meta-label">拍摄时间</span>
              <span class="meta-value">{{ formatDate(currentFile.exifTimestamp, currentFile.exifTimezoneOffset) }}</span>
            </div>
            <div class="meta-item" v-if="currentFile.createTime">
              <span class="meta-label">创建时间</span>
              <span class="meta-value">{{ formatDate(currentFile.createTime) }}</span>
            </div>
            <div class="meta-item" v-if="currentFile.modifyTime">
              <span class="meta-label">修改时间</span>
              <span class="meta-value">{{ formatDate(currentFile.modifyTime) }}</span>
            </div>
          </div>

          <!-- 拍摄设备 -->
          <div class="meta-group">
            <div class="meta-item" v-if="currentFile.cameraMake">
              <span class="meta-label">相机厂商</span>
              <span class="meta-value">{{ currentFile.cameraMake }}</span>
            </div>
            <div class="meta-item" v-if="currentFile.cameraModel">
              <span class="meta-label">相机型号</span>
              <span class="meta-value">{{ currentFile.cameraModel }}</span>
            </div>
            <div class="meta-item" v-if="currentFile.lensModel">
              <span class="meta-label">镜头型号</span>
              <span class="meta-value">{{ currentFile.lensModel }}</span>
            </div>
          </div>

          <!-- 拍摄参数 -->
          <div class="meta-group">
            <div class="meta-item" v-if="currentFile.exposureTime">
              <span class="meta-label">快门速度</span>
              <span class="meta-value">{{ formatExposureTime(currentFile.exposureTime) }}</span>
            </div>
            <div class="meta-item" v-if="currentFile.aperture">
              <span class="meta-label">光圈</span>
              <span class="meta-value">f/{{ currentFile.aperture }}</span>
            </div>
            <div class="meta-item" v-if="currentFile.iso">
              <span class="meta-label">ISO</span>
              <span class="meta-value">{{ currentFile.iso }}</span>
            </div>
            <div class="meta-item" v-if="currentFile.focalLength">
              <span class="meta-label">焦距</span>
              <span class="meta-value">{{ currentFile.focalLength }}</span>
            </div>
          </div>

          <!-- 文件信息 -->
          <div class="meta-group">
            <div class="meta-item" v-if="currentFile.width && currentFile.height">
              <span class="meta-label">尺寸</span>
              <span class="meta-value">{{ currentFile.width }} × {{ currentFile.height }}</span>
            </div>
            <div class="meta-item" v-if="currentFile.fileSize">
              <span class="meta-label">文件大小</span>
              <span class="meta-value">{{ formatFileSize(currentFile.fileSize) }}</span>
            </div>
            <div class="meta-item" v-if="currentFile.duration">
              <span class="meta-label">时长</span>
              <span class="meta-value">{{ formatDuration(currentFile.duration) }}</span>
            </div>
            <div class="meta-item" v-if="currentFile.videoCodec">
              <span class="meta-label">视频编码</span>
              <span class="meta-value">{{ currentFile.videoCodec }}</span>
            </div>
          </div>
        </div>
      </div>
      
      <button class="close-btn" @click="close">×</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { fileApi } from '@/services/api'
import { useScreenSize } from '@/composables/useScreenSize'
import { formatDuration, formatFileSize, formatDate, formatExposureTime, downloadFile } from '@/utils/format'
import type { MediaFile } from '@/types'

const { isMobile: isSmallScreen } = useScreenSize()

const props = defineProps<{
  file: MediaFile
  neighbors: MediaFile[]
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'change', file: MediaFile): void
}>()

const currentFile = ref<MediaFile | null>(props.file)
const currentIndex = ref(props.neighbors.findIndex(f => f.id === props.file.id))
const currentImageUrl = ref<string | undefined>(undefined)
const currentVideoUrl = ref<string | undefined>(undefined)
const thumbnailUrl = ref<string | undefined>(undefined)
const videoRef = ref<HTMLVideoElement | null>(null)
const showDetailInfo = ref(false)
const isLoading = ref(false)
const isConverting = ref<boolean>(false)
const videoError = ref<string | null>(null)
const isImageLoaded = ref(false)

// 用于触发窗口尺寸变化时的重新计算
const windowResizeKey = ref(0)

// 用于防止竞态条件：跟踪当前加载的世代
let loadGeneration = 0

// 计算属性
const isImage = computed(() => currentFile.value?.fileType === 'image')
const isVideo = computed(() => currentFile.value?.fileType === 'video')

const hasPrev = computed(() => currentIndex.value > 0)
const hasNext = computed(() => currentIndex.value < props.neighbors.length - 1)

// 图片占位符显示条件：加载中且图片尚未加载完成
const showImagePlaceholder = computed(() => isImage.value && isLoading.value && !isImageLoaded.value)

// 图片容器样式：根据原始图片尺寸计算固定的宽高比，避免从 large 切换到 full 时视觉跳变
const containerStyle = computed(() => {
  // 依赖 windowResizeKey，当窗口大小变化时触发重新计算
  void windowResizeKey.value

  if (!currentFile.value?.width || !currentFile.value?.height) {
    return {}
  }

  const maxWidth = window.innerWidth * 0.9
  const maxHeight = window.innerHeight * 0.8

  const imgAspectRatio = currentFile.value.width / currentFile.value.height
  const containerAspectRatio = maxWidth / maxHeight

  let width: number, height: number
  if (imgAspectRatio > containerAspectRatio) {
    // 图片更宽，以宽度为基准
    width = maxWidth
    height = maxWidth / imgAspectRatio
  } else {
    // 图片更高或相等，以高度为基准
    height = maxHeight
    width = maxHeight * imgAspectRatio
  }

  return {
    width: `${width}px`,
    height: `${height}px`
  }
})


// 导航操作
const prev = () => {
  if (hasPrev.value && currentIndex.value > 0) {
    currentIndex.value--
    currentFile.value = props.neighbors[currentIndex.value]
  }
}

const next = () => {
  if (hasNext.value && currentIndex.value < props.neighbors.length - 1) {
    currentIndex.value++
    currentFile.value = props.neighbors[currentIndex.value]
  }
}

// 关闭查看器
const close = () => {
  emit('close')
}

const toggleInfo = () => {
  showDetailInfo.value = !showDetailInfo.value
}

const downloadMedia = async () => {
  if (!currentFile.value) return

  try {
    const response = await fileApi.getOriginalFile(currentFile.value.id)
    downloadFile(response.data, currentFile.value.fileName)
  } catch (error) {
    console.error('下载文件失败:', error)
    alert('下载文件失败，请稍后重试')
  }
}

// 加载媒体文件
const loadMedia = async () => {
  if (!currentFile.value) return

  // 重置图片加载状态
  isImageLoaded.value = false

  // 增加世代计数，用于防止竞态条件
  const currentGeneration = ++loadGeneration

  try {
    if (isImage.value) {
      isLoading.value = true
      const isHeif = Boolean(currentFile.value.fileName) &&
        (currentFile.value.fileName.toLowerCase().endsWith('.heic') ||
         currentFile.value.fileName.toLowerCase().endsWith('.heif'))
      isConverting.value = isHeif

      // 小屏设备只请求 large 尺寸，大屏设备并行请求 full 和 large
      if (isSmallScreen.value) {
        // 小屏设备：只使用 large 尺寸，避免加载过大的图片
        try {
          const result = await fileApi.getThumbnail(currentFile.value.id, 'large')
          if (currentGeneration !== loadGeneration) return
          currentImageUrl.value = URL.createObjectURL(new Blob([result.data]))
        } catch (e) {
          console.error('加载媒体文件失败:', e)
          currentImageUrl.value = undefined
        }
      } else {
        // 大屏设备：并行请求 full 和 large，优先显示先返回的
        // full: 全尺寸转码图（JPEG格式，节省流量）
        // large: 大尺寸缩略图作为备选/占位图
        const fullRequest = fileApi.getThumbnail(currentFile.value.id, 'full')
        const largeRequest = fileApi.getThumbnail(currentFile.value.id, 'large')

        try {
          // 使用 Promise.race 竞争，先完成的先处理
          const winner: { result: { data: BlobPart }; isFull: boolean } = await Promise.race([
            fullRequest.then(result => ({ result, isFull: true })),
            largeRequest.then(result => ({ result, isFull: false }))
          ])

          // 检查世代是否匹配（可能被翻页中断）
          if (currentGeneration !== loadGeneration) return

          if (winner.isFull) {
            // full 先返回，直接显示
            currentImageUrl.value = URL.createObjectURL(new Blob([winner.result.data]))
          } else {
            // large 先返回，直接作为占位图显示
            currentImageUrl.value = URL.createObjectURL(new Blob([winner.result.data]))

            // 继续等待 full，完成后替换
            const fullResult = await fullRequest

            // 再次检查世代是否匹配
            if (currentGeneration !== loadGeneration) return

            currentImageUrl.value = URL.createObjectURL(new Blob([fullResult.data]))
          }
        } catch {
          // 如果 full 失败，尝试使用 large 作为备选
          if (currentImageUrl.value === undefined && currentGeneration === loadGeneration) {
            try {
              const largeResult = await largeRequest
              if (currentGeneration !== loadGeneration) return
              currentImageUrl.value = URL.createObjectURL(new Blob([largeResult.data]))
            } catch (e) {
              console.error('加载媒体文件失败:', e)
              currentImageUrl.value = undefined
            }
          }
        }
      }
    } else if (isVideo.value) {
      // 视频使用流式播放，直接使用URL，无需下载到内存
      // 后端支持 Range 请求，浏览器可以自动进行 seek 和流式播放
      thumbnailUrl.value = fileApi.getThumbnailUrl(currentFile.value.id, 'large')
      currentVideoUrl.value = fileApi.getOriginalFileUrl(currentFile.value.id)
    }
  } finally {
    isLoading.value = false
    isConverting.value = false
  }
}

const handleImageLoad = () => {
  isImageLoaded.value = true
}

const handleError = () => {
  console.error('媒体文件加载失败')
}

const onVideoMetadataLoaded = () => {
  videoError.value = null
  // 视频元数据加载后聚焦到 video 元素，使空格键可以控制播放/暂停
  videoRef.value?.focus()
}

const onVideoError = (e: Event) => {
  const videoElement = e.target as HTMLVideoElement
  const errorCode = videoElement.error?.code
  let errorMessage = '未知错误'
  
  switch (errorCode) {
    case MediaError.MEDIA_ERR_ABORTED:
      errorMessage = '视频加载被中断'
      break
    case MediaError.MEDIA_ERR_NETWORK:
      errorMessage = '网络错误，无法加载视频'
      break
    case MediaError.MEDIA_ERR_DECODE:
      errorMessage = '视频解码失败，可能是不支持的格式'
      break
    case MediaError.MEDIA_ERR_SRC_NOT_SUPPORTED:
      errorMessage = '不支持的视频格式或编码'
      break
  }
  
  videoError.value = errorMessage
  console.error('视频播放错误:', errorMessage, e)
}

const onVideoPlay = () => {
  // Video playback started - can be used for analytics or UI state
}

const onVideoPause = () => {
  // Video playback paused - can be used for analytics or UI state
}

const onVideoTimeUpdate = () => {
  // 可以在这里更新播放进度
}

// 监听 neighbors 变化，更新当前索引
watch(() => props.neighbors, (newNeighbors) => {
  if (currentFile.value) {
    const newIndex = newNeighbors.findIndex(f => f.id === currentFile.value?.id)
    if (newIndex !== -1) {
      currentIndex.value = newIndex
    }
  }
}, { deep: true })

// 监听 file prop 变化
watch(() => props.file, (newFile) => {
  currentFile.value = newFile
  currentIndex.value = props.neighbors.findIndex(f => f.id === newFile.id)
})

// 监听当前文件变化
watch(currentFile, () => {
  currentImageUrl.value = undefined
  currentVideoUrl.value = undefined
  loadMedia()
  
  if (currentFile.value) {
    emit('change', currentFile.value)
  }
})

// 键盘事件监听
const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    close()
  } else if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
    // 视频播放时不拦截方向键，让 video 控件自行处理快进快退
    if (isVideo.value && videoRef.value && !videoRef.value.paused) {
      return
    }
    if (e.key === 'ArrowLeft') {
      prev()
    } else {
      next()
    }
  }
}

// 窗口大小变化时触发重新计算
const handleResize = () => {
  windowResizeKey.value++
}

// 初始化
loadMedia()

// 添加窗口大小变化监听
onMounted(() => {
  window.addEventListener('resize', handleResize)
})

// 清理窗口大小变化监听和键盘事件监听
onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
  document.removeEventListener('keydown', handleKeydown)
  // 释放 ObjectURL 防止内存泄漏
  if (currentImageUrl.value) {
    URL.revokeObjectURL(currentImageUrl.value)
    currentImageUrl.value = undefined
  }
})

// 添加键盘事件监听
document.addEventListener('keydown', handleKeydown)

// 清理事件监听
defineExpose({
  cleanup: () => {
    document.removeEventListener('keydown', handleKeydown)
  }
})
</script>

<style scoped>
.photo-viewer {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.9);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.viewer-content {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.nav-btn {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  background: rgba(255, 255, 255, 0.2);
  border: none;
  color: white;
  font-size: 2rem;
  width: 50px;
  height: 50px;
  border-radius: 50%;
  cursor: pointer;
  z-index: 1001;
  backdrop-filter: blur(5px);
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
}

.nav-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.3);
}

.nav-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.nav-btn.prev {
  left: 20px;
}

.nav-btn.next {
  right: 20px;
}

.media-container {
  display: flex;
  align-items: center;
  justify-content: center;
}

/* 图片加载占位符 */
.image-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  width: 100%;
  height: 100%;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid rgba(255, 255, 255, 0.2);
  border-top-color: rgba(255, 255, 255, 0.8);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.placeholder-filename {
  color: rgba(255, 255, 255, 0.7);
  font-size: 0.9rem;
  max-width: 80%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 图片容器固定尺寸，避免从 large 切换到 full 时视觉跳变 */
.media-container img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

/* 视频时减少高度，留出空间给底部文件信息栏 */
.media-container video {
  max-height: 75vh;
}

.video-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
}

.video-wrapper {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.video-placeholder video {
  max-width: 100%;
  max-height: 75vh;
}

.video-error {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  backdrop-filter: blur(10px);
}

.error-content {
  text-align: center;
  color: white;
  padding: 24px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  backdrop-filter: blur(10px);
  max-width: 400px;
}

.error-content svg {
  width: 64px;
  height: 64px;
  margin: 0 auto 16px;
  color: #ff6b6b;
}

.error-content p {
  margin: 8px 0;
  font-size: 1rem;
}

.error-message {
  font-size: 0.875rem;
  color: rgba(255, 255, 255, 0.7);
  margin-bottom: 16px;
}

.error-content .download-btn {
  display: inline-block;
  min-width: 100px;
  text-align: center;
  width: auto;
  height: auto;
  padding: 8px 12px;
  border-radius: 6px;
  margin-top: 8px;
}

.error-content .download-btn:hover {
  background: #45a049;
}

.file-info {
  background: rgba(0, 0, 0, 0.7);
  color: white;
  padding: 12px 16px;
  border-radius: 8px;
  max-width: 80%;
  backdrop-filter: blur(5px);
  transition: all 0.3s ease;
}

.file-info--bottom {
  position: absolute;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
}

.info-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.info-basic {
  flex: 1;
  min-width: 0;
}

.info-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-info h3 {
  margin: 0 0 4px 0;
  font-size: 1rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.time-display {
  margin: 0;
  font-size: 0.85rem;
  color: rgba(255, 255, 255, 0.8);
}

.info-toggle-btn {
  flex-shrink: 0;
  background: rgba(255, 255, 255, 0.1);
  border: none;
  color: white;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s ease;
}

.info-toggle-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.info-toggle-btn svg {
  width: 18px;
  height: 18px;
  display: block;
  margin: 0;
}

.download-btn {
  flex-shrink: 0;
  background: rgba(255, 255, 255, 0.1);
  border: none;
  color: white;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s ease;
}

.download-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.download-btn svg {
  width: 18px;
  height: 18px;
  display: block;
  margin: 0;
}

.download-btn i {
  font-size: 18px;
}

.meta-info {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.2);
  font-size: 0.85rem;
}

.meta-group {
  margin-bottom: 12px;
}

.meta-group:last-child {
  margin-bottom: 0;
}

.meta-group .meta-item {
  display: inline-block;
  vertical-align: top;
  min-width: 140px;
  margin-right: 8px;
  margin-bottom: 8px;
}

.meta-label {
  display: block;
  font-size: 0.7rem;
  color: rgba(255, 255, 255, 0.5);
}

.meta-value {
  display: block;
  color: rgba(255, 255, 255, 0.9);
  word-break: break-word;
}

.close-btn {
  position: absolute;
  top: 20px;
  right: 20px;
  background: rgba(255, 255, 255, 0.2);
  border: none;
  color: white;
  font-size: 1.5rem;
  width: 40px;
  height: 40px;
  border-radius: 50%;
  cursor: pointer;
  backdrop-filter: blur(5px);
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.3);
}

.loading {
  color: white;
  font-size: 1.2rem;
}
</style>