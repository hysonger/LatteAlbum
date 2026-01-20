<template>
  <div
    class="refresh-button"
    :class="{
      success: refreshStatus === 'success',
      error: refreshStatus === 'error',
      scanning: isScanning
    }"
    @click="emit('click')"
  >
    <!-- 扫描进度圆环 -->
    <svg v-if="isScanning" class="progress-ring" viewBox="0 0 36 36">
      <path
        class="progress-ring-bg"
        d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
      />
      <path
        class="progress-ring-fill"
        :stroke-dasharray="`${Math.min(progressPercentage, 100)}, 100`"
        d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
      />
    </svg>

    <!-- 状态图标 -->
    <i v-if="!isScanning && refreshStatus === 'success'" class="fas fa-check"></i>
    <i v-if="!isScanning && refreshStatus === 'error'" class="fas fa-times"></i>
    <i v-if="!isScanning && refreshStatus === 'default'" class="fas fa-sync-alt"></i>
  </div>
</template>

<script setup lang="ts">
interface Props {
  refreshStatus: 'default' | 'refreshing' | 'success' | 'error'
  isScanning: boolean
  progressPercentage?: number
}

withDefaults(defineProps<Props>(), {
  refreshStatus: 'default',
  isScanning: false,
  progressPercentage: 0
})

const emit = defineEmits<{
  click: []
}>()
</script>

<style scoped>
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

.refresh-button.scanning {
  color: #409eff;
  border-color: #409eff;
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

/* 响应式 */
@media (max-width: 768px) {
  .refresh-button {
    width: 32px;
    height: 32px;
  }
}

@media (max-width: 375px) {
  .refresh-button {
    width: 28px;
    height: 28px;
  }
}
</style>
