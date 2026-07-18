/**
 * PhotoViewer GPS 折叠懒加载测试
 *
 * 位置信息属于敏感信息，应单独折叠：展开详情面板时不请求 GPS，
 * 仅当用户再次点击展开 GPS 折叠区时才调用 /gps 端点。
 */
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import PhotoViewer from '@/components/PhotoViewer.vue'
import { fileApi } from '@/services/api'
import type { MediaFile } from '@/types'

vi.mock('@/services/api', () => ({
  fileApi: {
    getThumbnailUrl: vi.fn(() => 'thumb-url'),
    getOriginalFileUrl: vi.fn(() => 'video-url'),
    getOriginalFile: vi.fn(),
    getFileGps: vi.fn()
  }
}))

const getFileGps = vi.mocked(fileApi.getFileGps)

const imageFile: MediaFile = {
  id: 'file-1',
  fileName: 'photo.jpg',
  fileType: 'image',
  width: 4000,
  height: 3000
}

const mountViewer = (file: MediaFile = imageFile) =>
  mount(PhotoViewer, {
    props: { file, neighbors: [file] }
  })

describe('PhotoViewer GPS 折叠懒加载', () => {
  beforeEach(() => {
    getFileGps.mockReset()
  })

  it('展开详情面板时不请求 GPS', async () => {
    const wrapper = mountViewer()

    await wrapper.find('.info-toggle-btn').trigger('click')

    expect(wrapper.find('.meta-info').isVisible()).toBe(true)
    expect(getFileGps).not.toHaveBeenCalled()
  })

  it('点击展开 GPS 折叠区后才请求并显示坐标', async () => {
    getFileGps.mockResolvedValue({
      data: { hasGps: true, latitude: 31.23, longitude: 121.47 }
    } as any)
    const wrapper = mountViewer()

    // GPS 折叠头应始终存在于详情面板中，展开详情时不请求
    await wrapper.find('.info-toggle-btn').trigger('click')
    expect(wrapper.find('.gps-toggle').exists()).toBe(true)
    expect(getFileGps).not.toHaveBeenCalled()

    // 再次点击展开 GPS 折叠区，此时才请求
    await wrapper.find('.gps-toggle').trigger('click')
    expect(getFileGps).toHaveBeenCalledTimes(1)
    expect(getFileGps).toHaveBeenCalledWith('file-1')

    await nextTick()
    await nextTick()
    expect(wrapper.text()).toContain('(31.2300, 121.4700)')
  })

  it('请求返回无 GPS 时显示无位置信息', async () => {
    getFileGps.mockResolvedValue({ data: { hasGps: false } } as any)
    const wrapper = mountViewer()

    await wrapper.find('.info-toggle-btn').trigger('click')
    await wrapper.find('.gps-toggle').trigger('click')
    await nextTick()
    await nextTick()

    expect(wrapper.text()).toContain('无位置信息')
  })
})
