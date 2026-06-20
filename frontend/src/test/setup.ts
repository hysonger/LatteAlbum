/**
 * Vitest 全局 setup
 *
 * jsdom 不实现部分浏览器 API，这里统一补齐 mock，
 * 使依赖它们的生产代码（downloadFile、Element Plus 响应式组件、懒加载等）可在测试中运行。
 *
 * 仅放「全局缺失」的 API mock；依赖真实布局尺寸（offsetWidth/Height、getBoundingClientRect）
 * 的 mock 不放这里，而由具体用例按需 stub，避免污染其它测试。
 */

/** 补齐 URL.createObjectURL / revokeObjectURL（format.ts 的 downloadFile 会用到） */
if (!('createObjectURL' in URL) || typeof URL.createObjectURL !== 'function') {
  URL.createObjectURL = () => 'blob:mock-url'
}
if (!('revokeObjectURL' in URL) || typeof URL.revokeObjectURL !== 'function') {
  URL.revokeObjectURL = () => undefined
}

/** 补齐 window.matchMedia（Element Plus 等响应式组件会读取断点） */
if (!window.matchMedia) {
  window.matchMedia = (query: string): MediaQueryList => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => {}, // 旧版 API，部分库仍调用
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => false,
  })
}

/** 补齐 IntersectionObserver（图片懒加载场景） */
if (typeof IntersectionObserver === 'undefined') {
  class IntersectionObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
    takeRecords() {
      return []
    }
  }
  ;(window as any).IntersectionObserver = IntersectionObserver
}

/** 补齐 ResizeObserver（尺寸监听场景） */
if (typeof ResizeObserver === 'undefined') {
  class ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
  }
  ;(window as any).ResizeObserver = ResizeObserver
}

// 每个测试后恢复 mock 状态，避免用例间相互影响
afterEach(() => {
  vi.restoreAllMocks()
})
