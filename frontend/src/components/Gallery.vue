<template>
  <div class="gallery-container" ref="container">
    <div
      v-for="column in columns"
      :key="column.id"
      class="gallery-column"
    >
      <MediaCard
        v-for="item in column.items"
        :key="item.id"
        :item="item"
        :thumbnail-size="thumbnailSize"
        @click="handleClick(item)"
      />
      <!-- 列底部哨兵，用于触发加载更多 -->
      <div
        v-if="displayHasMore"
        :ref="(el) => setColumnSentinel(el, column.id)"
        class="column-sentinel"
      ></div>
    </div>
    <div v-if="displayIsLoading" class="loading">
      <div class="spinner"></div>
    </div>
    <div v-else-if="displayIsEmpty" class="empty">
      暂无数据
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import MediaCard from './MediaCard.vue'
import type { MediaFile } from '@/types'

// 防抖函数
const debounce = <T extends (...args: any[]) => any>(
  func: T,
  wait: number
): ((...args: Parameters<T>) => void) => {
  let timeout: ReturnType<typeof setTimeout> | null = null
  return (...args: Parameters<T>) => {
    if (timeout) clearTimeout(timeout)
    timeout = setTimeout(() => func(...args), wait)
  }
}

interface Props {
  items: MediaFile[]
  isLoading?: boolean
  isEmpty?: boolean
  hasMore?: boolean
  enableScrollLoad?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  items: () => [],
  isLoading: false,
  isEmpty: false,
  hasMore: false,
  enableScrollLoad: true
})

const container = ref<HTMLElement>()
const columnCount = ref(4)

// 列哨兵引用集合
const columnSentinels = ref<Map<number, HTMLElement>>(new Map())
const sentinelObserver = ref<IntersectionObserver | null>(null)
let isLoadingMore = false // 防止重复触发加载更多

// 计算缩略图尺寸
const thumbnailSize = computed(() => {
  const width = window.innerWidth
  return width < 768 ? 'small' : 'medium'
})

// 使用传入的items作为唯一数据源
const displayItems = computed(() => props.items)
const displayIsLoading = computed(() => props.isLoading)
const displayIsEmpty = computed(() => props.isEmpty)
const displayHasMore = computed(() => props.hasMore)

// 响应式列数计算
const updateColumnCount = () => {
  const width = window.innerWidth
  if (width < 768) {
    columnCount.value = 2
  } else if (width < 1024) {
    columnCount.value = 3
  } else {
    columnCount.value = 4
  }
}

// 列宽度跟踪，用于自适应高度计算
const columnWidth = ref(200) // 初始默认值

// 计算实际列宽度
const updateColumnWidth = () => {
  if (!container.value) return

  const containerWidth = container.value.offsetWidth
  const gap = 10 // 与CSS中的gap值一致
  const totalGapWidth = gap * (columnCount.value - 1)
  const availableWidth = containerWidth - totalGapWidth
  columnWidth.value = Math.floor(availableWidth / columnCount.value)
}

// 统一的布局更新函数
const updateLayout = debounce(() => {
  updateColumnCount()
  updateColumnWidth()
  // 布局变化后需要重新观察所有哨兵
  reobserveSentinels()
}, 100) // 100ms防抖，避免频繁重排

// 设置列哨兵引用
const setColumnSentinel = (el: any, columnId: number) => {
  if (el) {
    columnSentinels.value.set(columnId, el)
  } else {
    columnSentinels.value.delete(columnId)
  }
}

// 重新观察所有哨兵
const reobserveSentinels = () => {
  if (!sentinelObserver.value) return

  // 取消观察所有现有哨兵
  columnSentinels.value.forEach((el) => {
    sentinelObserver.value?.unobserve(el)
  })

  // 重新观察所有哨兵
  columnSentinels.value.forEach((el) => {
    sentinelObserver.value?.observe(el)
  })
}

// 触发加载更多
const loadMore = () => {
  if (props.enableScrollLoad && !isLoadingMore && displayHasMore.value) {
    isLoadingMore = true
    emit('load-more')
    // 延迟重置，防止快速连续触发
    setTimeout(() => {
      isLoadingMore = false
    }, 500)
  }
}

const columns = computed(() => {
  // 初始化列数组
  const cols = Array.from({ length: columnCount.value }, (_, i) => ({
    id: i,
    items: [] as MediaFile[]
  }))

  // 按顺序分配算法：将图片按顺序依次分配到每一列
  // 这样可以确保在纵向方向上图片是从新到旧排序的
  displayItems.value.forEach((item, index) => {
    // 计算图片应该分配到的列索引
    const columnIndex = index % columnCount.value
    cols[columnIndex].items.push(item)
  })

  return cols
})

// 定义事件
const emit = defineEmits<{
  (e: 'click', item: MediaFile): void
  (e: 'load-more'): void
}>()

const handleClick = (item: MediaFile) => {
  // 触发查看详情事件
  emit('click', item)
}

// 键盘导航支持
const handleKeydown = (e: KeyboardEvent) => {
  // 键盘导航逻辑可由父组件实现
  if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
    // 保留键盘导航接口，未来可扩展
  }
}

onMounted(() => {
  // 初始化哨兵 Observer
  sentinelObserver.value = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          loadMore()
        }
      })
    },
    {
      rootMargin: '400px', // 提前 400px 触发加载更多
      threshold: 0
    }
  )

  // 观察所有列哨兵
  columnSentinels.value.forEach((el) => {
    sentinelObserver.value?.observe(el)
  })

  window.addEventListener('resize', updateLayout)
  window.addEventListener('keydown', handleKeydown)
  // 使用requestAnimationFrame确保DOM已渲染
  requestAnimationFrame(() => {
    updateColumnCount()
    updateColumnWidth()
  })
})

onUnmounted(() => {
  sentinelObserver.value?.disconnect()
  window.removeEventListener('resize', updateLayout)
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.gallery-container {
  display: flex;
  flex-direction: row;
  padding: 10px;
  gap: 10px;
}

.gallery-column {
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex: 1;
}

.loading, .empty {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  text-align: center;
  padding: 20px;
  color: #666;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 4px solid #f3f3f3;
  border-top: 4px solid #3498db;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.column-sentinel {
  height: 1px;
  width: 100%;
}
</style>
