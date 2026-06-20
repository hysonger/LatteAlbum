import { ref, computed, onMounted, onUnmounted } from 'vue'
import type { Ref, ComputedRef, CSSProperties } from 'vue'

/** 将值限制在 [min, max] 区间 */
function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value))
}

/**
 * 以锚点 c 为中心，将缩放从 s0 变到 s1 时，保持锚点下同一图像点不动的偏移。
 * 推导：图像局部点 p = (c - o) / s，缩放后需 (c - o1) / s1 = p，
 *   解得 o1 = c - (c - o0) * (s1 / s0)。
 */
function zoomOffset(o0: number, c: number, s0: number, s1: number): number {
  return c - (c - o0) * (s1 / s0)
}

/**
 * 限制偏移使缩放后的图片始终覆盖容器（不露黑边）。
 * 缩放后图片跨度为 size*scale，需覆盖 [0, size]：
 *   offset ∈ [size * (1 - scale), 0]。
 * 1x 时固定为 0（不可平移）。
 */
function clampPan(offset: number, size: number, scale: number): number {
  if (scale <= 1) return 0
  return clamp(offset, size * (1 - scale), 0)
}

interface UseImageZoomOptions {
  /** 是否启用缩放（仅图片类型启用，视频禁用） */
  enabled: ComputedRef<boolean>
  /** 最小缩放倍率，默认 1 */
  minScale?: number
  /** 最大缩放倍率，默认 5 */
  maxScale?: number
}

/**
 * 图片大图查看器的缩放与拖拽 composable。
 *
 * 基于 CSS transform（translate + scale, transform-origin: 0 0）实现，
 * GPU 合成、零依赖。滚轮以光标为锚点缩放，键盘以容器中心为锚点缩放，
 * 双击在 1x/2x 间切换，鼠标点按拖拽平移并按容器边界 clamp。
 *
 * @param containerRef 容器元素 ref（坐标参照系，须与 <img> 等大）
 */
