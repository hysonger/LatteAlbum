<template>
  <div class="home">
    <main class="main-content">
      <Gallery
        :items="displayItems"
        :is-loading="displayIsLoading"
        :is-empty="displayIsEmpty"
        :has-more="displayHasMore"
        :enable-scroll-load="!galleryStore.showDateResults"
        @click="handleClick"
        @load-more="handleLoadMore"
      />

      <!-- 日期筛选结果头部信息 -->
      <div v-if="galleryStore.showDateResults" class="date-results-header">
        <h2>{{ selectedDate }} 的照片</h2>
      </div>
    </main>

    <!-- 固定在底部的导航栏 -->
    <footer class="footer">
      <div class="controls">
        <DateNavigator @date-selected="handleDateSelected" @clear="backToGallery" />

        <!-- 桌面端：排序和过滤按钮 -->
        <template v-if="!isMobile">
          <FilterControls
            :filter-type="filterType"
            @update:filter-type="selectFilter"
          />
          <SortControls
            :sort-by="sortBy"
            :sort-order="sortOrder"
            :is-mobile="isMobile"
            @update:sort-by="selectSort"
            @update:sort-order="toggleSortOrder"
          />
        </template>

        <RefreshButton
          :refresh-status="refreshStatus"
          :is-scanning="isScanning"
          :progress-percent="scanProgressPercentage"
          @click="handleRefresh"
        />

        <!-- 手机端更多按钮 -->
        <div class="more-button" @click="toggleMobileMenu" v-if="isMobile">
          <i class="fas fa-ellipsis-h"></i>
        </div>
      </div>

      <!-- 手机端折叠菜单 -->
      <MobileMenu
        :show="showMobileMenu && isMobile"
        :sort-by="sortBy"
        :sort-order="sortOrder"
        :filter-type="filterType"
        @close="showMobileMenu = false"
        @sort-change="selectSort"
        @sort-order-change="toggleSortOrder"
        @filter-change="selectFilter"
      />

      <!-- 扫描进度弹窗 -->
      <ScanProgressPopup
        :progress-data="scanProgressData"
        :is-visible="showScanPopup"
        :is-mobile="isMobile"
        :refresh-status="refreshStatus"
        @close="showScanPopup = false"
        @stop="handleStopScan"
      />
    </footer>

    <PhotoViewer
      v-if="showViewer && currentFile"
      :file="currentFile"
      :neighbors="currentNeighbors"
      @close="closeViewer"
      @change="handleChangeFile"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useGalleryStore } from '@/stores/gallery'
import { useScreenSize } from '@/composables/useScreenSize'
import { ElMessageBox } from 'element-plus'
import Gallery from '@/components/Gallery.vue'
import DateNavigator from '@/components/DateNavigator.vue'
import PhotoViewer from '@/components/PhotoViewer.vue'
import SortControls from '@/components/SortControls.vue'
import FilterControls from '@/components/FilterControls.vue'
import RefreshButton from '@/components/RefreshButton.vue'
import MobileMenu from '@/components/MobileMenu.vue'
import ScanProgressPopup from '@/components/ScanProgressPopup.vue'
import type { MediaFile } from '@/types'
import { systemApi } from '@/services/api'
import { scanProgressWs, type ScanProgressMessage } from '@/services/websocket'

const galleryStore = useGalleryStore()

// 排序相关
const sortBy = ref(galleryStore.sortBy)
const sortOrder = ref<'asc' | 'desc'>(galleryStore.sortOrder as 'asc' | 'desc')

// 过滤相关
const filterType = ref(galleryStore.filterType)

// 统一的数据源管理 - 使用 galleryStore 的 displayItems
const displayItems = computed(() => galleryStore.displayItems)

const displayIsLoading = computed(() => {
  return galleryStore.showDateResults ? false : galleryStore.isLoading
})

const displayIsEmpty = computed(() => {
  return galleryStore.isEmpty
})

const displayHasMore = computed(() => {
  return galleryStore.showDateResults ? false : galleryStore.hasMore
})

const handleLoadMore = () => {
  if (!galleryStore.showDateResults) {
    galleryStore.loadNextPage()
  }
}

// 刷新相关
const refreshStatus = ref<'default' | 'refreshing' | 'success' | 'error'>('default')
const scanProgressData = ref<{
  scanning: boolean
  status: 'progress' | 'completed' | 'error' | 'idle' | 'cancelled'
  phase?: string
  totalFiles: number
  successCount: number
  failureCount: number
  progressPercentage: string
  filesToAdd?: number
  filesToUpdate?: number
  filesToDelete?: number
}>({
  scanning: false,
  status: 'idle',
  totalFiles: 0,
  successCount: 0,
  failureCount: 0,
  progressPercentage: ''
})

