/**
 * composable + DOM 测试模板
 *
 * useScreenSize 依赖 window.innerWidth 与 resize 事件，
 * 需在组件上下文触发 onMounted（注册监听）后再 dispatch resize。
 * 后续给 composables/ 新增组合式函数时照此扩展。
 */
import { mount } from '@vue/test-utils'
import { defineComponent, h, nextTick } from 'vue'
import { useScreenSize } from '@/composables/useScreenSize'

/** 覆盖 jsdom 的 window.innerWidth（只读属性，需 defineProperty） */
const setInnerWidth = (w: number) => {
  Object.defineProperty(window, 'innerWidth', { configurable: true, writable: true, value: w })
}

/** 挂载一个消费 useScreenSize 的组件，渲染当前断点名便于断言 */
const mountScreen = () =>
  mount(
    defineComponent({
      setup() {
        const size = useScreenSize()
        return () => {
          if (size.isMobile.value) return h('div', 'mobile')
          if (size.isTablet.value) return h('div', 'tablet')
          return h('div', 'desktop')
        }
      }
    })
  )

/** 改宽度并触发 resize，等待响应式更新 */
async function resizeTo(w: number) {
  setInnerWidth(w)
  window.dispatchEvent(new Event('resize'))
  await nextTick()
}

describe('useScreenSize', () => {
  it('< 768 为移动端', async () => {
    const wrapper = mountScreen()
    await resizeTo(500)
    expect(wrapper.text()).toBe('mobile')
    wrapper.unmount()
  })

  it('768 ~ 1023 为平板', async () => {
    const wrapper = mountScreen()
    await resizeTo(900)
    expect(wrapper.text()).toBe('tablet')
    wrapper.unmount()
  })

  it('>= 1024 为桌面', async () => {
    const wrapper = mountScreen()
    await resizeTo(1280)
    expect(wrapper.text()).toBe('desktop')
    wrapper.unmount()
  })

  it('resize 后断点实时切换', async () => {
    const wrapper = mountScreen()
    await resizeTo(1280)
    expect(wrapper.text()).toBe('desktop')
    await resizeTo(500)
    expect(wrapper.text()).toBe('mobile')
    wrapper.unmount()
  })

  it('多个消费方之一卸载后，其余消费方仍响应 resize（回归：共享监听器被去重移除的问题）', async () => {
    const a = mountScreen()
    const b = mountScreen()
    await resizeTo(1280)
    expect(a.text()).toBe('desktop')

    // 模拟关闭灯箱：其中一个消费方卸载
    b.unmount()

    await resizeTo(500)
    expect(a.text()).toBe('mobile')
    a.unmount()
  })
})
