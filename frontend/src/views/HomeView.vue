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
          <!-- 可折叠式排序按钮组件 -->
          <div class="sort-container">
            <div 
              class="sort-button" 
              @click="toggleSortMenu"
            >
              <i class="fas fa-sort"></i>
              <span>{{ getSortLabel(sortBy) }}</span>
              <i class="fas fa-chevron-up"></i>
            </div>
            <transition name="fade">
              <div v-if="showSortMenu" class="sort-menu">
                <div 
                  v-for="option in sortOptions" 
                  :key="option.value"
                  class="sort-option"
                  :class="{ active: sortBy === option.value }"
                  @click="selectSort(option.value)"
                >
                  {{ option.label }}
                </div>
              </div>
            </transition>
          </div>
          
          <!-- 排序方向切换按钮 -->
          <div 
            class="sort-order-button"
            @click="toggleSortOrder"
            :title="sortOrder === 'desc' ? '倒序' : '正序'"
          >
            <i :class="sortOrder === 'desc' ? 'fas fa-sort-amount-down' : 'fas fa-sort-amount-up'"></i>
          </div>
          
          <!-- 水平排列的过滤图标按钮 -->
          <div class="filter-container">
            <div 
              v-for="option in filterOptions" 
              :key="option.value"
              class="filter-button"
              :class="{ active: filterType === option.value }"
              @click="selectFilter(option.value)"
            >
              <i :class="option.icon"></i>
            </div>
          </div>
        </template>
        
        <!-- 刷新功能按钮 -->
        <div
          class="refresh-button"
          :class="{
            success: refreshStatus === 'success',
            error: refreshStatus === 'error',
            refreshing: isRefreshing
          }"
          @click="handleRefresh"
          :disabled="isRefreshing"
        >
          <svg v-if="isRefreshing" class="progress-ring" viewBox="0 0 36 36">
            <path
              class="progress-ring-bg"
              d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
            />
            <path
              class="progress-ring-fill"
              :stroke-dasharray="`${parseFloat(scanProgressData?.progressPercentage || '0')}, 100`"
              d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
            />
          </svg>
          <i v-if="!isRefreshing && refreshStatus === 'success'" class="fas fa-check"></i>
          <i v-if="!isRefreshing && refreshStatus === 'error'" class="fas fa-times"></i>
          <i v-if="!isRefreshing && refreshStatus === 'default'" class="fas fa-sync-alt"></i>
        </div>
        
        <!-- 手机端更多按钮 -->
        <div class="more-button" @click="toggleMobileMenu" v-if="isMobile">
          <i class="fas fa-ellipsis-h"></i>
        </div>
      </div>
      
      <!-- 手机端折叠菜单 -->
      <transition name="slide-up">
        <div v-if="showMobileMenu && isMobile" class="mobile-menu">
          <!-- 排序选项 -->
          <div class="mobile-menu-section">
            <div class="mobile-menu-title">
              <i class="fas fa-sort"></i>
              <span>排序方式</span>
            </div>
            <div class="mobile-sort-options">
              <div 
                v-for="option in sortOptions" 
                :key="option.value"
                class="mobile-sort-option"
                :class="{ active: sortBy === option.value }"
                @click="selectSort(option.value)"
              >
                {{ option.label }}
              </div>
            </div>
            <!-- 排序方向切换 -->
            <div 
              class="mobile-sort-order-toggle"
              @click="toggleSortOrder"
            >
              <i :class="sortOrder === 'desc' ? 'fas fa-sort-amount-down' : 'fas fa-sort-amount-up'"></i>
              <span>{{ sortOrder === 'desc' ? '倒序' : '正序' }}</span>
            </div>
          </div>
          
          <!-- 过滤选项 -->
          <div class="mobile-menu-section">
            <div class="mobile-menu-title">
              <i class="fas fa-filter"></i>
              <span>文件类型</span>
            </div>
            <div class="mobile-filter-buttons">
              <div 
                v-for="option in filterOptions" 
                :key="option.value"
                class="mobile-filter-button"
                :class="{ active: filterType === option.value }"
                @click="selectFilter(option.value)"
              >
                <i :class="option.icon"></i>
                <span>{{ option.label }}</span>
              </div>
            </div>
          </div>
        </div>
      </transition>
    </footer>
    
    <PhotoViewer
      v-if="showViewer && currentFile"
      :file="currentFile"
      :neighbors="currentNeighbors"
      @close="closeViewer"
      @change="handleChangeFile"
    />

    <!-- 扫描进度对话框 -->
    <el-dialog
      v-model="showScanDialog"
      title="扫描进度"
      :close-on-click-modal="false"
      width="90%"
      class="scan-progress-dialog"
      :append-to-body="true"
    >
      <!-- 阶段信息 -->
      <div class="phase-info">
        <span class="phase-text">{{ scanProgressData?.phaseMessage || '初始化中...' }}</span>
      </div>

      <!-- 进度条 -->
      <el-progress
        :percentage="parseFloat(scanProgressData?.progressPercentage || '0')"
        :stroke-width="10"
      />

      <!-- 详细统计 -->
      <div class="scan-stats">
        <div class="stat-item">
          <span class="stat-label">新增</span>
          <span class="stat-value add">{{ scanProgressData?.filesToAdd || 0 }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">修改</span>
          <span class="stat-value update">{{ scanProgressData?.filesToUpdate || 0 }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">删除</span>
          <span class="stat-value delete">{{ scanProgressData?.filesToDelete || 0 }}</span>
        </div>
      </div>

      <!-- 处理进度 -->
      <div class="processing-info">
        <div class="info-row">
          <span>已处理</span>
          <span>{{ scanProgressData?.successCount || 0 }} / {{ scanProgressData?.totalFiles || 0 }}</span>
        </div>
        <div class="info-row" v-if="scanProgressData && (scanProgressData.failureCount || 0) > 0">
          <span>失败</span>
          <span class="error">{{ scanProgressData.failureCount }}</span>
        </div>
      </div>

      <!-- 底部按钮 -->
      <template #footer>
        <el-button @click="showScanDialog = false">隐藏</el-button>
        <el-button type="danger" @click="handleStopScan">停止扫描</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useGalleryStore } from '@/stores/gallery'
import { ElMessageBox } from 'element-plus'
import Gallery from '@/components/Gallery.vue'
import DateNavigator from '@/components/DateNavigator.vue'
import PhotoViewer from '@/components/PhotoViewer.vue'
import type { MediaFile } from '@/types'
import { systemApi } from '@/services/api'
import { scanProgressWs, type ScanProgressMessage } from '@/services/websocket'

const galleryStore = useGalleryStore()

// 排序相关
const sortBy = ref(galleryStore.sortBy)
const sortOrder = ref(galleryStore.sortOrder)
const showSortMenu = ref(false)
const sortOptions = [
  { label: '按拍摄时间', value: 'exifTimestamp' },
  { label: '按创建时间', value: 'createTime' },
  { label: '按修改时间', value: 'modifyTime' },
  { label: '按文件名', value: 'fileName' }
]

// 过滤相关
const filterType = ref(galleryStore.filterType)
const filterOptions = [
  { label: '全部', value: 'all', icon: 'fas fa-th-large' },
  { label: '图片', value: 'image', icon: 'fas fa-image' },
  { label: '视频', value: 'video', icon: 'fas fa-video' }
]

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
const isRefreshing = ref(false)
const refreshStatus = ref<'default' | 'refreshing' | 'success' | 'error'>('default')
const scanProgressData = ref<{
  scanning: boolean
  phase?: string
  phaseMessage?: string
  totalFiles: number
  successCount: number
  failureCount: number
  progressPercentage: string
  filesToAdd?: number
  filesToUpdate?: number
  filesToDelete?: number
} | null>(null)

// 扫描进度对话框
const showScanDialog = ref(false)

// 其他状态
const showViewer = ref(false)
const currentFile = ref<MediaFile | null>(null)
const currentNeighbors = ref<MediaFile[]>([])
const selectedDate = ref('')

// 手机端相关
const showMobileMenu = ref(false)
const isMobile = ref(false)

// 检测是否为移动设备
const checkMobile = () => {
  isMobile.value = window.innerWidth < 768
}

// 切换手机端菜单
const toggleMobileMenu = () => {
  showMobileMenu.value = !showMobileMenu.value
  // 关闭排序菜单
  showSortMenu.value = false
}

// 点击外部关闭菜单
const handleClickOutside = (event: MouseEvent) => {
  const target = event.target as HTMLElement
  const mainContent = document.querySelector('.main-content')
  
  // 只在点击主内容区域时关闭菜单
  if (mainContent && mainContent.contains(target)) {
    showMobileMenu.value = false
    showSortMenu.value = false
  }
}

// 获取排序标签
const getSortLabel = (value: string) => {
  const option = sortOptions.find(opt => opt.value === value)
  return option ? option.label : '排序方式'
}

// 切换排序菜单
const toggleSortMenu = () => {
  if (isMobile.value) {
    // 移动端：关闭移动菜单
    showMobileMenu.value = false
  } else {
    // 桌面端：切换排序菜单
    showSortMenu.value = !showSortMenu.value
  }
}

// 选择排序方式
const selectSort = (value: string) => {
  sortBy.value = value
  galleryStore.sortBy = value
  galleryStore.refresh()
  showSortMenu.value = false
  // 关闭手机端菜单
  showMobileMenu.value = false
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
  // 关闭手机端菜单
  showMobileMenu.value = false
}

// 处理 WebSocket 进度消息
const handleScanProgress = (progress: ScanProgressMessage) => {
  console.log('[HomeView] 收到进度消息:', progress)

  switch (progress.status) {
    case 'started':
      refreshStatus.value = 'refreshing'
      scanProgressData.value = {
        scanning: true,
        phase: progress.phase,
        phaseMessage: progress.phaseMessage,
        totalFiles: progress.totalFiles || 0,
        successCount: 0,
        failureCount: 0,
        progressPercentage: '0',
        filesToAdd: progress.filesToAdd,
        filesToUpdate: progress.filesToUpdate,
        filesToDelete: progress.filesToDelete
      }
      break

    case 'progress':
      scanProgressData.value = {
        scanning: true,
        phase: progress.phase,
        phaseMessage: progress.phaseMessage,
        totalFiles: progress.totalFiles,
        successCount: progress.successCount,
        failureCount: progress.failureCount,
        progressPercentage: progress.progressPercentage,
        filesToAdd: progress.filesToAdd,
        filesToUpdate: progress.filesToUpdate,
        filesToDelete: progress.filesToDelete
      }
      break

    case 'completed':
      showScanDialog.value = false
      scanProgressData.value = {
        scanning: false,
        phase: 'completed',
        phaseMessage: '扫描完成',
        totalFiles: progress.totalFiles,
        successCount: progress.successCount,
        failureCount: progress.failureCount,
        progressPercentage: '100',
        filesToAdd: progress.filesToAdd,
        filesToUpdate: progress.filesToUpdate,
        filesToDelete: progress.filesToDelete
      }
      isRefreshing.value = false
      refreshStatus.value = 'success'

      // 2秒后恢复默认状态
      setTimeout(() => {
        refreshStatus.value = 'default'
        scanProgressData.value = null
      }, 2000)
      // 刷新相册数据
      galleryStore.refresh()
      break

    case 'error':
      isRefreshing.value = false
      refreshStatus.value = 'error'
      scanProgressData.value = null

      // 3秒后恢复默认状态
      setTimeout(() => {
        refreshStatus.value = 'default'
      }, 3000)
      break

    case 'cancelled':
      isRefreshing.value = false
      refreshStatus.value = 'default'
      scanProgressData.value = null
      break
  }
}

// 刷新功能
const handleRefresh = async () => {
  // 如果正在扫描，打开进度对话框
  if (isRefreshing.value) {
    showScanDialog.value = true
    return
  }

  try {
    isRefreshing.value = true
    refreshStatus.value = 'refreshing'
    scanProgressData.value = null

    // 调用重新扫描接口
    await systemApi.rescan()
    // WebSocket 会自动接收进度更新
  } catch (error) {
    console.error('刷新失败:', error)
    isRefreshing.value = false
    refreshStatus.value = 'error'

    // 3秒后恢复默认状态
    setTimeout(() => {
      refreshStatus.value = 'default'
    }, 3000)
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
      showScanDialog.value = false
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
  // 如果galleryStore中没有数据，则加载第一页
  if (galleryStore.items.length === 0 && !galleryStore.isLoading) {
    galleryStore.loadPage(0)
  }

  // 检测移动设备
  checkMobile()
  window.addEventListener('resize', checkMobile)
  // 添加点击外部关闭菜单事件
  document.addEventListener('click', handleClickOutside)

  // 连接 WebSocket 并订阅进度消息
  try {
    await scanProgressWs.connect()
    scanProgressWs.onProgress(handleScanProgress)
    console.log('[HomeView] WebSocket 订阅已设置')

    // 检查是否有正在进行的扫描
    try {
      const statusResponse = await systemApi.getStatus()
      if (statusResponse.data.scanning) {
        // 标记正在扫描，刷新按钮显示进度环
        isRefreshing.value = true
        refreshStatus.value = 'refreshing'
        // 尝试获取当前进度
        try {
          const progressResponse = await systemApi.getScanProgress()
          if (progressResponse.data.scanning) {
            scanProgressData.value = {
              scanning: true,
              phase: 'processing',
              phaseMessage: '正在恢复扫描进度...',
              totalFiles: progressResponse.data.totalFiles || 0,
              successCount: progressResponse.data.successCount || 0,
              failureCount: progressResponse.data.failureCount || 0,
              progressPercentage: progressResponse.data.progressPercentage || '0',
              filesToAdd: progressResponse.data.filesToAdd,
              filesToUpdate: progressResponse.data.filesToUpdate,
              filesToDelete: progressResponse.data.filesToDelete
            }
          }
        } catch (e) {
          console.error('[HomeView] 获取扫描进度失败:', e)
        }
      }
    } catch (e) {
      console.error('[HomeView] 检查扫描状态失败:', e)
    }
  } catch (error) {
    console.error('[HomeView] WebSocket 连接失败:', error)
  }
})

// 清理事件监听
onUnmounted(() => {
  window.removeEventListener('resize', checkMobile)
  document.removeEventListener('click', handleClickOutside)
  // 清理 WebSocket
  scanProgressWs.offProgress()
  // 不主动断开连接，让连接保持以便下次使用
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
  padding: 10px 10px 80px 10px; /* 添加底部内边距，避免被底栏遮挡 */
}

/* 日期筛选模式下的顶部间距 */
.main-content:has(.date-results-header) {
  padding-top: 70px; /* 为固定头部留出空间 */
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

/* 排序组件样式 */
.sort-container {
  position: relative;
  z-index: 10;
}

.sort-button {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  height: 32px;
  line-height: 32px;
  border: 1px solid #dcdfe6;
  border-radius: 4px;
  background: white;
  cursor: pointer;
  font-size: 13px;
  transition: all 0.3s ease;
  box-sizing: border-box;
}

.sort-button:hover {
  border-color: #409eff;
  background: #ecf5ff;
}

.sort-menu {
  position: absolute;
  bottom: 100%;
  left: 0;
  margin-bottom: 4px;
  padding: 4px 0;
  border: 1px solid #dcdfe6;
  border-radius: 4px;
  background: white;
  box-shadow: 0 -2px 12px 0 rgba(0, 0, 0, 0.1);
  min-width: 150px;
  max-height: 200px;
  overflow-y: auto;
  z-index: 200;
}

.sort-option {
  padding: 8px 16px;
  cursor: pointer;
  transition: all 0.3s ease;
  font-size: 13px;
}

.sort-option:hover {
  background: #ecf5ff;
}

.sort-option.active {
  background: #409eff;
  color: white;
}

/* 排序方向切换按钮样式 */
.sort-order-button {
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

.sort-order-button:hover {
  border-color: #409eff;
  background: #ecf5ff;
  color: #409eff;
}

.sort-order-button i {
  transition: transform 0.3s ease;
}

.sort-order-button:active i {
  transform: scale(0.9);
}

/* 过滤组件样式 */
.filter-container {
  display: flex;
  gap: 8px;
  background: white;
  border: 1px solid #dcdfe6;
  border-radius: 4px;
  padding: 2px;
  box-sizing: border-box;
  height: 32px;
  align-items: center;
  justify-content: center;
}

.filter-button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 32px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.3s ease;
  font-size: 16px;
  box-sizing: border-box;
}

.filter-button:hover {
  background: #ecf5ff;
}

.filter-button.active {
  background: #409eff;
  color: white;
}

/* 刷新组件样式 */
.refresh-button {
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
  position: relative;
}

.refresh-button:hover:not(:disabled) {
  border-color: #409eff;
  background: #ecf5ff;
}

.refresh-button:disabled {
  cursor: not-allowed;
  opacity: 0.8;
}

.refresh-button.success {
  color: #67c23a;
  border-color: #67c23a;
}

.refresh-button.error {
  color: #f56c6c;
  border-color: #f56c6c;
}

/* 圆环进度条 */
.progress-ring {
  position: absolute;
  width: 24px;
  height: 24px;
  transform: rotate(-90deg);
  pointer-events: none;
}

.progress-ring-bg {
  fill: none;
  stroke: #ebeef5;
  stroke-width: 2.5;
}

.progress-ring-fill {
  fill: none;
  stroke: #409eff;
  stroke-width: 2.5;
  stroke-linecap: round;
  transition: stroke-dasharray 0.3s ease;
}

/* 动画效果 */
@keyframes rotate {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
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

/* 手机端折叠菜单 */
.mobile-menu {
  position: absolute;
  bottom: 100%;
  left: 0;
  right: 0;
  background: white;
  border-radius: 8px 8px 0 0;
  box-shadow: 0 -2px 12px rgba(0, 0, 0, 0.15);
  padding: 12px;
  z-index: 150;
}

.mobile-menu-section {
  margin-bottom: 16px;
}

.mobile-menu-section:last-child {
  margin-bottom: 0;
}

.mobile-menu-title {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
  font-size: 14px;
  font-weight: 500;
  color: #303133;
  border-bottom: 1px solid #f0f0f0;
  margin-bottom: 8px;
}

.mobile-menu-title i {
  font-size: 16px;
  color: #606266;
}

.mobile-menu-title span {
  font-size: 14px;
  color: #303133;
}

.mobile-sort-options {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.mobile-sort-option {
  padding: 10px 12px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 14px;
  color: #606266;
}

.mobile-sort-option:hover {
  background: #f5f7fa;
}

.mobile-sort-option.active {
  background: #409eff;
  color: white;
}

/* 移动端排序方向切换 */
.mobile-sort-order-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 10px 12px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 14px;
  color: #606266;
  background: #f5f7fa;
  border: 1px solid #dcdfe6;
  margin-top: 8px;
}

.mobile-sort-order-toggle:hover {
  border-color: #409eff;
  background: #ecf5ff;
  color: #409eff;
}

.mobile-sort-order-toggle i {
  font-size: 16px;
}

.mobile-sort-order-toggle span {
  font-size: 14px;
}

.mobile-filter-buttons {
  display: flex;
  gap: 8px;
}

.mobile-filter-button {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 8px 12px;
  border: 1px solid #dcdfe6;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s ease;
  flex: 1;
}

.mobile-filter-button:hover {
  border-color: #409eff;
  background: #ecf5ff;
}

.mobile-filter-button.active {
  border-color: #409eff;
  background: #409eff;
  color: white;
}

.mobile-filter-button i {
  font-size: 18px;
}

.mobile-filter-button span {
  font-size: 12px;
}

/* 滑动动画 */
.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.3s ease;
}

.slide-up-enter-from,
.slide-up-leave-to {
  opacity: 0;
  transform: translateY(20px);
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
  
  .sort-container {
    display: none;
  }
  
  .sort-order-button {
    display: none;
  }
  
  .filter-container {
    display: none;
  }
  
  .sort-button span {
    display: none;
  }
  
  .filter-button {
    width: 32px;
    height: 32px;
  }
  
  .refresh-button {
    width: 32px;
    height: 32px;
  }
  
  .more-button {
    width: 32px;
    height: 32px;
  }
}

.main-content {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
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
  
  :deep(.el-date-editor--date) {
    min-width: 90px;
    max-width: 100px;
  }
  
  :deep(.el-input__wrapper) {
    padding: 0 4px;
  }
  
  :deep(.el-button) {
    padding: 0 6px;
    min-width: 28px;
  }
  
  .refresh-button {
    width: 28px;
    height: 28px;
  }
  
  .more-button {
    width: 28px;
    height: 28px;
  }
  
  .mobile-sort-order-toggle {
    padding: 8px 10px;
    font-size: 13px;
  }
  
  .mobile-sort-order-toggle i {
    font-size: 14px;
  }
  
  .mobile-sort-order-toggle span {
    font-size: 13px;
  }
}

/* 扫描进度对话框样式 */
.scan-progress-dialog {
  --el-dialog-margin-top: 15vh;
}

.scan-progress-dialog .el-dialog {
  max-width: 450px;
}

.scan-progress-dialog .phase-info {
  margin-bottom: 16px;
  text-align: center;
}

.scan-progress-dialog .phase-text {
  font-size: 16px;
  font-weight: 500;
  color: #409eff;
}

.scan-progress-dialog .scan-stats {
  display: flex;
  justify-content: space-around;
  margin: 20px 0;
  padding: 12px;
  background: #f5f7fa;
  border-radius: 8px;
}

.scan-progress-dialog .stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.scan-progress-dialog .stat-label {
  font-size: 12px;
  color: #909399;
}

.scan-progress-dialog .stat-value {
  font-size: 18px;
  font-weight: 600;
}

.scan-progress-dialog .stat-value.add {
  color: #67c23a;
}

.scan-progress-dialog .stat-value.update {
  color: #409eff;
}

.scan-progress-dialog .stat-value.delete {
  color: #f56c6c;
}

.scan-progress-dialog .processing-info {
  margin-top: 16px;
  padding: 12px;
  background: #fafafa;
  border-radius: 8px;
}

.scan-progress-dialog .info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 0;
  font-size: 14px;
  color: #606266;
}

.scan-progress-dialog .info-row .error {
  color: #f56c6c;
  font-weight: 500;
}

/* 移动端适配 */
@media (max-width: 480px) {
  .scan-progress-dialog .el-dialog {
    width: 90% !important;
    margin: 0 auto !important;
  }

  .scan-progress-dialog .scan-stats {
    padding: 8px;
  }

  .scan-progress-dialog .stat-value {
    font-size: 16px;
  }

  .scan-progress-dialog .phase-text {
    font-size: 14px;
  }
}
</style>