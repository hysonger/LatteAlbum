/**
 * 组件挂载测试模板
 *
 * 使用 @vue/test-utils 的 mount 挂载单文件组件，
 * 验证 props 驱动的渲染与 emit 事件。
 * RefreshButton 是纯 props + emit、零外部依赖的组件，适合作为链路样板。
 */
import { mount } from '@vue/test-utils'
import RefreshButton from '@/components/RefreshButton.vue'

describe('RefreshButton', () => {
  it('默认状态不附加状态 class 且渲染同步图标', () => {
    const wrapper = mount(RefreshButton, {
      props: { refreshStatus: 'default', isScanning: false }
    })
    const classes = wrapper.find('.refresh-button').classes()

    expect(classes).not.toContain('success')
    expect(classes).not.toContain('error')
    expect(classes).not.toContain('scanning')
    expect(wrapper.html()).toContain('fa-sync-alt')
  })

  it('success 状态附加 class 与对勾图标', () => {
    const wrapper = mount(RefreshButton, {
      props: { refreshStatus: 'success', isScanning: false }
    })
    expect(wrapper.find('.refresh-button').classes()).toContain('success')
    expect(wrapper.html()).toContain('fa-check')
  })

  it('error 状态附加 class 与叉号图标', () => {
    const wrapper = mount(RefreshButton, {
      props: { refreshStatus: 'error', isScanning: false }
    })
    expect(wrapper.find('.refresh-button').classes()).toContain('error')
    expect(wrapper.html()).toContain('fa-times')
  })

  it('扫描中渲染进度圆环', () => {
    const wrapper = mount(RefreshButton, {
      props: { refreshStatus: 'default', isScanning: true, progressPercentage: 42 }
    })
    expect(wrapper.find('.progress-ring').exists()).toBe(true)
    expect(wrapper.find('.progress-ring-fill').attributes('stroke-dasharray')).toContain('42')
  })

  it('点击触发 click 事件', async () => {
    const wrapper = mount(RefreshButton, {
      props: { refreshStatus: 'default', isScanning: false }
    })
    await wrapper.find('.refresh-button').trigger('click')
    expect(wrapper.emitted('click')).toHaveLength(1)
  })
})
