/**
 * 复杂交互 composable 测试模板
 *
 * useImageZoom 依赖容器布局尺寸（offsetWidth/Height、getBoundingClientRect），
 * 而 jsdom 不实现真实布局，故在 setupZoom 内按需 stub，
 * 不污染全局 setup。后续给含 DOM 交互的 composable 扩展用例时照此处理。
 */
import { mount } from '@vue/test-utils'
import { defineComponent, h, ref, computed } from 'vue'
import type { Ref } from 'vue'
import { useImageZoom } from '@/composables/useImageZoom'

/** 挂载消费 useImageZoom 的组件并 stub 容器尺寸为 400x400 */
function setupZoom(enabled = true) {
  const containerRef: Ref<HTMLElement | null> = ref(null)
  let zoom: ReturnType<typeof useImageZoom> | undefined

  const wrapper = mount(
    defineComponent({
      setup() {
        zoom = useImageZoom(containerRef, {
          enabled: computed(() => enabled),
          maxScale: 5
        })
        return () => h('div', { ref: containerRef })
      }
    })
  )

  const el = containerRef.value as HTMLElement
  Object.defineProperty(el, 'offsetWidth', { configurable: true, value: 400 })
  Object.defineProperty(el, 'offsetHeight', { configurable: true, value: 400 })
  el.getBoundingClientRect = () =>
    ({
      left: 0,
      top: 0,
      width: 400,
      height: 400,
      right: 400,
      bottom: 400,
      x: 0,
      y: 0,
      toJSON: () => ({})
    }) as DOMRect

  return { wrapper, zoom: zoom! }
}

