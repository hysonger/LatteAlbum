<template>
  <div class="photo-viewer" @click.self="close">
    <div class="viewer-content">
      <button class="nav-btn prev" @click="prev" :disabled="!hasPrev">‹</button>
      
      <div class="media-container">
        <img 
          v-if="isImage" 
          :src="currentImageUrl ?? undefined" 
          :alt="currentFile?.fileName"
          @load="handleImageLoad"
          @error="handleError"
        />
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
import { ref, computed, watch } from 'vue'
import { fileApi } from '@/services/api'
import type { MediaFile } from '@/types'

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

// 用于防止竞态条件：跟踪当前加载的世代
let loadGeneration = 0

// 计算属性
const isImage = computed(() => currentFile.value?.fileType === 'image')
const isVideo = computed(() => currentFile.value?.fileType === 'video')

const hasPrev = computed(() => currentIndex.value > 0)
const hasNext = computed(() => currentIndex.value < props.neighbors.length - 1)

// 格式化日期
const formatDate = (dateString: string, timezoneOffset?: string) => {
  const date = new Date(dateString)
  
  if (!timezoneOffset) {
    // 未知时区，显示为UTC时间
    return `${date.toLocaleString('zh-CN')}`
  }
  
  // 解析时区偏移量（如"+08:00"）
  const offsetHours = parseInt(timezoneOffset.substring(1, 3))
  const offsetMinutes = parseInt(timezoneOffset.substring(4, 6))
  const offsetSign = timezoneOffset[0] === '+' ? 1 : -1
  const totalOffsetMinutes = offsetSign * (offsetHours * 60 + offsetMinutes)
  
  // 计算原始时区的本地时间
  const originalDate = new Date(date.getTime() + totalOffsetMinutes * 60000)
  
  // 检查是否与用户本地时区一致
  const userOffset = date.getTimezoneOffset()
  const isSameTimezone = userOffset === -totalOffsetMinutes
  
  if (isSameTimezone) {
    // 与用户时区一致，正常显示
    return originalDate.toLocaleString('zh-CN')
  } else {
    // 时区不同，显示原始时区时间并标注
    const timezoneLabel = `UTC${timezoneOffset}`
    return `${originalDate.toLocaleString('zh-CN')} (${timezoneLabel})`
  }
}

// 格式化文件大小
const formatFileSize = (bytes: number) => {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB'
}

const formatDuration = (seconds: number) => {
  if (!seconds) return ''
  const minutes = Math.floor(seconds / 60)
  const remainingSeconds = Math.floor(seconds % 60)
  return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`
}

// 格式化快门速度（如 "1/125" 显示为 "1/125s"）
const formatExposureTime = (exposureTime: string) => {
  if (!exposureTime) return ''
  // 如果是分数形式如 "1/125.5"，精确到小数点后3位
  if (exposureTime.startsWith('1/')) {
    const denominator = parseFloat(exposureTime.substring(2))
    if (!isNaN(denominator)) {
      return `1/${denominator.toFixed(3)}s`
    }
  }
  return `${exposureTime}s`
}

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

const downloadOriginal = async () => {
  if (!currentFile.value) return

  try {
    const response = await fileApi.getOriginalFile(currentFile.value.id)
    const blob = new Blob([response.data])
    const url = URL.createObjectURL(blob)
    
    const link = document.createElement('a')
    link.href = url
    link.download = currentFile.value.fileName
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    
    URL.revokeObjectURL(url)
  } catch (error) {
    console.error('下载文件失败:', error)
    alert('下载文件失败，请稍后重试')
  }
}

// 加载媒体文件
const loadMedia = async () => {
  if (!currentFile.value) return

  // 增加世代计数，用于防止竞态条件
  const currentGeneration = ++loadGeneration

  try {
    if (isImage.value) {
      isLoading.value = true
      const isHeif = Boolean(currentFile.value.fileName) &&
        (currentFile.value.fileName.toLowerCase().endsWith('.heic') ||
         currentFile.value.fileName.toLowerCase().endsWith('.heif'))
      isConverting.value = isHeif

      // 并行请求 full 和 large，优先显示先返回的
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
  console.log('视频开始播放')
}

const onVideoPause = () => {
  console.log('视频暂停')
}

const onVideoTimeUpdate = () => {
  // 可以在这里更新播放进度
}

const downloadVideo = async () => {
  if (!currentFile.value) return
  
  try {
    const response = await fileApi.getOriginalFile(currentFile.value.id)
    const blob = new Blob([response.data])
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = currentFile.value.fileName
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  } catch (error) {
    console.error('下载视频失败:', error)
  }
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

// 初始化
loadMedia()

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
  max-width: 90%;
  max-height: 90%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.media-container img,
.media-container video {
  max-width: 100%;
  max-height: 80vh;
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

.download-btn {
  background: #4CAF50;
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.9rem;
  font-weight: 500;
  transition: background 0.2s;
}

.download-btn:hover {
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