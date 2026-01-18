<template>
  <div class="media-card" ref="cardRef" @click="$emit('click', item)">
    <div class="thumbnail-container">
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
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { fileApi } from '@/services/api'
import type { MediaFile } from '@/types'

const props = defineProps<{
  item: MediaFile
  thumbnailSize?: 'small' | 'medium' | 'large' | 'full'
}>()

defineEmits<{
  (e: 'click', item: MediaFile): void
}>()

const cardRef = ref<HTMLElement | null>(null)
const thumbnailUrl = ref<string | null>(null)
const isLoading = ref(false)
const isLoaded = ref(false)
const observer = ref<IntersectionObserver | null>(null)

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
}

const formatDuration = (seconds: number) => {
  if (!seconds) return ''
  const minutes = Math.floor(seconds / 60)
  const remainingSeconds = Math.floor(seconds % 60)
  return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`
}

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