import { ref, computed, onMounted, onUnmounted } from 'vue'

// 屏幕宽度响应式状态（全局共享，避免重复监听）
const screenWidth = ref(window.innerWidth)

// 响应式断点定义
const isMobile = computed(() => screenWidth.value < 768)
const isTablet = computed(() => screenWidth.value >= 768 && screenWidth.value < 1024)
const isDesktop = computed(() => screenWidth.value >= 1024)

/**
 * 屏幕尺寸响应式 composable
 * 统一管理移动端/平板/桌面端的屏幕宽度检测
 *
 * @example
 * ```typescript
 * const { isMobile, isTablet, isDesktop } = useScreenSize()
 * ```
 */
export function useScreenSize() {
  // 注意：必须为每个调用方创建独立的监听器函数。
  // 若共享同一个模块级函数，addEventListener 会去重为单次注册，
  // 任一组件卸载时 removeEventListener 会把所有组件的监听一并移除，
  // 导致其余组件不再响应窗口尺寸变化（如关闭灯箱后断点失效）。
  function handleResize() {
    screenWidth.value = window.innerWidth
  }

  onMounted(() => {
    window.addEventListener('resize', handleResize, { passive: true })
  })

  onUnmounted(() => {
    window.removeEventListener('resize', handleResize)
  })

  return {
    /** 当前屏幕宽度（响应式） */
    screenWidth,
    /** 是否为移动端（< 768px） */
    isMobile,
    /** 是否为平板设备（768px - 1024px） */
    isTablet,
    /** 是否为桌面设备（>= 1024px） */
    isDesktop
  }
}

/** 便捷导出：用于只需要判断小屏的场景 */
export const isSmallScreen = isMobile
