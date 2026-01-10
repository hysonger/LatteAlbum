<template>
  <div class="date-navigator">
    <el-button @click="navigateDate(1)" :disabled="!canNavigatePrev" size="small">
      <i class="fas fa-arrow-left"></i>
    </el-button>
    <el-date-picker
      v-model="selectedDate"
      type="date"
      placeholder="选择日期"
      format="YYYY/MM/DD"
      value-format="YYYY-MM-DD"
      @change="handleDateChange"
      @clear="handleClear"
      clearable
      size="small"
      :disabled-date="disabledDate"
    />
    <el-button @click="navigateDate(-1)" :disabled="!canNavigateNext" size="small">
      <i class="fas fa-arrow-right"></i>
    </el-button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useGalleryStore } from '@/stores/gallery'
import { fileApi } from '@/services/api'
import type { DateInfo } from '@/types'

const galleryStore = useGalleryStore()
const selectedDate = ref<string | null>(null)
const datesWithPhotos = ref<DateInfo[]>([])
const emit = defineEmits<{
  (e: 'date-selected', files: any[], date: string): void
  (e: 'clear'): void
}>()

const currentIndex = computed(() => {
  if (!selectedDate.value) return -1
  return datesWithPhotos.value.findIndex(d => d.date === selectedDate.value)
})

const canNavigatePrev = computed(() => {
  return currentIndex.value > 0
})

const canNavigateNext = computed(() => {
  return currentIndex.value >= 0 && currentIndex.value < datesWithPhotos.value.length - 1
})

const disabledDate = (date: Date) => {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  const dateStr = `${year}-${month}-${day}`
  const hasPhoto = datesWithPhotos.value.some(d => d.date === dateStr)
  return !hasPhoto
}

const loadDates = async () => {
  try {
    const response = await fileApi.getDates({
      sortBy: galleryStore.sortBy,
      filterType: galleryStore.filterType,
      cameraModel: undefined
    })
    datesWithPhotos.value = response.data
  } catch (error) {
    console.error('获取日期列表失败:', error)
  }
}

const handleDateChange = async (value: string | null) => {
  selectedDate.value = value

  if (value) {
    try {
      const response = await fileApi.getFiles({ 
        date: value, 
        page: 0, 
        size: 1000,
        sortBy: galleryStore.sortBy,
        order: galleryStore.sortOrder,
        filterType: galleryStore.filterType
      })
      emit('date-selected', response.data.content, value)
    } catch (error) {
      console.error('获取指定日期文件失败:', error)
    }
  }
}

const navigateDate = async (direction: number) => {
  const index = currentIndex.value
  if (index < 0) return

  const newIndex = index + direction
  if (newIndex < 0 || newIndex >= datesWithPhotos.value.length) return

  const newDate = datesWithPhotos.value[newIndex].date
  selectedDate.value = newDate

  try {
    const response = await fileApi.getFiles({ 
      date: newDate, 
      page: 0, 
      size: 1000,
      sortBy: galleryStore.sortBy,
      order: galleryStore.sortOrder,
      filterType: galleryStore.filterType
    })
    emit('date-selected', response.data.content, newDate)
  } catch (error) {
    console.error('获取指定日期文件失败:', error)
  }
}

const handleClear = () => {
  selectedDate.value = null
  emit('clear')
}

watch(() => [galleryStore.sortBy, galleryStore.sortOrder, galleryStore.filterType], async () => {
  await loadDates()
  
  if (selectedDate.value) {
    try {
      const response = await fileApi.getFiles({ 
        date: selectedDate.value, 
        page: 0, 
        size: 1000,
        sortBy: galleryStore.sortBy,
        order: galleryStore.sortOrder,
        filterType: galleryStore.filterType
      })
      emit('date-selected', response.data.content, selectedDate.value)
    } catch (error) {
      console.error('重新加载日期文件失败:', error)
    }
  }
}, { deep: true })

onMounted(() => {
  loadDates()
})
</script>

<style scoped>
.date-navigator {
  display: flex;
  gap: 6px;
  align-items: center;
  padding: 0;
  background: transparent;
  border-radius: 4px;
  box-shadow: none;
  height: 32px;
}

:deep(.el-date-editor--date) {
  height: 32px;
  padding: 0;
  box-sizing: border-box;
  flex: 1;
  min-width: 100px;
  max-width: 120px;
}

:deep(.el-input__wrapper) {
  height: 32px;
  padding: 0 6px;
  box-sizing: border-box;
}

:deep(.el-input__inner) {
  height: 32px;
  line-height: 32px;
  box-sizing: border-box;
  font-size: 13px;
}

:deep(.el-button) {
  height: 32px;
  line-height: 32px;
  padding: 0 8px;
  box-sizing: border-box;
  min-width: 32px;
}

:deep(.el-button:disabled) {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>