export function useImageZoom(
  containerRef: Ref<HTMLElement | null>,
  options: UseImageZoomOptions,
) {
  const minScale = options.minScale ?? 1
  const maxScale = options.maxScale ?? 5
  const enabled = options.enabled

  const scale = ref(1)
  const offsetX = ref(0)
  const offsetY = ref(0)
  const isDragging = ref(false)

  // 拖拽过程中的非响应式状态
  let dragging = false
  let dragStartX = 0
  let dragStartY = 0
  let dragStartOffsetX = 0
  let dragStartOffsetY = 0

  const cursor = computed(() => {
    if (!enabled.value) return 'default'
    if (isDragging.value) return 'grabbing'
    if (scale.value > 1) return 'grab'
    return 'default'
  })

  /** 绑定到 <img> 的样式 */
  const imgStyle = computed<CSSProperties>(() => ({
    transform: `translate(${offsetX.value}px, ${offsetY.value}px) scale(${scale.value})`,
    transformOrigin: '0 0',
    cursor: cursor.value,
    userSelect: 'none',
    WebkitUserDrag: 'none',
  }))

  /** 读取容器当前布局尺寸（不受 transform 影响） */
  const readSize = () => {
    const el = containerRef.value
    if (!el) return null
    return { W: el.offsetWidth, H: el.offsetHeight, rect: el.getBoundingClientRect() }
  }

  /**
   * 以指定锚点 (cx, cy) 将缩放变到 s1，并 clamp 偏移。
   * 调用方需保证 s1 已 clamp 到 [minScale, maxScale]。
   */
  const zoomAt = (cx: number, cy: number, s0: number, s1: number) => {
    const size = readSize()
    if (!size || s1 === s0) return
    const ox = zoomOffset(offsetX.value, cx, s0, s1)
    const oy = zoomOffset(offsetY.value, cy, s0, s1)
    scale.value = s1
    offsetX.value = clampPan(ox, size.W, s1)
    offsetY.value = clampPan(oy, size.H, s1)
  }

  /** 滚轮缩放：以光标为锚点 */
  const onWheel = (e: WheelEvent) => {
    if (!enabled.value) return
    const el = containerRef.value
    if (!el) return
    e.preventDefault()
    const rect = el.getBoundingClientRect()
    const cx = e.clientX - rect.left
    const cy = e.clientY - rect.top
    const s0 = scale.value
    // 既能处理鼠标离散滚轮（deltaY≈±100 → 倍率≈1.16），也能平滑处理触控板惯性滚动
    const factor = Math.pow(1.0015, -e.deltaY)
    const s1 = clamp(s0 * factor, minScale, maxScale)
    zoomAt(cx, cy, s0, s1)
  }

  /** 键盘缩放：以容器中心为锚点，按倍率 step 缩放 */
  const zoomByCenter = (step: number) => {
    if (!enabled.value) return
    const size = readSize()
    if (!size) return
    const s0 = scale.value
    const s1 = clamp(s0 * step, minScale, maxScale)
    zoomAt(size.W / 2, size.H / 2, s0, s1)
  }

  /** 双击：在 1x 与 2x 间切换，以双击点为锚点 */
  const onDoubleClick = (e: MouseEvent) => {
    if (!enabled.value) return
    const el = containerRef.value
    if (!el) return
    const rect = el.getBoundingClientRect()
    const cx = e.clientX - rect.left
    const cy = e.clientY - rect.top
    const s0 = scale.value
    const target = s0 > 1 ? 1 : 2
    zoomAt(cx, cy, s0, target)
  }

  const onPointerDown = (e: PointerEvent) => {
    if (!enabled.value) return
    if (e.pointerType !== 'mouse') return
    if (scale.value <= 1) return // 1x 不可拖
    dragging = true
    isDragging.value = true
    dragStartX = e.clientX
    dragStartY = e.clientY
    dragStartOffsetX = offsetX.value
    dragStartOffsetY = offsetY.value
    const target = e.target as Element | null
    target?.setPointerCapture?.(e.pointerId)
  }

  const onPointerMove = (e: PointerEvent) => {
    if (!dragging) return
    const size = readSize()
    if (!size) return
    const dx = e.clientX - dragStartX
    const dy = e.clientY - dragStartY
    offsetX.value = clampPan(dragStartOffsetX + dx, size.W, scale.value)
    offsetY.value = clampPan(dragStartOffsetY + dy, size.H, scale.value)
  }

  const endDrag = (e: PointerEvent) => {
    if (!dragging) return
    dragging = false
    isDragging.value = false
    const target = e.target as Element | null
    target?.releasePointerCapture?.(e.pointerId)
  }

  /** 容器尺寸变化后重新 clamp，避免缩放态下露出黑边 */
  const reclamp = () => {
    const size = readSize()
    if (!size) return
    offsetX.value = clampPan(offsetX.value, size.W, scale.value)
    offsetY.value = clampPan(offsetY.value, size.H, scale.value)
  }

  /** 重置到 1x（切换图片时调用） */
  const reset = () => {
    scale.value = 1
    offsetX.value = 0
    offsetY.value = 0
    dragging = false
    isDragging.value = false
  }

  // wheel 需以 passive:false 手动挂载，确保 preventDefault 生效（阻止页面滚动）
  let wheelTarget: HTMLElement | null = null
  onMounted(() => {
    wheelTarget = containerRef.value
    wheelTarget?.addEventListener('wheel', onWheel, { passive: false })
  })
  onUnmounted(() => {
    wheelTarget?.removeEventListener('wheel', onWheel)
    wheelTarget = null
  })

  return {
    scale,
    offsetX,
    offsetY,
    imgStyle,
    cursor,
    onWheel,
    zoomByCenter,
    onDoubleClick,
    onPointerDown,
    onPointerMove,
    onPointerUp: endDrag,
    onPointerCancel: endDrag,
    reclamp,
    reset,
  }
}