// 扫描进度弹窗
const showScanPopup = ref(false)

// 扫描进度百分比（用于圆环显示）
const scanProgressPercentage = computed(() => {
  return Math.min(parseFloat(scanProgressData.value?.progressPercentage || '0'), 100)
})

// 可靠的是否正在扫描状态
const isScanning = computed(() => {
  const status = scanProgressData.value?.status
  return status === 'progress'
})

// 其他状态
const showViewer = ref(false)
const currentFile = ref<MediaFile | null>(null)
const currentNeighbors = ref<MediaFile[]>([])
const selectedDate = ref('')

// 手机端相关
const showMobileMenu = ref(false)
const { isMobile } = useScreenSize()

// 切换手机端菜单
const toggleMobileMenu = () => {
  showMobileMenu.value = !showMobileMenu.value
}

// 选择排序方式
const selectSort = (value: string) => {
  sortBy.value = value
  galleryStore.sortBy = value
  galleryStore.refresh()
}

// 切换排序方向
const toggleSortOrder = () => {
  sortOrder.value = sortOrder.value === 'desc' ? 'asc' : 'desc'
  galleryStore.sortOrder = sortOrder.value
  galleryStore.refresh()
}

// 选择过滤方式
const selectFilter = (value: string) => {
  filterType.value = value
  galleryStore.filterType = value
  galleryStore.refresh()
}

// 处理 WebSocket 进度消息
const handleScanProgress = (progress: ScanProgressMessage) => {
  if (progress.status === 'progress') {
    if (refreshStatus.value !== 'refreshing') {
      refreshStatus.value = 'refreshing'
    }

    scanProgressData.value = {
      scanning: true,
      status: 'progress',
      phase: progress.phase,
      totalFiles: progress.totalFiles || 0,
      successCount: progress.successCount || 0,
      failureCount: progress.failureCount || 0,
      progressPercentage: progress.progressPercentage || '0',
      filesToAdd: progress.filesToAdd || 0,
      filesToUpdate: progress.filesToUpdate || 0,
      filesToDelete: progress.filesToDelete || 0
    }
    return
  }

  switch (progress.status) {
    case 'completed':
      if (refreshStatus.value !== 'success') {
        refreshStatus.value = 'success'
      }
      showScanPopup.value = false
      scanProgressData.value = {
        scanning: false,
        status: 'completed',
        phase: 'completed',
        totalFiles: progress.totalFiles,
        successCount: progress.successCount,
        failureCount: progress.failureCount,
        progressPercentage: '100',
        filesToAdd: progress.filesToAdd,
        filesToUpdate: progress.filesToUpdate,
        filesToDelete: progress.filesToDelete
      }

      // 2秒后恢复默认状态
      setTimeout(() => {
        refreshStatus.value = 'default'
        if (scanProgressData.value) {
          scanProgressData.value.status = 'idle'
        }
      }, 2000)
      // 刷新相册数据
      galleryStore.refresh()
      break

    case 'error':
      refreshStatus.value = 'error'
      scanProgressData.value = {
        scanning: false,
        status: 'error',
        phase: 'error',
        totalFiles: progress.totalFiles || 0,
        successCount: progress.successCount || 0,
        failureCount: progress.failureCount || 0,
        progressPercentage: '100',
        filesToAdd: progress.filesToAdd || 0,
        filesToUpdate: progress.filesToUpdate || 0,
        filesToDelete: progress.filesToDelete || 0
      }

      // 3秒后恢复默认状态
      setTimeout(() => {
        refreshStatus.value = 'default'
        if (scanProgressData.value) {
          scanProgressData.value.status = 'idle'
        }
      }, 3000)
      break

    case 'cancelled':
      showScanPopup.value = false
      refreshStatus.value = 'default'
      if (scanProgressData.value) {
        scanProgressData.value.status = 'idle'
      }
      galleryStore.refresh()
      break
  }
}

// 刷新功能
const handleRefresh = async () => {
  // 如果正在扫描，点击按钮切换弹窗显示
  if (isScanning.value) {
    showScanPopup.value = !showScanPopup.value
    return
  }

  // 不在扫描状态，点击按钮触发新扫描
  try {
    refreshStatus.value = 'refreshing'
    await systemApi.rescan()
  } catch (error) {
    console.error('刷新失败:', error)
    if (scanProgressData.value) {
      scanProgressData.value.status = 'idle'
    }
    refreshStatus.value = 'error'
    showScanPopup.value = false

    setTimeout(() => {
      refreshStatus.value = 'default'
    }, 3000)
  }
}