describe('useImageZoom', () => {
  it('初始为 1x 且无偏移', () => {
    const { wrapper, zoom } = setupZoom()
    expect(zoom.scale.value).toBe(1)
    expect(zoom.offsetX.value).toBe(0)
    expect(zoom.offsetY.value).toBe(0)
    wrapper.unmount()
  })

  it('zoomByCenter 以容器中心放大', () => {
    const { wrapper, zoom } = setupZoom()
    zoom.zoomByCenter(2)
    expect(zoom.scale.value).toBe(2)
    wrapper.unmount()
  })

  it('缩放不低于 minScale', () => {
    const { wrapper, zoom } = setupZoom()
    zoom.zoomByCenter(2)
    zoom.zoomByCenter(0.1)
    expect(zoom.scale.value).toBe(1)
    wrapper.unmount()
  })

  it('缩放不超过 maxScale', () => {
    const { wrapper, zoom } = setupZoom()
    zoom.zoomByCenter(100)
    expect(zoom.scale.value).toBe(5)
    wrapper.unmount()
  })

  it('双击在 1x 与 2x 间切换', () => {
    const { wrapper, zoom } = setupZoom()
    zoom.onDoubleClick({ clientX: 200, clientY: 200 } as MouseEvent)
    expect(zoom.scale.value).toBe(2)
    zoom.onDoubleClick({ clientX: 200, clientY: 200 } as MouseEvent)
    expect(zoom.scale.value).toBe(1)
    wrapper.unmount()
  })

  it('滚轮以光标为锚点放大并产生偏移', () => {
    const { wrapper, zoom } = setupZoom()
    zoom.onWheel({
      deltaY: -100,
      clientX: 200,
      clientY: 200,
      preventDefault: () => {}
    } as WheelEvent)
    expect(zoom.scale.value).toBeGreaterThan(1)
    // 光标在容器右侧，放大后图像向左偏移以保持锚点不动
    expect(zoom.offsetX.value).toBeLessThan(0)
    wrapper.unmount()
  })

  it('鼠标滚轮大步长保持原灵敏度（deltaY=-100 → 倍率≈1.16）', () => {
    const { wrapper, zoom } = setupZoom()
    zoom.onWheel({
      deltaY: -100,
      deltaMode: 0,
      clientX: 200,
      clientY: 200,
      preventDefault: () => {}
    } as WheelEvent)
    expect(zoom.scale.value).toBeCloseTo(Math.exp(0.15), 3)
    wrapper.unmount()
  })

  it('触控板小步长滚动使用更高灵敏度', () => {
    const { wrapper, zoom } = setupZoom()
    zoom.onWheel({
      deltaY: -5,
      deltaMode: 0,
      clientX: 200,
      clientY: 200,
      preventDefault: () => {}
    } as WheelEvent)
    // 小步长按触控板系数 0.01 计算：exp(0.05)
    expect(zoom.scale.value).toBeCloseTo(Math.exp(0.05), 3)
    wrapper.unmount()
  })

  it('触控板捏合（ctrlKey）使用更高灵敏度', () => {
    const { wrapper, zoom } = setupZoom()
    zoom.onWheel({
      deltaY: -20,
      deltaMode: 0,
      ctrlKey: true,
      clientX: 200,
      clientY: 200,
      preventDefault: () => {}
    } as WheelEvent)
    expect(zoom.scale.value).toBeCloseTo(Math.exp(0.2), 3)
    wrapper.unmount()
  })

  it('行模式（Firefox 滚轮）按约 33px/行归一化，速度同像素模式鼠标', () => {
    const { wrapper, zoom } = setupZoom()
    zoom.onWheel({
      deltaY: -3,
      deltaMode: 1,
      clientX: 200,
      clientY: 200,
      preventDefault: () => {}
    } as WheelEvent)
    // -3 行 ≈ -99px，按鼠标系数 0.0015：exp(0.1485)
    expect(zoom.scale.value).toBeCloseTo(Math.exp(0.1485), 3)
    wrapper.unmount()
  })

  it('reset 复位到 1x', () => {
    const { wrapper, zoom } = setupZoom()
    zoom.zoomByCenter(2)
    zoom.reset()
    expect(zoom.scale.value).toBe(1)
    expect(zoom.offsetX.value).toBe(0)
    expect(zoom.offsetY.value).toBe(0)
    wrapper.unmount()
  })

  it('1x 时拖拽不启动', () => {
    const { wrapper, zoom } = setupZoom()
    zoom.onPointerDown({
      pointerType: 'mouse',
      clientX: 100,
      clientY: 100,
      target: null,
      pointerId: 1
    } as PointerEvent)
    zoom.onPointerMove({ clientX: 200, clientY: 100 } as PointerEvent)
    expect(zoom.offsetX.value).toBe(0)
    wrapper.unmount()
  })

  it('放大后可拖拽平移', () => {
    const { wrapper, zoom } = setupZoom()
    zoom.zoomByCenter(2) // offsetX 变为 -200
    zoom.onPointerDown({
      pointerType: 'mouse',
      clientX: 100,
      clientY: 200,
      target: null,
      pointerId: 1
    } as PointerEvent)
    zoom.onPointerMove({ clientX: 150, clientY: 200 } as PointerEvent)
    expect(zoom.offsetX.value).toBeCloseTo(-150, 5)
    wrapper.unmount()
  })

  /** 触屏 pointer 事件 stub（带 setPointerCapture 的 target） */
  const touchEvent = (pointerId: number, x: number, y: number) =>
    ({
      pointerId,
      pointerType: 'touch',
      clientX: x,
      clientY: y,
      target: { setPointerCapture: () => {}, releasePointerCapture: () => {} }
    }) as unknown as PointerEvent

  it('双指捏合以双指中点为锚点缩放', () => {
    const { wrapper, zoom } = setupZoom()
    zoom.onPointerDown(touchEvent(1, 100, 200))
    zoom.onPointerDown(touchEvent(2, 300, 200))
    // 距离 200 → 400，倍率 x2，中点 (300, 200)
    zoom.onPointerMove(touchEvent(2, 500, 200))
    expect(zoom.scale.value).toBeCloseTo(2, 5)
    // 锚点 (300,200)：o = c - (c - 0) * 2 → (-300, -200)
    expect(zoom.offsetX.value).toBeCloseTo(-300, 5)
    expect(zoom.offsetY.value).toBeCloseTo(-200, 5)
    wrapper.unmount()
  })

  it('捏合缩放受 maxScale 限制', () => {
    const { wrapper, zoom } = setupZoom()
    zoom.onPointerDown(touchEvent(1, 100, 200))
    zoom.onPointerDown(touchEvent(2, 300, 200))
    zoom.onPointerMove(touchEvent(2, 3000, 200)) // 距离 2900，倍率远超 maxScale
    expect(zoom.scale.value).toBe(5)
    wrapper.unmount()
  })

  it('捏合结束剩一指时以该指为基准继续平移', () => {
    const { wrapper, zoom } = setupZoom()
    zoom.onPointerDown(touchEvent(1, 100, 200))
    zoom.onPointerDown(touchEvent(2, 300, 200))
    zoom.onPointerMove(touchEvent(2, 500, 200)) // scale=2, offset=(-300,-200)
    zoom.onPointerUp(touchEvent(2, 500, 200))
    // 剩余手指从 (100,200) 移到 (150,220)：dx=50, dy=20
    zoom.onPointerMove(touchEvent(1, 150, 220))
    expect(zoom.offsetX.value).toBeCloseTo(-250, 5)
    expect(zoom.offsetY.value).toBeCloseTo(-180, 5)
    zoom.onPointerUp(touchEvent(1, 150, 220))
    wrapper.unmount()
  })

  it('单指触屏在放大后可平移', () => {
    const { wrapper, zoom } = setupZoom()
    zoom.zoomByCenter(2) // offset (-200, -200)
    zoom.onPointerDown(touchEvent(1, 100, 100))
    zoom.onPointerMove(touchEvent(1, 120, 130))
    expect(zoom.offsetX.value).toBeCloseTo(-180, 5)
    expect(zoom.offsetY.value).toBeCloseTo(-170, 5)
    wrapper.unmount()
  })

  it('1x 时单指触屏不平移', () => {
    const { wrapper, zoom } = setupZoom()
    zoom.onPointerDown(touchEvent(1, 100, 100))
    zoom.onPointerMove(touchEvent(1, 150, 150))
    expect(zoom.offsetX.value).toBe(0)
    expect(zoom.offsetY.value).toBe(0)
    wrapper.unmount()
  })

  it('enabled=false 时缩放无效', () => {
    const { wrapper, zoom } = setupZoom(false)
    zoom.zoomByCenter(2)
    expect(zoom.scale.value).toBe(1)
    wrapper.unmount()
  })
})
