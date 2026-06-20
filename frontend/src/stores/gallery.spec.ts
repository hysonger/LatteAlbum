/**
 * Pinia store 测试模板
 *
 * 用 vi.mock 替换 @/services/api，避免真实网络请求；
 * 用 createPinia + setActivePinia 为每个用例提供独立 store 实例。
 * 后续给 stores/ 新增 store 时照此扩展。
 */
vi.mock('@/services/api', () => ({
  fileApi: {
    getFiles: vi.fn()
  }
}))

import { createPinia, setActivePinia } from 'pinia'
import { fileApi } from '@/services/api'
import { useGalleryStore } from '@/stores/gallery'
import type { MediaFile } from '@/types'

const getFiles = vi.mocked(fileApi.getFiles)

const sampleItems = (count: number): MediaFile[] =>
  Array.from({ length: count }, (_, i) => ({
    id: String(i),
    fileName: `file-${i}.jpg`,
    fileType: 'image'
  }))

/** 让 getFiles 返回指定分页结构 */
function mockPage(page: number, totalPages: number, count = 1) {
  getFiles.mockResolvedValue({
    data: {
      items: sampleItems(count),
      total: count * totalPages,
      page,
      size: 100,
      totalPages
    }
  } as any)
}

describe('gallery store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    getFiles.mockReset()
  })

  it('loadPage(0) 替换首屏 items', async () => {
    mockPage(0, 5, 2)
    const store = useGalleryStore()

    await store.loadPage(0)

    expect(store.items).toHaveLength(2)
    expect(store.currentPage).toBe(0)
    expect(store.hasMore).toBe(true)
  })

  it('loadPage 追加而非替换', async () => {
    mockPage(0, 5, 2)
    const store = useGalleryStore()
    await store.loadPage(0)

    mockPage(1, 5, 3)
    await store.loadPage(1)

    expect(store.items).toHaveLength(5)
    expect(store.currentPage).toBe(1)
  })

  it('到达最后一页后 hasMore 为 false', async () => {
    mockPage(4, 5, 1)
    const store = useGalleryStore()

    await store.loadPage(4)

    expect(store.hasMore).toBe(false)
  })

  it('loadNextPage 在 loading 时跳过', async () => {
    mockPage(0, 5, 2)
    const store = useGalleryStore()
    await store.loadPage(0)

    store.isLoading = true
    await store.loadNextPage()

    // 仅首屏调用了一次 getFiles
    expect(getFiles).toHaveBeenCalledTimes(1)
  })

  it('setDateResults / clearDateResults 切换 displayItems', () => {
    const store = useGalleryStore()
    const files = sampleItems(2)

    store.setDateResults(files)
    expect(store.showDateResults).toBe(true)
    // displayItems 经 Pinia reactive 包装为代理，按内容深度相等比较
    expect(store.displayItems).toEqual(files)

    store.clearDateResults()
    expect(store.showDateResults).toBe(false)
    expect(store.displayItems).toEqual(store.items)
  })

  it('初始 isEmpty 为 true', () => {
    const store = useGalleryStore()
    expect(store.isEmpty).toBe(true)
  })
})
