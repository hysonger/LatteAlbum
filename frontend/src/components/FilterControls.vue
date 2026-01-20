<template>
  <div class="filter-container">
    <div
      v-for="option in filterOptions"
      :key="option.value"
      class="filter-button"
      :class="{ active: filterType === option.value }"
      @click="emit('update:filterType', option.value)"
    >
      <i :class="option.icon"></i>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  filterType: string
}

withDefaults(defineProps<Props>(), {
  filterType: 'all'
})

const emit = defineEmits<{
  'update:filterType': [value: string]
}>()

const filterOptions = [
  { label: '全部', value: 'all', icon: 'fas fa-th-large' },
  { label: '图片', value: 'image', icon: 'fas fa-image' },
  { label: '视频', value: 'video', icon: 'fas fa-video' }
]
</script>

<style scoped>
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

@media (max-width: 768px) {
  .filter-container {
    display: none;
  }
}
</style>
