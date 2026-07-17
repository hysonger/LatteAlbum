/**
 * 通用格式化工具函数
 */

/**
 * 格式化时长（秒数转换为 MM:SS 格式）
 * @param seconds 秒数
 * @returns 格式化后的字符串，如 "1:30"
 */
export const formatDuration = (seconds: number): string => {
  if (!seconds) return ''
  const minutes = Math.floor(seconds / 60)
  const remainingSeconds = Math.floor(seconds % 60)
  return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`
}

/**
 * 格式化文件大小
 * @param bytes 字节数
 * @returns 格式化后的字符串，如 "1.5 MB"
 */
export const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB'
}

/**
 * 格式化日期时间
 * @param dateString ISO 格式的日期字符串
 * @param timezoneOffset 可选的时区偏移量（如 "+08:00"）
 * @returns 格式化的日期字符串
 */
export const formatDate = (dateString: string, timezoneOffset?: string): string => {
  const date = new Date(dateString)

  if (!timezoneOffset) {
    return date.toLocaleString('zh-CN')
  }

  const offsetHours = parseInt(timezoneOffset.substring(1, 3))
  const offsetMinutes = parseInt(timezoneOffset.substring(4, 6))
  const offsetSign = timezoneOffset[0] === '+' ? 1 : -1
  const totalOffsetMinutes = offsetSign * (offsetHours * 60 + offsetMinutes)

  const userOffset = date.getTimezoneOffset()
  const isSameTimezone = userOffset === -totalOffsetMinutes

  if (isSameTimezone) {
    return date.toLocaleString('zh-CN')
  } else {
    const timezoneLabel = `UTC${timezoneOffset}`
    return `${date.toLocaleString('zh-CN')} (${timezoneLabel})`
  }
}

/**
 * 格式化快门速度
 * @param exposureTime 快门速度字符串（如 "1/125"）
 * @returns 格式化后的字符串（如 "1/125s"）
 */
export const formatExposureTime = (exposureTime: string): string => {
  if (!exposureTime) return ''
  if (exposureTime.startsWith('1/')) {
    const denominator = parseFloat(exposureTime.substring(2))
    if (!isNaN(denominator)) {
      return `1/${denominator.toFixed(3)}s`
    }
  }
  return `${exposureTime}s`
}

/**
 * 下载文件通用函数
 * @param data Blob 数据
 * @param fileName 文件名
 */
export const downloadFile = (data: BlobPart, fileName: string): void => {
  const blob = new Blob([data])
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = fileName
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
  URL.revokeObjectURL(url)
}

/**
 * 防抖函数 - 限制函数在指定时间间隔内最多执行一次
 * @param func 要执行的函数
 * @param wait 等待时间（毫秒）
 * @returns 防抖处理后的函数
 */
export const debounce = <T extends (...args: any[]) => any>(
  func: T,
  wait: number
): ((...args: Parameters<T>) => void) => {
  let timeout: ReturnType<typeof setTimeout> | null = null
  return (...args: Parameters<T>) => {
    if (timeout) clearTimeout(timeout)
    timeout = setTimeout(() => func(...args), wait)
  }
}

/**
 * 把十进制度数格式化为 DMS（度分秒）字符串，并附加半球标识。
 * @param decimal 十进制度数（lat 范围 [-90,90]，lon 范围 [-180,180]）
 * @param isLatitude true 表示纬度（N/S），false 表示经度（E/W）
 * @returns 形如 `39°54'12"N` 或 `116°23'30"E`
 */
export const formatDmsCoord = (decimal: number, isLatitude: boolean): string => {
  const hemisphere = isLatitude
    ? (decimal >= 0 ? 'N' : 'S')
    : (decimal >= 0 ? 'E' : 'W')
  const abs = Math.abs(decimal)
  const deg = Math.floor(abs)
  const minFloat = (abs - deg) * 60
  const min = Math.floor(minFloat)
  const sec = Math.round((minFloat - min) * 60)
  // 当秒数四舍五入到 60 时进位到分；分到 60 时进位到度
  let adjSec = sec
  let adjMin = min
  let adjDeg = deg
  if (adjSec >= 60) {
    adjSec = 0
    adjMin += 1
  }
  if (adjMin >= 60) {
    adjMin = 0
    adjDeg += 1
  }
  return `${adjDeg}°${adjMin.toString().padStart(2, '0')}'${adjSec.toString().padStart(2, '0')}"${hemisphere}`
}

/**
 * 将十进制经纬度格式化为人类可读字符串，同时给出 DMS 与十进制双格式。
 * @returns 形如 `39°54'12"N 116°23'30"E (39.9033, 116.3917)`
 */
export const formatCoordinate = (lat: number, lon: number): string => {
  const dms = `${formatDmsCoord(lat, true)} ${formatDmsCoord(lon, false)}`
  const decLat = lat.toFixed(4)
  const decLon = lon.toFixed(4)
  return `${dms} (${decLat}, ${decLon})`
}

/**
 * 构造 OpenStreetMap 跳转链接。
 */
export const buildOsmUrl = (lat: number, lon: number): string => {
  const zoom = 14
  return `https://www.openstreetmap.org/?mlat=${lat}&mlon=${lon}#map=${zoom}/${lat}/${lon}`
}
