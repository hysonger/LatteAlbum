import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { fileApi } from '@/services/api'
import type { MediaFile } from '@/types'

export const useGalleryStore = defineStore('gallery', () => {
  // 状态
  const items = ref<MediaFile[]>([])
  const currentPage = ref(0)
  const hasMore = ref(true)
  const isLoading = ref(false)
  const sortBy = ref('exifTimestamp')
  const sortOrder = ref('desc')
  const filterType = ref('all')
  const currentPath = ref('')
  const pageSize = ref(50)
  const showDateResults = ref(false)
  const dateResults = ref<MediaFile[]>([])

  // 计算属性
  const isEmpty = computed(() => displayItems.value.length === 0)
  const displayItems = computed(() => {
    return showDateResults.value ? dateResults.value : items.value
  })

  // 动作
  async function loadPage(page: number) {
    isLoading.value = true
    try {
      const response = await fileApi.getFiles({
        path: currentPath.value,
        page,
        size: pageSize.value,
        sortBy: sortBy.value,
        order: sortOrder.value,
        filterType: filterType.value
      })
      
      if (page === 0) {
        items.value = response.data.content
      } else {
        items.value.push(...response.data.content)
      }
      
      hasMore.value = !response.data.last
      currentPage.value = page
      
      // 预加载下一批图片
      if (hasMore.value) {
        preloadNextPage()
      }
    } catch (error) {
      console.error('加载页面失败:', error)
    } finally {
      isLoading.value = false
    }
  }
  
  function preloadNextPage() {
    // 获取下一批图片的缩略图URL并预加载
    // 这里可以实现预加载逻辑
  }
  
  async function loadNextPage() {
    if (!isLoading.value && hasMore.value) {
      await loadPage(currentPage.value + 1)
    }
  }
  
  async function refresh() {
    await loadPage(0)
  }
  
  function reset() {
    items.value = []
    currentPage.value = 0
    hasMore.value = true
  }

  function setDateResults(files: MediaFile[]) {
    dateResults.value = files
    showDateResults.value = true
  }

  function clearDateResults() {
    dateResults.value = []
    showDateResults.value = false
  }

  return {
    // 状态
    items,
    currentPage,
    hasMore,
    isLoading,
    sortBy,
    sortOrder,
    filterType,
    currentPath,
    pageSize,
    showDateResults,
    dateResults,
    isEmpty,
    displayItems,

    // 动作
    loadPage,
    loadNextPage,
    refresh,
    reset,
    preloadNextPage,
    setDateResults,
    clearDateResults
  }
})