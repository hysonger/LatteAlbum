<template>
  <transition name="fade">
    <div
      v-if="isVisible"
      :class="['scan-progress-container', { 'mobile': isMobile }]"
      @click.self="handleClickOutside"
    >
      <!-- 电脑端：气泡样式 -->
      <div v-if="!isMobile" class="scan-progress-popup" ref="popupRef">
        <div class="popup-header">
          <span class="popup-title">扫描进度</span>
          <i class="fas fa-times close-icon" @click="emit('close')"></i>
        </div>
        <!-- 阶段信息 -->
        <div class="phase-info">
          <span class="phase-text">{{ getPhaseMessage(progressData) }}</span>
        </div>
        <!-- 进度条 -->
        <el-progress
          :percentage="Math.min(parseFloat(progressData?.progressPercentage || '0'), 100)"
          :stroke-width="8"
        />
        <!-- 统计信息 -->
        <div class="scan-stats">
          <div class="stat-item">
            <span class="stat-label">新增</span>
            <span class="stat-value add">{{ progressData?.filesToAdd || 0 }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">修改</span>
            <span class="stat-value update">{{ progressData?.filesToUpdate || 0 }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">删除</span>
            <span class="stat-value delete">{{ progressData?.filesToDelete || 0 }}</span>
          </div>
        </div>
        <!-- 处理进度 -->
        <div class="processing-info">
          <span>已处理 {{ progressData?.successCount || 0 }} / {{ progressData?.totalFiles || 0 }}</span>
          <span v-if="(progressData?.failureCount || 0) > 0" class="error">失败 {{ progressData?.failureCount }}</span>
        </div>
        <!-- 底部按钮 -->
        <div class="popup-actions">
          <el-button
            v-if="progressData?.status === 'progress'"
            type="danger"
            size="small"
            @click="emit('stop')"
          >
            停止扫描
          </el-button>
          <el-button
            v-else
            size="small"
            @click="emit('close')"
          >
            关闭
          </el-button>
        </div>
      </div>

      <!-- 移动端：底部弹窗 -->
      <div v-else class="scan-progress-mobile" @click.self="emit('close')">
        <div class="mobile-popup-content">
          <div class="popup-header">
            <span class="popup-title">扫描进度</span>
            <i class="fas fa-times close-icon" @click="emit('close')"></i>
          </div>
          <!-- 阶段信息 -->
          <div class="phase-info">
            <span class="phase-text">{{ progressData ? getPhaseMessage(progressData) : '初始化中...' }}</span>
          </div>
          <!-- 进度条 -->
          <el-progress
            v-if="progressData"
            :percentage="Math.min(parseFloat(progressData.progressPercentage || '0'), 100)"
            :stroke-width="10"
          />
          <!-- 统计信息 -->
          <div class="scan-stats">
            <div class="stat-item">
              <span class="stat-label">新增</span>
              <span class="stat-value add">{{ progressData?.filesToAdd || 0 }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">修改</span>
              <span class="stat-value update">{{ progressData?.filesToUpdate || 0 }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">删除</span>
              <span class="stat-value delete">{{ progressData?.filesToDelete || 0 }}</span>
            </div>
          </div>
          <!-- 处理进度 -->
          <div class="processing-info">
            <span>已处理 {{ progressData?.successCount || 0 }} / {{ progressData?.totalFiles || 0 }}</span>
            <span v-if="progressData && (progressData.failureCount || 0) > 0" class="error">失败 {{ progressData.failureCount }}</span>
          </div>
          <!-- 底部按钮 -->
          <div class="popup-actions">
            <el-button
              v-if="progressData?.status === 'progress'"
              type="danger"
              @click="emit('stop')"
            >
              停止扫描
            </el-button>
            <el-button
              v-else
              @click="emit('close')"
            >
              关闭
            </el-button>
          </div>
        </div>
      </div>
    </div>
  </transition>
</template>

<script setup lang="ts">
import type { ScanProgressMessage } from '@/services/websocket'

// 阶段信息映射
const phaseMessages: Record<string, string> = {
  idle: '就绪',
  collecting: '收集中',
  counting: '检查中',
  processing: '处理中',
  writing: '保存中',
  deleting: '清理中',
  completed: '完成',
  error: '错误',
  cancelled: '已取消'
}

// 获取阶段中文信息
const getPhaseMessage = (data?: ScanProgressMessage): string => {
  if (!data?.phase) return '初始化中...'
  return phaseMessages[data.phase] || data.phase
}

interface ProgressData {
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
}

interface Props {
  progressData?: ProgressData
  isVisible: boolean
  isMobile: boolean
  refreshStatus: 'default' | 'refreshing' | 'success' | 'error'
}

withDefaults(defineProps<Props>(), {
  progressData: () => ({
    scanning: false,
    status: 'idle',
    totalFiles: 0,
    successCount: 0,
    failureCount: 0,
    progressPercentage: '0'
  })
})

const emit = defineEmits<{
  close: []
  stop: []
}>()

const handleClickOutside = (event: MouseEvent) => {
  // 移动端点击背景关闭
  const target = event.target as HTMLElement
  if (target.classList.contains('scan-progress-mobile')) {
    emit('close')
  }
}
</script>

<style scoped>
.scan-progress-container {
  position: absolute;
  bottom: 100%;
  left: 0;
  right: 0;
  z-index: 200;
}

/* 电脑端气泡样式 */
.scan-progress-popup {
  position: absolute;
  bottom: 100%;
  left: 0;
  margin-bottom: 4px;
  width: 280px;
  background: white;
  border: 1px solid #dcdfe6;
  border-radius: 6px;
  box-shadow: 0 -2px 12px rgba(0, 0, 0, 0.1);
  padding: 12px;
}

.scan-progress-popup .popup-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.scan-progress-popup .popup-title {
  font-size: 14px;
  font-weight: 500;
  color: #303133;
}

.scan-progress-popup .close-icon {
  cursor: pointer;
  color: #909399;
  padding: 4px;
  font-size: 14px;
}

.scan-progress-popup .close-icon:hover {
  color: #409eff;
}

.scan-progress-popup .phase-info {
  margin-bottom: 8px;
  text-align: center;
}

.scan-progress-popup .phase-text {
  font-size: 13px;
  font-weight: 500;
  color: #409eff;
}

.scan-progress-popup .scan-stats {
  display: flex;
  gap: 12px;
  justify-content: center;
  margin: 12px 0;
  padding: 8px;
  background: #f5f7fa;
  border-radius: 6px;
}

.scan-progress-popup .stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.scan-progress-popup .stat-label {
  font-size: 11px;
  color: #909399;
}

.scan-progress-popup .stat-value {
  font-size: 14px;
  font-weight: 600;
}

.scan-progress-popup .stat-value.add {
  color: #67c23a;
}

.scan-progress-popup .stat-value.update {
  color: #409eff;
}

.scan-progress-popup .stat-value.delete {
  color: #f56c6c;
}

.scan-progress-popup .processing-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  color: #606266;
  margin-bottom: 8px;
}

.scan-progress-popup .processing-info .error {
  color: #f56c6c;
  font-weight: 500;
}

.scan-progress-popup .popup-actions {
  display: flex;
  justify-content: center;
  margin-top: 12px;
}

/* 移动端底部弹窗样式 */
.scan-progress-mobile {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  top: 0;
  background: rgba(0, 0, 0, 0.3);
  z-index: 200;
  display: flex;
  align-items: flex-end;
  justify-content: center;
}

.mobile-popup-content {
  width: 100%;
  max-width: 1200px;
  background: white;
  border-radius: 12px 12px 0 0;
  box-shadow: 0 -2px 12px rgba(0, 0, 0, 0.15);
  padding: 16px;
  max-height: 70vh;
  overflow-y: auto;
}

.mobile-popup-content .popup-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.mobile-popup-content .popup-title {
  font-size: 16px;
  font-weight: 500;
  color: #303133;
}

.mobile-popup-content .close-icon {
  cursor: pointer;
  color: #909399;
  padding: 6px;
  font-size: 16px;
}

.mobile-popup-content .close-icon:hover {
  color: #409eff;
}

.mobile-popup-content .phase-info {
  margin-bottom: 12px;
  text-align: center;
}

.mobile-popup-content .phase-text {
  font-size: 14px;
  font-weight: 500;
  color: #409eff;
}

.mobile-popup-content .scan-stats {
  display: flex;
  gap: 16px;
  justify-content: center;
  margin: 16px 0;
  padding: 12px;
  background: #f5f7fa;
  border-radius: 8px;
}

.mobile-popup-content .stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.mobile-popup-content .stat-label {
  font-size: 12px;
  color: #909399;
}

.mobile-popup-content .stat-value {
  font-size: 16px;
  font-weight: 600;
}

.mobile-popup-content .stat-value.add {
  color: #67c23a;
}

.mobile-popup-content .stat-value.update {
  color: #409eff;
}

.mobile-popup-content .stat-value.delete {
  color: #f56c6c;
}

.mobile-popup-content .processing-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 14px;
  color: #606266;
  padding: 8px 0;
}

.mobile-popup-content .processing-info .error {
  color: #f56c6c;
  font-weight: 500;
}

.mobile-popup-content .popup-actions {
  display: flex;
  justify-content: center;
  margin-top: 16px;
}

.mobile-popup-content .popup-actions .el-button {
  flex: 1;
  max-width: 200px;
}

/* 动画效果 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