// 点击外部关闭弹窗和菜单
const handleClickOutside = (event: MouseEvent) => {
  const target = event.target as HTMLElement
  const mainContent = document.querySelector('.main-content')

  if (mainContent && mainContent.contains(target)) {
    showMobileMenu.value = false
  }
}

// 停止扫描
const handleStopScan = () => {
  ElMessageBox.confirm(
    '确定要停止扫描吗？已完成的处理将保留。',
    '停止扫描确认',
    {
      confirmButtonText: '确定停止',
      cancelButtonText: '继续扫描',
      type: 'warning'
    }
  ).then(async () => {
    try {
      await systemApi.cancelScan()
      showScanPopup.value = false
    } catch (error) {
      console.error('停止扫描失败:', error)
    }
  }).catch(() => {
    // 用户取消，继续扫描
  })
}

const handleDateSelected = (files: MediaFile[], date: string) => {
  galleryStore.setDateResults(files)
  selectedDate.value = date
}

const backToGallery = () => {
  galleryStore.clearDateResults()
}

const closeViewer = () => {
  showViewer.value = false
  currentFile.value = null
  currentNeighbors.value = []
}

const handleClick = (item: MediaFile) => {
  currentFile.value = item
  currentNeighbors.value = galleryStore.displayItems
  showViewer.value = true
}

const handleChangeFile = (file: MediaFile) => {
  currentFile.value = file
}

// 初始化加载
onMounted(async () => {
  if (galleryStore.items.length === 0 && !galleryStore.isLoading) {
    galleryStore.loadPage(0)
  }

  document.addEventListener('click', handleClickOutside)

  try {
    await scanProgressWs.connect()
    scanProgressWs.onProgress(handleScanProgress)

    // 尝试获取当前扫描进度
    try {
      const progressResponse = await systemApi.getScanProgress()
      if (progressResponse.data.status === 'progress') {
        scanProgressData.value = {
          scanning: true,
          status: 'progress',
          phase: progressResponse.data.phase || 'processing',
          totalFiles: progressResponse.data.totalFiles || 0,
          successCount: progressResponse.data.successCount || 0,
          failureCount: progressResponse.data.failureCount || 0,
          progressPercentage: progressResponse.data.progressPercentage || '0',
          filesToAdd: progressResponse.data.filesToAdd,
          filesToUpdate: progressResponse.data.filesToUpdate,
          filesToDelete: progressResponse.data.filesToDelete
        }
        refreshStatus.value = 'refreshing'
      }
    } catch (e) {
      console.error('[HomeView] 获取扫描进度失败:', e)
    }
  } catch (error) {
    console.error('[HomeView] WebSocket 连接失败:', error)
  }
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
  scanProgressWs.offProgress()
})
</script>

<style scoped>
.home {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.footer {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 10px 16px;
  background: rgba(255, 255, 255, 0.85);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.1);
  display: flex;
  justify-content: center;
  align-items: center;
  flex-wrap: nowrap;
  gap: 12px;
  z-index: 100;
  border-top: 1px solid rgba(228, 231, 237, 0.5);
}

.main-content {
  flex: 1;
  overflow-y: auto;
  padding: 10px 10px 80px 10px;
}

/* 日期筛选模式下的顶部间距 */
.main-content:has(.date-results-header) {
  padding-top: 70px;
}

.controls {
  display: flex;
  gap: 12px;
  flex-wrap: nowrap;
  align-items: center;
  justify-content: center;
  position: relative;
  width: 100%;
  max-width: 1200px;
  margin: 0 auto;
}

/* 手机端更多按钮 */
.more-button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 32px;
  border: 1px solid #dcdfe6;
  border-radius: 4px;
  background: white;
  cursor: pointer;
  transition: all 0.3s ease;
  font-size: 16px;
  box-sizing: border-box;
}

.more-button:hover {
  border-color: #409eff;
  background: #ecf5ff;
}

.date-results-header {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  background: white;
  padding: 15px 20px;
  border-bottom: 1px solid #e4e7ed;
  display: flex;
  justify-content: space-between;
  align-items: center;
  z-index: 50;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.date-results-header h2 {
  margin: 0;
  font-size: 18px;
  color: #303133;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .footer {
    padding: 8px 12px;
    gap: 8px;
  }

  .controls {
    gap: 8px;
    justify-content: center;
  }

  .more-button {
    width: 32px;
    height: 32px;
  }
}

/* 超小屏幕优化 */
@media (max-width: 375px) {
  .footer {
    padding: 4px 6px;
    gap: 4px;
  }

  .controls {
    gap: 4px;
    justify-content: center;
  }

  .more-button {
    width: 28px;
    height: 28px;
  }
}
</style>
