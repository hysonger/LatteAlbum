<template>
  <div class="sort-controls">
    <!-- 排序下拉按钮 -->
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
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'

const sortOptions = [
  { label: '按拍摄时间', value: 'exifTimestamp' },
  { label: '按创建时间', value: 'createTime' },
  { label: '按修改时间', value: 'modifyTime' },
  { label: '按文件名', value: 'fileName' }
]

interface Props {
  sortBy: string
  sortOrder: 'asc' | 'desc'
  isMobile: boolean
}

const props = withDefaults(defineProps<Props>(), {
  sortBy: 'exifTimestamp',
  sortOrder: 'desc'
})

const emit = defineEmits<{
  'update:sortBy': [value: string]
  'update:sortOrder': [value: 'asc' | 'desc']
}>()

const showSortMenu = ref(false)

// 获取排序标签
const getSortLabel = (value: string): string => {
  const option = sortOptions.find(opt => opt.value === value)
  return option ? option.label : '排序方式'
}

// 切换排序菜单
const toggleSortMenu = () => {
  showSortMenu.value = !showSortMenu.value
}

// 选择排序方式
const selectSort = (value: string) => {
  emit('update:sortBy', value)
  showSortMenu.value = false
}

// 切换排序方向
const toggleSortOrder = () => {
  const newOrder = props.sortOrder === 'desc' ? 'asc' : 'desc'
  emit('update:sortOrder', newOrder)
}

// 点击外部关闭菜单
const handleClickOutside = (event: MouseEvent) => {
  const target = event.target as HTMLElement
  if (!target.closest('.sort-container')) {
    showSortMenu.value = false
  }
}

// 监听 isMobile，当切换到移动端时关闭菜单
watch(() => props.isMobile, (mobile) => {
  if (mobile) {
    showSortMenu.value = false
  }
})

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<style scoped>
.sort-controls {
  display: flex;
  align-items: center;
  gap: 8px;
}

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

.sort-button i:last-child {
  transition: transform 0.3s ease;
}

.sort-button:hover i:last-child {
  transform: rotate(180deg);
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

/* 动画效果 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

/* 移动端隐藏 */
@media (max-width: 768px) {
  .sort-controls {
    display: none;
  }
}
</style>
