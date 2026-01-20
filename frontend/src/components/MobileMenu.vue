<template>
  <transition name="slide-up">
    <div v-if="show" class="mobile-menu">
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
            @click="handleSortChange(option.value)"
          >
            {{ option.label }}
          </div>
        </div>
        <!-- 排序方向切换 -->
        <div
          class="mobile-sort-order-toggle"
          @click="handleSortOrderChange"
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
            @click="handleFilterChange(option.value)"
          >
            <i :class="option.icon"></i>
            <span>{{ option.label }}</span>
          </div>
        </div>
      </div>
    </div>
  </transition>
</template>

<script setup lang="ts">
interface Props {
  show: boolean
  sortBy: string
  sortOrder: 'asc' | 'desc'
  filterType: string
}

withDefaults(defineProps<Props>(), {
  show: false,
  sortBy: 'exifTimestamp',
  sortOrder: 'desc',
  filterType: 'all'
})

const emit = defineEmits<{
  close: []
  'sortChange': [value: string]
  'sortOrderChange': []
  'filterChange': [value: string]
}>()

const sortOptions = [
  { label: '按拍摄时间', value: 'exifTimestamp' },
  { label: '按创建时间', value: 'createTime' },
  { label: '按修改时间', value: 'modifyTime' },
  { label: '按文件名', value: 'fileName' }
]

const filterOptions = [
  { label: '全部', value: 'all', icon: 'fas fa-th-large' },
  { label: '图片', value: 'image', icon: 'fas fa-image' },
  { label: '视频', value: 'video', icon: 'fas fa-video' }
]

const handleSortChange = (value: string) => {
  emit('sortChange', value)
}

const handleSortOrderChange = () => {
  emit('sortOrderChange')
}

const handleFilterChange = (value: string) => {
  emit('filterChange', value)
}
</script>

<style scoped>
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
</style>
