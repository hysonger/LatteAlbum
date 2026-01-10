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
      
      <div class="file-info" v-if="currentFile">
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
          <p v-if="currentFile.exifTimestamp">
            拍摄时间: {{ formatDate(currentFile.exifTimestamp, currentFile.exifTimezoneOffset) }}
          </p>
          <p v-if="currentFile.createTime">
            创建时间: {{ formatDate(currentFile.createTime) }}
          </p>
          <p v-if="currentFile.modifyTime">
            修改时间: {{ formatDate(currentFile.modifyTime) }}
          </p>
          <p v-if="cameraDisplay">
            相机型号: {{ cameraDisplay }}
          </p>
          <p v-if="currentFile.width && currentFile.height">
            尺寸: {{ currentFile.width }} × {{ currentFile.height }}
          </p>
          <p v-if="currentFile.duration">
            时长: {{ formatDuration(currentFile.duration) }}
          </p>
          <p v-if="currentFile.videoCodec">
            视频编码: {{ currentFile.videoCodec }}
          </p>
          <p v-if="currentFile.fileSize">
            文件大小: {{ formatFileSize(currentFile.fileSize) }}
          </p>
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

// 计算属性
const isImage = computed(() => currentFile.value?.fileType === 'image')
const isVideo = computed(() => currentFile.value?.fileType === 'video')

const hasPrev = computed(() => currentIndex.value > 0)
const hasNext = computed(() => currentIndex.value < props.neighbors.length - 1)

const cameraDisplay = computed(() => {
  const make = currentFile.value?.cameraMake
  const model = currentFile.value?.cameraModel
  
  if (!model) return make || ''
  if (!make) return model
  
  const modelLower = model.toLowerCase()
  const makeLower = make.toLowerCase()
  
  if (modelLower.startsWith(makeLower)) {
    return model
  }
  
  return `${make} ${model}`
})

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

  try {
    // 加载缩略图（large size用于预览）
    const thumbResponse = await fileApi.getThumbnail(currentFile.value.id, 'large')
    const thumbBlob = new Blob([thumbResponse.data])
    thumbnailUrl.value = URL.createObjectURL(thumbBlob)

    // 加载大图（使用异步接口，HEIF自动转换为JPEG）
    if (isImage.value) {
      isLoading.value = true
      const isHeif = Boolean(currentFile.value.fileName) && 
        (currentFile.value.fileName.toLowerCase().endsWith('.heic') || 
         currentFile.value.fileName.toLowerCase().endsWith('.heif'))
      
      isConverting.value = isHeif
      
      try {
        const response = await fileApi.getThumbnail(currentFile.value.id, 'full')
        const blob = new Blob([response.data])
        currentImageUrl.value = URL.createObjectURL(blob)
      } finally {
        isLoading.value = false
        isConverting.value = false
      }
    } else if (isVideo.value) {
      const response = await fileApi.getOriginalFile(currentFile.value.id)
      const blob = new Blob([response.data])
      currentVideoUrl.value = URL.createObjectURL(blob)
    }
  } catch (error) {
    console.error('加载媒体文件失败:', error)
    // 出错时设置为undefined而不是null
    currentImageUrl.value = undefined
    currentVideoUrl.value = undefined
    thumbnailUrl.value = undefined
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
  } else if (e.key === 'ArrowLeft') {
    prev()
  } else if (e.key === 'ArrowRight') {
    next()
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
  max-height: 80vh;
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
  position: absolute;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.7);
  color: white;
  padding: 12px 16px;
  border-radius: 8px;
  max-width: 80%;
  backdrop-filter: blur(5px);
  transition: all 0.3s ease;
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

.meta-info p {
  margin: 4px 0;
  color: rgba(255, 255, 255, 0.9);
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
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.3);
}

.loading {
  color: white;
  font-size: 1.2rem;
}
</style>