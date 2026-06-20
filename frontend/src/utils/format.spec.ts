/**
 * 纯函数测试模板
 *
 * 这是最简单的测试层级：被测函数无副作用、无 DOM 依赖，
 * 输入→输出直接断言。后续给 utils/ 下新增工具函数时照此扩展。
 */
import {
  formatDuration,
  formatFileSize,
  formatExposureTime,
  formatDate,
  debounce,
  downloadFile
} from '@/utils/format'

describe('formatDuration', () => {
  it('空/无效秒数返回空字符串', () => {
    expect(formatDuration(0)).toBe('')
    expect(formatDuration(NaN)).toBe('')
  })

  it('秒数转换为 M:SS', () => {
    expect(formatDuration(59)).toBe('0:59')
    expect(formatDuration(60)).toBe('1:00')
    expect(formatDuration(90)).toBe('1:30')
    expect(formatDuration(125)).toBe('2:05')
  })
})

describe('formatFileSize', () => {
  it('字节级直接显示 B', () => {
    expect(formatFileSize(0)).toBe('0 B')
    expect(formatFileSize(1023)).toBe('1023 B')
  })

  it('KB / MB / GB 跨档位换算', () => {
    expect(formatFileSize(1024)).toBe('1.0 KB')
    expect(formatFileSize(1024 * 1024)).toBe('1.0 MB')
    expect(formatFileSize(1024 * 1024 * 1024)).toBe('1.0 GB')
  })
})

describe('formatExposureTime', () => {
  it('空值返回空', () => {
    expect(formatExposureTime('')).toBe('')
  })

  it('分数快门补零到三位小数', () => {
    expect(formatExposureTime('1/125')).toBe('1/125.000s')
  })

  it('非分数快门原样附加 s', () => {
    expect(formatExposureTime('0.5')).toBe('0.5s')
  })
})

describe('formatDate', () => {
  it('无时区偏移时直接本地化', () => {
    const spy = vi.spyOn(Date.prototype, 'toLocaleString')
    formatDate('2024-01-01T12:00:00Z')
    expect(spy).toHaveBeenCalledWith('zh-CN')
  })

  it('同时区时不附加 UTC 标签', () => {
    // getTimezoneOffset 返回 -480 表示 UTC+8
    vi.spyOn(Date.prototype, 'getTimezoneOffset').mockReturnValue(-480)
    vi.spyOn(Date.prototype, 'toLocaleString').mockReturnValue('LOCAL')
    expect(formatDate('2024-01-01T12:00:00Z', '+08:00')).toBe('LOCAL')
  })

  it('异时区时附加 UTC 标签', () => {
    vi.spyOn(Date.prototype, 'getTimezoneOffset').mockReturnValue(-480)
    vi.spyOn(Date.prototype, 'toLocaleString').mockReturnValue('LOCAL')
    expect(formatDate('2024-01-01T12:00:00Z', '-05:00')).toBe('LOCAL (UTC-05:00)')
  })
})

describe('debounce', () => {
  it('等待时间内多次调用只执行最后一次', () => {
    vi.useFakeTimers()
    const fn = vi.fn()
    const d = debounce(fn, 100)

    d()
    d()
    d()

    expect(fn).not.toHaveBeenCalled()
    vi.advanceTimersByTime(100)
    expect(fn).toHaveBeenCalledTimes(1)

    vi.useRealTimers()
  })
})

describe('downloadFile', () => {
  it('创建并点击下载链接', () => {
    const createUrl = vi.spyOn(URL, 'createObjectURL')
    const appendSpy = vi.spyOn(document.body, 'appendChild')

    downloadFile(new Blob(['x']), 'test.txt')

    expect(createUrl).toHaveBeenCalled()
    expect(appendSpy).toHaveBeenCalled()
  })
})
