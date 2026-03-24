<template>
  <div class="media-card" ref="cardRef" @click="$emit('click', item)">
    <div
      class="thumbnail-container"
      :class="{ truncated: isTruncated }"
      :style="isTruncated ? { maxHeight: maxHeight } : {}"
    >
      <img
        v-if="thumbnailUrl"
        :src="thumbnailUrl"
        :alt="item.fileName"
        :class="['thumbnail', { loaded: isLoaded }]"
        @load="onImageLoad"
        @error="onImageError"
      />
      <div v-else class="placeholder">
        <span v-if="item.fileType === 'video'">▶</span>
        <span v-else>📷</span>
      </div>
      <div v-if="item.fileType === 'video' && item.duration" class="video-duration">
        {{ formatDuration(item.duration) }}
      </div>
      <!-- 截断指示器 -->
      <div v-if="isTruncated" class="truncate-indicator">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M7 16V4M7 4L3 8M7 4L11 8"/>
          <path d="M17 8V20M17 20L21 16M17 20L13 16"/>
        </svg>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from 'vue'
import { fileApi } from '@/services/api'
import { formatDuration } from '@/utils/format'
import type { MediaFile } from '@/types'

const props = defineProps<{
  item: MediaFile
  thumbnailSize?: 'small' | 'medium' | 'large' | 'full'
  columnWidth?: number
}>()

defineEmits<{
  (e: 'click', item: MediaFile): void
}>()

const cardRef = ref<HTMLElement | null>(null)
const thumbnailUrl = ref<string | null>(null)
const isLoading = ref(false)
const isLoaded = ref(false)
const observer = ref<IntersectionObserver | null>(null)

// 计算宽高比
const aspectRatio = computed(() => {
  if (!props.item.width || !props.item.height) return 1
  return props.item.width / props.item.height
})

// 是否应该截断（宽高比 < 0.5）
const isTruncated = computed(() => {
  return aspectRatio.value < 0.5
})

// 最大高度（用于截断显示）
const maxHeight = computed(() => {
  if (!props.columnWidth) return 'none'
  return `${props.columnWidth * 1.5}px`
})

// 计算缩略图URL
const loadThumbnail = async () => {
  if (isLoading.value) return

  isLoading.value = true
  try {
    const response = await fileApi.getThumbnail(props.item.id, props.thumbnailSize || 'small')
    const blob = new Blob([response.data])
    thumbnailUrl.value = URL.createObjectURL(blob)
  } catch (error) {
    console.error('加载缩略图失败:', error)
  } finally {
    isLoading.value = false
  }
}

const onImageLoad = () => {
  isLoaded.value = true
}

const onImageError = () => {
  // 显示错误占位符
  console.error('加载缩略图失败:', props.item.id)
}

const revokeThumbnailUrl = () => {
  if (thumbnailUrl.value) {
    URL.revokeObjectURL(thumbnailUrl.value)
    thumbnailUrl.value = null
  }
}

// 监听 item 变化时释放旧的 ObjectURL
watch(() => props.item.id, () => {
  revokeThumbnailUrl()
  isLoaded.value = false
})

onMounted(() => {
  // 使用 Intersection Observer 实现懒加载
  observer.value = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          loadThumbnail()
          // 加载后取消观察，避免重复触发
          observer.value?.unobserve(entry.target)
        }
      })
    },
    {
      rootMargin: '200px', // 提前 200px 开始加载
      threshold: 0
    }
  )

  if (cardRef.value) {
    observer.value.observe(cardRef.value)
  }
})

onUnmounted(() => {
  observer.value?.disconnect()
  revokeThumbnailUrl()
})
</script>

<style scoped>
.media-card {
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  background: white;
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
}

.media-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
}

.thumbnail-container {
    position: relative;
    width: 100%;
    overflow: hidden;
  }

  .thumbnail {
    width: 100%;
    height: auto;
    object-fit: contain;
    display: block;
    opacity: 0;
    transition: opacity 0.5s ease-in-out;
  }

  .thumbnail.loaded {
    opacity: 1;
  }

  .video-duration {
    position: absolute;
    bottom: 8px;
    right: 8px;
    background: rgba(0, 0, 0, 0.7);
    color: white;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 500;
    backdrop-filter: blur(4px);
  }

  /* 截断容器样式 */
  .thumbnail-container.truncated {
    max-height: var(--max-height, none);
  }

  .thumbnail-container.truncated .thumbnail {
    object-fit: cover;
    height: 100%;
  }

  /* 底部渐变效果 */
  .thumbnail-container.truncated::after {
    content: '';
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 30%;
    background: linear-gradient(
      to bottom,
      transparent,
      rgba(255, 255, 255, 0.9)
    );
    pointer-events: none;
  }

  /* 截断指示器图标 */
  .truncate-indicator {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 24px;
    height: 24px;
    background: rgba(0, 0, 0, 0.6);
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    cursor: pointer;
    transition: background 0.2s;
    z-index: 1;
  }

  .truncate-indicator:hover {
    background: rgba(0, 0, 0, 0.8);
  }

.placeholder {
  width: 100%;
  min-height: 150px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #f5f5f5;
  font-size: 2em;
}


</style>